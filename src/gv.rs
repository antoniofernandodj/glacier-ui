//! `.gv` — o markup de blocos do Glacier.
//!
//! A sintaxe está descrita no `AGENTS.md` dos projetos gerados. O motor não interpreta o texto do
//! `.gv` diretamente: [`desugar`] o traduz para o XML que o parser
//! ([`UiNode::parse_xml_with_source`]) consome, e é esse XML — um formato
//! **interno**, que ninguém escreve à mão — que alimenta `eval`, builtins e
//! diagnósticos. Eles não sabem que o `.gv` existe.
//!
//! **Cada linha do `.gv` emite na mesma linha do XML.** É o truque do
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
use crate::eval;
use crate::parser::UiNode;

/// A extensão do markup de blocos.
pub const EXTENSAO: &str = "gv";

/// Ações embutidas que o motor despacha em Rust pelo prefixo `nome:arg`
/// (`src/lib.rs`, `dispatch`). Na forma de chamada (`textarea_end("log")`) elas
/// viram `nome:arg`, e não a chamada JSON de um handler Luau.
pub const ACOES_EMBUTIDAS_COM_PREFIXO: &[&str] =
    &["textarea_end", "textarea_top", "clipboard", "open", "audio", "whatsthis"];

/// Se `path` é um template `.gv`.
pub fn eh_gv(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSAO))
}

/// O markup de um template em `UiNode`, mais o `script` que ele carregava.
///
/// `content` já é o XML interno: a leitura (ver
/// [`crate::asset_source::read_markup`]) dessugarou o `.gv` antes de chegar
/// aqui. Tira o `script`, normaliza as diretivas nuas (`else`) e parseia.
pub(crate) fn parse_markup(path: Option<&str>, content: &str) -> Result<(UiNode, Option<String>)> {
    let (markup, script) = eval::strip_script(content);
    let markup = eval::normalize_bare_directives(&markup);
    // `content` (e não `markup`) como fonte dos trechos: o erro deve mostrar a
    // linha que o autor escreveu, não a que o pré-processamento produziu.
    Ok((
        UiNode::parse_xml_with_source(&markup, content, path)?,
        script,
    ))
}

/// Dessugara o texto de um [`Template::Inline`](crate::Template::Inline): o mesmo
/// `.gv`, só que embutido no binário.
pub(crate) fn desugar_inline(src: &str) -> Result<String> {
    desugar(src).map_err(|d| GlacierError::Xml(Box::new(d.with_source(src))))
}

/// Dessugara um `.gv` para o XML interno equivalente, linha por linha.
// O `Diagnostic` é grande (posição, trecho, dica), e o clippy pede um `Box`.
// Não compensa: isto roda uma vez por arquivo, não num laço quente, e trocar o
// tipo do erro quebraria a API pública.
#[allow(clippy::result_large_err)]
pub fn desugar(src: &str) -> std::result::Result<String, Diagnostic> {
    desugar_with(src, true)
}

/// Como [`desugar`], mas **sem interpolação**: `@nome` e `@{nome}` ficam como estão.
/// É o dessugar de um `.gv` que chega como dado — o texto de um artigo vindo do
/// banco, num `prose(syntax = gv, content = "…")` —, em que `a@b.com` é um e-mail, e
/// não uma ligação a uma chave do contexto.
#[allow(clippy::result_large_err)] // ver `desugar`
pub fn desugar_literal(src: &str) -> std::result::Result<String, Diagnostic> {
    let mut p = Parser::new(src);
    p.literal = true;
    let nodes = p.file().map_err(|e| e.into_diagnostic())?;
    let mut out = Out::default();
    for n in &nodes {
        out.node(n);
    }
    Ok(out.buf)
}

/// Como [`desugar`], mas com `strict = false` aceita uma ligação sem o `:`
/// (`value = chave`), que é como os `.gv` eram escritos antes da marca. Existe
/// para as ferramentas de migração; o motor usa sempre [`desugar`].
#[allow(clippy::result_large_err)] // ver `desugar`
/// [`desugar`] de um arquivo: o mesmo resultado, e as formas depreciadas que ele
/// usa avisadas no `stderr` — uma linha por forma, com a primeira ocorrência e
/// quantas mais há no arquivo (o `glacier fmt --migrate` reescreve todas).
pub fn desugar_em(src: &str, file: &str) -> std::result::Result<String, Diagnostic> {
    let mut p = Parser::new(src);
    let nodes = p.file().map_err(|e| e.into_diagnostic())?;
    let mut out = Out::default();
    for n in &nodes {
        out.node(n);
    }
    let mut vistas: Vec<(&'static str, u32, &'static str, usize)> = Vec::new();
    for (linha, forma, msg) in &p.avisos {
        match vistas.iter_mut().find(|v| v.0 == *forma) {
            Some(v) => v.3 += 1,
            None => vistas.push((forma, *linha, msg, 0)),
        }
    }
    for (_, linha, msg, mais) in vistas {
        let resto = if mais > 0 { format!(" (e mais {mais} neste arquivo)") } else { String::new() };
        eprintln!("glacier-ui: {file}:{linha}: {msg}{resto} — `glacier fmt --migrate` reescreve");
    }
    Ok(out.buf)
}

/// O corpo de `script` de um `.gv` (o Luau da tela ou do componente), vazio
/// quando não há. É o que o `glacier check` lê para conferir os handlers contra
/// as ações do template (Frente 6 do `TODO.md`).
pub fn script_do_gv(src: &str) -> String {
    match desugar(src) {
        Ok(xml) => eval::strip_script(&xml).1.unwrap_or_default(),
        Err(_) => String::new(),
    }
}

/// As formas depreciadas de um `.gv`: (linha, mensagem), na ordem do arquivo. É o
/// que o `glacier check` lista.
pub fn formas_depreciadas(src: &str) -> std::result::Result<Vec<(u32, &'static str)>, Diagnostic> {
    let mut p = Parser::new(src);
    p.file().map_err(|e| e.into_diagnostic())?;
    Ok(p.avisos.into_iter().map(|(l, _, m)| (l, m)).collect())
}

pub fn desugar_with(src: &str, strict: bool) -> std::result::Result<String, Diagnostic> {
    let mut p = Parser::new(src);
    p.strict = strict;
    let nodes = p.file().map_err(|e| e.into_diagnostic())?;
    let mut out = Out::default();
    for n in &nodes {
        out.node(n);
    }
    Ok(out.buf)
}

impl UiNode {
    /// O mesmo que [`UiNode::parse_xml_in`], para um `.gv`: dessugara e
    /// entrega ao parser de XML, citando o `.gv` (não o XML gerado) nos erros.
    pub fn parse_gv_in(src: &str, file: Option<&str>) -> Result<Self> {
        let xml = match match file {
            Some(f) => desugar_em(src, f),
            None => desugar(src),
        } {
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

#[derive(Debug, Clone, Default)]
struct Attr {
    name: String,
    /// Já interpolado (`@x` → `{x}`), ainda sem escape de XML.
    value: String,
    line: u32,
}

#[derive(Debug, Clone)]
struct Text {
    value: String,
    line: u32,
}

/// Qualquer coisa que vira uma tag: um elemento, um `if`/`else`, um `each`.
#[derive(Debug, Clone)]
struct Node {
    tag: String,
    attrs: Vec<Attr>,
    text: Option<Text>,
    children: Vec<Node>,
    line: u32,
    end_line: u32,
}

// ──────────────────────────── erro ─────────────────────────────

#[derive(Debug)]
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
    /// Recusa uma ligação sem o `:` (ver `check_bindings`).
    strict: bool,
    /// As tags cujo bloco está aberto: o pai de quem está sendo lido.
    open: Vec<String>,
    /// Não interpola: `@nome` fica como está. Para um `.gv` que vem de **dado**
    /// (ver [`desugar_literal`]), onde um `@` é o de um e-mail, não uma ligação.
    literal: bool,
    /// As formas depreciadas encontradas: (linha, chave da forma, mensagem).
    /// [`desugar_em`] as imprime, uma por forma e por arquivo.
    avisos: Vec<(u32, &'static str, &'static str)>,
    /// As linhas em que um `:for_each` foi GERADO pelo `each = @x as v` — a forma
    /// nova, que não pode ser avisada como a antiga que ela produz.
    for_each_gerado: Vec<u32>,
    /// Os atributos do elemento corrente escritos com um `:` de expressão
    /// (`:disabled = @a && @b`, Frente 10): são escolha de quem escreveu, e o
    /// `check_bindings` não pode pedir o `:` de ligação neles.
    uma_via: Vec<String>,
}

/// O que pode vir dentro de um bloco: um filho — ou uma cadeia
/// `if`/`else if`/`else`, que rende vários nós irmãos.
struct Item(Vec<Node>);

impl Parser {
    fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            line_start: 0,
            strict: true,
            open: Vec::new(),
            literal: false,
            avisos: Vec::new(),
            for_each_gerado: Vec::new(),
            uma_via: Vec::new(),
        }
    }

