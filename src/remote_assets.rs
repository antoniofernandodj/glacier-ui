//! Assets servidos por HTTP — o que faz o Glacier abrir um app a partir de uma URL.
//!
//! [`HttpAssets`] é um [`AssetSource`] como o disco e o embutido, então o resto do
//! motor não muda: `GlacierDaemon::new().assets(Arc::new(HttpAssets::assinado(…)))`
//! abre o `app.gv` remoto inteiro, e um [`RoteadorAssets`](crate::RoteadorAssets)
//! monta só um prefixo (`remoto/`) na rede, deixando o resto do app local.
//!
//! # Por que a assinatura não é opcional
//!
//! Markup e `<script>` vindos de uma URL são **código remoto**: quem controla o
//! servidor, ou intercepta a conexão, roda Luau na máquina do usuário. Por isso o
//! construtor normal, [`HttpAssets::assinado`], exige a chave pública ed25519 do
//! autor — fixada no binário do cliente — e só aceita um app cujo
//! `manifest.json` esteja assinado por ela e cujos arquivos batam, byte a byte,
//! com os hashes SHA-256 do manifesto. O `https` protege o transporte; a
//! assinatura protege contra um servidor comprometido.
//! [`HttpAssets::sem_verificacao`] existe para desenvolvimento local e o nome diz
//! o que ele faz.
//!
//! # O manifesto
//!
//! ```json
//! { "versao": 1,
//!   "arquivos": { "views/app.gv": "<sha256 hex>", "views/app.gss": "…" },
//!   "assinatura": "<ed25519, base64>" }
//! ```
//!
//! A assinatura cobre o texto canônico de [`mensagem_assinada`]: os pares
//! `caminho\thash\n`, em ordem de caminho — não o JSON, que não tem forma canônica.
//! [`publicar_manifesto`] gera e assina o arquivo para uma pasta.
//!
//! # Cache, offline e atualização ao vivo
//!
//! Os arquivos ficam em disco endereçados pelo hash (`blobs/<sha256>`): o hash do
//! manifesto já é o validador, então um arquivo que não mudou nunca é baixado de
//! novo. O `manifest.json` é revalidado com `ETag`/`If-None-Match`; sem rede, o
//! último manifesto em cache (reverificado) e seus arquivos continuam servindo.
//!
//! Uma thread revalida o manifesto a cada [`HttpAssets::atualizar_a_cada`] e, se
//! um arquivo já carregado mudou, o baixa, confere o hash e avança o
//! [`modified`](AssetSource::modified) — que é o que o hot-reload já observa.
//! Uma atualização adulterada **não** substitui a versão boa: o erro fica em
//! [`HttpAssets::ultimo_erro`] e vai para o `stderr`.

use std::collections::{BTreeMap, HashMap};
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use base64::Engine as _;
use iced::time::SystemTime;
use ring::signature::KeyPair as _;
use ring::{digest, signature};

use crate::asset_source::AssetSource;

const NOME_MANIFESTO: &str = "manifest.json";
const PREFIXO_ASSINATURA: &str = "glacier-manifest-v1\n";

/// Tamanho de uma chave pública ed25519, em bytes.
pub const TAMANHO_CHAVE_PUBLICA: usize = 32;

#[derive(Debug, Clone)]
enum Modo {
    Assinado(Vec<u8>),
    SemVerificacao,
}

#[derive(Debug, Clone)]
struct Arquivo {
    bytes: Arc<Vec<u8>>,
    etag: Option<String>,
    modificado: SystemTime,
}

#[derive(Debug, Default)]
struct Estado {
    /// caminho lógico → sha256 hex, do último manifesto verificado.
    manifesto: Option<BTreeMap<String, String>>,
    manifesto_bruto: Vec<u8>,
    manifesto_etag: Option<String>,
    arquivos: HashMap<String, Arquivo>,
    ultimo_erro: Option<String>,
}

#[derive(Debug)]
struct Inner {
    base: String,
    origem: String,
    cache: Option<PathBuf>,
    modo: Modo,
    timeout: Duration,
    intervalo: Duration,
    poller_iniciado: AtomicBool,
    estado: Mutex<Estado>,
}

