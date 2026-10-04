//! Traduz `.gv` → `.gvb`, mantendo os comentários, e CONFERE cada tradução:
//! o `.gvb` gerado é dessugarado e parseado de volta, e a árvore tem de sair
//! igual à do `.gv` original (ignorando o contador global de `node_id` e a
//! ordem dos campos de um mapa).
//!
//!     cargo run --example gvb_convert -- <saida> <arquivo.gv>...
//!
//! Cada `examples/X/Y.gv` vira `<saida>/X/Y.gvb`. O que o conversor não sabe
//! traduzir (texto misturado com elementos, por exemplo) é listado e pulado — a
//! tradução fiel ou nenhuma, nunca uma que mude a tela.
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use glacier_ui::parser::UiNode;

const W: usize = 80;

// ───────────────────────── XML mínimo, com comentários ─────────────────────────

enum X {
    Elem {
        tag: String,
        attrs: Vec<(String, String)>,
        kids: Vec<X>,
    },
    Text(String),
    Comment(String),
    /// Corpo cru de um `<script>`/`<style>`.
    Raw(String),
}

fn decode(s: &str) -> String {
    let mut o = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        o.push_str(&rest[..i]);
        rest = &rest[i..];
        let Some(end) = rest.find(';') else { break };
        let ent = &rest[1..end];
        match ent {
            "lt" => o.push('<'),
            "gt" => o.push('>'),
            "amp" => o.push('&'),
            "quot" => o.push('"'),
            "apos" => o.push('\''),
            "nbsp" => o.push('\u{A0}'),
            _ => {
                let n = ent
                    .strip_prefix("#x")
                    .and_then(|h| u32::from_str_radix(h, 16).ok())
                    .or_else(|| ent.strip_prefix('#').and_then(|d| d.parse().ok()));
                match n.and_then(char::from_u32) {
                    Some(c) => o.push(c),
                    None => o.push_str(&rest[..=end]),
                }
            }
        }
        rest = &rest[end + 1..];
    }
    o.push_str(rest);
    o
}

