//! A rede completa **em Rust**: o mesmo que o `scripts/app.luau` do `rede_completa_luau` faz,
//! mas num `Component` — pela API pública `glacier_ui::net` (`Request`, `Client`,
//! `AbortHandle`, `Multipart`, `clear_cookies`), que usa a MESMA camada de rede do `fetch` do
//! Luau (o mesmo jar de cookies, o mesmo multipart, o mesmo cancelamento).
//!
//! `net::Client` é o equivalente Rust do `http()` do Luau: `base_url`, cabeçalhos, parâmetros,
//! retries, hooks, `extend`, cookies, multipart, cancelamento — e um atalho por método
//! (`get`, `post`, `put`, `patch`, `delete`, `head`, `options`, `query`).
//!
//! Compartilhado pelos exemplos `rede_completa` e `rede_completa`.

use std::sync::Arc;

use glacier_ui::net::{AbortHandle, Client, Multipart, MultipartFile, Request, clear_cookies};
use glacier_ui::{Component, Context, EffectOutcome, Template};
use serde_json::{Value, json};

const BASE: &str = "http://127.0.0.1:8096";
const AMOSTRA: &str = "examples/rede_completa/rede_amostra.txt";

// ───────────────────────────────────── os 21 casos ─────────────────────────────────────

/// Resume o eco JSON do servidor numa linha.
fn eco(corpo: &str) -> String {
    let j: Value = serde_json::from_str(corpo).unwrap_or(Value::Null);
    let achatar = |v: &Value| -> String {
        let mut itens: Vec<String> = v
            .as_object()
            .map(|o| o.iter().map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or(""))).collect())
            .unwrap_or_default();
        itens.sort();
        itens.join(", ")
    };
    format!(
        "{} {}  query[{}]  headers[{}]  corpo={}",
        j["method"].as_str().unwrap_or("nil"),
        j["path"].as_str().unwrap_or("nil"),
        achatar(&j["query"]).replace(", ", ","),
        achatar(&j["headers"]),
        j["body"].as_str().unwrap_or("")
    )
}

fn multipart_amostra(campos: &[(&str, &str)], nome_no_servidor: Option<&str>) -> Multipart {
    Multipart {
        fields: campos.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        files: vec![MultipartFile {
            name: "anexo".into(),
            path: AMOSTRA.into(),
            filename: nome_no_servidor.map(String::from),
            content_type: nome_no_servidor.is_none().then(|| "text/plain".to_string()),
        }],
    }
}

/// Cancela `ctl` daqui a 300 ms, de outra thread (o equivalente do `after(300, …)` do Luau).
fn cancelar_em_300ms(ctl: &AbortHandle) {
    let c = ctl.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        c.abort();
    });
}

