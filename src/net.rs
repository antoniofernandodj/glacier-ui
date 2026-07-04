//! Camada HTTP mínima sobre [`hyper`], usada pelo `fetch` da camada Lua
//! (ver [`crate::lua`]). Faz uma requisição assíncrona (GET/POST/…) e devolve
//! um [`FetchResult`] — sem bloquear a thread de UI: o future roda no executor
//! do `iced` e seu resultado volta como [`crate::EngineMessage::LuaResume`].

use std::sync::OnceLock;

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::Method;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

use crate::component::{FetchResult, PendingFetch};

type HttpsClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

/// Cliente compartilhado (pool de conexões + config TLS), construído uma vez.
fn client() -> &'static HttpsClient {
    static CLIENT: OnceLock<HttpsClient> = OnceLock::new();
    CLIENT.get_or_init(|| {
        // rustls 0.23 não tem provider default embutido: instala o ring uma vez.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();
        Client::builder(TokioExecutor::new()).build(https)
    })
}

/// Executa a requisição descrita por `req` e devolve o resultado. Nunca entra
/// em pânico: qualquer erro (URL inválida, DNS, TLS, timeout de conexão…) vira
/// um [`FetchResult`] com `ok = false` e a mensagem em `error`.
pub(crate) async fn perform(req: PendingFetch) -> FetchResult {
    match send(&req).await {
        Ok(result) => result,
        Err(e) => FetchResult::error(e.to_string()),
    }
}

async fn send(req: &PendingFetch) -> Result<FetchResult, Box<dyn std::error::Error + Send + Sync>> {
    let method = Method::from_bytes(req.method.to_uppercase().as_bytes())?;
    let body = Full::new(Bytes::from(req.body.clone().unwrap_or_default()));

    let mut builder = hyper::Request::builder().method(method).uri(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    let request = builder.body(body)?;

    let response = client().request(request).await?;
    let status = response.status().as_u16();
    let bytes = response.into_body().collect().await?.to_bytes();
    let text = String::from_utf8_lossy(&bytes).into_owned();

    Ok(FetchResult {
        ok: (200..300).contains(&status),
        status,
        body: text,
        error: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test real de rede (HTTPS via hyper + rustls). Ignorado por padrão
    /// para não depender de rede na CI; rode com:
    /// `cargo test --lib net::tests::https_smoke -- --ignored`.
    #[test]
    #[ignore]
    fn https_smoke() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let req = PendingFetch::new(1, "https://example.com".into(), "GET".into(), None, Vec::new());
        let res = rt.block_on(perform(req));
        assert!(res.ok, "falhou: status={} erro={}", res.status, res.error);
        assert!(res.body.contains("Example Domain"), "corpo inesperado");
    }
}
