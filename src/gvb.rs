//! `.gvb` — o markup de blocos, dessugarado para o XML do `.gv`.
//!
//! A especificação é `rascunhos/MARKUP.md`. A regra que manda aqui é a de lá:
//! **o `.gvb` não muda uma vírgula da semântica do motor** — [`desugar`] produz
//! o XML que o autor teria escrito à mão, e quem o lê é o mesmo
//! [`UiNode::parse_xml_with_source`] de sempre. `eval`, builtins e diagnósticos
//! não sabem que o `.gvb` existe.
//!
//! **Cada linha do `.gvb` emite na mesma linha do XML.** É o truque do
//! `protect_style_bodies`: o `line` que o roxmltree reporta passa a ser o do
//! arquivo que o autor escreveu. A coluna não é preservada (o XML gerado não
//! tem a mesma largura), então o erro aponta a linha certa e a coluna só serve
//! de dica. Na prática isso sai de graça por construção: o emissor só avança
//! no número de linha ([`Out::at`]), nunca recua, e cada token é posto na
//! linha em que a fonte o tinha.
//!
//! O pipeline é `texto → AST ([`Node`]) → XML`. A AST existe porque a tag abre
//! antes de os atributos acabarem: o emissor precisa de cada nó inteiro (com o
//! texto e as linhas) para escrever abertura, corpo e fechamento.

use crate::error::{Diagnostic, GlacierError, Result};
use crate::parser::UiNode;

/// A extensão do markup de blocos.
pub const EXTENSAO: &str = "gvb";

/// Se `path` é um template `.gvb`.
pub fn eh_gvb(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSAO))
}

/// Dessugara um `.gvb` para o XML equivalente do `.gv`, linha por linha.
pub fn desugar(src: &str) -> std::result::Result<String, Diagnostic> {
    let nodes = Parser::new(src).file().map_err(|e| e.into_diagnostic())?;
    let mut out = Out::default();
    for n in &nodes {
        out.node(n);
    }
    Ok(out.buf)
}

impl UiNode {
    /// O mesmo que [`UiNode::parse_xml_in`], para um `.gvb`: dessugara e
    /// entrega ao parser de XML, citando o `.gvb` (não o XML gerado) nos erros.
    pub fn parse_gvb_in(src: &str, file: Option<&str>) -> Result<Self> {
        let xml = match desugar(src) {
            Ok(x) => x,
            Err(d) => {
                let d = match file {
                    Some(f) => d.in_file(f, src),
                    None => d.with_source(src),
                };
                return Err(GlacierError::Xml(Box::new(d)));
            }
        };
        Self::parse_xml_with_source(&xml, src, file)
    }
}

// ───────────────────────────── AST ─────────────────────────────

#[derive(Debug)]
struct Attr {
    name: String,
    /// Já interpolado (`@x` → `{x}`), ainda sem escape de XML.
    value: String,
    line: u32,
}

#[derive(Debug)]
struct Text {
    value: String,
    line: u32,
}

/// Qualquer coisa que vira uma tag: um elemento, um `if`/`else`, um `each`.
#[derive(Debug)]
struct Node {
    tag: String,
    attrs: Vec<Attr>,
    text: Option<Text>,
    children: Vec<Node>,
    line: u32,
    end_line: u32,
}

// ──────────────────────────── erro ─────────────────────────────

struct Error {
    line: u32,
    col: u32,
    msg: String,
    hint: Option<&'static str>,
}

impl Error {
    fn into_diagnostic(self) -> Diagnostic {
        let d = Diagnostic::new(self.line, self.col, self.msg);
        match self.hint {
            Some(h) => d.with_hint(h),
            None => d,
        }
    }
}

type Res<T> = std::result::Result<T, Error>;

// ─────────────────────────── o parser ──────────────────────────

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    /// Índice em `chars` do início da linha atual, para a coluna.
    line_start: usize,
}

/// O que pode vir dentro de um bloco: um atributo ou um filho.
enum Item {
    Attr(Attr),
    /// Uma cadeia `if`/`else if`/`else` rende vários nós irmãos.
    Nodes(Vec<Node>),
}