/// [`AssetSource`] que busca os arquivos por HTTP/HTTPS. Ver o módulo.
#[derive(Debug, Clone)]
pub struct HttpAssets {
    inner: Arc<Inner>,
}

impl HttpAssets {
    /// O construtor normal: só aceita o que estiver assinado por `chave_publica`
    /// (32 bytes, ed25519 — ver [`chave_publica_de_hex`]).
    pub fn assinado(url: &str, chave_publica: impl AsRef<[u8]>) -> Self {
        Self::novo(url, Modo::Assinado(chave_publica.as_ref().to_vec()))
    }

    /// **Sem assinatura nem hash**: o que o servidor mandar é o app, e roda — script
    /// incluso. Serve para desenvolver contra um servidor local; um app publicado
    /// usa [`assinado`](Self::assinado).
    pub fn sem_verificacao(url: &str) -> Self {
        Self::novo(url, Modo::SemVerificacao)
    }

    fn novo(url: &str, modo: Modo) -> Self {
        let base = url.trim_end_matches('/').to_string();
        let origem = origem_de(&base);
        let cache = cache_padrao(&base);
        Self {
            inner: Arc::new(Inner {
                base,
                origem,
                cache,
                modo,
                timeout: Duration::from_secs(10),
                intervalo: Duration::from_secs(2),
                poller_iniciado: AtomicBool::new(false),
                estado: Mutex::new(Estado::default()),
            }),
        }
    }

    fn mudar(self, f: impl FnOnce(&mut Inner)) -> Self {
        // Só vale antes do primeiro uso: o `Arc` ainda é único.
        let mut inner = Arc::try_unwrap(self.inner)
            .expect("configure o HttpAssets antes de passá-lo ao daemon");
        f(&mut inner);
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Pasta do cache em disco. O padrão é `~/.cache/glacier-ui/<origem>`.
    pub fn cache_em(self, dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        self.mudar(|i| i.cache = Some(dir))
    }

    /// Sem cache em disco: nada é gravado e não há modo offline.
    pub fn sem_cache(self) -> Self {
        self.mudar(|i| i.cache = None)
    }

    /// De quanto em quanto tempo o servidor é consultado por mudanças.
    /// `Duration::ZERO` desliga a atualização ao vivo.
    pub fn atualizar_a_cada(self, intervalo: Duration) -> Self {
        self.mudar(|i| i.intervalo = intervalo)
    }

    /// Teto de cada requisição (padrão: 10s).
    pub fn timeout(self, timeout: Duration) -> Self {
        self.mudar(|i| i.timeout = timeout)
    }

    /// O último erro de integridade ou de rede que a atualização ao vivo viu — o
    /// que a tela deve mostrar quando um servidor passa a servir algo adulterado.
    pub fn ultimo_erro(&self) -> Option<String> {
        self.inner.estado.lock().ok()?.ultimo_erro.clone()
    }
}

// ── leitura ──────────────────────────────────────────────────────────────────

impl Inner {
    fn url(&self, caminho: &str) -> String {
        format!("{}/{}", self.base, codificar(caminho))
    }

    fn estado(&self) -> std::sync::MutexGuard<'_, Estado> {
        self.estado.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn registrar_erro(&self, msg: String) {
        eprintln!("[glacier] assets remotos ({}): {msg}", self.origem);
        self.estado().ultimo_erro = Some(msg);
    }

    /// Garante um manifesto verificado em memória: rede primeiro, cache em disco
    /// (reverificado) se a rede falhar.
    fn garantir_manifesto(&self) -> Result<()> {
        if self.estado().manifesto.is_some() {
            return Ok(());
        }
        match self.buscar_manifesto() {
            Ok(_) => Ok(()),
            Err(erro_rede) => {
                let cacheado = self
                    .cache
                    .as_ref()
                    .and_then(|d| std::fs::read(d.join(NOME_MANIFESTO)).ok());
                match cacheado {
                    Some(bruto) => {
                        self.aceitar_manifesto(bruto, None).map_err(|e| {
                            Error::new(
                                e.kind(),
                                format!("sem rede ({erro_rede}) e o manifesto em cache falhou: {e}"),
                            )
                        })?;
                        eprintln!("[glacier] {}: sem rede, usando o cache ({erro_rede})", self.origem);
                        Ok(())
                    }
                    None => Err(erro_rede),
                }
            }
        }
    }