fn parse_xml(src: &str) -> Result<Vec<X>, String> {
    // Pilha de (tag, attrs, filhos) abertos; o fundo é o nível do arquivo.
    let mut stack: Vec<(String, Vec<(String, String)>, Vec<X>)> =
        vec![(String::new(), vec![], vec![])];
    let b = src.as_bytes();
    let mut i = 0;
    while i < src.len() {
        if b[i] != b'<' {
            let j = src[i..].find('<').map_or(src.len(), |k| i + k);
            stack.last_mut().unwrap().2.push(X::Text(decode(&src[i..j])));
            i = j;
            continue;
        }
        let rest = &src[i..];
        if rest.starts_with("<!--") {
            let e = rest.find("-->").ok_or("comentário sem fim")?;
            stack
                .last_mut()
                .unwrap()
                .2
                .push(X::Comment(rest[4..e].to_string()));
            i += e + 3;
        } else if rest.starts_with("<![CDATA[") {
            let e = rest.find("]]>").ok_or("CDATA sem fim")?;
            stack.last_mut().unwrap().2.push(X::Raw(rest[9..e].to_string()));
            i += e + 3;
        } else if rest.starts_with("<?") {
            i += rest.find("?>").ok_or("<? sem fim")? + 2;
        } else if let Some(r) = rest.strip_prefix("</") {
            let e = r.find('>').ok_or("</ sem >")?;
            let name = r[..e].trim();
            let (tag, attrs, kids) = stack.pop().ok_or("fecha demais")?;
            if tag != name {
                return Err(format!("</{name}> fecha <{tag}>"));
            }
            stack
                .last_mut()
                .ok_or("fecha demais")?
                .2
                .push(X::Elem { tag, attrs, kids });
            i += 2 + e + 1;
        } else {
            // abertura: nome, atributos, `>` ou `/>`
            let mut j = i + 1;
            while j < src.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' && b[j] != b'/' {
                j += 1;
            }
            let tag = src[i + 1..j].to_string();
            let mut attrs = Vec::new();
            let self_close;
            loop {
                while j < src.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j >= src.len() {
                    return Err(format!("<{tag}> sem fim"));
                }
                if b[j] == b'>' {
                    self_close = false;
                    j += 1;
                    break;
                }
                if src[j..].starts_with("/>") {
                    self_close = true;
                    j += 2;
                    break;
                }
                let s = j;
                while j < src.len()
                    && !b[j].is_ascii_whitespace()
                    && b[j] != b'='
                    && b[j] != b'>'
                    && b[j] != b'/'
                {
                    j += 1;
                }
                let name = src[s..j].to_string();
                while j < src.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < src.len() && b[j] == b'=' {
                    j += 1;
                    while j < src.len() && b[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    let q = b[j];
                    if q != b'"' && q != b'\'' {
                        return Err(format!("atributo `{name}` sem aspas"));
                    }
                    let e = src[j + 1..]
                        .find(q as char)
                        .ok_or("atributo sem fim")?;
                    // normalização de valor de atributo do XML: \n \t \r → espaço
                    let raw: String = src[j + 1..j + 1 + e]
                        .chars()
                        .map(|c| if matches!(c, '\n' | '\t' | '\r') { ' ' } else { c })
                        .collect();
                    attrs.push((name, decode(&raw)));
                    j += e + 2;
                } else {
                    attrs.push((name, String::new())); // `else` sem valor
                }
            }
            let lower = tag.to_ascii_lowercase();
            if self_close {
                stack
                    .last_mut()
                    .unwrap()
                    .2
                    .push(X::Elem { tag, attrs, kids: vec![] });
            } else if lower == "script" || lower == "style" {
                let close = format!("</{lower}>");
                let e = src[j..]
                    .to_ascii_lowercase()
                    .find(&close)
                    .ok_or("script/style sem fim")?;
                let body = src[j..j + e].to_string();
                stack.last_mut().unwrap().2.push(X::Elem {
                    tag,
                    attrs,
                    kids: vec![X::Raw(body)],
                });
                j += e + close.len();
            } else {
                stack.push((tag, attrs, vec![]));
            }
            i = j;
        }
    }
    if stack.len() != 1 {
        return Err("tag aberta no fim do arquivo".into());
    }
    Ok(stack.pop().unwrap().2)
}

// ───────────────────────────── valores ─────────────────────────────

fn is_ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn is_path(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        && !s.starts_with('.')
        && !s.ends_with('.')
}

/// `{x}` → `@x`; `{x|d}` → `@{x|d}`; `@` → `@@`.
fn interp(v: &str) -> String {
    let cs: Vec<char> = v.chars().collect();
    let mut o = String::new();
    let mut i = 0;
    while i < cs.len() {
        match cs[i] {
            '@' => {
                o.push_str("@@");
                i += 1;
            }
            // `{{marcador}}` — o que o scaffold do CLI substitui; passa intacto.
            '{' if cs.get(i + 1) == Some(&'{') => {
                match (i + 2..cs.len().saturating_sub(1)).find(|&k| cs[k] == '}' && cs[k + 1] == '}') {
                    Some(k) => {
                        o.extend(&cs[i..k + 2]);
                        i = k + 2;
                    }
                    None => {
                        o.push('{');
                        i += 1;
                    }
                }
            }
            '{' => match cs[i + 1..].iter().position(|&c| c == '}') {
                Some(n) => {
                    let inner: String = cs[i + 1..i + 1 + n].iter().collect();
                    let next = cs.get(i + n + 2).copied();
                    let after = cs.get(i + n + 3).copied();
                    let glued = next.is_some_and(|c| c.is_alphanumeric() || c == '_')
                        || (next == Some('.')
                            && after.is_some_and(|c| c.is_alphanumeric() || c == '_'));
                    if is_path(&inner) && !glued {
                        let _ = write!(o, "@{inner}");
                    } else {
                        let _ = write!(o, "@{{{inner}}}");
                    }
                    i += n + 2;
                }
                None => {
                    o.push('{');
                    i += 1;
                }
            },
            c => {
                o.push(c);
                i += 1;
            }
        }
    }
    o
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn bare_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '+' | '@' | ':' | '#' | '%'))
        && !s.contains("//")
}

