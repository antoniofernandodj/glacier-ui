//! Reprodução de áudio local para o `<audio>`.
//!
//! Atrás da feature **`audio`** (`rodio` + `symphonia`: MP3, WAV, OGG/Vorbis, FLAC). Sem
//! ela o módulo existe com a mesma API e tudo devolve "áudio não compilado" — um app que
//! não toca som não precisa de `libasound` (Linux) para compilar.
//!
//! # Desenho
//!
//! O `rodio` guarda o dispositivo de saída num objeto que não pode trocar de thread em
//! todas as plataformas, e a UI roda em outra thread que o tempo todo pergunta "em que
//! posição está?". Por isso o dispositivo e os `Player`s moram numa **thread própria**
//! (`glacier-audio`), que recebe comandos por um canal e publica, a cada 50 ms, um
//! retrato do estado (`Status`) numa tabela compartilhada. A UI nunca fala com o `rodio`:
//! escreve comandos e lê retratos. Nada bloqueia o desenho.
//!
//! O dispositivo só é aberto no primeiro som (um app com `<audio>` que o usuário nunca
//! aperta não toca no hardware). Se não houver dispositivo, o erro aparece no `Status` e
//! no evento `on_error`, não derruba o app.
//!
//! # O que o motor faz com isto
//!
//! O nó `<audio name src …>` registra a fonte ([`registra`], idempotente: o mesmo `name`
//! com o mesmo `src` é no-op) e desenha um player — botão, barra de posição e tempo. As
//! ações embutidas `audio:toggle:NOME`, `audio:play:NOME`, `audio:pause:NOME`,
//! `audio:stop:NOME` e `audio:seek:NOME` (com o valor em segundos) e o global Luau `audio`
//! fazem o mesmo de fora.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// O retrato de uma fonte, como a UI a enxerga.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Status {
    /// O arquivo foi aberto e decodificado.
    pub carregado: bool,
    pub tocando: bool,
    /// Posição em segundos.
    pub posicao: f64,
    /// Duração em segundos, quando o formato a informa (um MP3 sem cabeçalho pode não).
    pub duracao: Option<f64>,
    /// Chegou ao fim (e não está em `loop`).
    pub terminou: bool,
    pub erro: Option<String>,
}

/// Algo que a UI precisa saber que aconteceu, drenado por [`drena_eventos`].
#[derive(Debug, Clone, PartialEq)]
pub enum Evento {
    Terminou(String),
    Erro(String, String),
}

/// As opções de uma fonte (os atributos do `<audio>`).
#[derive(Debug, Clone, PartialEq)]
pub struct Opcoes {
    pub autoplay: bool,
    pub repetir: bool,
    /// 0.0 a 1.0 (acima de 1.0 amplifica).
    pub volume: f32,
}

impl Default for Opcoes {
    fn default() -> Self {
        Opcoes { autoplay: false, repetir: false, volume: 1.0 }
    }
}

/// As ações de evento declaradas no markup, por nome (já com o namespace do componente).
#[derive(Debug, Clone, Default)]
pub struct Acoes {
    pub ao_terminar: Option<String>,
    pub ao_errar: Option<String>,
}

fn acoes() -> &'static Mutex<HashMap<String, Acoes>> {
    static A: OnceLock<Mutex<HashMap<String, Acoes>>> = OnceLock::new();
    A.get_or_init(Default::default)
}

/// A ação declarada para o `evento` da fonte `nome`, se houver.
pub fn acao_do_evento(ev: &Evento) -> Option<String> {
    let a = acoes().lock().unwrap_or_else(|e| e.into_inner());
    match ev {
        Evento::Terminou(n) => a.get(n)?.ao_terminar.clone(),
        Evento::Erro(n, _) => a.get(n)?.ao_errar.clone(),
    }
}

