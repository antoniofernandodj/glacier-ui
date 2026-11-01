//! O modelo de blocos do `prose` e os quatro jeitos de chegar a ele.
//!
//! Um `prose` é uma lista de [`Bloco`]s — parágrafo, título, citação, lista,
//! figura (imagem com legenda), código e régua. Eles vêm de:
//!
//! - **filhos escritos no markup** (`.gv`), lidos na carga;
//! - **conteúdo dinâmico** (`prose(syntax = html, content = @corpo)`), lido na
//!   avaliação a partir de uma chave do contexto, em `gv`, `html` ou
//!   `markdown` — o texto de um artigo que veio de um arquivo, de um `fetch` ou de
//!   um banco.
//!
//! Os dois primeiros passam por uma árvore intermediária, [`No`]: o `.gv`
//! dessugarado (XML interno) entra por [`de_xml`], o HTML por [`parse_html`], e o mesmo
//! construtor, [`blocos`], faz o resto. O Markdown tem renderizador próprio
//! (`crate::markdown`), então só passa adiante como [`Bloco::Markdown`].
//!
//! # Conteúdo dinâmico não é de confiança
//!
//! O que vem de um banco ou de uma URL não foi escrito por quem escreveu o app. Por
//! isso, em conteúdo dinâmico ([`Opcoes::confiavel`] falso): só vale `href` em link, e
//! só `http`, `https` e `mailto` — `on_click` e `navigate_to` são **recusados**, já
//! que um registro com `on_click="app::apagar"` dispararia uma ação do app; e imagem só
//! de caminho local (a regra de `crate::markdown::caminho_local`).

use crate::parser::{SpanLink, TextSpan};

/// Teto do conteúdo dinâmico: acima disso vira um [`Bloco::Erro`], em vez de travar a
/// tela moldando um texto sem fim.
pub const LIMITE_BYTES: usize = 2 * 1024 * 1024;
/// Teto de aninhamento do HTML.
const LIMITE_PROFUNDIDADE: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub enum Bloco {
    Paragrafo(Vec<TextSpan>),
    Titulo {
        nivel: u8,
        spans: Vec<TextSpan>,
    },
    /// Citação: um ou mais parágrafos e, opcionalmente, a autoria (`<footer>` ou
    /// `<cite>`).
    Citacao {
        paragrafos: Vec<Vec<TextSpan>>,
        autoria: Vec<TextSpan>,
    },
    Lista {
        ordenada: bool,
        itens: Vec<Vec<TextSpan>>,
    },
    /// Imagem **local** com legenda (rica) opcional.
    Figura {
        src: String,
        alt: String,
        legenda: Vec<TextSpan>,
    },
    Codigo(String),
    Regua,
    /// Markdown a renderizar (`syntax="markdown"`).
    Markdown(String),
    /// Conteúdo que não deu para ler: aparece na tela, em vermelho, em vez de sumir.
    Erro(String),
}

impl Bloco {
    /// O texto do bloco sem estilo — para o `selectable`, que perde estilo e link.
    pub fn texto_simples(&self) -> String {
        let junta = |s: &[TextSpan]| s.iter().map(|t| t.content.as_str()).collect::<String>();
        match self {
            Self::Paragrafo(s) | Self::Titulo { spans: s, .. } => junta(s),
            Self::Citacao { paragrafos, autoria } => {
                let mut t = paragrafos.iter().map(|p| junta(p)).collect::<Vec<_>>().join("\n\n");
                if !autoria.is_empty() {
                    t.push_str("\n— ");
                    t.push_str(&junta(autoria));
                }
                t
            }
            Self::Lista { itens, .. } => itens
                .iter()
                .map(|i| format!("• {}", junta(i)))
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Figura { alt, legenda, .. } => {
                let l = junta(legenda);
                if l.is_empty() { alt.clone() } else { l }
            }
            Self::Codigo(c) | Self::Markdown(c) => c.clone(),
            Self::Regua => String::new(),
            Self::Erro(e) => e.clone(),
        }
    }
}

/// A sintaxe do texto de um `prose(syntax = "…", content = "…")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sintaxe {
    Gv,
    Html,
    Markdown,
}

impl Sintaxe {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "gv" => Some(Self::Gv),
            "html" => Some(Self::Html),
            "markdown" | "md" => Some(Self::Markdown),
            _ => None,
        }
    }
}