/// O valor de um atributo, no formato mais curto que a leitura aceita.
fn val(v: &str) -> String {
    // Uma referência a outro template (`from="x.gv"`) aponta para a tradução.
    let v = &if let Some(r) = v.strip_suffix(".gv") {
        match r.strip_prefix("examples/") {
            Some(r) => format!("gvb/{r}.gvb"),
            None => format!("{r}.gvb"),
        }
    } else {
        v.to_string()
    };
    let s = interp(v);
    if bare_ok(&s) { s } else { quote(&s) }
}

// ───────────────────────────── layout ─────────────────────────────

struct Conv {
    errs: Vec<String>,
}

fn pad(n: usize) -> String {
    " ".repeat(n)
}

fn pack(items: &[String], indent: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for it in items {
        if !cur.is_empty() && indent + cur.len() + 1 + it.len() > W {
            lines.push(format!("{}{}", pad(indent), cur));
            cur.clear();
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(it);
    }
    if !cur.is_empty() {
        lines.push(format!("{}{}", pad(indent), cur));
    }
    lines
}

fn ws_only(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_whitespace())
}

/// O texto de um elemento já em `.gvb`: `"curto"` ou um `"""` quebrado a 80.
fn text_lit(t: &str, indent: usize) -> Result<String, String> {
    let words: Vec<&str> = t
        .split(|c: char| c.is_ascii_whitespace())
        .filter(|w| !w.is_empty())
        .collect();
    let body = interp(&words.join(" "));
    if body.is_empty() {
        return Ok(String::new());
    }
    let plain = !body.contains('"') && !body.contains('\\');
    if plain && indent + body.len() + 6 <= W {
        return Ok(format!("\"{body}\""));
    }
    if body.contains("\"\"\"") || body.ends_with('"') {
        // Aspas no fim (ou três seguidas) não cabem num `"""`: vai entre aspas
        // simples, com `\"` escapado, e a quebra de linha continua livre dentro.
        let esc = |w: &str| w.replace('\\', "\\\\").replace('"', "\\\"");
        let inner = pad(indent + 2);
        let mut out = String::from("\"");
        let mut line = String::new();
        for w in body.split(' ') {
            if !line.is_empty() && indent + 2 + line.chars().count() + 1 + w.chars().count() > W {
                out.push_str(&line);
                out.push('\n');
                out.push_str(&inner);
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(&esc(w));
        }
        out.push_str(&line);
        out.push('"');
        return Ok(out);
    }
    let inner = pad(indent + 2);
    let mut out = String::from("\"\"\"\n");
    let mut line = String::new();
    for w in interp(&words.join(" ")).split(' ') {
        if !line.is_empty() && indent + 2 + line.chars().count() + 1 + w.chars().count() > W {
            let _ = writeln!(out, "{inner}{line}");
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(w);
    }
    let _ = writeln!(out, "{inner}{line}");
    let _ = write!(out, "{}\"\"\"", pad(indent));
    Ok(out)
}

fn comment_lines(body: &str, indent: usize) -> Vec<String> {
    let ls: Vec<&str> = body.lines().collect();
    let common = ls
        .iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut out = Vec::new();
    for (i, l) in ls.iter().enumerate() {
        let t = if i == 0 {
            l.trim()
        } else {
            let cut = common.min(l.len() - l.trim_start().len());
            l[cut..].trim_end()
        };
        if i == 0 && t.is_empty() {
            continue;
        }
        if t.is_empty() {
            out.push(format!("{}//", pad(indent)));
        } else {
            out.push(format!("{}// {}", pad(indent), t));
        }
    }
    while out.last().is_some_and(|l| l.trim_end() == format!("{}//", pad(indent)).trim_end()) {
        out.pop();
    }
    out
}

/// Como um elemento cabe numa condição/`each`/seletor.
enum Head {
    If,
    ElseIf,
    Else,
}

impl Conv {
    fn err(&mut self, m: impl Into<String>) {
        self.errs.push(m.into());
    }

    /// A tag e seus atributos, todos por extenso (`class:` e `id:` inclusive —
    /// o `.gvb` não tem `tag.classe`).
    fn selector(&mut self, tag: &str, attrs: &[(String, String)]) -> (String, Vec<String>) {
        let mut rest: Vec<String> = Vec::new();
        for (k, v) in attrs {
            if !is_ident(k) {
                self.err(format!("atributo `{k}` com nome que a leitura não aceita"));
            }
            rest.push(format!("{k}: {}", val(v)));
        }
        (tag.to_string(), rest)
    }

    /// Classifica `<template if=…>`, `<if>`, `<else>`…
    fn classify(
        &mut self,
        tag: &str,
        attrs: &[(String, String)],
    ) -> Option<(Head, String, Vec<String>)> {
        let get = |n: &str| attrs.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
        let others = |skip: &[&str], this: &mut Self| -> Vec<String> {
            attrs
                .iter()
                .filter(|(k, _)| !skip.contains(&k.as_str()))
                .map(|(k, v)| {
                    if !is_ident(k) {
                        this.err(format!("atributo `{k}` inválido"));
                    }
                    format!("{k}: {}", val(v))
                })
                .collect()
        };
        if tag == "else" {
            let o = others(&[], self);
            return Some((Head::Else, "else".into(), o));
        }
        if tag != "template" && tag != "if" {
            return None;
        }
        if tag == "template" && get("else").is_some() && get("if").is_none() && get("else_if").is_none() {
            let o = others(&["else"], self);
            return Some((Head::Else, "else".into(), o));
        }
        let (key, head) = match tag {
            "if" if get("cond").is_some() => ("cond", Head::If),
            "template" if get("if").is_some() => ("if", Head::If),
            "template" if get("else_if").is_some() => ("else_if", Head::ElseIf),
            _ => return None,
        };
        let k = get(key)?;
        let name = k.strip_prefix('{')?.strip_suffix('}')?;
        if !is_path(name) {
            return None;
        }
        let ops: Vec<&str> = ["equals", "not_equals", "contains", "empty"]
            .into_iter()
            .filter(|o| get(o).is_some())
            .collect();
        if ops.len() > 1 {
            return None;
        }
        let kw = if matches!(head, Head::If) { "if" } else { "else if" };
        let cond = match ops.first().copied() {
            None => format!("{kw} @{name}"),
            Some("equals") => format!("{kw} @{name} == {}", quote(&interp(&get("equals")?))),
            Some("not_equals") => {
                format!("{kw} @{name} != {}", quote(&interp(&get("not_equals")?)))
            }
            Some("contains") => format!("{kw} {} in @{name}", quote(&interp(&get("contains")?))),
            Some("empty") => {
                if get("empty")? != "true" {
                    return None;
                }
                format!("{kw} @{name} is empty")
            }
            _ => return None,
        };
        let mut skip = vec![key];
        skip.extend(ops.iter().copied());
        let o = others(&skip, self);
        Some((head, cond, o))
    }

    fn kids(&mut self, kids: &[X], indent: usize, out: &mut Vec<String>, first_in_block: bool) {
        let mut prev_cond = false; // o irmão anterior foi um ramo if/else if
        let mut comment_between = false;
        let mut prev_multi = false;
        let mut prev_was_comment = false;
        let mut any = !first_in_block;
        let _ = &mut any;
        for k in kids {
            match k {
                X::Text(t) if ws_only(t) => {}
                X::Text(_) => self.err("texto misturado com elementos"),
                X::Raw(_) => self.err("corpo cru fora de script/style"),
                X::Comment(c) => {
                    let ls = comment_lines(c, indent);
                    if ls.is_empty() {
                        continue;
                    }
                    if !out.is_empty() && !prev_was_comment && !out.last().unwrap().is_empty() {
                        // comentário abre um parágrafo novo, salvo entre ramos
                        if !prev_cond {
                            out.push(String::new());
                        } else {
                            out.push(String::new());
                        }
                    }
                    out.extend(ls);
                    prev_was_comment = true;
                    if prev_cond {
                        comment_between = true;
                    }
                }
                X::Elem { tag, attrs, kids: ek } => {
                    let class = self.classify(tag, attrs);
                    let mut lines = match class {
                        Some((head, cond, rest)) => {
                            let hd = (head, cond, rest);
                            self.render_cond(hd, ek, indent)
                        }
                        None => self.render_elem(tag, attrs, ek, indent),
                    };
                    let is_branch = self.classify(tag, attrs).map(|c| c.0);
                    let joined = matches!(is_branch, Some(Head::ElseIf | Head::Else))
                        && prev_cond
                        && !comment_between
                        && out.last().is_some_and(|l| l.trim() == "}");
                    if joined {
                        let last = out.pop().unwrap();
                        let first = lines.remove(0);
                        out.push(format!("{last} {}", first.trim_start()));
                    } else {
                        let multi = lines.len() > 1;
                        if !out.is_empty()
                            && !prev_was_comment
                            && (multi || prev_multi)
                            && !out.last().unwrap().is_empty()
                        {
                            out.push(String::new());
                        }
                        let first = lines.remove(0);
                        out.push(first);
                        prev_multi = multi;
                    }
                    if joined {
                        prev_multi = true;
                    }
                    out.extend(lines);
                    prev_cond = matches!(is_branch, Some(Head::If | Head::ElseIf));
                    comment_between = false;
                    prev_was_comment = false;
                }
            }
        }
    }

    fn render_cond(
        &mut self,
        (head, cond, rest): (Head, String, Vec<String>),
        ek: &[X],
        indent: usize,
    ) -> Vec<String> {
        let _ = head;
        let mut lines = Vec::new();
        let hdr = format!("{}{} {{", pad(indent), cond);
        lines.push(hdr);
        lines.extend(pack(&rest, indent + 2));
        let mut body = Vec::new();
        self.kids(ek, indent + 2, &mut body, true);
        if !rest.is_empty() && !body.is_empty() {
            lines.push(String::new());
        }
        lines.extend(body);
        lines.push(format!("{}}}", pad(indent)));
        if lines.len() == 3 && lines[1].trim().is_empty() {
            lines.remove(1);
        }
        // `if @x { }` — bloco vazio numa linha
        if lines.len() == 2 {
            return vec![format!("{}{} {{ }}", pad(indent), cond)];
        }
        lines
    }

    fn render_elem(
        &mut self,
        tag: &str,
        attrs: &[(String, String)],
        ek: &[X],
        indent: usize,
    ) -> Vec<String> {
        // foreach → each
        let mut attrs_v: Vec<(String, String)> = attrs.to_vec();
        let mut each: Option<String> = None;
        if tag == "foreach" {
            let items = attrs.iter().find(|(k, _)| k == "items").map(|(_, v)| v.clone());
            let var = attrs.iter().find(|(k, _)| k == "var").map(|(_, v)| v.clone());
            if let (Some(i), Some(v)) = (items, var)
                && is_path(&i)
                && is_ident(&v)
            {
                each = Some(format!("each @{i} as {v}"));
                attrs_v.retain(|(k, _)| k != "items" && k != "var");
            }
        }
        let (head, rest) = match &each {
            Some(h) => {
                let mut r = Vec::new();
                for (k, v) in &attrs_v {
                    if !is_ident(k) {
                        self.err(format!("atributo `{k}` inválido"));
                    }
                    r.push(format!("{k}: {}", val(v)));
                }
                (h.clone(), r)
            }
            None => self.selector(tag, &attrs_v),
        };
        if matches!(tag, "if" | "else" | "each") && each.is_none() {
            self.err(format!("tag <{tag}> que a leitura toma por palavra-chave"));
        }

        // o conteúdo: texto, corpo cru ou filhos
        let has_elems = ek
            .iter()
            .any(|k| matches!(k, X::Elem { .. } | X::Comment(_)));
        let raw: Option<&String> = ek.iter().find_map(|k| match k {
            X::Raw(r) => Some(r),
            _ => None,
        });
        let text: String = ek
            .iter()
            .filter_map(|k| match k {
                X::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        let has_text = !ws_only(&text);
        if has_text && has_elems {
            self.err(format!("<{tag}> mistura texto e elementos"));
        }

        let ind = pad(indent);
        let mut lit = String::new();
        if let Some(r) = raw {
            if r.contains("\"\"\"") || r.trim_end_matches([' ', '\t', '\n']).ends_with('"') && !r.ends_with(['\n', ' ']) {
                self.err("corpo cru com `\"\"\"` ou terminado em aspas");
            }
            if !r.is_empty() {
                lit = format!("\"\"\"{r}\"\"\"");
            }
        } else if has_text {
            match text_lit(&text, indent) {
                Ok(l) => lit = l,
                Err(e) => self.err(e),
            }
        }

        let mut lines: Vec<String> = Vec::new();
        let kids_present = has_elems;
        let mut first = format!("{ind}{head}");
        if !lit.is_empty() {
            first.push(' ');
            first.push_str(&lit);
        }
        if !kids_present {
            if rest.is_empty() {
                lines.extend(first.split('\n').map(str::to_string));
                return lines;
            }
            // cabe numa linha?
            let one = format!("{first} {{ {} }}", rest.join(" "));
            if !one.contains('\n') && one.len() <= W {
                return vec![one];
            }
            let mut f: Vec<String> = first.split('\n').map(str::to_string).collect();
            let last = f.pop().unwrap();
            lines.extend(f);
            lines.push(format!("{last} {{"));
            lines.extend(pack(&rest, indent + 2));
            lines.push(format!("{ind}}}"));
            return lines;
        }
        lines.push(format!("{first} {{"));
        lines.extend(pack(&rest, indent + 2));
        let mut body = Vec::new();
        self.kids(ek, indent + 2, &mut body, true);
        if !rest.is_empty() && !body.is_empty() && !body[0].is_empty() {
            lines.push(String::new());
        }
        lines.extend(body);
        lines.push(format!("{ind}}}"));
        lines
    }
}

fn convert(src: &str) -> Result<String, Vec<String>> {
    let xml = parse_xml(src).map_err(|e| vec![e])?;
    let mut c = Conv { errs: vec![] };
    let mut out = Vec::new();
    c.kids(&xml, 0, &mut out, true);
    if !c.errs.is_empty() {
        c.errs.dedup();
        return Err(c.errs);
    }
    while out.first().is_some_and(|l| l.is_empty()) {
        out.remove(0);
    }
    Ok(out.join("\n") + "\n")
}

// ───────────────────────────── conferência ─────────────────────────────

/// Mesmo caminho que `parse_markup` (script fora, diretivas nuas) → árvore.
fn tree(xml: &str, original: &str) -> Result<(String, Option<String>), String> {
    let (markup, script) = glacier_ui::eval::strip_script(xml);
    let markup = glacier_ui::eval::normalize_bare_directives(&markup);
    let t = UiNode::parse_xml_with_source(&markup, original, None).map_err(|e| e.to_string())?;
    let mut ls: Vec<String> = format!("{t:#?}")
        .lines()
        .filter(|l| !l.contains("node_id:") && !l.trim_start().starts_with("line:"))
        .map(|l| l.trim().to_string())
        .collect();
    ls.sort();
    Ok((ls.join("\n"), script))
}

/// O que o scaffold do CLI faz com `{{marcador}}` — a conferência parseia o
/// resultado, não o marcador (um `<app id="{{nome_projeto}}">` não é um id válido).
fn marcadores(s: &str) -> String {
    s.replace("{{nome_projeto}}", "meu-app")
        .replace("{{nome_crate}}", "meu_app")
        .replace("{{titulo}}", "Meu App")
        .replace("{{versao_motor}}", "0.0.0")
}

fn check(gv: &str, gvb: &str) -> Result<(), String> {
    let (gv, gvb) = (&marcadores(gv), &marcadores(gvb));
    let mutated = std::env::var("GVB_MUTATE").ok().map(|m| gvb.replacen(&m, "zzz", 1));
    let gvb = mutated.as_deref().unwrap_or(gvb);
    let xml = glacier_ui::gvb::desugar(gvb).map_err(|d| format!("desugar: {}", d.message))?;
    let a = tree(gv, gv).map_err(|e| format!("original: {e}"))?;
    let mut b = tree(&xml, gvb).map_err(|e| format!("tradução: {e}"))?;
    // as referências `.gv` foram reescritas para `.gvb`; desfaz para comparar
    b.0 = b.0.replace(".gvb\"", ".gv\"").replace("\"gvb/", "\"examples/");
    if a.1 != b.1 {
        return Err("o <script> mudou".into());
    }
    if a.0 != b.0 {
        let (la, lb): (Vec<_>, Vec<_>) = (a.0.lines().collect(), b.0.lines().collect());
        let only_a: Vec<_> = la.iter().filter(|l| !lb.contains(l)).take(3).collect();
        let only_b: Vec<_> = lb.iter().filter(|l| !la.contains(l)).take(3).collect();
        return Err(format!("árvores diferentes — só no .gv: {only_a:?} · só no .gvb: {only_b:?}"));
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `--in-place a.gv b.gv`: grava `a.gvb` ao lado de cada arquivo.
    let in_place = args[0] == "--in-place";
    let out_dir = PathBuf::from(&args[0]);
    let args: Vec<String> = if in_place { args[1..].to_vec() } else { args };
    let args = if in_place { [vec![String::new()], args].concat() } else { args };
    let (mut ok, mut bad) = (0, 0);
    for f in &args[1..] {
        let p = Path::new(f);
        let rel = p
            .components()
            .skip(1)
            .collect::<PathBuf>()
            .with_extension("gvb");
        let rel = if f.starts_with("templates") {
            Path::new("templates").join(rel)
        } else {
            rel
        };
        let (out_dir, rel) = if in_place {
            (p.parent().unwrap().to_path_buf(), PathBuf::from(p.file_name().unwrap()).with_extension("gvb"))
        } else {
            (out_dir.clone(), rel)
        };
        let src = std::fs::read_to_string(p).expect("lendo");
        match convert(&src) {
            Err(es) => {
                bad += 1;
                println!("PULADO  {f}: {}", es.join("; "));
            }
            Ok(gvb) => match check(&src, &gvb) {
                Err(e) => {
                    bad += 1;
                    // grava assim mesmo, ao lado, para inspeção
                    let dest = out_dir.join("_falhas").join(&rel);
                    std::fs::create_dir_all(dest.parent().unwrap()).ok();
                    std::fs::write(&dest, &gvb).ok();
                    println!("FALHOU  {f}: {e}");
                }
                Ok(()) => {
                    ok += 1;
                    let dest = out_dir.join(&rel);
                    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
                    std::fs::write(&dest, gvb).unwrap();
                }
            },
        }
    }
    println!("\n{ok} traduzidos e conferidos, {bad} pendentes");
}