/// `m:ss` (ou `h:mm:ss` a partir de uma hora).
pub fn formata_tempo(segundos: f64) -> String {
    let s = segundos.max(0.0).round() as u64;
    let (h, m, s) = (s / 3600, (s / 60) % 60, s % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

#[cfg(feature = "audio")]
mod real {
    use super::*;
    use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, Source};
    use std::fs::File;
    use std::io::BufReader;
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::time::Duration;

    enum Cmd {
        Registra { nome: String, src: String, opts: Opcoes },
        Play(String),
        Pause(String),
        Toggle(String),
        Stop(String),
        Seek(String, f64),
        Volume(String, f32),
    }

    struct Slot {
        src: String,
        player: Player,
        opts: Opcoes,
        duracao: Option<f64>,
        /// Já tocou e esvaziou: o fim foi (ou vai ser) reportado.
        terminou: bool,
        /// Quer tocar mas o `Player` está pausado de propósito.
        tocando: bool,
    }

    struct Estado {
        retratos: Mutex<HashMap<String, Status>>,
        eventos: Mutex<Vec<Evento>>,
        tx: Mutex<Option<Sender<Cmd>>>,
    }

    fn estado() -> &'static Estado {
        static E: OnceLock<Estado> = OnceLock::new();
        E.get_or_init(|| Estado {
            retratos: Default::default(),
            eventos: Default::default(),
            tx: Default::default(),
        })
    }

    /// Instante (em ms desde o primeiro uso) do último comando: o tick da UI fica ligado
    /// por um instante depois dele, para pegar a resposta da thread (que é assíncrona).
    static ULTIMO_CMD: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    fn agora_ms() -> u64 {
        static T0: OnceLock<std::time::Instant> = OnceLock::new();
        T0.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64 + 1
    }

    /// Manda um comando, subindo a thread na primeira vez.
    fn envia(c: Cmd) {
        ULTIMO_CMD.store(agora_ms(), std::sync::atomic::Ordering::Relaxed);
        let e = estado();
        let mut tx = e.tx.lock().unwrap_or_else(|x| x.into_inner());
        if tx.is_none() {
            let (s, r) = mpsc::channel();
            std::thread::Builder::new()
                .name("glacier-audio".into())
                .spawn(move || laco(r))
                .expect("thread de áudio");
            *tx = Some(s);
        }
        let _ = tx.as_ref().unwrap().send(c);
    }

    fn abre(src: &str) -> Result<(rodio::Decoder<BufReader<File>>, Option<f64>), String> {
        let f = File::open(src).map_err(|e| format!("não abri `{src}`: {e}"))?;
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        let dec = rodio::Decoder::builder()
            .with_data(BufReader::new(f))
            .with_byte_len(len)
            .with_seekable(true)
            .build()
            .map_err(|e| format!("não decodifiquei `{src}`: {e}"))?;
        let dur = dec.total_duration().map(|d| d.as_secs_f64());
        Ok((dec, dur))
    }

    fn laco(rx: Receiver<Cmd>) {
        let mut saida: Option<Result<MixerDeviceSink, String>> = None;
        let mut slots: HashMap<String, Slot> = HashMap::new();
        let e = estado();

        let publica = |slots: &HashMap<String, Slot>| {
            let mut r = e.retratos.lock().unwrap_or_else(|x| x.into_inner());
            for (nome, s) in slots {
                let pos = if s.terminou {
                    s.duracao.unwrap_or(0.0)
                } else {
                    s.player.get_pos().as_secs_f64()
                };
                let st = r.entry(nome.clone()).or_default();
                st.carregado = true;
                st.tocando = s.tocando && !s.terminou && !s.player.is_paused();
                st.posicao = pos;
                st.duracao = s.duracao;
                st.terminou = s.terminou;
            }
        };

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(cmd) => {
                    // O dispositivo é aberto sob demanda, uma vez.
                    let precisa_saida = matches!(
                        cmd,
                        Cmd::Registra { .. } | Cmd::Play(_) | Cmd::Toggle(_)
                    );
                    if precisa_saida && saida.is_none() {
                        saida = Some(
                            DeviceSinkBuilder::open_default_sink()
                                .map_err(|e| format!("sem dispositivo de áudio: {e}")),
                        );
                    }
                    let mixer = match &saida {
                        Some(Ok(s)) => Some(s.mixer()),
                        _ => None,
                    };
                    match cmd {
                        Cmd::Registra { nome, src, opts } => {
                            if let Some(s) = slots.get_mut(&nome) {
                                if s.src == src {
                                    s.opts = opts.clone();
                                    s.player.set_volume(opts.volume);
                                    continue;
                                }
                                s.player.stop();
                                slots.remove(&nome);
                            }
                            let Some(mixer) = mixer else {
                                let msg = match &saida {
                                    Some(Err(m)) => m.clone(),
                                    _ => "sem dispositivo de áudio".into(),
                                };
                                falha(&nome, msg);
                                continue;
                            };
                            match abre(&src) {
                                Ok((dec, dur)) => {
                                    let player = Player::connect_new(mixer);
                                    player.set_volume(opts.volume);
                                    player.pause();
                                    player.append(dec);
                                    let tocando = opts.autoplay;
                                    if tocando {
                                        player.play();
                                    }
                                    e.retratos.lock().unwrap_or_else(|x| x.into_inner()).remove(&nome);
                                    slots.insert(
                                        nome,
                                        Slot { src, player, opts, duracao: dur, terminou: false, tocando },
                                    );
                                }
                                Err(m) => falha(&nome, m),
                            }
                        }
                        Cmd::Play(n) | Cmd::Toggle(n) if matches!(&saida, Some(Err(_))) => {
                            if let Some(Err(m)) = &saida {
                                falha(&n, m.clone());
                            }
                        }
                        Cmd::Play(n) => reproduz(&mut slots, &n),
                        Cmd::Toggle(n) => {
                            let tocando = slots.get(&n).is_some_and(|s| s.tocando && !s.terminou);
                            if tocando {
                                pausa(&mut slots, &n);
                            } else {
                                reproduz(&mut slots, &n);
                            }
                        }
                        Cmd::Pause(n) => pausa(&mut slots, &n),
                        Cmd::Stop(n) => {
                            if let Some(s) = slots.get_mut(&n) {
                                s.player.pause();
                                let _ = s.player.try_seek(Duration::ZERO);
                                s.tocando = false;
                                s.terminou = false;
                            }
                        }
                        Cmd::Seek(n, secs) => {
                            if let Some(s) = slots.get_mut(&n) {
                                let alvo = Duration::from_secs_f64(secs.max(0.0));
                                // Voltar de um fim: recarrega, porque a fila esvaziou.
                                if s.terminou || s.player.empty() {
                                    if let (Ok((dec, _)), Some(Ok(sink))) = (abre(&s.src), &saida) {
                                        s.player.stop();
                                        s.player = Player::connect_new(sink.mixer());
                                        s.player.set_volume(s.opts.volume);
                                        s.player.append(dec);
                                        if !s.tocando {
                                            s.player.pause();
                                        }
                                        s.terminou = false;
                                    }
                                }
                                if let Err(err) = s.player.try_seek(alvo) {
                                    eprintln!("[glacier audio] seek em `{n}`: {err}");
                                }
                            }
                        }
                        Cmd::Volume(n, v) => {
                            if let Some(s) = slots.get_mut(&n) {
                                s.opts.volume = v;
                                s.player.set_volume(v);
                            }
                        }
                    }
                    publica(&slots);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Detecta o fim de cada fonte que estava tocando.
                    let mut reiniciar = Vec::new();
                    for (nome, s) in slots.iter_mut() {
                        if s.tocando && !s.terminou && s.player.empty() {
                            if s.opts.repetir {
                                reiniciar.push(nome.clone());
                            } else {
                                s.terminou = true;
                                s.tocando = false;
                                e.eventos
                                    .lock()
                                    .unwrap_or_else(|x| x.into_inner())
                                    .push(Evento::Terminou(nome.clone()));
                            }
                        }
                    }
                    for nome in reiniciar {
                        if let (Some(s), Some(Ok(sink))) = (slots.get_mut(&nome), &saida) {
                            if let Ok((dec, _)) = abre(&s.src) {
                                s.player = Player::connect_new(sink.mixer());
                                s.player.set_volume(s.opts.volume);
                                s.player.append(dec);
                            }
                        }
                    }
                    publica(&slots);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn reproduz(slots: &mut HashMap<String, Slot>, n: &str) {
        if let Some(s) = slots.get_mut(n) {
            if s.terminou || s.player.empty() {
                // Tocar de novo depois do fim: volta ao começo.
                if let Some(Ok(dec)) = Some(abre(&s.src).map(|x| x.0)) {
                    s.player.append(dec);
                }
                s.terminou = false;
            }
            s.tocando = true;
            s.player.play();
        }
    }

    fn pausa(slots: &mut HashMap<String, Slot>, n: &str) {
        if let Some(s) = slots.get_mut(n) {
            s.tocando = false;
            s.player.pause();
        }
    }

    fn falha(nome: &str, msg: String) {
        let e = estado();
        eprintln!("[glacier audio] `{nome}`: {msg}");
        e.retratos
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .entry(nome.to_string())
            .or_default()
            .erro = Some(msg.clone());
        e.eventos.lock().unwrap_or_else(|x| x.into_inner()).push(Evento::Erro(nome.to_string(), msg));
    }

    pub fn registra(nome: &str, src: &str, opts: Opcoes) {
        envia(Cmd::Registra { nome: nome.into(), src: src.into(), opts });
    }
    pub fn play(n: &str) {
        envia(Cmd::Play(n.into()));
    }
    pub fn pause(n: &str) {
        envia(Cmd::Pause(n.into()));
    }
    pub fn toggle(n: &str) {
        envia(Cmd::Toggle(n.into()));
    }
    pub fn stop(n: &str) {
        envia(Cmd::Stop(n.into()));
    }
    pub fn seek(n: &str, segundos: f64) {
        envia(Cmd::Seek(n.into(), segundos));
    }
    pub fn set_volume(n: &str, v: f32) {
        envia(Cmd::Volume(n.into(), v));
    }
    pub fn status(n: &str) -> Status {
        estado().retratos.lock().unwrap_or_else(|x| x.into_inner()).get(n).cloned().unwrap_or_default()
    }
    pub fn ativo() -> bool {
        let recente = agora_ms()
            .saturating_sub(ULTIMO_CMD.load(std::sync::atomic::Ordering::Relaxed))
            < 600
            && ULTIMO_CMD.load(std::sync::atomic::Ordering::Relaxed) != 0;
        recente
            || estado().retratos.lock().unwrap_or_else(|x| x.into_inner()).values().any(|s| s.tocando)
    }
    pub fn drena_eventos() -> Vec<Evento> {
        std::mem::take(&mut *estado().eventos.lock().unwrap_or_else(|x| x.into_inner()))
    }
    pub fn compilado() -> bool {
        true
    }
}

#[cfg(not(feature = "audio"))]
mod real {
    use super::*;
    pub fn registra(nome: &str, _src: &str, _opts: Opcoes) {
        eprintln!("[glacier audio] `{nome}`: este build não tem a feature `audio` (cargo add glacier-ui --features audio)");
    }
    pub fn play(_: &str) {}
    pub fn pause(_: &str) {}
    pub fn toggle(_: &str) {}
    pub fn stop(_: &str) {}
    pub fn seek(_: &str, _: f64) {}
    pub fn set_volume(_: &str, _: f32) {}
    pub fn status(_: &str) -> Status {
        Status { erro: Some("feature `audio` desligada".into()), ..Default::default() }
    }
    pub fn ativo() -> bool {
        false
    }
    pub fn drena_eventos() -> Vec<Evento> {
        Vec::new()
    }
    pub fn compilado() -> bool {
        false
    }
}

pub use real::{ativo, compilado, drena_eventos, pause, play, registra, seek, set_volume, status, stop, toggle};

/// Registra também as ações de evento do nó (ver [`Acoes`]).
pub fn registra_acoes(nome: &str, a: Acoes) {
    acoes().lock().unwrap_or_else(|e| e.into_inner()).insert(nome.to_string(), a);
}

/// Aplica uma ação embutida `audio:<verbo>:<nome>` (com `valor` para `seek`/`volume`).
/// `true` se a ação era de áudio.
pub fn aplica_acao(acao: &str, valor: Option<&str>) -> bool {
    let Some(resto) = acao.strip_prefix("audio:") else { return false };
    let (verbo, nome) = resto.split_once(':').unwrap_or((resto, ""));
    let num = valor.and_then(|v| v.trim().parse::<f64>().ok());
    match verbo {
        "toggle" => toggle(nome),
        "play" => play(nome),
        "pause" => pause(nome),
        "stop" => stop(nome),
        "seek" => {
            if let Some(s) = num {
                seek(nome, s)
            }
        }
        "volume" => {
            if let Some(v) = num {
                set_volume(nome, v as f32)
            }
        }
        _ => return false,
    }
    true
}