/// De onde vêm os blocos de um `prose` dinâmico.
#[derive(Debug, Clone, PartialEq)]
pub struct FonteProse {
    pub sintaxe: Sintaxe,
    /// O atributo `content`: um modelo a interpolar (`{corpo}`) **antes** de ser lido.
    pub content: String,
}

// ───────────────────────────── a árvore intermediária ─────────────────────────────

/// Um nó de XML ou de HTML, só com o que o `prose` precisa.
#[derive(Debug, Clone)]
pub enum No {
    Elem {
        tag: String,
        attrs: Vec<(String, String)>,
        filhos: Vec<No>,
        /// Posição (byte) no documento, para o erro apontar a linha.
        pos: Option<usize>,
    },
    Texto(String, Option<usize>),
}

impl No {
    fn attr(&self, nomes: &[&str]) -> Option<&str> {
        let No::Elem { attrs, .. } = self else { return None };
        nomes
            .iter()
            .find_map(|n| attrs.iter().find(|(k, _)| k.eq_ignore_ascii_case(n)))
            .map(|(_, v)| v.as_str())
    }
}

/// Os filhos de um nó do roxmltree como [`No`].
pub fn de_xml(node: roxmltree::Node) -> Vec<No> {
    node.children()
        .filter_map(|c| {
            if c.is_text() {
                Some(No::Texto(c.text().unwrap_or_default().to_string(), Some(c.range().start)))
            } else if c.is_element() {
                Some(No::Elem {
                    tag: c.tag_name().name().to_ascii_lowercase(),
                    attrs: c
                        .attributes()
                        .map(|a| (a.name().to_string(), a.value().to_string()))
                        .collect(),
                    filhos: de_xml(c),
                    pos: Some(c.range().start),
                })
            } else {
                None // comentário
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erro {
    pub msg: String,
    pub dica: Option<String>,
    pub pos: Option<usize>,
}

impl Erro {
    fn novo(msg: impl Into<String>, pos: Option<usize>) -> Self {
        Self { msg: msg.into(), dica: None, pos }
    }
    fn dica(mut self, d: impl Into<String>) -> Self {
        self.dica = Some(d.into());
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Opcoes {
    /// Escrito por quem escreveu o app (o markup): pode ter `on_click`/`navigate_to`.
    pub confiavel: bool,
    /// Tolerante como um navegador: tag desconhecida some e o texto fica. Só o HTML.
    pub leniente: bool,
}

const TAGS_BLOCO: &[&str] = &[
    "p", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "ul", "ol", "figure", "pre", "hr",
    "img",
];
/// Contêineres que o HTML usa só para agrupar: o conteúdo deles é do `prose`.
const TAGS_TRANSPARENTES: &[&str] = &[
    "div", "section", "article", "main", "header", "footer", "aside", "nav", "body", "html",
    "center",
];
/// Elementos cujo conteúdo nunca é texto de leitura: somem com tudo dentro.
const TAGS_DESCARTADAS: &[&str] = &[
    "script", "style", "head", "title", "iframe", "object", "embed", "form", "input", "button",
    "select", "textarea", "noscript", "template", "svg", "canvas", "audio", "video", "link", "meta",
    "base",
];
const TAGS_INLINE: &[&str] = &[
    "b", "strong", "i", "em", "code", "u",
    "underline", "s", "del", "strike", "a", "span", "br", "mark", "cite",
    "small", "abbr", "kbd", "samp", "var", "sub", "sup",
];

fn e_inline(tag: &str) -> bool {
    TAGS_INLINE.contains(&tag)
}

/// Caminho de imagem aceitável: relativo, sem esquema, sem `..`.
pub fn caminho_local(url: &str) -> bool {
    let u = url.trim();
    !u.is_empty()
        && !u.contains("://")
        && !u.starts_with("data:")
        && !u.starts_with('/')
        && !u.starts_with('\\')
        && !u.split(['/', '\\']).any(|p| p == "..")
        && !u.contains(':')
}

/// `href` que o Glacier abre no navegador do SO: só web e e-mail. Qualquer outro
/// esquema (`file:`, `javascript:`, um protocolo registrado por outro programa) é
/// recusado — o link vem de dado.
pub fn href_seguro(href: &str) -> bool {
    let h = href.trim().to_ascii_lowercase();
    h.starts_with("http://") || h.starts_with("https://") || h.starts_with("mailto:")
}

// ───────────────────────────────── os blocos ─────────────────────────────────

/// Constrói os blocos de uma lista de nós (os filhos de um `prose`).
pub fn blocos(filhos: &[No], o: Opcoes) -> Result<Vec<Bloco>, Erro> {
    let mut out = Vec::new();
    let mut pend: Vec<&No> = Vec::new();
    nivel_bloco(filhos, o, &mut out, &mut pend)?;
    flush(&mut pend, o, &mut out)?;
    Ok(out)
}

fn flush(pend: &mut Vec<&No>, o: Opcoes, out: &mut Vec<Bloco>) -> Result<(), Erro> {
    if pend.is_empty() {
        return Ok(());
    }
    let nos: Vec<No> = pend.drain(..).cloned().collect();
    let spans = spans_de(&nos, o)?;
    if !spans.is_empty() {
        out.push(Bloco::Paragrafo(spans));
    }
    Ok(())
}

fn nivel_bloco<'a>(
    filhos: &'a [No],
    o: Opcoes,
    out: &mut Vec<Bloco>,
    pend: &mut Vec<&'a No>,
) -> Result<(), Erro> {
    for no in filhos {
        match no {
            No::Texto(t, pos) => {
                if t.trim().is_empty() {
                    if !pend.is_empty() {
                        pend.push(no);
                    }
                } else if o.leniente {
                    pend.push(no);
                } else {
                    return Err(Erro::novo("texto solto dentro de prose", *pos)
                        .dica("o prose só contém blocos (p, h2, blockquote, ul, figure…): escreva prose { p(\"o texto\") }"));
                }
            }
            No::Elem { tag, filhos: fs, pos, .. } => {
                let tag = tag.as_str();
                if TAGS_DESCARTADAS.contains(&tag) {
                    continue;
                }
                if e_inline(tag) {
                    if o.leniente {
                        pend.push(no);
                        continue;
                    }
                    return Err(Erro::novo(format!("{tag} direto no prose"), *pos)
                        .dica("o prose só contém blocos; as tags de estilo vão dentro de um p"));
                }
                if o.leniente && TAGS_TRANSPARENTES.contains(&tag) {
                    flush(pend, o, out)?;
                    nivel_bloco(fs, o, out, pend)?;
                    flush(pend, o, out)?;
                    continue;
                }
                if !TAGS_BLOCO.contains(&tag) {
                    if o.leniente {
                        // Desconhecida: o texto dela fica, no fluxo.
                        pend.push(no);
                        continue;
                    }
                    return Err(Erro::novo(format!("{tag} direto no prose"), *pos).dica(
                        "dentro de um prose cabem p, h1–h6, blockquote, ul, ol, figure, img, pre e hr",
                    ));
                }
                flush(pend, o, out)?;
                bloco(no, o, out)?;
            }
        }
    }
    Ok(())
}

fn bloco(no: &No, o: Opcoes, out: &mut Vec<Bloco>) -> Result<(), Erro> {
    let No::Elem { tag, filhos, pos, .. } = no else { return Ok(()) };
    match tag.as_str() {
        "p" => {
            let s = spans_de(filhos, o)?;
            if !s.is_empty() {
                out.push(Bloco::Paragrafo(s));
            }
        }
        t @ ("h1" | "h2" | "h3" | "h4" | "h5" | "h6") => {
            let s = spans_de(&conteudo_de(no), o)?;
            if !s.is_empty() {
                out.push(Bloco::Titulo { nivel: t.as_bytes()[1] - b'0', spans: s });
            }
        }
        "hr" => out.push(Bloco::Regua),
        "pre" => {
            let mut t = String::new();
            texto_cru(filhos, &mut t);
            out.push(Bloco::Codigo(t.trim_matches('\n').to_string()));
        }
        "ul" | "ol" => {
            let mut itens = Vec::new();
            for f in filhos {
                match f {
                    No::Elem { tag, .. } if tag == "li" => {
                        let s = spans_de(&conteudo_de(f), o)?;
                        if !s.is_empty() {
                            itens.push(s);
                        }
                    }
                    No::Texto(t, _) if t.trim().is_empty() => {}
                    No::Elem { tag, .. } if o.leniente && TAGS_DESCARTADAS.contains(&tag.as_str()) => {}
                    other => {
                        if o.leniente {
                            continue;
                        }
                        let (nome, p) = match other {
                            No::Elem { tag, pos, .. } => (format!("{tag}"), *pos),
                            No::Texto(_, p) => ("texto solto".to_string(), *p),
                        };
                        return Err(Erro::novo(format!("{nome} dentro de {tag}"), p)
                            .dica("uma lista só contém li"));
                    }
                }
            }
            out.push(Bloco::Lista { ordenada: tag == "ol", itens });
        }
        "blockquote" => {
            let mut paragrafos = Vec::new();
            let mut autoria = Vec::new();
            let mut soltos: Vec<No> = Vec::new();
            let flush_soltos = |soltos: &mut Vec<No>, paragrafos: &mut Vec<Vec<TextSpan>>| -> Result<(), Erro> {
                if soltos.is_empty() {
                    return Ok(());
                }
                let s = spans_de(soltos, o)?;
                soltos.clear();
                if !s.is_empty() {
                    paragrafos.push(s);
                }
                Ok(())
            };
            for f in filhos {
                match f {
                    No::Elem { tag, filhos, .. } if tag == "p" => {
                        flush_soltos(&mut soltos, &mut paragrafos)?;
                        let s = spans_de(filhos, o)?;
                        if !s.is_empty() {
                            paragrafos.push(s);
                        }
                    }
                    No::Elem { tag, .. } if tag == "footer" || tag == "cite" => {
                        autoria = spans_de(&conteudo_de(f), o)?;
                    }
                    No::Texto(t, p) => {
                        if !t.trim().is_empty() && !o.leniente {
                            return Err(Erro::novo("texto solto dentro de blockquote", *p)
                                .dica("a citação contém p e, se quiser, um footer com a autoria"));
                        }
                        soltos.push(f.clone());
                    }
                    No::Elem { tag, pos, .. } => {
                        if !o.leniente {
                            return Err(Erro::novo(format!("{tag} dentro de blockquote"), *pos)
                                .dica("a citação só contém p e footer"));
                        }
                        soltos.push(f.clone());
                    }
                }
            }
            flush_soltos(&mut soltos, &mut paragrafos)?;
            out.push(Bloco::Citacao { paragrafos, autoria });
        }
        "img" => out.push(figura(no, &[], o)?),
        "figure" => {
            let mut img: Option<&No> = None;
            let mut legenda: Vec<No> = Vec::new();
            for f in filhos {
                match f {
                    No::Elem { tag, .. } if tag == "img" => img = Some(f),
                    No::Elem { tag, .. } if tag == "figcaption" => legenda = conteudo_de(f),
                    No::Texto(t, _) if t.trim().is_empty() => {}
                    No::Texto(_, p) | No::Elem { pos: p, .. } if !o.leniente => {
                        return Err(Erro::novo("só img e figcaption cabem num figure", *p));
                    }
                    _ => {}
                }
            }
            match img {
                Some(i) => out.push(figura(i, &legenda, o)?),
                None if o.leniente => {}
                None => {
                    return Err(Erro::novo("figure sem img", *pos)
                        .dica("figure { img(src = \"…\", alt = \"…\") figcaption(\"legenda\") }"));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn figura(img: &No, legenda: &[No], o: Opcoes) -> Result<Bloco, Erro> {
    let pos = if let No::Elem { pos, .. } = img { *pos } else { None };
    let src = img.attr(&["src", "source"]).unwrap_or_default().trim().to_string();
    if src.is_empty() && !o.leniente {
        return Err(Erro::novo("img sem src", pos));
    }
    Ok(Bloco::Figura {
        src,
        alt: img.attr(&["alt"]).unwrap_or_default().to_string(),
        legenda: spans_de(legenda, o)?,
    })
}

/// Os filhos de um elemento, com o atributo `content` na frente: é como o `.gv` escreve
/// o texto de `b("x")`, `h3("x")` ou `li("x")`.
fn conteudo_de(no: &No) -> Vec<No> {
    let No::Elem { filhos, .. } = no else { return Vec::new() };
    let mut v = Vec::with_capacity(filhos.len() + 1);
    if let Some(c) = no.attr(&["content", "text"]) {
        v.push(No::Texto(c.to_string(), None));
    }
    v.extend(filhos.iter().cloned());
    v
}

fn texto_cru(nos: &[No], out: &mut String) {
    for n in nos {
        match n {
            No::Texto(t, _) => out.push_str(t),
            No::Elem { filhos, .. } => texto_cru(filhos, out),
        }
    }
}

// ───────────────────────────────── os trechos ─────────────────────────────────

/// Colapsa espaço em branco como o HTML, **preservando uma borda**: entre `Olá ` e
/// `<b>mundo</b>` o espaço é do texto.
pub fn colapsar_espaco(raw: &str) -> String {
    let mut out = String::new();
    let mut pendente = false;
    for ch in raw.chars() {
        if ch == '\u{00A0}' {
            out.push(' ');
            pendente = false;
        } else if ch.is_whitespace() {
            pendente = true;
        } else {
            if pendente {
                out.push(' ');
                pendente = false;
            }
            out.push(ch);
        }
    }
    if pendente {
        out.push(' ');
    }
    out
}

fn spans_de(filhos: &[No], o: Opcoes) -> Result<Vec<TextSpan>, Erro> {
    let mut spans = Vec::new();
    coletar(filhos, &TextSpan::default(), o, &mut spans, 0)?;

    // Espaço sobrando: nas pontas e dobrado entre trechos vizinhos.
    let mut anterior_termina_em_espaco = true;
    for i in 0..spans.len() {
        if spans[i].content == "\n" {
            anterior_termina_em_espaco = true;
            continue;
        }
        if anterior_termina_em_espaco {
            let t = spans[i].content.trim_start().to_string();
            spans[i].content = t;
        }
        if !spans[i].content.is_empty() {
            anterior_termina_em_espaco = spans[i].content.ends_with(' ');
        }
    }
    if let Some(ultimo) = spans.iter_mut().rev().find(|s| !s.content.is_empty() && s.content != "\n") {
        ultimo.content = ultimo.content.trim_end().to_string();
    }
    spans.retain(|s| !s.content.is_empty());
    while spans.last().is_some_and(|s| s.content == "\n") {
        spans.pop();
    }
    Ok(spans)
}

fn coletar(
    filhos: &[No],
    base: &TextSpan,
    o: Opcoes,
    out: &mut Vec<TextSpan>,
    fundo: usize,
) -> Result<(), Erro> {
    if fundo > LIMITE_PROFUNDIDADE {
        return Err(Erro::novo("conteúdo aninhado demais", None));
    }
    for f in filhos {
        match f {
            No::Texto(t, _) => {
                let content = colapsar_espaco(t);
                if !content.is_empty() {
                    out.push(TextSpan { content, ..base.clone() });
                }
            }
            No::Elem { tag, pos, .. } => {
                let tag = tag.as_str();
                if TAGS_DESCARTADAS.contains(&tag) {
                    continue;
                }
                let mut estilo = base.clone();
                match tag {
                    "br" => {
                        out.push(TextSpan { content: "\n".into(), ..base.clone() });
                        continue;
                    }
                    "b" | "strong" => estilo.bold = true,
                    "i" | "em" | "cite" => estilo.italic = true,
                    "code" | "kbd" | "samp" => estilo.code = true,
                    "u" | "underline" | "ins" => estilo.underline = true,
                    "s" | "del" | "strike" => estilo.strike = true,
                    "mark" => estilo.bold = true,
                    "a" => {
                        let link = ler_link(f, o, *pos)?;
                        estilo.link = link;
                    }
                    "span" => {
                        estilo.bold |= f.attr(&["bold"]).is_some_and(|v| v == "true" || v == "1");
                    }
                    "small" | "abbr" | "var" | "sub" | "sup" => {}
                    other => {
                        if !o.leniente {
                            return Err(Erro::novo(format!("{other} dentro de um parágrafo"), *pos)
                                .dica("um parágrafo só tem texto e as tags b, i, code, u, s, span e a (um p não entra em outro p)"));
                        }
                    }
                }
                if let Some(c) = f.attr(&["color"]).filter(|c| cor_valida(c)) {
                    estilo.color = Some(c.trim().to_string());
                }
                coletar(&conteudo_de(f), &estilo, o, out, fundo + 1)?;
            }
        }
    }
    Ok(())
}

fn cor_valida(c: &str) -> bool {
    let c = c.trim();
    c.starts_with('#') && matches!(c.len(), 4 | 7 | 9) && c[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

fn ler_link(a: &No, o: Opcoes, pos: Option<usize>) -> Result<Option<SpanLink>, Erro> {
    let href = a.attr(&["href", "url"]).map(str::to_string);
    let navigate = a.attr(&["navigate_to", "navigate"]);
    let on_click = a.attr(&["on_click", "onclick"]);
    if o.confiavel {
        return Ok(Some(SpanLink {
            href,
            navigate: navigate.map(str::to_string),
            on_click: on_click.map(str::to_string),
        }));
    }
    // Dinâmico: nunca uma ação do app.
    if let Some(attr) = navigate.map(|_| "navigate_to").or(on_click.map(|_| "on_click")) {
        if !o.leniente {
            return Err(Erro::novo(format!("`{attr}` não é permitido em conteúdo dinâmico"), pos)
                .dica("um texto vindo de dado só pode ter links `href` (http, https ou mailto): uma ação do app não pode vir de um registro"));
        }
    }
    Ok(match href {
        Some(h) if href_seguro(&h) => Some(SpanLink { href: Some(h.trim().to_string()), navigate: None, on_click: None }),
        // Esquema recusado (ou sem href): o texto fica, sem ser link.
        _ => None,
    })
}

// ─────────────────────────────────── HTML ────────────────────────────────────

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];
const CRU: &[&str] = &["script", "style", "textarea", "title"];

fn decodifica(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut resto = s;
    while let Some(i) = resto.find('&') {
        out.push_str(&resto[..i]);
        resto = &resto[i..];
        let fim = resto.find(';').filter(|f| *f <= 10);
        match fim {
            Some(f) => {
                let nome = &resto[1..f];
                match entidade(nome) {
                    Some(c) => {
                        out.push(c);
                        resto = &resto[f + 1..];
                    }
                    None => {
                        out.push('&');
                        resto = &resto[1..];
                    }
                }
            }
            None => {
                out.push('&');
                resto = &resto[1..];
            }
        }
    }
    out.push_str(resto);
    out
}

fn entidade(nome: &str) -> Option<char> {
    if let Some(num) = nome.strip_prefix('#') {
        let cp = if let Some(h) = num.strip_prefix(['x', 'X']) {
            u32::from_str_radix(h, 16).ok()?
        } else {
            num.parse().ok()?
        };
        return char::from_u32(cp).filter(|c| *c != '\0');
    }
    Some(match nome {
        "amp" => '&', "lt" => '<', "gt" => '>', "quot" => '"', "apos" => '\'',
        "nbsp" => '\u{a0}', "mdash" => '—', "ndash" => '–', "hellip" => '…',
        "laquo" => '«', "raquo" => '»', "ldquo" => '“', "rdquo" => '”',
        "lsquo" => '‘', "rsquo" => '’', "copy" => '©', "reg" => '®', "trade" => '™',
        "euro" => '€', "middot" => '·', "bull" => '•', "deg" => '°', "sect" => '§',
        "aacute" => 'á', "eacute" => 'é', "iacute" => 'í', "oacute" => 'ó', "uacute" => 'ú',
        "agrave" => 'à', "egrave" => 'è', "acirc" => 'â', "ecirc" => 'ê', "ocirc" => 'ô',
        "atilde" => 'ã', "otilde" => 'õ', "ccedil" => 'ç', "uuml" => 'ü', "ntilde" => 'ñ',
        "Aacute" => 'Á', "Eacute" => 'É', "Iacute" => 'Í', "Oacute" => 'Ó', "Uacute" => 'Ú',
        "Agrave" => 'À', "Acirc" => 'Â', "Ecirc" => 'Ê', "Ocirc" => 'Ô', "Atilde" => 'Ã',
        "Otilde" => 'Õ', "Ccedil" => 'Ç',
        _ => return None,
    })
}

/// Um parser de HTML **tolerante**, só do que um texto de leitura usa: tags, atributos,
/// comentários, entidades, elementos vazios, `<p>` e `<li>` que fecham sozinhos e
/// fechamentos fora de ordem. Não é o algoritmo do HTML5 — o `prose` descarta o que
/// não reconhece de qualquer jeito.
pub fn parse_html(src: &str) -> Vec<No> {
    struct Aberto {
        tag: String,
        attrs: Vec<(String, String)>,
        filhos: Vec<No>,
        pos: usize,
    }
    let b = src.as_bytes();
    let mut i = 0;
    let mut raiz: Vec<No> = Vec::new();
    let mut pilha: Vec<Aberto> = Vec::new();

    fn fecha(pilha: &mut Vec<Aberto>, raiz: &mut Vec<No>) {
        if let Some(a) = pilha.pop() {
            let no = No::Elem { tag: a.tag, attrs: a.attrs, filhos: a.filhos, pos: Some(a.pos) };
            match pilha.last_mut() {
                Some(pai) => pai.filhos.push(no),
                None => raiz.push(no),
            }
        }
    }
    fn empurra(pilha: &mut [Aberto], raiz: &mut Vec<No>, no: No) {
        match pilha.last_mut() {
            Some(pai) => pai.filhos.push(no),
            None => raiz.push(no),
        }
    }
    let ci_starts = |i: usize, p: &str| b.get(i..i + p.len()).is_some_and(|x| x.eq_ignore_ascii_case(p.as_bytes()));

    while i < b.len() {
        if b[i] != b'<' {
            let fim = src[i..].find('<').map_or(b.len(), |f| i + f);
            empurra(&mut pilha, &mut raiz, No::Texto(decodifica(&src[i..fim]), Some(i)));
            i = fim;
            continue;
        }
        if src[i..].starts_with("<!--") {
            i = src[i..].find("-->").map_or(b.len(), |f| i + f + 3);
            continue;
        }
        if src[i..].starts_with("<!") || src[i..].starts_with("<?") {
            i = src[i..].find('>').map_or(b.len(), |f| i + f + 1);
            continue;
        }
        if src[i..].starts_with("</") {
            let fim = src[i..].find('>').map_or(b.len(), |f| i + f);
            let nome = src[i + 2..fim].trim().to_ascii_lowercase();
            if let Some(idx) = pilha.iter().rposition(|a| a.tag == nome) {
                while pilha.len() > idx {
                    fecha(&mut pilha, &mut raiz);
                }
            }
            i = (fim + 1).min(b.len());
            continue;
        }
        // Abertura: `<` seguido de letra; senão é um `<` solto, texto.
        if !b.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic()) {
            empurra(&mut pilha, &mut raiz, No::Texto("<".into(), Some(i)));
            i += 1;
            continue;
        }
        let inicio = i;
        let mut j = i + 1;
        while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-' || b[j] == b':') {
            j += 1;
        }
        let tag = src[i + 1..j].to_ascii_lowercase();
        let mut attrs: Vec<(String, String)> = Vec::new();
        let mut auto_fechada = false;
        loop {
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= b.len() {
                break;
            }
            if b[j] == b'>' {
                j += 1;
                break;
            }
            if b[j] == b'/' {
                auto_fechada = true;
                j += 1;
                continue;
            }
            let ini = j;
            while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'=' | b'>' | b'/') {
                j += 1;
            }
            let nome = src[ini..j].to_ascii_lowercase();
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            let mut valor = String::new();
            if b.get(j) == Some(&b'=') {
                j += 1;
                while j < b.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if let Some(&q) = b.get(j).filter(|c| **c == b'"' || **c == b'\'') {
                    let vi = j + 1;
                    let vf = src[vi..].find(q as char).map_or(b.len(), |f| vi + f);
                    valor = decodifica(&src[vi..vf]);
                    j = (vf + 1).min(b.len());
                } else {
                    let vi = j;
                    while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' {
                        j += 1;
                    }
                    valor = decodifica(&src[vi..j]);
                }
            }
            if !nome.is_empty() {
                attrs.push((nome, valor));
            }
        }
        i = j;

        // Elementos cujo corpo é texto cru: pula até o fechamento e descarta.
        if CRU.contains(&tag.as_str()) && !auto_fechada {
            let fecho = format!("</{tag}");
            let mut k = i;
            let fim = loop {
                match src[k..].find("</") {
                    Some(f) if ci_starts(k + f, &fecho) => break k + f,
                    Some(f) => k += f + 2,
                    None => break b.len(),
                }
            };
            empurra(&mut pilha, &mut raiz, No::Elem { tag, attrs, filhos: vec![], pos: Some(inicio) });
            i = src[fim..].find('>').map_or(b.len(), |f| fim + f + 1);
            continue;
        }

        // `<p>` e `<li>` fecham o anterior do mesmo tipo; blocos fecham um `<p>` aberto.
        let fecha_p = matches!(
            tag.as_str(),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "blockquote" | "pre"
                | "figure" | "hr" | "div" | "table"
        );
        if fecha_p {
            if let Some(idx) = pilha.iter().rposition(|a| a.tag == "p") {
                while pilha.len() > idx {
                    fecha(&mut pilha, &mut raiz);
                }
            }
        }
        if tag == "li" {
            if let Some(idx) = pilha.iter().rposition(|a| a.tag == "li") {
                let dentro_de_lista = pilha[idx..].iter().all(|a| a.tag != "ul" && a.tag != "ol");
                if dentro_de_lista {
                    while pilha.len() > idx {
                        fecha(&mut pilha, &mut raiz);
                    }
                }
            }
        }

        if VOID.contains(&tag.as_str()) || auto_fechada {
            empurra(&mut pilha, &mut raiz, No::Elem { tag, attrs, filhos: vec![], pos: Some(inicio) });
        } else if pilha.len() >= LIMITE_PROFUNDIDADE {
            // Fundo demais: a tag vira irmã em vez de mais um nível.
            empurra(&mut pilha, &mut raiz, No::Elem { tag, attrs, filhos: vec![], pos: Some(inicio) });
        } else {
            pilha.push(Aberto { tag, attrs, filhos: vec![], pos: inicio });
        }
    }
    while !pilha.is_empty() {
        fecha(&mut pilha, &mut raiz);
    }
    raiz
}

// ───────────────────────────── o conteúdo dinâmico ─────────────────────────────

/// Lê o texto de um `prose(syntax = "…", content = "…")`. Nunca falha: um conteúdo ruim
/// vira um [`Bloco::Erro`] visível na tela, e o resto da tela segue de pé.
pub fn de_conteudo(sintaxe: Sintaxe, conteudo: &str) -> Vec<Bloco> {
    if conteudo.len() > LIMITE_BYTES {
        return vec![Bloco::Erro(format!(
            "prose: conteúdo de {} bytes passa do limite de {LIMITE_BYTES}",
            conteudo.len()
        ))];
    }
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<(Sintaxe, String), Vec<Bloco>>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    let chave = (sintaxe, conteudo.to_string());
    if let Some(b) = CACHE.with(|c| c.borrow().get(&chave).cloned()) {
        return b;
    }
    let resultado = ler(sintaxe, conteudo);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= 64 {
            c.clear();
        }
        c.insert(chave, resultado.clone());
    });
    resultado
}

fn ler(sintaxe: Sintaxe, conteudo: &str) -> Vec<Bloco> {
    let erro = |msg: String| vec![Bloco::Erro(format!("prose ({}): {msg}", nome(sintaxe)))];
    let o_estrito = Opcoes { confiavel: false, leniente: false };
    match sintaxe {
        Sintaxe::Markdown => vec![Bloco::Markdown(conteudo.to_string())],
        Sintaxe::Html => {
            let nos = parse_html(conteudo);
            match blocos(&nos, Opcoes { confiavel: false, leniente: true }) {
                Ok(b) => b,
                Err(e) => erro(e.msg),
            }
        }
        Sintaxe::Gv => {
            let xml = match crate::gv::desugar_literal(conteudo) {
                Ok(x) => x,
                Err(d) => return erro(format!("linha {}: {}", d.line, d.message)),
            };
            let envolto = format!("<prose>{xml}</prose>");
            let doc = match roxmltree::Document::parse(&envolto) {
                Ok(d) => d,
                Err(e) => return erro(format!("XML inválido — {e}")),
            };
            let nos = de_xml(doc.root_element());
            match blocos(&nos, o_estrito) {
                Ok(b) => b,
                Err(e) => {
                    let linha = e.pos.map(|p| envolto[..p.min(envolto.len())].matches('\n').count() + 1);
                    let onde = linha.map(|l| format!("linha {l}: ")).unwrap_or_default();
                    let dica = e.dica.map(|d| format!(" ({d})")).unwrap_or_default();
                    erro(format!("{onde}{}{dica}", e.msg))
                }
            }
        }
    }
}

fn nome(s: Sintaxe) -> &'static str {
    match s {
        Sintaxe::Gv => "gv",
        Sintaxe::Html => "html",
        Sintaxe::Markdown => "markdown",
    }
}