impl Parser {
    fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            line_start: 0,
        }
    }

    // — baixo nível —

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.line_start = self.pos;
        }
        Some(c)
    }

    fn starts_with(&self, s: &str) -> bool {
        s.chars().enumerate().all(|(i, c)| self.peek_at(i) == Some(c))
    }

    fn err<T>(&self, msg: impl Into<String>) -> Res<T> {
        Err(Error {
            line: self.line,
            col: (self.pos - self.line_start + 1) as u32,
            msg: msg.into(),
            hint: None,
        })
    }

    fn err_hint<T>(&self, msg: impl Into<String>, hint: &'static str) -> Res<T> {
        Err(Error {
            hint: Some(hint),
            ..self.err::<()>(msg).unwrap_err()
        })
    }

    /// Espaço e comentários (`//` e `/* … */`). A quebra de linha não significa
    /// nada (regra 1), então aqui ela é só contada.
    fn skip_ws(&mut self) -> Res<()> {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek_at(1) == Some('/') => {
                    while self.peek().is_some_and(|c| c != '\n') {
                        self.bump();
                    }
                }
                Some('/') if self.peek_at(1) == Some('*') => {
                    let (line, col) = (self.line, self.pos - self.line_start + 1);
                    self.bump();
                    self.bump();
                    loop {
                        if self.starts_with("*/") {
                            self.bump();
                            self.bump();
                            break;
                        }
                        if self.bump().is_none() {
                            return Err(Error {
                                line,
                                col: col as u32,
                                msg: "comentário `/*` nunca fechado".into(),
                                hint: None,
                            });
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn is_ident(c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '-'
    }

    fn ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek().filter(|&c| Self::is_ident(c)) {
            s.push(c);
            self.bump();
        }
        s
    }

    /// Salva/restaura a posição, para o `else` que pode não vir.
    fn mark(&self) -> (usize, u32, usize) {
        (self.pos, self.line, self.line_start)
    }

    fn reset(&mut self, m: (usize, u32, usize)) {
        (self.pos, self.line, self.line_start) = m;
    }

    // — gramática —

    fn file(&mut self) -> Res<Vec<Node>> {
        let mut out = Vec::new();
        loop {
            self.skip_ws()?;
            match self.peek() {
                None => return Ok(out),
                Some('}') => return self.err("`}` sem um `{` que o abra"),
                _ => {}
            }
            match self.item()? {
                Item::Nodes(n) => out.extend(n),
                Item::Attr(a) => {
                    return Err(Error {
                        line: a.line,
                        col: 1,
                        msg: format!("o atributo `{}:` está fora de qualquer bloco", a.name),
                        hint: Some("atributo só existe dentro das chaves de um elemento"),
                    });
                }
            }
        }
    }

    /// O corpo de um `{ … }`: consome até o `}` e devolve a linha dele.
    fn block(&mut self, raw: bool) -> Res<(Vec<Attr>, Vec<Node>, Option<Text>, u32)> {
        let (open_line, open_col) = (self.line, (self.pos - self.line_start + 1) as u32);
        self.bump(); // `{`
        let mut attrs = Vec::new();
        let mut children = Vec::new();
        let mut text = None;
        loop {
            self.skip_ws()?;
            match self.peek() {
                None => {
                    return Err(Error {
                        line: open_line,
                        col: open_col,
                        msg: "este `{` nunca é fechado".into(),
                        hint: None,
                    });
                }
                Some('}') => {
                    let end = self.line;
                    self.bump();
                    return Ok((attrs, children, text, end));
                }
                Some('"') => {
                    // Texto dentro do bloco: a forma de um `script`/`style`
                    // com atributos (`script { lang: python """…""" }`).
                    if text.is_some() {
                        return self.err("dois corpos de texto no mesmo bloco");
                    }
                    let line = self.line;
                    let value = self.string(raw)?.0;
                    text = Some(Text { value, line });
                }
                _ => match self.item()? {
                    Item::Attr(a) => attrs.push(a),
                    Item::Nodes(n) => children.extend(n),
                },
            }
        }
    }

    fn item(&mut self) -> Res<Item> {
        let line = self.line;
        let Some(c) = self.peek() else {
            return self.err("fim do arquivo inesperado");
        };
        if !Self::is_ident(c) {
            return self.err_hint(
                format!("não esperava `{c}` aqui"),
                "um item começa por um nome: `tag { … }` ou `atributo: valor`",
            );
        }
        let name = self.ident();

        // `nome:` — atributo. O `:` tem de encostar no nome (como no CSS).
        if self.peek() == Some(':') {
            self.bump();
            self.skip_ws()?;
            let value = self.value()?;
            return Ok(Item::Attr(Attr { name, value, line }));
        }

        match name.as_str() {
            "if" => self.if_chain(line).map(Item::Nodes),
            "each" => self.each(line).map(|n| Item::Nodes(vec![n])),
            "else" => self.err_hint(
                "`else` sem um `if` antes",
                "o `else` tem de vir logo depois do `}` do `if`: `} else { … }`",
            ),
            _ => self.element(name, line).map(|n| Item::Nodes(vec![n])),
        }
    }

    /// `tag.a.b#id "texto" { … }` — o texto e o bloco são opcionais.
    fn element(&mut self, tag: String, line: u32) -> Res<Node> {
        // `script` e `style` carregam código de outra linguagem: o corpo é cru —
        // sem `@`, sem escape, sem colapso de espaço (e a emissão não o escapa).
        let raw = matches!(tag.as_str(), "script" | "style");
        // `tag.classe` e `tag#id` não existem: classe e id são atributos como
        // os outros (`class:`, `id:`), e o ponto fica livre para ser só ponto.
        if matches!(self.peek(), Some('.' | '#')) {
            return self.err_hint(
                format!("`{tag}{}` — classe e id não vão na tag", self.peek().unwrap()),
                "escreva `tag { class: nome }` (ou `id: nome`)",
            );
        }

        let mut node = Node {
            tag,
            attrs: Vec::new(),
            text: None,
            children: Vec::new(),
            line,
            end_line: line,
        };

        // O texto: uma string solta logo depois da tag (regra 6) — ou depois do
        // bloco, ver mais abaixo.
        let m = self.mark();
        self.skip_ws()?;
        if self.peek() == Some('"') {
            let tline = self.line;
            let (value, end) = self.string(raw)?;
            node.text = Some(Text { value, line: tline });
            node.end_line = end;
        } else {
            self.reset(m);
        }

        let m = self.mark();
        self.skip_ws()?;
        if self.peek() == Some('{') {
            let (attrs, children, body_text, end) = self.block(raw)?;
            if body_text.is_some() {
                if node.text.is_some() {
                    return self.err("o texto aparece antes e dentro do bloco");
                }
                node.text = body_text;
            }
            node.attrs = attrs;
            node.children = children;
            node.end_line = end;

            // O texto também pode vir DEPOIS do bloco — `text { class: nota }
            // "oi"` —, e é a forma que se escreve: as chaves primeiro, o
            // conteúdo por último. Só vale sem filhos (texto misturado com
            // elementos não existe), e um texto já lido antes do bloco não
            // admite um segundo.
            if node.children.is_empty() {
                let m = self.mark();
                self.skip_ws()?;
                if self.peek() == Some('"') {
                    if node.text.is_some() {
                        return self.err("o texto aparece antes e depois do bloco");
                    }
                    let tline = self.line;
                    let (value, end) = self.string(raw)?;
                    node.text = Some(Text { value, line: tline });
                    node.end_line = end;
                } else {
                    self.reset(m);
                }
            }
        } else {
            self.reset(m);
        }

        Ok(node)
    }

    // — condicional —

    /// `if COND { … }` (o `if` já consumido), com os `else if`/`else` que o
    /// seguem. Cada ramo vira um `<template>` irmão — é assim que o `.gv` os
    /// escreve.
    fn if_chain(&mut self, line: u32) -> Res<Vec<Node>> {
        let mut chain = vec![self.branch("if", line)?];
        loop {
            let m = self.mark();
            self.skip_ws()?;
            if !self.word_is("else") {
                self.reset(m);
                return Ok(chain);
            }
            let l = self.line;
            self.ident();
            self.skip_ws()?;
            if self.word_is("if") {
                self.ident();
                chain.push(self.branch("else_if", l)?);
                continue;
            }
            if self.peek() != Some('{') {
                return self.err("esperava `{` (ou `if`) depois do `else`");
            }
            let (body_attrs, children, _, end_line) = self.block(false)?;
            let mut attrs = vec![Attr { name: "else".into(), value: String::new(), line: l }];
            attrs.extend(body_attrs);
            chain.push(Node { tag: "template".into(), attrs, text: None, children, line: l, end_line });
            return Ok(chain);
        }
    }

    /// Um ramo com condição: `COND { … }`.
    fn branch(&mut self, key: &str, line: u32) -> Res<Node> {
        self.skip_ws()?;
        let mut attrs = self.condition(key, line)?;
        self.skip_ws()?;
        if self.peek() != Some('{') {
            return self.err("esperava `{` depois da condição");
        }
        let (body_attrs, children, _, end_line) = self.block(false)?;
        attrs.extend(body_attrs);
        Ok(Node { tag: "template".into(), attrs, text: None, children, line, end_line })
    }

    /// `COND` da regra 8, já traduzida para os atributos do `<template>`.
    /// `key` é `if` ou `else_if`.
    fn condition(&mut self, key: &str, line: u32) -> Res<Vec<Attr>> {
        let at = |name: &str, value: String| Attr {
            name: name.into(),
            value,
            line,
        };
        let first = self.operand()?;
        self.skip_inline();

        // `"api" in @marcados`
        if self.word_is("in") {
            self.ident();
            self.skip_inline();
            let target = self.operand()?;
            return Ok(vec![at(key, target.as_ref_value(self)?), at("contains", first.literal())]);
        }
        // `@x is empty`
        if self.word_is("is") {
            self.ident();
            self.skip_inline();
            if !self.word_is("empty") {
                return self.err("depois de `is` só existe `empty`");
            }
            self.ident();
            return Ok(vec![at(key, first.as_ref_value(self)?), at("empty", "true".into())]);
        }
        // `@x == v`, `@x != v`
        let op = if self.starts_with("==") {
            Some("equals")
        } else if self.starts_with("!=") {
            Some("not_equals")
        } else {
            None
        };
        if let Some(op) = op {
            self.bump();
            self.bump();
            self.skip_inline();
            let rhs = self.operand()?;
            return Ok(vec![at(key, first.as_ref_value(self)?), at(op, rhs.literal())]);
        }
        // `@x` — truthy
        Ok(vec![at(key, first.as_ref_value(self)?)])
    }

    /// Espaço sem quebra de linha — a condição acaba no `{`, mas o `{` pode
    /// estar na linha de baixo, então só o espaço entre operadores é "inline".
    fn skip_inline(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.bump();
        }
    }

    fn word_is(&self, w: &str) -> bool {
        self.starts_with(w) && !self.peek_at(w.chars().count()).is_some_and(Self::is_ident)
    }

    fn operand(&mut self) -> Res<Operand> {
        match self.peek() {
            Some('@') => {
                self.bump();
                let braced = self.peek() == Some('{');
                if braced {
                    self.bump();
                }
                let name = self.ref_name();
                if name.is_empty() {
                    return self.err("esperava o nome de uma chave depois do `@`");
                }
                if braced {
                    if self.peek() != Some('}') {
                        return self.err("`@{` sem o `}` que o fecha");
                    }
                    self.bump();
                }
                Ok(Operand::Ref(name))
            }
            Some('"') => Ok(Operand::Lit(self.string(false)?.0)),
            Some(c) if Self::is_ident(c) => Ok(Operand::Lit(self.ident())),
            _ => self.err("esperava `@chave`, uma string ou um nome"),
        }
    }

    /// Nome de chave depois de um `@`: letras, dígitos, `_` e `.` (caminho,
    /// `@c.id`) — mas o `.` só conta se vier mais nome depois, para a frase
    /// "vale @preco." não engolir o ponto final.
    fn ref_name(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            let ok = c.is_alphanumeric()
                || c == '_'
                || (c == '.' && self.peek_at(1).is_some_and(|n| n.is_alphanumeric() || n == '_'));
            if !ok {
                break;
            }
            s.push(c);
            self.bump();
        }
        s
    }

    // — each —

    fn each(&mut self, line: u32) -> Res<Node> {
        self.skip_ws()?;
        let items = match self.operand()? {
            Operand::Ref(n) => n,
            Operand::Lit(_) => {
                return self.err_hint(
                    "`each` quer uma `@lista`",
                    "each @servicos as s { … }",
                );
            }
        };
        self.skip_ws()?;
        if !self.word_is("as") {
            return self.err("esperava `as` depois da lista: `each @itens as item { … }`");
        }
        self.ident();
        self.skip_ws()?;
        let var = self.ident();
        if var.is_empty() {
            return self.err("esperava o nome da variável depois do `as`");
        }
        self.skip_ws()?;
        if self.peek() != Some('{') {
            return self.err("esperava `{` depois do `each`");
        }
        let (body_attrs, children, _, end_line) = self.block(false)?;
        let mut attrs = vec![
            Attr { name: "items".into(), value: items, line },
            Attr { name: "var".into(), value: var, line },
        ];
        attrs.extend(body_attrs);
        Ok(Node {
            tag: "foreach".into(),
            attrs,
            text: None,
            children,
            line,
            end_line,
        })
    }

    // — valores —

    /// O valor de um `nome:`. Entre aspas, ou nu até o espaço/`{`/`}` — e um
    /// `//` no meio de um nu é comentário (a primeira armadilha da spec).
    fn value(&mut self) -> Res<String> {
        match self.peek() {
            Some('"') => Ok(self.string(false)?.0),
            None | Some('}') | Some('{') => self.err("esperava um valor depois do `:`"),
            _ => {
                let mut raw = String::new();
                while let Some(c) = self.peek() {
                    if c.is_whitespace() || c == '{' || c == '}' || self.starts_with("//") {
                        break;
                    }
                    raw.push(c);
                    self.bump();
                }
                Ok(interpolate(&raw))
            }
        }
    }

    /// Uma string `"…"` ou `"""…"""`. Devolve o conteúdo já interpolado e a
    /// linha em que ela termina.
    fn string(&mut self, raw_body: bool) -> Res<(String, u32)> {
        let (sline, scol) = (self.line, (self.pos - self.line_start + 1) as u32);
        let unclosed = Error {
            line: sline,
            col: scol,
            msg: "string nunca fechada".into(),
            hint: None,
        };
        let mut raw = String::new();
        if self.starts_with("\"\"\"") {
            for _ in 0..3 {
                self.bump();
            }
            loop {
                if self.starts_with("\"\"\"") {
                    for _ in 0..3 {
                        self.bump();
                    }
                    break;
                }
                match self.bump() {
                    Some(c) => raw.push(c),
                    None => return Err(unclosed),
                }
            }
        } else {
            self.bump();
            loop {
                match self.bump() {
                    Some('"') => break,
                    Some('\\') if matches!(self.peek(), Some('"' | '\\')) => {
                        raw.push(self.bump().unwrap());
                    }
                    Some(c) => raw.push(c),
                    None => return Err(unclosed),
                }
            }
        }
        if raw_body {
            return Ok((raw, self.line));
        }
        Ok((interpolate(&raw), self.line))
    }
}