pub async fn rodar_tudo() -> Vec<(String, String)> {
    let mut s: Vec<String> = Vec::new();
    let url = |c: &str| format!("{BASE}{c}");

    // ── fetch ──
    let r = Request::get(url("/json?busca=glacier&pagina=2")).send().await;
    s.push(format!("status {}: {}", r.status, eco(&r.body)));
    let r = Request::post(url("/json"))
        .header("Content-Type", "application/json")
        .header("X-Trace", "abc")
        .body(r#"{"idade":30,"nome":"ana"}"#)
        .send()
        .await;
    s.push(eco(&r.body));
    let r = Request::get(url("/lento")).timeout_ms(300).send().await;
    s.push(format!("ok={} status={} error=\"{}\"", r.ok, r.status, r.error));
    let r = Request::get(format!("file://{AMOSTRA}")).send().await;
    s.push(format!("ok={}, {} caracteres lidos de {AMOSTRA}", r.ok, r.body.chars().count()));
    let dest = "/tmp/glacier_rede_download_rust.bin";
    let r = Request::get(url("/download")).download_to(dest).send().await;
    let tam = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    s.push(format!("ok={}; {tam} bytes gravados em {dest} (esperado: 2000)", r.ok));

    // ── cookies ──
    clear_cookies(None);
    Request::get(url("/set")).send().await;
    s.push(Request::get(url("/eco")).send().await.body);
    s.push(Request::get(url("/eco")).no_cookies().send().await.body);
    Request::get(url("/apaga")).send().await;
    s.push(Request::get(url("/eco")).send().await.body);
    clear_cookies(None);
    s.push(Request::get(url("/eco")).send().await.body);

    // ── multipart ──
    let r = Request::post(url("/upload"))
        .multipart(multipart_amostra(&[("titulo", "olá"), ("versao", "2")], None))
        .send()
        .await;
    s.push(if r.ok { r.body } else { format!("erro: {}", r.error) });

    // ── abort ──
    let ctl = AbortHandle::new();
    cancelar_em_300ms(&ctl);
    let r = Request::get(url("/lento")).abortable(&ctl).send().await;
    s.push(format!("ok={} error=\"{}\" aborted={}", r.ok, r.error, ctl.is_aborted()));

    // ── cliente ──
    let api = Client::new(BASE)
        .default_header("X-Api-Key", "segredo")
        .default_header("Accept", "application/json")
        .default_param("versao", "1");

    s.push(eco(&api.get("/json").param("busca", "x").send().await.text()));

    let r = api
        .post("/json")
        .json(&json!({ "nome": "bia", "tags": ["a", "b"] }))
        .send()
        .await;
    s.push(eco(r.text()));

    let api2 = Client::new(BASE)
        .on_request(|p| p.headers.push(("X-Trace".into(), "trace-42".into())))
        .on_response(|r| {
            r.meta.insert("marcado".into(), "true".into());
        });
    let c = api2.get("/json").send().await;
    s.push(format!("{}  marcado={}", eco(c.text()), c.meta.contains_key("marcado")));

    let api3 = Client::new(BASE).retries(3);
    let d = api3.get("/falha").param("ate", "2").send().await;
    let tentativa = d
        .json()
        .and_then(|j| j["tentativa"].as_i64())
        .map(|n| n.to_string())
        .unwrap_or("?".into());
    s.push(format!("ok={} depois de {tentativa} tentativa(s)", d.ok()));

    let erros: Arc<std::sync::Mutex<Vec<String>>> = Arc::default();
    let e2 = erros.clone();
    let api4 = Client::new(BASE).on_error(move |e| {
        e2.lock().unwrap().push(format!("{} {} → {}", e.method, e.url, e.status));
    });
    api4.get("/status/404").send().await;
    let primeiro = erros.lock().unwrap().first().cloned().unwrap_or("(nada)".into());
    s.push(format!("on_error recebeu: {primeiro}"));

    s.push(eco(api.extend("/v1").get("/json").send().await.text()));

    Request::get(url("/set")).send().await;
    let com = api2.get("/eco").send().await.text().to_string();
    let sem = Client::new(BASE).no_cookies().get("/eco").send().await.text().to_string();
    api2.clear_cookies();
    let apos = api2.get("/eco").send().await.text().to_string();
    s.push(format!("jar ligado: [{com}]  cookies=false: [{sem}]  após clear_cookies: [{apos}]"));

    let up = api
        .post("/upload")
        .multipart(multipart_amostra(&[("origem", "cliente")], Some("cliente.txt")))
        .send()
        .await;
    s.push(if up.ok() { up.text().to_string() } else { format!("erro: {}", up.error()) });

    let ctl2 = AbortHandle::new();
    cancelar_em_300ms(&ctl2);
    let f = api.get("/lento").abort(&ctl2).send().await;
    s.push(format!("ok={} error=\"{}\" (o cliente não repete uma chamada cancelada)", f.ok(), f.error()));

    // ── QUERY (RFC 10008): seguro e idempotente, com a consulta no corpo ──
    let q = api
        .query("/json")
        .json(&json!({ "termo": "glacier", "limite": 10 }))
        .send()
        .await;
    let mut linha = format!("status {}: {}", q.status(), eco(q.text()));
    // sem corpo/Content-Type o cliente nem envia (RFC 10008 §2)
    let ruim = api.query("/json").send().await;
    linha.push_str(&format!("\nsem corpo: ok={} status={} error=\"{}\"", ruim.ok(), ruim.status(), ruim.error()));
    s.push(linha);

    s.into_iter().enumerate().map(|(i, v)| (format!("s{}", i + 1), v)).collect()
}

// ─────────────────────────────────────── o componente ───────────────────────────────────────

pub struct RedeCompleta {
    template: &'static str,
}

impl RedeCompleta {
    pub fn novo(template: &'static str) -> Self {
        Self { template }
    }
}

impl Component for RedeCompleta {
    fn name(&self) -> &str {
        "rede_completa"
    }

    fn template(&self) -> Template {
        Template::File(self.template.into())
    }

    fn init(&mut self, ctx: &mut Context) {
        for i in 1..=21 {
            ctx.set(&format!("s{i}"), "…".to_string());
        }
        // Os 21 casos rodam fora da thread da UI; o resultado volta como dados no contexto.
        ctx.perform(async { EffectOutcome::data(rodar_tudo().await) });
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {}
}