    fn interpola(&self, raw: &str) -> String {
        if self.literal {
            raw.to_string()
        } else {
            interpolate(raw)
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
        s.chars()
            .enumerate()
            .all(|(i, c)| self.peek_at(i) == Some(c))
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
                None => {
                    fundir_estado(&mut out);
                    check_else_positions(&out)?;
                    return Ok(out);
                }
                Some('}') => return self.err("`}` sem um `{` que o abra"),
                _ => {}
            }
            out.extend(self.item()?.0);
        }
    }

    /// O corpo de um `{ … }`: só filhos. Consome até o `}` e devolve a linha dele.
    fn block(&mut self) -> Res<(Vec<Node>, u32)> {
        let (open_line, open_col) = (self.line, (self.pos - self.line_start + 1) as u32);
        self.bump(); // `{`
        let mut children = Vec::new();
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
                    fundir_estado(&mut children);
                    check_else_positions(&children)?;
                    return Ok((children, end));
                }
                Some('"') => {
                    return self.err_hint(
                        "texto solto dentro das chaves",
                        "o texto vai logo depois da tag: `text(class = nota) \"oi\"`",
                    );
                }
                _ => children.extend(self.item()?.0),
            }
        }
    }

    /// Os atributos de `( nome = valor, … )`, com o `(` sob o cursor. Devolve
    /// também a linha do `)`. A vírgula separa; a última é opcional; a quebra
    /// de linha não significa nada.
    fn attrs(&mut self, tag: &str) -> Res<(Vec<Attr>, u32)> {
        let (open_line, open_col) = (self.line, (self.pos - self.line_start + 1) as u32);
        self.bump(); // `(`
        self.uma_via.clear();
        let mut attrs = Vec::new();
        loop {
            self.skip_ws()?;
            match self.peek() {
                None => {
                    return Err(Error {
                        line: open_line,
                        col: open_col,
                        msg: "este `(` nunca é fechado".into(),
                        hint: None,
                    });
                }
                Some(')') => {
                    let end = self.line;
                    self.bump();
                    return Ok((attrs, end));
                }
                Some(c) if Self::is_ident(c) || c == ':' || c == '"' => {}
                // Expressão sem nome, primeira da lista: `text(@max ? "a" : "b")`.
                Some('@' | '!' | '(' | '[') if attrs.is_empty() && posicional(tag).is_some() => {}
                Some(c) => {
                    return self.err_hint(
                        format!("não esperava `{c}` na lista de atributos"),
                        "um atributo é `nome = valor`, separados por vírgula",
                    );
                }
            }
            let line = self.line;
            // Expressão sem nome, **primeira** da lista: `text(@a ? "x" : "y")`,
            // `button(t("comum.salvar"), …)`. Vale como o texto da tag (`content`
            // num `text`, `text` num `button`), como a string sem nome abaixo.
            let funcao_posicional = attrs.is_empty()
                && crate::expr::FUNCOES.iter().any(|f| {
                    self.starts_with(f) && self.peek_at(f.chars().count()) == Some('(')
                });
            if let Some(nome) = posicional(tag)
                && attrs.is_empty()
                && (matches!(self.peek(), Some('@' | '!' | '(' | '[')) || funcao_posicional)
            {
                let value = match self.expression_value(nome, tag)? {
                    Some(v) => v,
                    None => self.value()?,
                };
                attrs.push(Attr {
                    name: nome.to_string(),
                    value,
                    line,
                });
                self.skip_ws()?;
                match self.peek() {
                    Some(',') => {
                        self.bump();
                    }
                    Some(')') => {}
                    _ => {
                        return self.err_hint(
                            "esperava `,` ou `)` depois do texto",
                            "os demais atributos vêm depois de uma vírgula: `text(@a ? \"x\" : \"y\", class = c)`",
                        );
                    }
                }
                continue;
            }
            // Texto sem nome, **primeiro** da lista: `b("negrito")`,
            // `a("o site", href = "…")`. Vale como `content` nas tags de texto
            // (ver `posicional`); é o que `content = "…"` diria, mais curto.
            if self.peek() == Some('"') || self.starts_with("l\"\"\"") {
                let Some(nome) = posicional(tag) else {
                    return self.err_hint(
                        format!("`{tag}` não aceita texto sem nome"),
                        "um atributo é `nome = valor`; o texto sem nome só vale nas tags que carregam texto (`text`, `b`, `i`, `a`, `span`…); `prose` e `p` só têm filhos",
                    );
                };
                if !attrs.is_empty() {
                    return self.err_hint(
                        "o texto sem nome vem primeiro na lista",
                        "o texto vai antes dos atributos: `tag(\"texto\", outro = valor)`",
                    );
                }
                let value = self.value()?;
                attrs.push(Attr {
                    name: nome.to_string(),
                    value,
                    line,
                });
                self.skip_ws()?;
                match self.peek() {
                    Some(',') => {
                        self.bump();
                    }
                    Some(')') => {}
                    _ => {
                        return self.err_hint(
                            "esperava `,` ou `)` depois do texto",
                            "os demais atributos vêm depois de uma vírgula: `a(\"o site\", href = x)`",
                        );
                    }
                }
                continue;
            }
            // Dois pontos antes do nome decidem o que o valor É (Frente 10 do
            // `TODO.md`): `::attr = @x` é o vínculo nos dois sentidos (a chave),
            // `:attr = expr` é o atributo recebendo uma expressão numa via. A
            // forma antiga — `:attr = nome`, o nome nu — continua valendo e é
            // avisada; o `glacier fmt --migrate` a troca pela nova.
            let dois = self.peek() == Some(':');
            let mut dos_sentidos = false;
            if dois {
                self.bump();
                dos_sentidos = self.peek() == Some(':');
                if dos_sentidos {
                    self.bump();
                }
            }
            let name = self.ident();
            // A classe condicional: `:class.nome = expr` (a forma) e
            // `class:nome = expr` (a de antes, depreciada). Juntada ao `class` no
            // fim do elemento (ver `merge_class_directives`).
            let classe = if name == "class" && matches!(self.peek(), Some(':') | Some('.')) {
                let sep = self.peek().unwrap();
                if sep == ':' && !dois {
                    self.avisos.push((
                        line,
                        "class-diretiva",
                        "`class:nome = expr` está depreciado — a classe condicional é `:class.nome = expr`",
                    ));
                } else if sep == '.' && !dois {
                    self.bump(); // o `.` de `:class.` só faz sentido depois do `:`
                    return self.err_hint(
                        "`class.nome` precisa do `:` antes: `:class.nome = expr`",
                        "a classe condicional é `:class.ativa = @view == @alvo`",
                    );
                }
                if sep == ':' && dois {
                    return self.err_hint(
                        "`:class:nome` não existe",
                        "a classe condicional é `:class.nome = expr`",
                    );
                }
                self.bump();
                let classe = self.ident();
                if classe.is_empty() {
                    return self.err_hint(
                        "esperava o nome da classe",
                        "class:nav-item--active = @view == @target",
                    );
                }
                Some(classe)
            } else {
                None
            };
            if let Some(classe) = classe {
                self.skip_ws()?;
                if self.peek() != Some('=') {
                    return self.err_hint(
                        format!("esperava `=` depois da classe `{classe}`"),
                        "a classe condicional leva a condição: `class:nome = @expr`",
                    );
                }
                self.bump();
                self.skip_ws()?;
                let (l0, c0) = (self.line, (self.pos - self.line_start + 1) as u32);
                let raw = self.expression_source(&[',', ')'])?;
                let src = raw.trim().to_string();
                if let Err(e) = crate::expr::parse(&src) {
                    return Err(Error {
                        line: l0,
                        col: c0 + e.pos as u32,
                        msg: format!("condição inválida em `class:{classe}`: {}", e.msg),
                        hint: Some("class:nome = @a && !@b"),
                    });
                }
                attrs.push(Attr {
                    name: format!("class:{classe}"),
                    value: src,
                    line,
                });
                self.skip_ws()?;
                if self.peek() == Some(',') {
                    self.bump();
                }
                continue;
            }
            // `each = @lista as v` — o laço na forma curta, num nó só (o par do
            // bloco `each @lista as v { … }`). Vira o `:for_each`/`var` de sempre.
            if !dois && name == "each" {
                self.skip_ws()?;
                if self.peek() != Some('=') {
                    return self.err_hint("esperava `=` depois de `each`", "each = @lista as item");
                }
                self.bump();
                self.skip_ws()?;
                let lista = match self.operand()? {
                    Operand::Ref(n) => n,
                    Operand::Lit => {
                        return self.err_hint("`each =` quer uma `@lista`", "PickerRow(each = @servicos as s, title = @s.name)");
                    }
                };
                self.skip_ws()?;
                if !self.word_is("as") {
                    return self.err_hint("esperava `as` depois da lista", "each = @lista as item");
                }
                self.ident();
                self.skip_ws()?;
                let var = self.ident();
                if var.is_empty() {
                    return self.err("esperava o nome da variável depois do `as`");
                }
                self.for_each_gerado.push(line);
                attrs.push(Attr { name: ":for_each".into(), value: lista, line });
                attrs.push(Attr { name: "var".into(), value: var, line });
                self.skip_ws()?;
                if self.peek() == Some(',') {
                    self.bump();
                }
                continue;
            }
            let rotulo = if dos_sentidos { format!("::{name}") } else { name.clone() };
            self.skip_ws()?;
            // Um atributo sem `= valor` é um marcador (`else`, `component`): o
            // mesmo que `nome=""` no XML.
            if matches!(self.peek(), Some(',' | ')')) {
                attrs.push(Attr {
                    name: rotulo,
                    value: String::new(),
                    line,
                });
                if self.peek() == Some(',') {
                    self.bump();
                }
                continue;
            }
            if self.peek() != Some('=') {
                return self.err_hint(
                    format!("esperava `=` depois de `{rotulo}`"),
                    "um atributo é `nome = valor` (e não `nome: valor`)",
                );
            }
            self.bump();
            self.skip_ws()?;
            // `::attr = @chave` — o vínculo de duas vias: vai para a ligação
            // interna do widget o NOME da chave. Não é vínculo o que o widget não
            // grava: dizer `::text = @x` seria pedir um silêncio.
            if dos_sentidos {
                let raw = self.expression_source(&[',', ')'])?.trim().to_string();
                let chave = raw
                    .strip_prefix('@')
                    .or_else(|| raw.strip_prefix("@{").map(|r| r.trim_end_matches('}').trim()))
                    .unwrap_or(&raw)
                    .trim()
                    .to_string();
                if chave.is_empty() || !crate::expr::eh_caminho(&format!("x{chave}")) {
                    return self.err_hint(
                        format!("`{rotulo} = {raw}` não é uma chave"),
                        "o vínculo de duas vias quer uma chave: `::value = @form.nome`",
                    );
                }
                if !crate::bindings::is_binding(tag, &name) {
                    return self.err_hint(
                        format!("`{name}` de `{tag}` não é um campo de formulário"),
                        "só vínculo nos dois sentidos o que o widget escreve; uma via é `:nome = expr`",
                    );
                }
                attrs.push(Attr { name: format!(":{name}"), value: chave, line });
                self.skip_ws()?;
                if self.peek() == Some(',') {
                    self.bump();
                }
                continue;
            }
            // `:attr = …` de UMA via. O nome nu depois do `:` é a ligação antiga
            // (`:value = progresso`), ainda aceita e avisada; o resto é expressão.
            if dois {
                let nu = matches!(self.peek(), Some(c) if c.is_alphabetic() || c == '_');
                if nu {
                    let m = self.mark();
                    let cru = self.value()?;
                    if crate::expr::eh_caminho(&cru) && !self.starts_with("(") {
                        self.avisos.push((
                            line,
                            "ligacao-nu",
                            "`:attr = nome` está depreciado — o vínculo nos dois sentidos é `::attr = @nome`, e uma via é `:attr = @nome`",
                        ));
                        attrs.push(Attr { name: format!(":{name}"), value: cru, line });
                        self.skip_ws()?;
                        if self.peek() == Some(',') {
                            self.bump();
                        }
                        continue;
                    }
                    self.reset(m);
                }
                let (l0, c0) = (self.line, (self.pos - self.line_start + 1) as u32);
                let raw = self.expression_source(&[',', ')'])?;
                let src = raw.trim().to_string();
                if src.is_empty() {
                    return self.err_hint(
                        format!("esperava a expressão de `{rotulo}`"),
                        "`:disabled = @a && @b`",
                    );
                }
                if let Err(e) = crate::expr::parse(&src) {
                    return Err(Error {
                        line: l0,
                        col: c0 + e.pos as u32,
                        msg: format!("expressão inválida em `{rotulo}`: {}", e.msg),
                        hint: Some("`:` é uma expressão: `:value = @a`, `:disabled = @a || !@b`"),
                    });
                }
                self.uma_via.push(name.clone());
                attrs.push(Attr { name, value: format!("{{{src}}}"), line });
                self.skip_ws()?;
                if self.peek() == Some(',') {
                    self.bump();
                }
                continue;
            }
            // `class = [btn, btn--primary, @extra]` e `size = [1280, 820]`
            // (Frente 10): `[…]` é sempre lista, e a lista vira o texto separado
            // por espaço que o atributo sempre esperou.
            if (name == "class" || LISTAS_DE_PALAVRAS.contains(&name.as_str()))
                && self.peek() == Some('[')
            {
                let v = self.lista_de_itens()?;
                attrs.push(Attr { name, value: v, line });
                self.skip_ws()?;
                if self.peek() == Some(',') {
                    self.bump();
                }
                continue;
            }
            let value = if let Some(v) = self.action_call(&name)? {
                v
            } else {
                match self.expression_value(&name, tag)? {
                    Some(v) => v,
                    None => self.value()?,
                }
            };
            attrs.push(Attr { name, value, line });
            self.skip_ws()?;
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some(')') => {}
                _ => {
                    return self.err_hint(
                        "esperava `,` ou `)` depois do valor",
                        "valor com espaço, vírgula ou parêntese vai entre aspas; atributos se separam por vírgula",
                    );
                }
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
                "um item começa por um nome: `tag(atributos) { filhos }`",
            );
        }
        let name = self.ident();

        // `state { … }` é a declaração de estado local de um `component` — um
        // bloco de `nome = valor`, não um elemento (Frente 2 do `TODO.md`).
        if name == "state" && !self.starts_with("=") {
            let m = self.mark();
            self.skip_ws()?;
            if matches!(self.peek(), Some('{') | Some('(')) {
                return self.state_block(line).map(|n| Item(vec![n]));
            }
            self.reset(m);
        }

        // `nome:` era o atributo solto dentro das chaves; agora é `tag(nome = v)`.
        if self.peek() == Some(':') {
            return self.err_hint(
                format!("`{name}:` — atributo fora dos parênteses"),
                "atributo vai no cabeçalho: `tag(nome = valor, outro = valor)`; as chaves são só dos filhos",
            );
        }

        match name.as_str() {
            "if" => self.if_chain(line).map(Item),
            "each" => self.each(line).map(|n| Item(vec![n])),
            "else" => self.err_hint(
                "`else` sem um `if` antes",
                "o `else` tem de vir logo depois do `}` do `if`: `} else { … }`",
            ),
            _ => self.element(name, line).map(|n| Item(vec![n])),
        }
    }

    /// `tag(a = 1, b = 2) { filhos }` — os parênteses e as chaves são opcionais.
    /// O texto é um atributo como os outros (`text = "…"` num `button`,
    /// `content = "…"` num `text`); só o corpo cru de um `script`/`style`
    /// (`script(lang = python) """…"""`) fica fora dos parênteses.
    fn element(&mut self, tag: String, line: u32) -> Res<Node> {
        // `script` e `style` carregam código de outra linguagem: o corpo é cru —
        // sem `@`, sem escape, sem colapso de espaço (e a emissão não o escapa).
        let raw = matches!(tag.as_str(), "script" | "style");
        // `tag.classe` e `tag#id` não existem: classe e id são atributos como
        // os outros (`class = `, `id = `), e o ponto fica livre para ser só ponto.
        if matches!(self.peek(), Some('.' | '#')) {
            return self.err_hint(
                format!(
                    "`{tag}{}` — classe e id não vão na tag",
                    self.peek().unwrap()
                ),
                "escreva `tag(class = nome)` (ou `id = nome`)",
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

        // Os atributos: `(` encostado na tag, ou depois de espaço.
        let m = self.mark();
        self.skip_ws()?;
        if self.peek() == Some('(') {
            let (attrs, end) = self.attrs(&node.tag)?;
            node.attrs = merge_class_directives(attrs);
            self.avisa_formas_antigas(&node);
            node.end_line = end;
            // `content` duas vezes (sem nome e com nome) é engano, não escolha.
            if node.attrs.iter().filter(|a| a.name == "content").count() > 1 {
                return self.err_hint(
                    format!("`{}` tem o texto duas vezes", node.tag),
                    "use o texto sem nome OU `content = \"…\"`, não os dois",
                );
            }
            self.check_bindings(&node)?;
        } else {
            self.reset(m);
        }

        // O corpo cru de um `script`/`style`: uma string solta depois do cabeçalho.
        let m = self.mark();
        self.skip_ws()?;
        if self.peek() == Some('"') && !raw {
            return self.err_hint(
                format!("texto solto depois de `{}`", node.tag),
                "o texto é um atributo: `button(text = \"Salvar\")`, `text(\"oi\")`",
            );
        }
        if self.peek() == Some('"') {
            let tline = self.line;
            let (value, end) = self.string(raw)?;
            node.text = Some(Text { value, line: tline });
            node.end_line = end;
        } else {
            self.reset(m);
        }

        // Os filhos.
        let m = self.mark();
        self.skip_ws()?;
        if self.peek() == Some('{') {
            self.open.push(node.tag.clone());
            let bloco = self.block();
            self.open.pop();
            let (children, end) = bloco?;
            node.children = children;
            node.end_line = end;
        } else {
            self.reset(m);
        }

        Ok(node)
    }

    /// Anota as formas que o plano da linguagem deprecia (Fase 4): cada uma tem
    /// a forma nova equivalente, e o `glacier fmt --migrate` reescreve.
    fn avisa_formas_antigas(&mut self, node: &Node) {
        let gerados = &self.for_each_gerado;
        let tem = |n: &str| {
            node.attrs
                .iter()
                .any(|a| a.name == n && !(n == ":for_each" && gerados.contains(&a.line)))
        };
        let condicional = tem("if") || tem("else_if") || tem("else-if");
        if node.tag == "template" && (condicional || tem("else")) {
            self.avisos.push((
                node.line,
                "template-if",
                "`template(if = …)` está depreciado — use o bloco `if cond { … } else { … }`",
            ));
        }
        if node.tag == "template" && tem(":for_each") {
            self.avisos.push((
                node.line,
                "template-for-each",
                "`template(:for_each = x, var = v)` está depreciado — use o bloco `each @x as v { … }`",
            ));
        } else if tem(":for_each") {
            self.avisos.push((
                node.line,
                "for-each-no",
                "`:for_each = x, var = v` está depreciado — use `each = @x as v`",
            ));
        }
        // `matches` (busca por palavras, sem caixa) não tem equivalente na
        // linguagem de expressão: continua sendo a forma, e não é avisado.
        const COMPARADORES: &[&str] = &[
            "equals", "eq", "not_equals", "ne", "one_of", "equals_any", "contains", "has", "empty",
            "not_empty",
        ];
        if condicional && node.attrs.iter().any(|a| COMPARADORES.contains(&a.name.as_str())) {
            self.avisos.push((
                node.line,
                "comparador",
                "`if = @x, equals = y` (e not_equals, one_of, contains, empty) está depreciado — use a expressão: `if = @x == \"y\"`, `if = @x in [\"a\", \"b\"]`, `if = @lista is empty`",
            ));
        }
        // Frente 6: ação é SEMPRE chamada. O nome solto (`on_click = salvar`)
        // parece um valor, e o `prefixo:arg` (`window:minimize`, `field:nome`) é a
        // mesma coisa escrita de outro jeito — `glacier fmt --migrate` troca todos
        // por `nome()`, e numa `feat!` o nome nu passa a ser erro de parse.
        for a in &node.attrs {
            if a.name.starts_with("on_") {
                let v = a.value.trim();
                if !v.is_empty() && !v.contains('(') && !v.starts_with('{') {
                    self.avisos.push((
                        a.line,
                        "acao-nua",
                        "`on_x = nome` está depreciado — ação é sempre chamada: `on_x = nome()` (`window:minimize` → `window.minimize()`, `field:nome` → `field(\"nome\")`)",
                    ));
                }
            }
            // Frente 10: `{chave|padrão}` é o único lugar em que o `|` é outra
            // coisa que `||`; a forma nova é a expressão com `??`.
            if tem_padrao_barra(&a.value) {
                self.avisos.push((
                    a.line,
                    "padrao-barra",
                    "`{chave|padrão}` está depreciado — use a expressão: `@chave ?? \"padrão\"`",
                ));
            }
            // Frente 1: um bool testado como texto.
            if a.value.contains("== \"true\"")
                || a.value.contains("!= \"true\"")
                || a.value.contains("== \"false\"")
                || a.value.contains("!= \"false\"")
            {
                self.avisos.push((
                    a.line,
                    "bool-como-texto",
                    "`@x == \"true\"` testa texto: com `app(truthy = typed)` o teste é `@x` (e o contrário, `!@x`)",
                ));
            }
        }
    }

    /// Uma ligação (ver [`crate::bindings`]) tem de levar o `:`: sem ele, `value =
    /// progresso` é o texto "progresso" e `value = @progresso` o valor da chave,
    /// e o widget que queria o nome dela falha em silêncio.
    fn check_bindings(&self, node: &Node) -> Res<()> {
        // O `check` de uma `tray` é um item de bandeja: o `checked` dele é o
        // valor (`@__notifications`), não o nome de uma chave.
        let na_bandeja = self
            .open
            .last()
            .is_some_and(|t| t.eq_ignore_ascii_case("tray"));
        if !self.strict || na_bandeja {
            return Ok(());
        }
        for a in &node.attrs {
            if a.name.starts_with(':') || self.uma_via_escrita(&a.name) {
                continue;
            }
            if !crate::bindings::is_binding(&node.tag, &a.name) {
                continue;
            }
            return Err(Error {
                line: a.line,
                col: 1,
                msg: format!(
                    "`{} = …` em `{}` é uma ligação e precisa do `:`",
                    a.name, node.tag
                ),
                hint: Some(
                    "escreva `:nome = chave`: o `:` diz que o valor é o NOME de uma chave do contexto (ligação nos dois sentidos); sem ele o valor é um texto, e `@chave` é o valor dela",
                ),
            });
        }
        Ok(())
    }

    // — condicional —

    /// `if COND { … }` (o `if` já consumido), com os `else if`/`else` que o
    /// seguem. Cada ramo vira um `template` irmão — é assim que o XML interno os
    /// representa.
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
            let (children, end_line) = self.block()?;
            let attrs = vec![Attr {
                name: "else".into(),
                value: String::new(),
                line: l,
            }];
            chain.push(Node {
                tag: "template".into(),
                attrs,
                text: None,
                children,
                line: l,
                end_line,
            });
            return Ok(chain);
        }
    }

    /// Um ramo com condição: `COND { … }`.
    fn branch(&mut self, key: &str, line: u32) -> Res<Node> {
        self.skip_ws()?;
        let attrs = self.condition(key, line)?;
        if self.peek() != Some('{') {
            return self.err("esperava `{` depois da condição");
        }
        let (children, end_line) = self.block()?;
        Ok(Node {
            tag: "template".into(),
            attrs,
            text: None,
            children,
            line,
            end_line,
        })
    }

    /// `COND` de um `if`/`else if`: uma **expressão** (`src/expr.rs`) — `@a && !@b`,
    /// `@n > 80`, `@tipo in ["int", "double"]`, `@lista is empty`. Vai até o `{`
    /// que abre o bloco (fora de aspas, parênteses e colchetes; a condição pode
    /// quebrar linha) e vira o atributo `key = "{expr}"`, que o `process_tpl`
    /// avalia. Um erro de sintaxe é erro de parse, apontado na coluna certa.
    /// `key` é `if` ou `else_if`.
    fn condition(&mut self, key: &str, line: u32) -> Res<Vec<Attr>> {
        let (l0, c0) = (self.line, (self.pos - self.line_start + 1) as u32);
        let raw = self.expression_source(&['{'])?;
        let src = raw.trim();
        if src.is_empty() {
            return self.err_hint(
                "esperava a condição do `if`",
                "if @aba == \"x\" { … }  ·  if @a && !@b { … }  ·  if @lista is empty { … }",
            );
        }
        if let Err(e) = crate::expr::parse(src) {
            return Err(Error {
                line: l0,
                col: c0 + e.pos as u32,
                msg: format!("condição inválida: {}", e.msg),
                hint: Some(
                    "a condição é uma expressão: `@a && !@b`, `@n > 80`, `@x == \"y\"`, `@t in [\"a\", \"b\"]`, `@lista is empty`",
                ),
            });
        }
        Ok(vec![Attr {
            name: key.into(),
            value: format!("{{{src}}}"),
            line,
        }])
    }

    /// O texto-fonte de uma expressão, até o primeiro caractere de `fim` fora de
    /// aspas, parênteses e colchetes (sem consumi-lo). Comentários `//` e `/* */`
    /// no meio saem; a quebra de linha vira espaço.
    fn expression_source(&mut self, fim: &[char]) -> Res<String> {
        let mut raw = String::new();
        let mut prof = 0i32;
        let mut aspas: Option<char> = None;
        loop {
            let Some(c) = self.peek() else {
                return self.err("a expressão acabou no fim do arquivo");
            };
            if let Some(q) = aspas {
                if c == '\\' {
                    raw.push(c);
                    self.bump();
                    if let Some(n) = self.bump() {
                        raw.push(n);
                    }
                    continue;
                }
                if c == q {
                    aspas = None;
                }
                raw.push(c);
                self.bump();
                continue;
            }
            if prof == 0 && fim.contains(&c) {
                return Ok(raw);
            }
            if self.starts_with("//") || self.starts_with("/*") {
                self.skip_ws()?;
                raw.push(' ');
                continue;
            }
            match c {
                '"' | '\'' => aspas = Some(c),
                '(' | '[' => prof += 1,
                ')' | ']' => prof -= 1,
                _ => {}
            }
            raw.push(if c == '\n' { ' ' } else { c });
            self.bump();
        }
    }

    /// Uma **ação com argumentos**: `nome(arg, arg)`, cada argumento uma
    /// expressão. Vira a string de ação `nome({json(arg, arg)})`, que o motor
    /// avalia como qualquer texto do template (`nome(["x",true])`) e o despacho
    /// Luau desfaz em argumentos separados, com o tipo do JSON. `a.b(x)` é a
    /// forma de chamada dos handlers embutidos com prefixo (`window.resize("nw")`)
    /// e vira a forma antiga deles, `a:b:x`.
    ///
    /// Só quando o nome não é uma função da linguagem de expressão (`t(…)`,
    /// `len(…)` são valores, não ações).
    fn action_call(&mut self, name: &str) -> Res<Option<String>> {
        if self.literal || name.starts_with(':') {
            return Ok(None);
        }
        let m = self.mark();
        let mut nome = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '.' {
                nome.push(c);
                self.bump();
            } else {
                break;
            }
        }
        let valido = !nome.is_empty()
            && !nome.starts_with('.')
            && !nome.ends_with('.')
            && !nome.chars().next().is_some_and(|c| c.is_ascii_digit())
            && !crate::expr::FUNCOES.contains(&nome.as_str());
        if !valido || self.peek() != Some('(') {
            self.reset(m);
            return Ok(None);
        }
        let (l0, c0) = (self.line, (self.pos - self.line_start + 1) as u32);
        self.bump(); // `(`
        let raw = self.expression_source(&[')'])?;
        self.bump(); // `)`
        let args = split_args(&raw);
        for a in &args {
            if let Err(e) = crate::expr::parse(a) {
                return Err(Error {
                    line: l0,
                    col: c0,
                    msg: format!("argumento inválido em `{nome}(…)`: `{a}`: {}", e.msg),
                    hint: Some(
                        "cada argumento da ação é uma expressão: `salvar(@item.id, true)`, `abrir(\"logs\")`",
                    ),
                });
            }
        }
        // `a.b(x)` (embutido com prefixo, `window.resize("nw")`) e os embutidos
        // que o motor trata em Rust pelo prefixo `nome:` (`textarea_end("log")`):
        // os dois voltam à forma `a:b:x` que o despacho em Rust reconhece. Sem
        // isto, `textarea_end("log")` virava a ação `textarea_end(["log"])`, que
        // nenhum prefixo casa — e o botão não fazia nada, sem erro.
        if nome.contains('.') || ACOES_EMBUTIDAS_COM_PREFIXO.contains(&nome.as_str()) {
            let mut out = nome.replace('.', ":");
            for a in &args {
                out.push(':');
                out.push_str(&format!("{{{a}}}"));
            }
            return Ok(Some(out));
        }
        Ok(Some(format!("{nome}({{json({})}})", args.join(", "))))
    }

    /// O valor de um atributo quando ele é uma **expressão**: começa por `@`, `!`,
    /// `(`, `[` ou por uma função da linguagem (`t(`, `len(`…) e não é só uma
    /// `@chave` — que continua pelo caminho de sempre (`@x` → `{x}`). Devolve
    /// `{expr}` (avaliado pelo `process_tpl`), ou `None` para o valor comum.
    ///
    /// Ações (`on_*`) e ligações (`:nome`) ficam de fora: o valor delas é um nome
    /// de handler ou de chave, não um dado.
    fn expression_value(&mut self, name: &str, tag: &str) -> Res<Option<String>> {
        if self.literal || name.starts_with(':') || name.starts_with("on_") {
            return Ok(None);
        }
        let comeca_funcao = crate::expr::FUNCOES.iter().any(|f| {
            self.starts_with(f) && self.peek_at(f.chars().count()) == Some('(')
        });
        if !matches!(self.peek(), Some('@' | '!' | '(' | '[')) && !comeca_funcao {
            return Ok(None);
        }
        let m = self.mark();
        let (l0, c0) = (self.line, (self.pos - self.line_start + 1) as u32);
        let raw = self.expression_source(&[',', ')'])?;
        let src = raw.trim();
        // Só `@chave` (ou `@{chave}`): o valor comum, que já sabe interpolar.
        if crate::expr::eh_caminho(src) || src.starts_with("@{") {
            self.reset(m);
            return Ok(None);
        }
        if let Err(e) = crate::expr::parse(src) {
            return Err(Error {
                line: l0,
                col: c0 + e.pos as u32,
                msg: format!("expressão inválida em `{name}`: {}", e.msg),
                hint: Some(
                    "um valor que começa com `@`, `!`, `(` ou `[` é uma expressão (`@a && !@b`, `@n > 0 ? \"x\" : \"y\"`); texto vai entre aspas",
                ),
            });
        }
        // Frente 10: quem escreve `:` diz que é expressão; sem ele, o nome nu é
        // texto e `@chave` é o valor da chave. Enquanto a heurística continua
        // aceitando o composto, ele é avisado — menos o conteúdo posicional
        // (`text(@ok ? "sim" : "não")`), que é expressão por definição.
        if posicional(tag) != Some(name) {
            self.avisos.push((
                l0,
                "expressao-sem-dos-pontos",
                "atributo com expressão sem `:` está depreciado — `:disabled = @a && @b` é uma via, `::value = @x` nos dois sentidos",
            ));
        }
        Ok(Some(format!("{{{src}}}")))
    }

    /// `state { aberto = false, itens = [] }` — o estado local de um `component`
    /// (Frente 2 do `TODO.md`). Cada linha vira um `prop(nome, default = …,
    /// type = …, estado)` do contrato do componente; `fundir_estado` junta essas
    /// declarações ao `props` escrito à mão. A forma explícita
    /// (`state { prop(name = aberto, type = bool, default = false) }`) também vale.
    fn state_block(&mut self, line: u32) -> Res<Node> {
        let mut decls: Vec<Node> = Vec::new();
        if self.peek() == Some('(') {
            let (attrs, _) = self.attrs("state")?;
            for a in attrs {
                decls.push(estado_decl(&a.name, &a.value, a.line));
            }
        } else {
            self.bump(); // `{`
            loop {
                self.skip_ws()?;
                match self.peek() {
                    None => return self.err("este `{` do `state` nunca é fechado"),
                    Some('}') => {
                        self.bump();
                        break;
                    }
                    _ => {}
                }
                let l = self.line;
                let nome = self.ident();
                if nome.is_empty() {
                    return self.err_hint(
                        "esperava `nome = valor` dentro de `state { … }`",
                        "state { aberto = false, filtros = [] }",
                    );
                }
                self.skip_ws()?;
                if nome == "prop" && self.peek() == Some('(') {
                    let (mut attrs, _) = self.attrs("prop")?;
                    attrs.push(Attr { name: "estado".into(), value: String::new(), line: l });
                    decls.push(Node {
                        tag: "prop".into(),
                        attrs,
                        text: None,
                        children: Vec::new(),
                        line: l,
                        end_line: l,
                    });
                } else if self.peek() == Some('=') {
                    self.bump();
                    self.skip_ws()?;
                    let valor = self.value()?;
                    decls.push(estado_decl(&nome, &valor, l));
                } else {
                    return self.err_hint(
                        format!("`{nome}` dentro de `state` precisa de um valor inicial"),
                        "state { aberto = false }  ·  state { texto = \"\" }",
                    );
                }
                self.skip_ws()?;
                if self.peek() == Some(',') {
                    self.bump();
                }
            }
        }
        let fim = self.line;
        Ok(Node {
            tag: "state".into(),
            attrs: Vec::new(),
            text: None,
            children: decls,
            line,
            end_line: fim,
        })
    }

    /// A lista de itens de um atributo de palavras (`class`, `size`, `padding`):
    /// nomes, números, `@chaves` e strings, unidos por espaço.
    fn lista_de_itens(&mut self) -> Res<String> {
        let (l0, c0) = (self.line, (self.pos - self.line_start + 1) as u32);
        let raw = self.expression_source(&[']'])?;
        if self.peek() != Some(']') {
            return self.err("faltou o `]` na lista");
        }
        self.bump();
        let mut out = String::new();
        for item in split_args(&raw) {
            let i = item.trim();
            if i.is_empty() {
                continue;
            }
            let pedaco = if let Some(k) = i.strip_prefix('@') {
                if !crate::expr::eh_caminho(k) {
                    return Err(Error {
                        line: l0,
                        col: c0,
                        msg: format!("`{i}` na lista de classes não é uma chave"),
                        hint: Some("class = [btn, btn--primary, @classe_extra]"),
                    });
                }
                format!("{{{k}}}")
            } else if i.starts_with('"') {
                i.trim_matches('"').to_string()
            } else if i.chars().all(|c| Self::is_ident(c) || c == '.' || c.is_ascii_digit()) {
                i.to_string()
            } else {
                return Err(Error {
                    line: l0,
                    col: c0,
                    msg: format!("`{i}` não é um nome de classe"),
                    hint: Some("class = [btn, btn--primary, @classe_extra]"),
                });
            };
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&pedaco);
        }
        Ok(out)
    }

    /// Uma `::attr = …` de uma via foi escrita de propósito: o `:` ali é o de
    /// expressão, não o de ligação, e o `check_bindings` não pode pedir outro.
    fn uma_via_escrita(&self, nome: &str) -> bool {
        self.uma_via.iter().any(|n| n == nome)
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
            Some('"') => {
                self.string(false)?;
                Ok(Operand::Lit)
            }
            Some(c) if Self::is_ident(c) => {
                self.ident();
                Ok(Operand::Lit)
            }
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
                || (c == '.'
                    && self
                        .peek_at(1)
                        .is_some_and(|n| n.is_alphanumeric() || n == '_'));
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
            Operand::Lit => {
                return self.err_hint("`each` quer uma `@lista`", "each @servicos as s { … }");
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
        // Os atributos do `foreach` (`fallback`, `on_reorder`…) vão entre
        // parênteses, depois da variável: `each @itens as item(fallback = X) { … }`.
        let mut body_attrs = Vec::new();
        if self.peek() == Some('(') {
            body_attrs = self.attrs("")?.0;
            // `key = id`: a identidade estável do item — hoje, a chave do arrasto
            // (`reorder_key`), que é onde o motor usa identidade por item.
            for a in &mut body_attrs {
                if a.name == "key" {
                    a.name = "reorder_key".into();
                }
            }
            self.skip_ws()?;
        }
        if self.peek() != Some('{') {
            return self.err("esperava `{` depois do `each`");
        }
        let (children, end_line) = self.block()?;
        let mut attrs = vec![
            Attr {
                name: "items".into(),
                value: items,
                line,
            },
            Attr {
                name: "var".into(),
                value: var,
                line,
            },
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

    /// O valor de um `nome =`. Entre aspas, ou nu até o espaço, a `,` ou o `)` —
    /// e um `//` no meio de um nu é comentário (a primeira armadilha da spec).
    fn value(&mut self) -> Res<String> {
        match self.peek() {
            // `"""…"""` é prosa: o espaço em branco colapsa como num texto
            // HTML, então a quebra de linha é livre.
            Some('"') if self.starts_with("\"\"\"") => Ok(collapse(&self.string(false)?.0)),
            // `l"""…"""` são LINHAS: as quebras ficam e a indentação comum sai.
            // Não é "raw" (o `@nome` continua interpolando): o `l` só avisa que a
            // quebra de linha é conteúdo, ao contrário do `"""` acima.
            Some('l') if self.starts_with("l\"\"\"") => {
                self.bump(); // o `l`
                let corpo = lines_block(&self.string(true)?.0);
                Ok(self.interpola(&corpo))
            }
            Some('"') => Ok(self.string(false)?.0),
            None | Some(',' | ')') => self.err("esperava um valor depois do `=`"),
            _ => {
                let mut raw = String::new();
                while let Some(c) = self.peek() {
                    if c.is_whitespace() || c == ',' || c == ')' || self.starts_with("//") {
                        break;
                    }
                    raw.push(c);
                    self.bump();
                }
                Ok(self.interpola(&raw))
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
        Ok((self.interpola(&raw), self.line))
    }
}

/// O tipo que um literal de `state { … }` declara.
fn tipo_do_valor(v: &str) -> &'static str {
    let t = v.trim();
    if t == "true" || t == "false" {
        "bool"
    } else if !t.is_empty() && t.parse::<f64>().is_ok() {
        "number"
    } else if t.starts_with('[') {
        "list"
    } else if t.starts_with('{') {
        "object"
    } else {
        "string"
    }
}

/// Um `nome = valor` de `state { … }` vira a `prop` marcada `estado` do contrato.
fn estado_decl(nome: &str, valor: &str, line: u32) -> Node {
    Node {
        tag: "prop".into(),
        attrs: vec![
            Attr { name: "name".into(), value: nome.to_string(), line },
            Attr { name: "default".into(), value: valor.to_string(), line },
            Attr { name: "type".into(), value: tipo_do_valor(valor).into(), line },
            Attr { name: "estado".into(), value: String::new(), line },
        ],
        text: None,
        children: Vec::new(),
        line,
        end_line: line,
    }
}

/// Junta o `state { … }` de um componente ao `props` dele — estado local e prop
/// compartilham o mesmo contrato no parser, e o que muda é só quem escreve: a
/// prop vem de quem usa, o estado é do componente (Frente 2 do `TODO.md`).
fn fundir_estado(nodes: &mut Vec<Node>) {
    for n in nodes {
        if n.children.is_empty() {
            continue;
        }
        let estados: Vec<Node> = n
            .children
            .iter()
            .filter(|c| c.tag == "state")
            .flat_map(|c| c.children.iter().cloned())
            .collect();
        // No lugar do `state` que havia, para o emissor continuar escrevendo cada
        // linha na linha do arquivo (ver o doc do módulo).
        let pos = n.children.iter().position(|c| c.tag == "state");
        n.children.retain(|c| c.tag != "state");
        if !estados.is_empty() {
            match n.children.iter_mut().find(|c| c.tag == "props") {
                Some(p) => p.children.extend(estados),
                None => {
                    let line = estados[0].line;
                    let props = Node {
                        tag: "props".into(),
                        attrs: Vec::new(),
                        text: None,
                        children: estados,
                        line,
                        end_line: line,
                    };
                    let i = pos.unwrap_or(0).min(n.children.len());
                    n.children.insert(i, props);
                }
            }
        }
        fundir_estado(&mut n.children);
    }
}

/// O que pode vir no lugar da lista de um `each`: uma `@chave` ou um literal
/// (que é erro).
enum Operand {
    Ref(String),
    Lit,
}


/// `{chave|padrão}`: um buraco cujo conteúdo tem um `|` que não é `||`. É o único
/// lugar da linguagem em que o `|` não é o ou, e a forma nova é a expressão com
/// `??` (Frente 10 do `TODO.md`).
fn tem_padrao_barra(v: &str) -> bool {
    let cs: Vec<char> = v.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] != '{' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        let mut aspas: Option<char> = None;
        while j < cs.len() {
            match (aspas, cs[j]) {
                (None, '}') => break,
                (None, q @ ('"' | '\'')) => aspas = Some(q),
                (Some(q), c) if c == q => aspas = None,
                (None, '|') if cs.get(j + 1) != Some(&'|') && j > i + 1 && cs[j - 1] != '|' => {
                    return true
                }
                _ => {}
            }
            j += 1;
        }
        i = (j + 1).min(cs.len());
    }
    false
}

/// Os atributos que aceitam `[…]` no lugar do texto separado por espaço
/// (Frente 10 do `TODO.md`): os pares de tamanho e a lista de classes, que é o
/// caso mais escrito. O `class` não está aqui porque já é tratado por nome.
const LISTAS_DE_PALAVRAS: &[&str] = &[
    "size",
    "min_size",
    "max_size",
    "fixed_size",
    "padding",
    "margem",
    "anchor_rect",
];

/// Os argumentos de uma chamada, pela vírgula de fora de aspas, parênteses e
/// colchetes. Vazio → nenhum argumento.
fn split_args(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut atual = String::new();
    let mut prof = 0i32;
    let mut aspas: Option<char> = None;
    for c in raw.chars() {
        if let Some(q) = aspas {
            if c == q {
                aspas = None;
            }
            atual.push(c);
            continue;
        }
        match c {
            '"' | '\'' => aspas = Some(c),
            '(' | '[' => prof += 1,
            ')' | ']' => prof -= 1,
            ',' if prof == 0 => {
                out.push(std::mem::take(&mut atual).trim().to_string());
                continue;
            }
            _ => {}
        }
        atual.push(c);
    }
    let fim = atual.trim().to_string();
    if !fim.is_empty() || !out.is_empty() {
        out.push(fim);
    }
    out
}

/// O `else` (e o `else_if`) **como atributo** vale para o irmão imediatamente
/// anterior, que precisa ter `if =` ou `else_if =`. Antes, um nó inserido entre
/// os dois mudava o sentido do `else` sem erro nenhum; agora a tela não
/// carrega, e o erro diz onde está o `else` e o que veio antes dele.
fn check_else_positions(irmaos: &[Node]) -> Res<()> {
    let tem = |n: &Node, nomes: &[&str]| n.attrs.iter().any(|a| nomes.contains(&a.name.as_str()));
    for (i, n) in irmaos.iter().enumerate() {
        let e_else = tem(n, &["else"]);
        let e_else_if = tem(n, &["else_if", "else-if"]);
        if !e_else && !e_else_if {
            continue;
        }
        let anterior = i.checked_sub(1).map(|j| &irmaos[j]);
        if anterior.is_some_and(|a| tem(a, &["if", "else_if", "else-if"])) {
            continue;
        }
        let qual = if e_else { "else" } else { "else_if" };
        let msg = match anterior {
            Some(a) => format!(
                "`{qual}` precisa vir logo depois de um irmão com `if =`; o irmão anterior (linha {}) é um `{}` sem condição",
                a.line, a.tag
            ),
            None => format!("`{qual}` precisa vir logo depois de um irmão com `if =`, e é o primeiro do bloco"),
        };
        return Err(Error {
            line: n.line,
            col: 1,
            msg,
            hint: Some(
                "mova o `else` para logo depois do nó com `if =`, ou use um bloco: `if cond { … } else { … }`",
            ),
        });
    }
    Ok(())
}

/// Junta as classes condicionais (`class:nome = expr`) ao `class` do elemento:
/// cada uma vira `{(expr) ? 'nome' : ''}` no fim da lista, que o motor já
/// interpola. Sem `class` no elemento, ela é criada.
fn merge_class_directives(attrs: Vec<Attr>) -> Vec<Attr> {
    if !attrs.iter().any(|a| a.name.starts_with("class:")) {
        return attrs;
    }
    let mut extra = String::new();
    let mut linha = None;
    let mut resto = Vec::with_capacity(attrs.len());
    for a in attrs {
        match a.name.strip_prefix("class:") {
            Some(classe) => {
                extra.push_str(&format!(" {{({}) ? '{classe}' : ''}}", a.value));
                linha.get_or_insert(a.line);
            }
            None => resto.push(a),
        }
    }
    match resto.iter_mut().find(|a| a.name == "class") {
        Some(c) => c.value.push_str(&extra),
        None => resto.push(Attr {
            name: "class".into(),
            value: extra.trim_start().to_string(),
            line: linha.unwrap_or(0),
        }),
    }
    resto
}

/// Prosa de um `"""`: sem espaço nas pontas, e cada sequência de espaço em branco
/// vira um espaço só (o `&nbsp;` fica), como `UiNode::normalize_text`.
fn collapse(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending = false;
    for c in s.chars() {
        if c == '\u{A0}' {
            out.push(' ');
            pending = false;
        } else if c.is_whitespace() {
            pending = true;
        } else {
            if pending && !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
            pending = false;
            out.push(c);
        }
    }
    out
}

/// O corpo de um `l"""`: as quebras de linha são conteúdo e a indentação comum sai.
///
/// - a primeira linha some se for só espaço (o texto começa na linha seguinte ao
///   `l"""`), e a última também (a linha do `"""` que fecha, com o recuo dele);
/// - o recuo comum às linhas não vazias é removido — o que vier a mais fica, é o
///   que permite um YAML ou um JSON recuado dentro do texto;
/// - linha em branco fica vazia; `\r\n` vira `\n`.
///
/// Roda sobre o texto **cru**, antes do `@nome` ser interpolado.
fn lines_block(raw: &str) -> String {
    let mut ls: Vec<&str> = raw
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    if ls.first().is_some_and(|l| l.trim().is_empty()) {
        ls.remove(0);
    }
    if ls.last().is_some_and(|l| l.trim().is_empty()) {
        ls.pop();
    }
    let recuo = ls
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);
    ls.iter()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                l.chars().skip(recuo).collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A regra 5: `@nome` / `@{nome}` viram `{nome}`; `@@` é um arroba literal.
/// Um `@` que não precede nome nem `{` fica como está (um e-mail não quebra).
fn interpolate(raw: &str) -> String {
    let cs: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < cs.len() {
        // `{ … }` é expressão (`src/expr.rs`) e fica como está: o `@` lá dentro é
        // da linguagem de expressão, não a regra 5 (virar `{nome}` aninharia
        // chaves). Respeita aspas, para um `'}'` num texto não fechar antes.
        if cs[i] == '{' {
            let mut j = i + 1;
            let mut aspas: Option<char> = None;
            while j < cs.len() {
                match (aspas, cs[j]) {
                    (None, '}') => break,
                    (None, q @ ('"' | '\'')) => aspas = Some(q),
                    (Some(q), c) if c == q => aspas = None,
                    _ => {}
                }
                j += 1;
            }
            let fim = (j + 1).min(cs.len());
            out.extend(&cs[i..fim]);
            i = fim;
            continue;
        }
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
                            && cs
                                .get(j + 1)
                                .is_some_and(|n| n.is_alphanumeric() || *n == '_')))
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
    /// Profundidade de tags **inline** (`prose`, `p`, `b`, `a`…) em que o emissor
    /// está. Dentro delas uma quebra de linha não pode virar texto: ver
    /// [`Out::quebra`].
    inline: u32,
}

