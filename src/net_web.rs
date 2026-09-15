//! O módulo `net` no alvo `wasm32`.
//!
//! O `net.rs` nativo é hyper + rustls + tungstenite sobre socket, e o navegador
//! não dá socket. Aqui ficam os mesmos tipos (o `EngineMessage` os carrega, então
//! eles precisam existir) e as três entradas respondendo com erro: um `fetch` que
//! falha diz por quê, em vez de uma corrotina que nunca acorda.
//!
//! Na prática, hoje ninguém chega aqui: quem pede `fetch`/SSE/WebSocket é a
//! camada Luau, que na web também não existe (ver `luau_web.rs`). A troca por
//! `window.fetch`/`WebSocket`/`EventSource` do navegador é o próximo passo
//! natural quando essa camada voltar.

use futures::Stream;

use crate::component::{FetchResult, PendingFetch, StreamKind};

const SEM_REDE: &str = "rede ainda não é suportada no alvo web (wasm32) — ver docs/WEB.md";

/// Ver `net.rs`.
pub type WsSender = futures::channel::mpsc::Sender<WsCommand>;

/// Ver `net.rs`.
#[derive(Debug, Clone)]
pub enum WsCommand {
    Send(String),
    Close,
}

/// Ver `net.rs`.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    Ready(WsSender),
    Open,
    Message(String),
    Error(String),
    Closed,
}

/// Ver `net.rs`.
#[derive(Debug, Clone, Hash)]
pub struct StreamKey {
    pub engine_id: u64,
    pub owner: String,
    pub id: u64,
    pub kind: StreamKind,
    pub url: String,
    pub headers: Vec<(String, String)>,
}

pub(crate) async fn perform(req: PendingFetch) -> FetchResult {
    FetchResult::error(format!("fetch(\"{}\"): {SEM_REDE}", req.url))
}

pub(crate) fn sse(url: String, _headers: Vec<(String, String)>) -> impl Stream<Item = StreamEvent> {
    falha(url)
}

pub(crate) fn websocket(
    url: String,
    _headers: Vec<(String, String)>,
) -> impl Stream<Item = StreamEvent> {
    falha(url)
}

fn falha(url: String) -> impl Stream<Item = StreamEvent> {
    futures::stream::iter([
        StreamEvent::Error(format!("{url}: {SEM_REDE}")),
        StreamEvent::Closed,
    ])
}
