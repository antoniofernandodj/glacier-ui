//! A rede completa **em Rust**: o mesmo que o `scripts/app.luau` do `rede_completa_luau` faz,
//! mas num `Component` — pela API pública `glacier_ui::net` (`Requisicao`, `AbortHandle`,
//! `Multipart`, `limpar_cookies`), que usa a MESMA camada de rede do `fetch` do Luau (o
//! mesmo jar de cookies, o mesmo multipart, o mesmo cancelamento).
//!
//! O cliente `http()` é do Luau (vive no prelúdio); aqui há um [`Cliente`] mínimo, escrito
//! em cima de `Requisicao`, com o mesmo contrato: `base_url`, cabeçalhos, query, retries,
//! interceptors, `on_error`, `extend`, cookies, multipart e cancelamento.
//!
//! Compartilhado pelos exemplos `rede_completa` e `gvb_rede_completa`.

use std::sync::Arc;

use glacier_ui::net::{AbortHandle, ArquivoMultipart, Multipart, Requisicao, limpar_cookies};
use glacier_ui::{Component, Context, EffectOutcome, FetchResult, Template};
use serde_json::Value;

const BASE: &str = "http://127.0.0.1:8096";
const AMOSTRA: &str = "examples/comum/rede_amostra.txt";

// ───────────────────────── um cliente mínimo (o `http()` do Luau, em Rust) ─────────────────────────

pub struct Pedido {
    pub metodo: String,
    pub caminho: String,
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub corpo: Option<String>,
    pub multipart: Option<Multipart>,
    pub signal: Option<AbortHandle>,
}

pub struct Resp {
    pub res: FetchResult,
    pub json: Option<Value>,
    pub marcado: bool,
}

pub struct Erro {
    pub status: u16,
    pub url: String,
    pub metodo: String,
}

type GanchoPedido = Arc<dyn Fn(&mut Pedido) + Send + Sync>;
type GanchoResp = Arc<dyn Fn(&mut Resp) + Send + Sync>;
type GanchoErro = Arc<dyn Fn(&Erro) + Send + Sync>;

#[derive(Clone)]
pub struct Cliente {
    base: String,
    headers: Vec<(String, String)>,
    query: Vec<(String, String)>,
    retries: u32,
    cookies: bool,
    ao_pedir: Vec<GanchoPedido>,
    ao_responder: Vec<GanchoResp>,
    ao_errar: Vec<GanchoErro>,
}

