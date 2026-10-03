//! Trava de **instância única** por `app_id`, para [`crate::GlacierDaemon::single_instance`].
//!
//! A primitiva é a nativa de cada sistema, escolhida em tempo de compilação com
//! `cfg(...)`:
//!
//! - **Unix** (Linux, macOS, BSDs): um *Unix domain socket*. Bind bem-sucedido =
//!   este processo é o dono. No Linux o socket é **abstrato** (sem arquivo no
//!   disco — o kernel o solta quando o processo morre, então não há socket
//!   órfão após um crash); nos demais Unix é um arquivo em `temp_dir()`, e um
//!   arquivo sobrado de um dono morto é detectado (o `connect` recusa) e
//!   reaproveitado.
//! - **Windows**: um *mutex nomeado* (`CreateMutexW`, namespace `Local\`, ou
//!   seja, por sessão de usuário). `ERROR_ALREADY_EXISTS` = já existe um dono;
//!   o SO solta o mutex quando o processo morre.
//! - **Android e navegador**: sem trava (o SO / a aba já definem a instância).
//!
//! Em todos, a segunda tentativa avisa o dono ("ping", sem payload) antes de
//! encerrar sem abrir janela: no Unix é uma conexão no próprio socket; no
//! Windows, uma abertura do *named pipe* que o dono escuta (o mutex só decide
//! quem é o dono, não carrega aviso).
//!
//! O dono guarda o recurso numa estática: só existe uma trava por processo (um
//! `GlacierDaemon` por processo), então não há necessidade de fiar nada
//! através do `Runtime` — [`event_stream`] só lê a estática, no mesmo espírito
//! do interruptor global de `crate::tray`.

/// Resultado de [`acquire`].
pub enum Lock {
    /// Este processo é o dono da trava — segue com o boot normal.
    Primary,
    /// Já havia um dono. O ping foi enviado (best-effort — se ele também
    /// acabou de sair, o envio falha e é ignorado). O chamador deve encerrar
    /// sem construir motor nem abrir janela.
    Secondary,
}

