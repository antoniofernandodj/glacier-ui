//! Trava de **instância única** por `app_id`, para [`crate::GlacierDaemon::single_instance`].
//!
//! A trava é um `TcpListener` em loopback (`127.0.0.1`), numa porta fixa
//! derivada de um hash do `app_id`. Bind bem-sucedido = este processo é o dono;
//! bind falhando com o endereço em uso = já existe um dono, e a segunda
//! tentativa manda um "ping" (uma conexão TCP, sem payload) pra ele antes de
//! encerrar sem abrir janela nenhuma.
//!
//! Por que TCP loopback em vez de socket Unix / mutex nomeado do Windows: é a
//! única primitiva de "só um dono por vez" disponível em `std` nos três
//! sistemas operacionais, sem depender de mais crates. A troca é aceitar uma
//! chance (pequena, e só entre apps que também escolheram essa faixa de porta)
//! de colisão com outro processo qualquer escutando a mesma porta — não um
//! app Glacier.
//!
//! O dono guarda o listener numa estática: só existe uma trava por processo (um
//! `GlacierDaemon` por processo), então não há necessidade de fiar o listener
//! através do `Runtime` — [`event_stream`] só lê a estática, no mesmo espírito
//! do interruptor global de `crate::tray`.

use std::net::{TcpListener, TcpStream};
use std::sync::OnceLock;

static LISTENER: OnceLock<TcpListener> = OnceLock::new();

/// Deriva uma porta estável (FNV-1a do `app_id`, mapeada em `[20000, 40000)`)
/// — mesmo `app_id` sempre cai na mesma porta, então uma segunda tentativa sabe
/// onde bater.
fn port_for(app_id: &str) -> u16 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in app_id.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    20000 + (hash % 20000) as u16
}

/// Resultado de [`acquire`].
pub enum Lock {
    /// Este processo é o dono da trava — segue com o boot normal.
    Primary,
    /// Já havia um dono. O ping foi enviado (best-effort — se ele também
    /// acabou de sair, o `connect` falha e é ignorado). O chamador deve
    /// encerrar sem construir motor nem abrir janela.
    Secondary,
}

/// Tenta se tornar o dono da trava de `app_id`. Ver [módulo](self).
pub fn acquire(app_id: &str) -> Lock {
    let port = port_for(app_id);
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            // Só há uma tentativa de `acquire` por processo (chamada uma vez em
            // `GlacierDaemon::run`), então `set` nunca encontra a estática já
            // preenchida.
            let _ = LISTENER.set(listener);
            Lock::Primary
        }
        Err(_) => {
            // Best-effort: o outro lado só precisa ver a conexão chegar
            // (`event_stream` não lê payload nenhum), não confirmar nada.
            let _ = TcpStream::connect(("127.0.0.1", port));
            Lock::Secondary
        }
    }
}

/// Stream que aceita conexões no listener guardado por [`acquire`] e emite `()`
/// a cada uma — o daemon mapeia isso pra reabrir/focar a janela principal,
/// mesmo caminho do "Open" da bandeja. `fn` (não closure), como
/// [`crate::tray::event_stream`], pra `Subscription::run` derivar a chave do
/// tipo a partir do tipo da função.
///
/// Usa `tokio::net::TcpListener::accept().await` — **não** uma thread dedicada
/// bloqueando em `std::net::TcpListener::accept()` ponteada por um
/// `std::sync::mpsc` (como a bandeja faz para o `tray-icon`, que é síncrono por
/// natureza e não tem alternativa). Aqui a alternativa async existe e é
/// obrigatória: `iced::stream::channel` roda o corpo dentro de
/// `futures::stream::select(receiver, stream::once(corpo))` — uma chamada de
/// `poll()` que nunca devolve `Poll::Pending` (por bloquear a thread de
/// verdade em vez de ceder via `.await`) morre de fome pro lado `receiver`
/// dentro do mesmo combinator: o item chega a ser mandado pro canal interno,
/// mas a metade que o entregaria pra fora nunca é repolada. Só descoberto
/// depurando com `eprintln!` — o ping chegava (confirmado via `ss` vendo o
/// accept+close no SO) e nunca surtia efeito nenhum.
pub fn event_stream() -> impl futures::Stream<Item = ()> {
    use futures::SinkExt;

    iced::stream::channel(16, |mut output: futures::channel::mpsc::Sender<()>| async move {
        let Some(listener) = LISTENER.get() else {
            return;
        };
        let Ok(std_listener) = listener.try_clone() else {
            return;
        };
        // `tokio::net::TcpListener::from_std` exige um fd não-bloqueante — o
        // `std::net::TcpListener` de `acquire` nasce bloqueante (default do
        // `std`).
        if std_listener.set_nonblocking(true).is_err() {
            return;
        }
        let Ok(listener) = tokio::net::TcpListener::from_std(std_listener) else {
            return;
        };

        while let Ok((stream, _addr)) = listener.accept().await {
            drop(stream);
            if output.send(()).await.is_err() {
                break;
            }
        }
    })
}

/// `true` quando este processo detém a trava (chamado por
/// `Runtime::subscription` pra decidir se registra [`event_stream`]).
pub fn has_lock() -> bool {
    LISTENER.get().is_some()
}