fn escapa(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

impl Cliente {
    pub fn novo(base: &str) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            headers: Vec::new(),
            query: Vec::new(),
            retries: 0,
            cookies: true,
            ao_pedir: Vec::new(),
            ao_responder: Vec::new(),
            ao_errar: Vec::new(),
        }
    }

    pub fn cabecalho(mut self, k: &str, v: &str) -> Self {
        self.headers.push((k.into(), v.into()));
        self
    }
    pub fn consulta(mut self, k: &str, v: &str) -> Self {
        self.query.push((k.into(), v.into()));
        self
    }
    pub fn retries(mut self, n: u32) -> Self {
        self.retries = n;
        self
    }
    pub fn sem_cookies(mut self) -> Self {
        self.cookies = false;
        self
    }
    pub fn ao_pedir(mut self, f: impl Fn(&mut Pedido) + Send + Sync + 'static) -> Self {
        self.ao_pedir.push(Arc::new(f));
        self
    }
    pub fn ao_responder(mut self, f: impl Fn(&mut Resp) + Send + Sync + 'static) -> Self {
        self.ao_responder.push(Arc::new(f));
        self
    }
    pub fn ao_errar(mut self, f: impl Fn(&Erro) + Send + Sync + 'static) -> Self {
        self.ao_errar.push(Arc::new(f));
        self
    }

    /// Um cliente derivado: herda tudo (inclusive os ganchos) e estende a base.
    pub fn estender(&self, caminho: &str) -> Self {
        let mut c = self.clone();
        c.base = format!("{}/{}", self.base, caminho.trim_matches('/'));
        c
    }

    pub fn limpar_cookies(&self) {
        let host = self
            .base
            .split_once("://")
            .map(|(_, r)| r.split('/').next().unwrap_or(r).to_string());
        limpar_cookies(host.as_deref());
    }

    pub fn pedido(metodo: &str, caminho: &str) -> Pedido {
        Pedido {
            metodo: metodo.into(),
            caminho: caminho.into(),
            headers: Vec::new(),
            query: Vec::new(),
            corpo: None,
            multipart: None,
            signal: None,
        }
    }

    pub async fn get(&self, caminho: &str) -> Resp {
        self.enviar(Self::pedido("GET", caminho)).await
    }

    pub async fn enviar(&self, mut p: Pedido) -> Resp {
        for g in &self.ao_pedir {
            g(&mut p);
        }
        let mut query = self.query.clone();
        query.extend(p.query.clone());
        let qs: Vec<String> = query.iter().map(|(k, v)| format!("{}={}", escapa(k), escapa(v))).collect();
        let url = format!(
            "{}{}{}",
            self.base,
            if p.caminho.starts_with('/') { p.caminho.clone() } else { format!("/{}", p.caminho) },
            if qs.is_empty() { String::new() } else { format!("?{}", qs.join("&")) }
        );
        let mut tentativa = 0u32;
        let mut res;
        loop {
            tentativa += 1;
            let mut r = Requisicao::new(url.clone(), &p.metodo);
            for (k, v) in self.headers.iter().chain(p.headers.iter()) {
                r = r.cabecalho(k.clone(), v.clone());
            }
            if let Some(c) = &p.corpo {
                r = r.corpo(c.clone());
            }
            if !self.cookies {
                r = r.sem_cookies();
            }
            if let Some(s) = &p.signal {
                r = r.cancelavel(s);
            }
            if let Some(m) = &p.multipart {
                r = r.multipart(m.clone());
            }
            res = r.enviar().await;
            let repetir = if res.error == "cancelado" {
                false // cancelar é decisão de quem chamou: não se repete
            } else {
                res.status == 0 || res.status == 429 || res.status >= 500
            };
            if res.ok || tentativa > self.retries || !repetir {
                break;
            }
        }
        let json = serde_json::from_str::<Value>(&res.body).ok();
        let mut resp = Resp { res, json, marcado: false };
        for g in &self.ao_responder {
            g(&mut resp);
        }
        if !resp.res.ok {
            let e = Erro { status: resp.res.status, url, metodo: p.metodo.clone() };
            for g in &self.ao_errar {
                g(&e);
            }
        }
        resp
    }
}