enum Operand {
    Ref(String),
    Lit(String),
}

impl Operand {
    /// O lado que é uma chave: vira `{nome}`.
    fn as_ref_value(&self, p: &Parser) -> Res<String> {
        match self {
            Operand::Ref(n) => Ok(format!("{{{n}}}")),
            Operand::Lit(_) => p.err_hint(
                "o lado esquerdo da condição tem de ser uma `@chave`",
                "if @aba == \"x\"  ·  if \"x\" in @lista  ·  if @lista is empty",
            ),
        }
    }

    /// O lado que é um valor. Uma `@chave` aqui é dado (`{nome}`), como num
    /// atributo.
    fn literal(&self) -> String {
        match self {
            Operand::Lit(s) => s.clone(),
            Operand::Ref(n) => format!("{{{n}}}"),
        }
    }
}

/// A regra 5: `@nome` / `@{nome}` viram `{nome}`; `@@` é um arroba literal.
/// Um `@` que não precede nome nem `{` fica como está (um e-mail não quebra).
fn interpolate(raw: &str) -> String {
    let cs: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < cs.len() {
        if cs[i] != '@' {
            out.push(cs[i]);
            i += 1;
            continue;
        }
        match cs.get(i + 1) {
            Some('@') => {
                out.push('@');
                i += 2;
            }
            Some('{') => match cs[i + 2..].iter().position(|&c| c == '}') {
                Some(end) => {
                    out.push('{');
                    out.extend(&cs[i + 2..i + 2 + end]);
                    out.push('}');
                    i += 2 + end + 1;
                }
                None => {
                    out.push('@');
                    i += 1;
                }
            },
            Some(&c) if c.is_alphanumeric() || c == '_' => {
                let mut j = i + 1;
                while j < cs.len()
                    && (cs[j].is_alphanumeric()
                        || cs[j] == '_'
                        || (cs[j] == '.'
                            && cs.get(j + 1).is_some_and(|n| n.is_alphanumeric() || *n == '_')))
                {
                    j += 1;
                }
                out.push('{');
                out.extend(&cs[i + 1..j]);
                out.push('}');
                i = j;
            }
            _ => {
                out.push('@');
                i += 1;
            }
        }
    }
    out
}