    /// Baixa o manifesto (condicional) e o aceita se for válido. `true` = mudou.
    fn buscar_manifesto(&self) -> Result<bool> {
        let etag = self.estado().manifesto_etag.clone();
        let r = crate::net::blocking_get(&self.url(NOME_MANIFESTO), etag.as_deref(), self.timeout)?;
        match r.status {
            304 => Ok(false),
            200 => {
                if self.estado().manifesto_bruto == r.body && self.estado().manifesto.is_some() {
                    return Ok(false);
                }
                self.aceitar_manifesto(r.body.clone(), r.etag)?;
                if let Some(dir) = &self.cache {
                    let _ = std::fs::create_dir_all(dir);
                    let _ = std::fs::write(dir.join(NOME_MANIFESTO), &r.body);
                }
                Ok(true)
            }
            s => Err(Error::other(format!(
                "{} respondeu HTTP {s}",
                self.url(NOME_MANIFESTO)
            ))),
        }
    }

    fn aceitar_manifesto(&self, bruto: Vec<u8>, etag: Option<String>) -> Result<()> {
        let Modo::Assinado(chave) = &self.modo else {
            return Err(Error::other("manifesto em modo sem verificação"));
        };
        let arquivos = verificar_manifesto(&bruto, chave)?;
        let mut estado = self.estado();
        estado.manifesto = Some(arquivos);
        estado.manifesto_bruto = bruto;
        estado.manifesto_etag = etag;
        Ok(())
    }