// ───────────────────────────────────── os 20 casos ─────────────────────────────────────

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
        files: vec![ArquivoMultipart {
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
    let r = Requisicao::get(url("/json?busca=glacier&pagina=2")).enviar().await;
    s.push(format!("status {}: {}", r.status, eco(&r.body)));
    let r = Requisicao::post(url("/json"))
        .cabecalho("Content-Type", "application/json")
        .cabecalho("X-Trace", "abc")
        .corpo(r#"{"idade":30,"nome":"ana"}"#)
        .enviar()
        .await;
    s.push(eco(&r.body));
    let r = Requisicao::get(url("/lento")).timeout_ms(300).enviar().await;
    s.push(format!("ok={} status={} error=\"{}\"", r.ok, r.status, r.error));
    let r = Requisicao::get(format!("file://{AMOSTRA}")).enviar().await;
    s.push(format!("ok={}, {} caracteres lidos de {AMOSTRA}", r.ok, r.body.chars().count()));
    let dest = "/tmp/glacier_rede_download_rust.bin";
    let r = Requisicao::get(url("/download")).baixar_para(dest).enviar().await;
    let tam = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    s.push(format!("ok={}; {tam} bytes gravados em {dest} (esperado: 2000)", r.ok));

    // ── cookies ──
    limpar_cookies(None);
    Requisicao::get(url("/set")).enviar().await;
    s.push(Requisicao::get(url("/eco")).enviar().await.body);
    s.push(Requisicao::get(url("/eco")).sem_cookies().enviar().await.body);
    Requisicao::get(url("/apaga")).enviar().await;
    s.push(Requisicao::get(url("/eco")).enviar().await.body);
    limpar_cookies(None);
    s.push(Requisicao::get(url("/eco")).enviar().await.body);

    // ── multipart ──
    let r = Requisicao::post(url("/upload"))
        .multipart(multipart_amostra(&[("titulo", "olá"), ("versao", "2")], None))
        .enviar()
        .await;
    s.push(if r.ok { r.body } else { format!("erro: {}", r.error) });

    // ── abort ──
    let ctl = AbortHandle::novo();
    cancelar_em_300ms(&ctl);
    let r = Requisicao::get(url("/lento")).cancelavel(&ctl).enviar().await;
    s.push(format!("ok={} error=\"{}\" aborted={}", r.ok, r.error, ctl.abortado()));

    // ── cliente ──
    let api = Cliente::novo(BASE)
        .cabecalho("X-Api-Key", "segredo")
        .cabecalho("Accept", "application/json")
        .consulta("versao", "1");

    let mut p = Cliente::pedido("GET", "/json");
    p.query.push(("busca".into(), "x".into()));
    s.push(eco(&api.enviar(p).await.res.body));

    let mut p = Cliente::pedido("POST", "/json");
    p.headers.push(("Content-Type".into(), "application/json".into()));
    p.corpo = Some(r#"{"nome":"bia","tags":["a","b"]}"#.into());
    s.push(eco(&api.enviar(p).await.res.body));

    let api2 = Cliente::novo(BASE)
        .ao_pedir(|p| p.headers.push(("X-Trace".into(), "trace-42".into())))
        .ao_responder(|r| r.marcado = true);
    let c = api2.get("/json").await;
    s.push(format!("{}  marcado={}", eco(&c.res.body), c.marcado));

    let api3 = Cliente::novo(BASE).retries(3);
    let mut p = Cliente::pedido("GET", "/falha");
    p.query.push(("ate".into(), "2".into()));
    let d = api3.enviar(p).await;
    let tentativa = d.json.as_ref().and_then(|j| j["tentativa"].as_i64()).map(|n| n.to_string()).unwrap_or("?".into());
    s.push(format!("ok={} depois de {tentativa} tentativa(s)", d.res.ok));

    let erros: Arc<std::sync::Mutex<Vec<String>>> = Arc::default();
    let e2 = erros.clone();
    let api4 = Cliente::novo(BASE).ao_errar(move |e| {
        e2.lock().unwrap().push(format!("{} {} → {}", e.metodo, e.url, e.status));
    });
    api4.get("/status/404").await;
    let primeiro = erros.lock().unwrap().first().cloned().unwrap_or("(nada)".into());
    s.push(format!("on_error recebeu: {primeiro}"));

    s.push(eco(&api.estender("/v1").get("/json").await.res.body));

    Requisicao::get(url("/set")).enviar().await;
    let com = api2.get("/eco").await.res.body;
    let sem = Cliente::novo(BASE).sem_cookies().get("/eco").await.res.body;
    api2.limpar_cookies();
    let apos = api2.get("/eco").await.res.body;
    s.push(format!("jar ligado: [{com}]  cookies=false: [{sem}]  após clear_cookies: [{apos}]"));

    let mut p = Cliente::pedido("POST", "/upload");
    p.multipart = Some(multipart_amostra(&[("origem", "cliente")], Some("cliente.txt")));
    let up = api.enviar(p).await;
    s.push(if up.res.ok { up.res.body } else { format!("erro: {}", up.res.error) });

    let ctl2 = AbortHandle::novo();
    cancelar_em_300ms(&ctl2);
    let mut p = Cliente::pedido("GET", "/lento");
    p.signal = Some(ctl2);
    let f = api.enviar(p).await;
    s.push(format!("ok={} error=\"{}\" (o cliente não repete uma chamada cancelada)", f.res.ok, f.res.error));

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
        for i in 1..=20 {
            ctx.set(&format!("s{i}"), "…".to_string());
        }
        // Os 20 casos rodam fora da thread da UI; o resultado volta como dados no contexto.
        ctx.perform(async { EffectOutcome::data(rodar_tudo().await) });
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {}
}