// ─────────────────────────── emissor ───────────────────────────

/// O XML sendo escrito, com o contador de linha que garante a regra de ouro:
/// só se avança.
struct Out {
    buf: String,
    line: u32,
}

impl Default for Out {
    fn default() -> Self {
        Self { buf: String::new(), line: 1 }
    }
}

impl Out {
    /// Leva a saída até a linha `line` (se já passou, não faz nada).
    fn at(&mut self, line: u32) {
        while self.line < line {
            self.buf.push('\n');
            self.line += 1;
        }
    }

    /// Escreve texto que pode conter quebras de linha, mantendo a contagem.
    fn push(&mut self, s: &str) {
        self.line += s.matches('\n').count() as u32;
        self.buf.push_str(s);
    }

    fn node(&mut self, n: &Node) {
        self.at(n.line);
        self.push(&format!("<{}", n.tag));
        for a in &n.attrs {
            self.at(a.line);
            self.push(&format!(" {}=\"{}\"", a.name, escape(&a.value, true)));
        }
        // `script`/`style` sempre abrem e fecham: o motor procura o `</script>`,
        // então `script { src: "x" }` sem corpo não pode sair como `<script/>`.
        let raw_tag = matches!(n.tag.as_str(), "script" | "style");
        if n.text.is_none() && n.children.is_empty() && !raw_tag {
            self.at(n.end_line);
            self.push("/>");
            return;
        }
        // O `>` só depois de alcançar a linha do texto: as quebras de linha
        // ficam DENTRO da tag de abertura (o XML as aceita), e não dentro do
        // corpo — que num `script`/`style` é código, e uma quebra a mais no
        // começo é conteúdo.
        if let Some(t) = &n.text {
            self.at(t.line);
        }
        self.push(">");
        if let Some(t) = &n.text {
            if matches!(n.tag.as_str(), "script" | "style") {
                self.push(&t.value);
            } else {
                self.push(&escape(&t.value, false));
            }
        }
        for c in &n.children {
            self.node(c);
        }
        self.at(n.end_line);
        self.push(&format!("</{}>", n.tag));
    }
}

fn escape(s: &str, attr: bool) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' if attr => o.push_str("&quot;"),
            // Um `\n` num atributo viraria espaço no parser de XML; a linha
            // física, porém, tem de ser contada — então vai como referência.
            '\n' if attr => o.push_str("&#10;"),
            _ => o.push(c),
        }
    }
    o
}