impl Default for Out {
    fn default() -> Self {
        Self {
            buf: String::new(),
            line: 1,
            inline: 0,
        }
    }
}

/// Tags cujo conteúdo é texto corrido — o `prose` e tudo o que vive dentro dele. Num
/// `.gv` cada filho fica na própria linha, e no XML a quebra entre dois elementos é um
/// espaço: `b(content = "Ana")` seguido de `span(content = "!")` viraria "Ana !". Quem
/// escreve `.gv` põe o espaço **no texto** (`content = "Olá, "`), então a quebra de
/// formatação não pode contar. (`pre` fica de fora: lá o espaço é conteúdo.)
fn tag_inline(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "prose" | "p" | "b" | "strong" | "negrito" | "i" | "em" | "italico" | "itálico"
            | "code" | "codigo" | "código" | "u" | "underline" | "sublinhado" | "s" | "del"
            | "strike" | "tachado" | "a" | "span"
            | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote" | "ul" | "ol" | "li"
            | "figure" | "figcaption" | "footer" | "cite"
    )
}

/// A que atributo corresponde o texto sem nome de `tag("…")`: `content`, nas tags que
/// **carregam** texto — as folhas (`text`, `markdown`, `b`, `i`, `a`, `span`…) e os
/// blocos de uma linha só (títulos, `li`, `figcaption`, `footer`, `cite`). Os
/// contêineres (`prose`, `p`, `ul`, `ol`, `blockquote`, `figure`) ficam de fora: o texto
/// vive nos filhos deles. Outras (`button` quer `text`, `textinput` quer
/// `placeholder`…) ficam de fora de propósito: lá "o texto" é ambíguo, e
/// `nome = valor` diz qual. O `button` (e o `choice` do `actions` de um `dialog`) entra com `text` (o rótulo);
/// os builtins `kbd` e `alert` entram com `keys` e `title`.
fn posicional(tag: &str) -> Option<&'static str> {
    let t = tag.to_ascii_lowercase();
    if matches!(t.as_str(), "button" | "choice") {
        return Some("text");
    }
    // Builtins de texto curto: `kbd("Ctrl+K")` é a tecla, `alert("Salvo")` é o
    // título do aviso. A forma com nome (`keys =`, `title =`) segue valendo.
    if t == "kbd" {
        return Some("keys");
    }
    if t == "alert" {
        return Some("title");
    }
    let carrega = matches!(t.as_str(), "text" | "texto" | "markdown" | "md")
        || (tag_inline(tag)
            && !matches!(t.as_str(), "prose" | "p" | "ul" | "ol" | "blockquote" | "figure"));
    carrega.then_some("content")
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

    /// Como [`Out::at`], mas para as quebras **entre** nós: dentro de uma tag
    /// inline elas vão dentro de um comentário XML (`<!--\n-->`), que mantém a
    /// contagem de linhas e não vira texto.
    fn quebra(&mut self, line: u32) {
        if self.inline == 0 || self.line >= line {
            return self.at(line);
        }
        let n = (line - self.line) as usize;
        self.buf.push_str("<!--");
        self.buf.push_str(&"\n".repeat(n));
        self.buf.push_str("-->");
        self.line = line;
    }

    fn node(&mut self, n: &Node) {
        self.quebra(n.line);
        self.push(&format!("<{}", n.tag));
        for a in &n.attrs {
            self.at(a.line);
            self.push(&format!(" {}=\"{}\"", a.name, escape(&a.value, true)));
        }
        // `script`/`style` sempre abrem e fecham: o motor procura o ``,
        // então `script { src: "x" }` sem corpo não pode sair como `script`.
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
        let inline = tag_inline(&n.tag);
        if inline {
            self.inline += 1;
        }
        for c in &n.children {
            self.node(c);
        }
        self.quebra(n.end_line);
        if inline {
            self.inline -= 1;
        }
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

// ─────────────────────────── testes da linguagem nova ───────────────────────────
//
// Cobrem as Frentes 1, 2, 6 e 10 do `TODO.md`: o `state`, os prefixos `:`/`::`, a
// classe condicional com ponto, a lista de classes, a chamada de ação e as formas
// antigas que continuam valendo — avisadas.
#[cfg(test)]
mod tests {
    use super::*;

    fn d(src: &str) -> String {
        desugar(src).unwrap_or_else(|e| panic!("{}\n{src}", e.message))
    }

    #[test]
    fn state_vira_prop_marcada_no_props() {
        let x = d(
            r#"component(name = Cartao) {
  state {
    aberto = false,
    total = 0,
    titulo = "",
    itens = [],
  }
  text("@titulo")
}"#,
        );
        assert!(x.contains("<props>"), "{x}");
        assert!(x.contains(r#"name="aberto""#), "{x}");
        assert!(x.contains(r#"default="false""#) && x.contains(r#"type="bool""#), "{x}");
        assert!(x.contains(r#"type="number""#), "{x}");
        assert!(x.contains(r#"type="list""#), "{x}");
        assert!(x.contains(r#"estado="""#), "{x}");
        // O `state` não desenha nada: vira declaração do componente.
        assert!(!x.contains("<state>"), "{x}");
    }

    #[test]
    fn state_se_funde_com_um_props_escrito_a_mao() {
        let x = d(
            r#"component(name = Cartao) {
  props {
    prop(name = rotulo, type = string)
  }
  state { aberto = true }
  text("x")
}"#,
        );
        assert_eq!(x.matches("<props>").count(), 1, "{x}");
        assert!(x.contains(r#"name="rotulo""#) && x.contains(r#"name="aberto""#), "{x}");
    }

    #[test]
    fn dois_pontos_sao_uma_via_e_dobres_sao_o_vinculo() {
        // `:attr = expr` — o atributo recebe o valor da expressão.
        let x = d(r#"progressbar(:value = @a + @b)"#);
        assert!(x.contains(r#"value="{a + b}""#), "{x}");
        assert!(!x.contains(":value"), "{x}");
        // `::attr = @chave` — a ligação nos dois sentidos, pelo NOME.
        let y = d(r#"textinput(::value = @form.nome)"#);
        assert!(y.contains(r#":value="form.nome""#), "{y}");
    }

    #[test]
    fn a_forma_antiga_de_ligacao_ainda_funciona_e_e_avisada() {
        let mut p = Parser::new(r#"textinput(:value = nome)"#);
        let nodes = p.file().unwrap();
        let xml = {
            let mut o = Out::default();
            for n in &nodes {
                o.node(n);
            }
            o.buf
        };
        assert!(xml.contains(r#":value="nome""#), "{xml}");
        assert!(p.avisos.iter().any(|(_, f, _)| *f == "ligacao-nu"), "{:?}", p.avisos);
    }

    #[test]
    fn vinculo_em_campo_que_nao_e_campo_e_erro() {
        let e = desugar(r#"text(::text = @x)"#).unwrap_err();
        assert!(e.message.contains("não é um campo"), "{e}");
    }

    #[test]
    fn classe_condicional_com_ponto() {
        let x = d(r#"row(:class.ativa = @view == @alvo)"#);
        // Fundida ao `class` como sempre, pela porta de trás.
        assert!(x.contains(r#"class="{(@view == @alvo) ? 'ativa' : ''}""#), "{x}");
        // A forma antiga continua valendo, e é avisada.
        let mut p = Parser::new(r#"row(class:ativa = @view)"#);
        p.file().unwrap();
        assert!(p.avisos.iter().any(|(_, f, _)| *f == "class-diretiva"), "{:?}", p.avisos);
    }

    #[test]
    fn lista_de_classes() {
        let x = d(r#"row(class = [btn, btn--primary, @extra]) { text("oi") }"#);
        assert!(x.contains(r#"class="btn btn--primary {extra}""#), "{x}");
    }

    #[test]
    fn acao_nua_e_avisada_e_a_chamada_nao() {
        let mut p = Parser::new("button(\"S\", on_click = salvar)");
        p.file().unwrap();
        assert!(p.avisos.iter().any(|(_, f, _)| *f == "acao-nua"), "{:?}", p.avisos);
        let mut q = Parser::new("button(\"S\", on_click = salvar(@id, true))");
        q.file().unwrap();
        assert!(!q.avisos.iter().any(|(_, f, _)| *f == "acao-nua"), "{:?}", q.avisos);
        // A chamada sem argumento é a forma canônica do `on_click = salvar`.
        let x = d("button(\"S\", on_click = salvar())");
        assert!(x.contains(r#"on_click="salvar()""#), "{x}");
    }

    #[test]
    fn bool_como_texto_e_verdade_tipada_avisadas() {
        for (src, forma) in [
            (r#"text("a", if = @x == "true")"#, "bool-como-texto"),
            (r#"text("a", if = "{x|nenhum"}")"#, "padrao-barra"),
            (r#"button("S", disabled = @a && @b)"#, "expressao-sem-dos-pontos"),
        ] {
            let mut p = Parser::new(src);
            p.file().unwrap_or_else(|e| panic!("{src}: {}", e.msg));
            assert!(p.avisos.iter().any(|(_, f, _)| *f == forma), "{src}: {:?}", p.avisos);
        }
    }

    #[test]
    fn coalescencia_e_lista_no_atributo() {
        let x = d(r#"text(:content = @a ?? "nada")"#);
        assert!(x.contains(r#"content="{@a ?? &quot;nada&quot;}"#), "{x}");
        let y = desugar("row(size = [1280, 820])").unwrap();
        // Par numérico: `[` é expressão pela heurística de sempre, e a lista vira
        // JSON — quem quer o par escrito vai de `size = "1280 820"` ou do `fmt`.
        assert!(y.contains("size="), "{y}");
    }

    #[test]
    fn a_regra_de_ouro_vale_para_as_formas_novas() {
        // Cada linha do `.gv` emite na mesma linha do XML: os erros apontam para o
        // arquivo que o autor escreveu, inclusive com `state` e `::`.
        let src = r#"app(id = x) {
  screen(principal) {
    textinput(
      ::value = @form.nome,
    )
    state {
      aberto = false,
    }
  }
}"#;
        let xml = d(src);
        assert_eq!(xml.matches('\n').count(), src.matches('\n').count(), "{xml}");
        assert!(xml.contains(":value=\"form.nome\""), "{xml}");
    }

    #[test]
    fn desugar_nao_perde_as_formas_de_sempre() {
        let x = d(
            r#"each @itens as i(class = linha) {
  button("Apagar", on_click = apagar(@i.id))
  text("{@i.nome}", if = @i.visivel && !@i.apagando)
}"#,
        );
        assert!(x.contains(r#"on_click="apagar({json(@i.id)})""#), "{x}");
        assert!(x.contains("foreach"), "{x}");
    }
}