    /// O arquivo `esperado` (sha256 hex), do cache em disco ou da rede, **sempre
    /// conferido** contra o hash do manifesto.
    fn baixar_verificado(&self, caminho: &str, esperado: &str) -> Result<Vec<u8>> {
        let blob = self.cache.as_ref().map(|d| d.join("blobs").join(esperado));
        if let Some(b) = &blob
            && let Ok(bytes) = std::fs::read(b)
            && hex(digest::digest(&digest::SHA256, &bytes).as_ref()) == esperado
        {
            return Ok(bytes);
        }
        let r = crate::net::blocking_get(&self.url(caminho), None, self.timeout)?;
        if r.status != 200 {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("{} respondeu HTTP {}", self.url(caminho), r.status),
            ));
        }
        let obtido = hex(digest::digest(&digest::SHA256, &r.body).as_ref());
        if obtido != esperado {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "integridade: '{caminho}' não bate com o manifesto assinado \
                     (esperado {}…, recebido {}…) — arquivo adulterado, recusado",
                    &esperado[..12.min(esperado.len())],
                    &obtido[..12]
                ),
            ));
        }
        if let Some(b) = &blob
            && let Some(pai) = b.parent()
            && std::fs::create_dir_all(pai).is_ok()
        {
            let _ = std::fs::write(b, &r.body);
        }
        Ok(r.body)
    }

    fn obter(self: &Arc<Self>, caminho: &str) -> Result<Arc<Vec<u8>>> {
        let chave = normalizar(caminho)?;
        if let Some(a) = self.estado().arquivos.get(&chave) {
            return Ok(a.bytes.clone());
        }
        let (bytes, etag) = match &self.modo {
            Modo::Assinado(_) => {
                self.garantir_manifesto()?;
                let esperado = self
                    .estado()
                    .manifesto
                    .as_ref()
                    .and_then(|m| m.get(&chave).cloned())
                    .ok_or_else(|| {
                        Error::new(
                            ErrorKind::NotFound,
                            format!("'{chave}' não consta no manifesto assinado"),
                        )
                    })?;
                (self.baixar_verificado(&chave, &esperado)?, None)
            }
            Modo::SemVerificacao => {
                let r = crate::net::blocking_get(&self.url(&chave), None, self.timeout)?;
                if r.status != 200 {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("{} respondeu HTTP {}", self.url(&chave), r.status),
                    ));
                }
                (r.body, r.etag)
            }
        };
        let bytes = Arc::new(bytes);
        self.estado().arquivos.insert(
            chave,
            Arquivo {
                bytes: bytes.clone(),
                etag,
                modificado: SystemTime::now(),
            },
        );
        self.iniciar_poller();
        Ok(bytes)
    }

    fn iniciar_poller(self: &Arc<Self>) {
        if self.intervalo.is_zero() || self.poller_iniciado.swap(true, Ordering::SeqCst) {
            return;
        }
        let fraco: Weak<Inner> = Arc::downgrade(self);
        let intervalo = self.intervalo;
        let _ = std::thread::Builder::new()
            .name("glacier-assets-remotos".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(intervalo);
                    // O `HttpAssets` foi largado: a thread encerra com ele.
                    let Some(inner) = fraco.upgrade() else { return };
                    inner.revalidar();
                }
            });
    }

    /// Um ciclo da atualização ao vivo.
    fn revalidar(&self) {
        match &self.modo {
            Modo::Assinado(_) => {
                match self.buscar_manifesto() {
                    Ok(false) => return,
                    Ok(true) => {}
                    Err(e) => {
                        // Rede fora do ar não é adulteração: segue com o que há.
                        // Manifesto com assinatura ruim é: fica registrado.
                        if e.kind() == ErrorKind::InvalidData {
                            self.registrar_erro(e.to_string());
                        }
                        return;
                    }
                }
                let novos: Vec<(String, String)> = {
                    let estado = self.estado();
                    let Some(m) = &estado.manifesto else { return };
                    estado
                        .arquivos
                        .keys()
                        .filter_map(|c| m.get(c).map(|h| (c.clone(), h.clone())))
                        .collect()
                };
                for (caminho, hash) in novos {
                    let atual = hex(digest::digest(
                        &digest::SHA256,
                        &self.estado().arquivos[&caminho].bytes,
                    )
                    .as_ref());
                    if atual == hash {
                        continue;
                    }
                    match self.baixar_verificado(&caminho, &hash) {
                        Ok(bytes) => {
                            self.estado().arquivos.insert(
                                caminho,
                                Arquivo {
                                    bytes: Arc::new(bytes),
                                    etag: None,
                                    modificado: SystemTime::now(),
                                },
                            );
                            self.estado().ultimo_erro = None;
                        }
                        Err(e) => self.registrar_erro(e.to_string()),
                    }
                }
            }
            Modo::SemVerificacao => {
                let lista: Vec<(String, Option<String>, Arc<Vec<u8>>)> = self
                    .estado()
                    .arquivos
                    .iter()
                    .map(|(c, a)| (c.clone(), a.etag.clone(), a.bytes.clone()))
                    .collect();
                for (caminho, etag, antigo) in lista {
                    let Ok(r) =
                        crate::net::blocking_get(&self.url(&caminho), etag.as_deref(), self.timeout)
                    else {
                        continue;
                    };
                    if r.status != 200 || *antigo == r.body {
                        continue;
                    }
                    self.estado().arquivos.insert(
                        caminho,
                        Arquivo {
                            bytes: Arc::new(r.body),
                            etag: r.etag,
                            modificado: SystemTime::now(),
                        },
                    );
                }
            }
        }
    }
}

impl AssetSource for HttpAssets {
    fn read_bytes(&self, path: &str) -> Result<std::borrow::Cow<'static, [u8]>> {
        self.inner
            .obter(path)
            .map(|b| std::borrow::Cow::Owned((*b).clone()))
    }

    fn read_to_string(&self, path: &str) -> Result<std::borrow::Cow<'static, str>> {
        let bytes = self.inner.obter(path)?;
        String::from_utf8((*bytes).clone())
            .map(std::borrow::Cow::Owned)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    fn exists(&self, path: &str) -> bool {
        let Ok(chave) = normalizar(path) else {
            return false;
        };
        if self.inner.estado().arquivos.contains_key(&chave) {
            return true;
        }
        match &self.inner.modo {
            Modo::Assinado(_) => {
                self.inner.garantir_manifesto().is_ok()
                    && self
                        .inner
                        .estado()
                        .manifesto
                        .as_ref()
                        .is_some_and(|m| m.contains_key(&chave))
            }
            Modo::SemVerificacao => self.inner.obter(&chave).is_ok(),
        }
    }

    fn modified(&self, path: &str) -> Option<SystemTime> {
        let chave = normalizar(path).ok()?;
        self.inner.estado().arquivos.get(&chave).map(|a| a.modificado)
    }

    fn supports_reload(&self) -> bool {
        !self.inner.intervalo.is_zero()
    }

    fn origem(&self) -> Option<String> {
        Some(self.inner.origem.clone())
    }
}