/// FNV-1a do `app_id` — mesmo `app_id` sempre dá o mesmo nome, então uma
/// segunda tentativa sabe onde bater.
#[cfg(any(unix, windows))]
fn hash_of(app_id: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in app_id.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

// ── Sem trava: navegador e Android ──────────────────────────────────────────

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn acquire(_app_id: &str) -> Lock {
    Lock::Primary
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn event_stream() -> impl futures::Stream<Item = ()> {
    futures::stream::empty()
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn has_lock() -> bool {
    false
}

// ── Unix: Unix domain socket ────────────────────────────────────────────────

#[cfg(all(unix, not(target_os = "android"), not(target_arch = "wasm32")))]
mod imp {
    use super::{Lock, hash_of};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::sync::OnceLock;

    static LISTENER: OnceLock<UnixListener> = OnceLock::new();

    #[cfg(target_os = "linux")]
    pub fn acquire(app_id: &str) -> Lock {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::SocketAddr;

        let name = format!("glacier-single-{:016x}", hash_of(app_id));
        let Ok(addr) = SocketAddr::from_abstract_name(name.as_bytes()) else {
            return Lock::Primary;
        };
        match UnixListener::bind_addr(&addr) {
            Ok(listener) => {
                let _ = LISTENER.set(listener);
                Lock::Primary
            }
            Err(_) => {
                let _ = UnixStream::connect_addr(&addr);
                Lock::Secondary
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn acquire(app_id: &str) -> Lock {
        // `$TMPDIR` já é por usuário no macOS; nos BSDs o uid vai no nome.
        let uid = std::env::var("UID").unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "glacier-single-{uid}-{:016x}.sock",
            hash_of(app_id)
        ));
        let listener = UnixListener::bind(&path).or_else(|e| {
            if e.kind() != std::io::ErrorKind::AddrInUse {
                return Err(e);
            }
            // Arquivo existe: dono vivo (connect aceita) ou sobra de um crash.
            if UnixStream::connect(&path).is_ok() {
                return Err(e);
            }
            let _ = std::fs::remove_file(&path);
            UnixListener::bind(&path)
        });
        match listener {
            Ok(listener) => {
                let _ = LISTENER.set(listener);
                Lock::Primary
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Lock::Secondary,
            // Não deu pra criar o socket (permissão, etc.): melhor abrir o app
            // do que impedi-lo de abrir.
            Err(_) => Lock::Primary,
        }
    }

    /// Ver a nota sobre `tokio` em [`event_stream`](super::event_stream).
    pub fn event_stream() -> impl futures::Stream<Item = ()> {
        use futures::SinkExt;

        iced::stream::channel(
            16,
            |mut output: futures::channel::mpsc::Sender<()>| async move {
                let Some(listener) = LISTENER.get() else {
                    return;
                };
                let Ok(std_listener) = listener.try_clone() else {
                    return;
                };
                // `tokio::net::UnixListener::from_std` exige fd não-bloqueante.
                if std_listener.set_nonblocking(true).is_err() {
                    return;
                }
                let Ok(listener) = tokio::net::UnixListener::from_std(std_listener) else {
                    return;
                };
                while let Ok((stream, _addr)) = listener.accept().await {
                    drop(stream);
                    if output.send(()).await.is_err() {
                        break;
                    }
                }
            },
        )
    }

    pub fn has_lock() -> bool {
        LISTENER.get().is_some()
    }
}

// ── Windows: mutex nomeado + named pipe de aviso ────────────────────────────

#[cfg(windows)]
mod imp {
    use super::{Lock, hash_of};
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    /// `HANDLE` do mutex, guardado como inteiro (ponteiro cru não é `Sync`).
    /// Nunca é fechado: viver até o fim do processo É a trava.
    static MUTEX: OnceLock<usize> = OnceLock::new();

    fn pipe_name(app_id: &str) -> String {
        format!(r"\\.\pipe\glacier-single-{:016x}", hash_of(app_id))
    }

    pub fn acquire(app_id: &str) -> Lock {
        let name: Vec<u16> = format!("Local\\glacier-single-{:016x}", hash_of(app_id))
            .encode_utf16()
            .chain(Some(0))
            .collect();
        // SAFETY: `name` é UTF-16 terminado em NUL e vive durante a chamada.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            // Não deu pra criar o mutex: melhor abrir o app do que impedi-lo.
            return Lock::Primary;
        }
        // SAFETY: lido logo após `CreateMutexW`, antes de qualquer outra chamada Win32.
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            // Best-effort: o dono só precisa ver a abertura do pipe.
            let _ = std::fs::OpenOptions::new()
                .write(true)
                .open(pipe_name(app_id));
            return Lock::Secondary;
        }
        let _ = MUTEX.set(handle as usize);
        // O nome do pipe é função só do `app_id`; guarda-o para o `event_stream`.
        let _ = PIPE.set(pipe_name(app_id));
        Lock::Primary
    }

    static PIPE: OnceLock<String> = OnceLock::new();

    pub fn event_stream() -> impl futures::Stream<Item = ()> {
        use futures::SinkExt;
        use tokio::net::windows::named_pipe::ServerOptions;

        iced::stream::channel(
            16,
            |mut output: futures::channel::mpsc::Sender<()>| async move {
                let Some(name) = PIPE.get() else {
                    return;
                };
                let Ok(mut server) = ServerOptions::new().create(name) else {
                    return;
                };
                loop {
                    if server.connect().await.is_err() {
                        break;
                    }
                    // A próxima instância nasce antes de soltar a conectada,
                    // para o pipe nunca ficar sem ouvinte entre dois pings.
                    let Ok(next) = ServerOptions::new().create(name) else {
                        break;
                    };
                    drop(std::mem::replace(&mut server, next));
                    if output.send(()).await.is_err() {
                        break;
                    }
                }
            },
        )
    }

    pub fn has_lock() -> bool {
        MUTEX.get().is_some()
    }
}

#[cfg(all(any(unix, windows), not(target_os = "android"), not(target_arch = "wasm32")))]
pub use imp::{acquire, event_stream, has_lock};