// ── manifesto: verificar, gerar, assinar ─────────────────────────────────────

/// O texto que a assinatura cobre: `glacier-manifest-v1\n` seguido dos pares
/// `caminho\thash\n` em ordem de caminho. Texto e não JSON de propósito — JSON não
/// tem forma canônica, e assinar "o JSON" deixaria a assinatura depender de
/// espaço em branco e ordem de chaves.
pub fn mensagem_assinada(arquivos: &BTreeMap<String, String>) -> Vec<u8> {
    let mut m = String::from(PREFIXO_ASSINATURA);
    for (caminho, hash) in arquivos {
        m.push_str(caminho);
        m.push('\t');
        m.push_str(hash);
        m.push('\n');
    }
    m.into_bytes()
}

fn verificar_manifesto(bruto: &[u8], chave_publica: &[u8]) -> Result<BTreeMap<String, String>> {
    let invalido = |m: String| Error::new(ErrorKind::InvalidData, m);
    if chave_publica.len() != TAMANHO_CHAVE_PUBLICA {
        return Err(invalido(format!(
            "a chave pública deve ter {TAMANHO_CHAVE_PUBLICA} bytes, tem {}",
            chave_publica.len()
        )));
    }
    let json: serde_json::Value = serde_json::from_slice(bruto)
        .map_err(|e| invalido(format!("manifest.json não é JSON válido: {e}")))?;
    let arquivos: BTreeMap<String, String> = json
        .get("arquivos")
        .and_then(|a| a.as_object())
        .ok_or_else(|| invalido("manifest.json sem o objeto \"arquivos\"".into()))?
        .iter()
        .map(|(k, v)| {
            v.as_str()
                .map(|h| (k.clone(), h.to_ascii_lowercase()))
                .ok_or_else(|| invalido(format!("hash de '{k}' não é texto")))
        })
        .collect::<Result<_>>()?;
    let assinatura = json
        .get("assinatura")
        .and_then(|a| a.as_str())
        .ok_or_else(|| invalido("manifest.json sem \"assinatura\" — recusado".into()))?;
    let assinatura = base64::prelude::BASE64_STANDARD
        .decode(assinatura)
        .map_err(|e| invalido(format!("assinatura não é base64: {e}")))?;
    signature::UnparsedPublicKey::new(&signature::ED25519, chave_publica)
        .verify(&mensagem_assinada(&arquivos), &assinatura)
        .map_err(|_| {
            invalido(
                "a assinatura do manifesto NÃO bate com a chave pública do cliente — \
                 app recusado"
                    .into(),
            )
        })?;
    Ok(arquivos)
}

/// Gera um par de chaves ed25519 e devolve o PKCS#8 (a chave **secreta** — guarde
/// fora do repositório). A pública sai de [`chave_publica_de`].
pub fn nova_chave() -> Result<Vec<u8>> {
    signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
        .map(|d| d.as_ref().to_vec())
        .map_err(|_| Error::other("não foi possível gerar a chave"))
}

/// A chave pública (32 bytes) de um PKCS#8 de [`nova_chave`].
pub fn chave_publica_de(pkcs8: &[u8]) -> Result<Vec<u8>> {
    signature::Ed25519KeyPair::from_pkcs8(pkcs8)
        .map(|p| p.public_key().as_ref().to_vec())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("chave inválida: {e}")))
}

/// Decodifica uma chave pública em hexadecimal (64 caracteres) — a forma de
/// escrevê-la no código do cliente.
pub fn chave_publica_de_hex(s: &str) -> Result<Vec<u8>> {
    let s = s.trim();
    if s.len() != TAMANHO_CHAVE_PUBLICA * 2 || !s.is_ascii() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "a chave pública são 64 caracteres hexadecimais",
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "hexadecimal inválido"))
        })
        .collect()
}

/// Varre `dir`, calcula o SHA-256 de cada arquivo, assina e grava `manifest.json`
/// na própria pasta. Os caminhos do manifesto são relativos a `dir` — que é a raiz
/// que o servidor serve. Devolve o conteúdo gravado.
pub fn publicar_manifesto(dir: &Path, pkcs8: &[u8]) -> Result<String> {
    let par = signature::Ed25519KeyPair::from_pkcs8(pkcs8)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("chave inválida: {e}")))?;
    let mut arquivos = BTreeMap::new();
    coletar(dir, dir, &mut arquivos)?;
    let sig = par.sign(&mensagem_assinada(&arquivos));
    let json = serde_json::json!({
        "versao": 1,
        "arquivos": arquivos,
        "assinatura": base64::prelude::BASE64_STANDARD.encode(sig.as_ref()),
    });
    let texto = serde_json::to_string_pretty(&json).map_err(Error::other)?;
    std::fs::write(dir.join(NOME_MANIFESTO), &texto)?;
    Ok(texto)
}

fn coletar(raiz: &Path, dir: &Path, saida: &mut BTreeMap<String, String>) -> Result<()> {
    for entrada in std::fs::read_dir(dir)? {
        let caminho = entrada?.path();
        let nome = caminho.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if nome.starts_with('.') || (dir == raiz && nome == NOME_MANIFESTO) {
            continue;
        }
        if caminho.is_dir() {
            coletar(raiz, &caminho, saida)?;
        } else {
            let rel = caminho
                .strip_prefix(raiz)
                .map_err(Error::other)?
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let bytes = std::fs::read(&caminho)?;
            saida.insert(rel, hex(digest::digest(&digest::SHA256, &bytes).as_ref()));
        }
    }
    Ok(())
}

// ── utilidades ───────────────────────────────────────────────────────────────

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Caminho lógico → chave do servidor. Recusa o que escapa da raiz (`..`),
/// caminho absoluto e barra invertida: o cliente nunca pede fora da base.
fn normalizar(path: &str) -> Result<String> {
    let mut partes: Vec<&str> = Vec::new();
    for parte in path.split('/') {
        match parte {
            "" | "." => {}
            ".." => {
                if partes.pop().is_none() {
                    return Err(Error::new(
                        ErrorKind::PermissionDenied,
                        format!("'{path}' sai da raiz dos assets remotos"),
                    ));
                }
            }
            p if p.contains('\\') || p.contains(':') => {
                return Err(Error::new(
                    ErrorKind::PermissionDenied,
                    format!("'{path}' não é um caminho relativo"),
                ));
            }
            p => partes.push(p),
        }
    }
    if partes.is_empty() {
        return Err(Error::new(ErrorKind::NotFound, "caminho vazio"));
    }
    Ok(partes.join("/"))
}

/// Percent-encoding do caminho, preservando as barras.
fn codificar(caminho: &str) -> String {
    let mut s = String::new();
    for b in caminho.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~' | b'/') {
            s.push(b as char);
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    s
}

/// `https://host:porta` de uma URL, sem caminho.
fn origem_de(url: &str) -> String {
    match url.split_once("://") {
        Some((esquema, resto)) => {
            let host = resto.split('/').next().unwrap_or(resto);
            format!("{}://{}", esquema.to_ascii_lowercase(), host.to_ascii_lowercase())
        }
        None => url.to_string(),
    }
}

/// `~/.cache/glacier-ui/<url sanitizada>` — uma pasta por base, para dois apps
/// remotos não dividirem cache.
fn cache_padrao(base: &str) -> Option<PathBuf> {
    let raiz = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    let nome: String = base
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    Some(raiz.join("glacier-ui").join(nome))
}

/// Nome de pasta seguro para uma origem — o `storage` de cada origem mora nela.
pub(crate) fn pasta_da_origem(origem: &str) -> String {
    origem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
        .collect()
}
