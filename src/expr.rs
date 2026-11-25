//! A linguagem de expressão do `.gv` (Fase 1 do `docs/PLANO_LINGUAGEM_GV_GSS.md`).
//!
//! Vale em três lugares: na condição de um bloco (`if @a && !@b { … }`), no valor de
//! um atributo (`disabled = @busy || !@valido`) e dentro de `{ … }` em qualquer
//! string (`text("{@cpu > 80 ? '⚠ ' : ''}@cpu")`). Os três chegam aqui pelo mesmo
//! caminho: o `.gv` emite a expressão entre chaves, e o `process_tpl` do `eval.rs`
//! a reconhece.
//!
//! ```text
//! expr    := ternary
//! ternary := default ( '?' expr ( ':' expr )? )?
//! default := or ( '|' texto-livre )?           (só na forma `{chave|padrão}`)
//! or      := and ( '||' and )*
//! and     := not ( '&&' not )*
//! not     := '!' not | cmp
//! cmp     := primary ( op primary )?
//!          | primary 'in' primary
//!          | primary 'is' 'not'? 'empty'
//! op      := '==' | '!=' | '<' | '<=' | '>' | '>='
//! primary := '@'? caminho | string | número | true | false | null
//!          | '[' expr ( ',' expr )* ']' | chamada | '(' expr ')'
//! chamada := nome '(' ( expr | nome '=' expr ) ( ',' … )* ')'
//! ```
//!
//! # Tipos
//!
//! O contexto guarda strings (o `ctx` tipado põe o tipo **ao lado**, e grava as
//! canônicas: `"true"`/`"false"`, número sem `.0`, JSON compacto). Uma leitura
//! devolve a string, e o tipo é **inferido** na hora da operação, do jeito mais
//! conservador possível:
//!
//! - `==`/`!=`: strings iguais são iguais; senão, se os dois lados são número,
//!   compara número (`"1.0" == 1`); senão, diferentes. Nunca há coerção de texto
//!   em booleano por igualdade.
//! - `<`, `<=`, `>`, `>=`: só número. Com um lado que não é número o resultado é
//!   falso (e o motor avisa uma vez por expressão).
//! - Verdade: falso é ausente, `""`, `false`, `0`, `no`, `off`, `null`, `[]` e
//!   `{}`; todo o resto é verdadeiro.
//! - `is empty`: ausente, `""`, `null`, `[]` e `{}`. Um texto qualquer **não** é
//!   vazio (o `empty` antigo achava que era: só entendia array).
//! - `in`: a lista pode ser literal (`["a", "b"]`), um array JSON do contexto ou um
//!   texto separado por vírgula, ponto e vírgula ou espaço.
//!
//! # Desempenho
//!
//! O parse acontece uma vez por texto de expressão (cache por thread em
//! [`parse_cached`]); a avaliação percorre a árvore lendo o contexto por uma
//! closure — que, no motor, é o `EvalCtx::get`, e por isso **registra cada
//! dependência**. Um ramo não avaliado (curto-circuito) não registra as chaves
//! dele, o que é correto: se o lado que decidiu mudar, a reavaliação acontece
//! por ele.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Uma expressão já lida.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal: string, número, `true`/`false`/`null`.
    Lit(Valor),
    /// Leitura de uma chave do contexto (`@x`, `x`, `@item.campo`).
    Ref(String),
    /// Lista literal `[a, b]`.
    Lista(Vec<Expr>),
    Nao(Box<Expr>),
    E(Box<Expr>, Box<Expr>),
    Ou(Box<Expr>, Box<Expr>),
    Cmp(Op, Box<Expr>, Box<Expr>),
    /// `item in lista`.
    Em(Box<Expr>, Box<Expr>),
    /// `x is empty` (`false`) / `x is not empty` (`true`).
    Vazio(Box<Expr>, bool),
    /// `c ? a : b` (sem `: b`, o falso dá `""`).
    Se(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    /// `{chave|padrão}`: a chave, ou o texto quando ela está ausente.
    Padrao(String, String),
    /// `nome(args, nomeado = expr)`.
    Chamada(String, Vec<Expr>, Vec<(String, Expr)>),
    /// Texto entre aspas com `@chave` dentro (`"@tab.id"`, `"v@versao"`): os
    /// pedaços literais e as chaves lidas, concatenados. Um texto entre aspas
    /// interpola em todo lugar do `.gv`, e dentro de uma expressão também —
    /// `if @atual == "@alvo"` compara com o VALOR de `alvo` (era assim antes da
    /// 0.169, quando o `if` de bloco virava `equals = "@alvo"`). `@@` é o `@`.
    Interp(Vec<Parte>),
}

/// Um pedaço de [`Expr::Interp`].
#[derive(Debug, Clone, PartialEq)]
pub enum Parte {
    Texto(String),
    Chave(String),
}

/// Quebra o conteúdo de um texto entre aspas em pedaços, se ele tiver alguma
/// `@chave`; `None` quando é só texto (o `@@` já vira `@`).
fn interpola(s: &str) -> Result<Expr, String> {
    let cs: Vec<char> = s.chars().collect();
    let mut partes = Vec::new();
    let mut txt = String::new();
    let mut tem_chave = false;
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '@' {
            if cs.get(i + 1) == Some(&'@') {
                txt.push('@');
                i += 2;
                continue;
            }
            // O mesmo nome de chave de `@x` fora das aspas (ver `caminho`).
            let mut k = String::new();
            let mut j = i + 1;
            while j < cs.len() {
                let c = cs[j];
                let continua = cs.get(j + 1).is_some_and(|n| n.is_alphanumeric() || *n == '_');
                let ok = c.is_alphanumeric() || c == '_' || ((c == '.' || c == '-') && continua && !k.is_empty());
                if !ok {
                    break;
                }
                k.push(c);
                j += 1;
            }
            if !k.is_empty() && !k.starts_with(|c: char| c.is_ascii_digit()) {
                if !txt.is_empty() {
                    partes.push(Parte::Texto(std::mem::take(&mut txt)));
                }
                partes.push(Parte::Chave(k));
                tem_chave = true;
                i = j;
                continue;
            }
        }
        txt.push(cs[i]);
        i += 1;
    }
    if !tem_chave {
        return Err(txt);
    }
    if !txt.is_empty() {
        partes.push(Parte::Texto(txt));
    }
    Ok(Expr::Interp(partes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Igual,
    Diferente,
    Menor,
    MenorIgual,
    Maior,
    MaiorIgual,
}

/// O valor de uma expressão.
#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    /// Chave ausente / `null`.
    Nulo,
    Bool(bool),
    Num(f64),
    /// Texto — inclusive o que veio do contexto, cujo tipo é inferido na hora.
    Texto(String),
    Lista(Vec<Valor>),
}

impl Valor {
    /// A string que o valor vira quando sai da expressão (atributo, texto).
    pub fn texto(&self) -> Cow<'_, str> {
        match self {
            Valor::Nulo => Cow::Borrowed(""),
            Valor::Bool(true) => Cow::Borrowed("true"),
            Valor::Bool(false) => Cow::Borrowed("false"),
            Valor::Num(n) => Cow::Owned(formata_num(*n)),
            Valor::Texto(s) => Cow::Borrowed(s),
            Valor::Lista(itens) => {
                let json: Vec<serde_json::Value> = itens.iter().map(Valor::json).collect();
                Cow::Owned(serde_json::Value::Array(json).to_string())
            }
        }
    }

    fn json(&self) -> serde_json::Value {
        match self {
            Valor::Nulo => serde_json::Value::Null,
            Valor::Bool(b) => serde_json::Value::Bool(*b),
            // Inteiro sai sem `.0` (`3`, não `3.0`) — a mesma forma canônica do `ctx`.
            Valor::Num(n) if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 => {
                serde_json::Value::from(*n as i64)
            }
            Valor::Num(n) => serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Valor::Texto(s) => serde_json::Value::String(s.clone()),
            Valor::Lista(v) => serde_json::Value::Array(v.iter().map(Valor::json).collect()),
        }
    }

    /// O número deste valor, se ele é (ou é um texto que é) um número finito.
    pub fn num(&self) -> Option<f64> {
        match self {
            Valor::Num(n) => Some(*n),
            Valor::Texto(s) => {
                let t = s.trim();
                if t.is_empty() {
                    return None;
                }
                t.parse::<f64>().ok().filter(|n| n.is_finite())
            }
            _ => None,
        }
    }

    /// A regra de verdade (ver o doc do módulo).
    pub fn verdadeiro(&self) -> bool {
        match self {
            Valor::Nulo => false,
            Valor::Bool(b) => *b,
            Valor::Num(n) => *n != 0.0,
            Valor::Lista(v) => !v.is_empty(),
            Valor::Texto(s) => texto_verdadeiro(s),
        }
    }

    /// `is empty`.
    pub fn vazio(&self) -> bool {
        match self {
            Valor::Nulo => true,
            Valor::Lista(v) => v.is_empty(),
            Valor::Texto(s) => matches!(s.trim(), "" | "null" | "[]" | "{}"),
            Valor::Bool(_) | Valor::Num(_) => false,
        }
    }

    /// Os itens deste valor, para o lado direito de um `in`.
    fn itens(&self) -> Vec<Valor> {
        match self {
            Valor::Lista(v) => v.clone(),
            Valor::Texto(s) => {
                let t = s.trim();
                if t.starts_with('[')
                    && let Ok(serde_json::Value::Array(a)) = serde_json::from_str(t)
                {
                    return a.into_iter().map(de_json).collect();
                }
                t.split([',', ';', ' ', '\t', '\n'])
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .map(|p| Valor::Texto(p.to_string()))
                    .collect()
            }
            Valor::Nulo => Vec::new(),
            outro => vec![outro.clone()],
        }
    }
}

/// A verdade de um texto — é também a do `if @x` simples (`eval::is_truthy`).
pub fn texto_verdadeiro(s: &str) -> bool {
    let t = s.trim();
    !(t.is_empty()
        || t.eq_ignore_ascii_case("false")
        || t.eq_ignore_ascii_case("no")
        || t.eq_ignore_ascii_case("nao")
        || t.eq_ignore_ascii_case("não")
        || t.eq_ignore_ascii_case("off")
        || t.eq_ignore_ascii_case("null")
        || t == "[]"
        || t == "{}"
        || t.parse::<f64>().is_ok_and(|n| n == 0.0))
}

fn de_json(v: serde_json::Value) -> Valor {
    match v {
        serde_json::Value::Null => Valor::Nulo,
        serde_json::Value::Bool(b) => Valor::Bool(b),
        serde_json::Value::Number(n) => Valor::Num(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Valor::Texto(s),
        serde_json::Value::Array(a) => Valor::Lista(a.into_iter().map(de_json).collect()),
        outro @ serde_json::Value::Object(_) => Valor::Texto(outro.to_string()),
    }
}

/// Inteiro sem `.0` quando cabe; senão o `f64` mais curto — a mesma forma
/// canônica do `ctx` tipado.
fn formata_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

fn iguais(a: &Valor, b: &Valor) -> bool {
    if a.texto() == b.texto() {
        return true;
    }
    match (a.num(), b.num()) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

// ───────────────────────────── avaliação ─────────────────────────────

/// Avalia `e`, lendo o contexto por `ler`. As chamadas `t`/`num`/`date` vão ao
/// i18n com os argumentos já avaliados.
pub fn avalia(e: &Expr, ler: &mut dyn FnMut(&str) -> Option<String>) -> Valor {
    match e {
        Expr::Lit(v) => v.clone(),
        Expr::Ref(k) => match ler(k) {
            Some(s) => Valor::Texto(s),
            None => Valor::Nulo,
        },
        Expr::Lista(itens) => Valor::Lista(itens.iter().map(|i| avalia(i, ler)).collect()),
        Expr::Nao(x) => Valor::Bool(!avalia(x, ler).verdadeiro()),
        Expr::E(a, b) => Valor::Bool(avalia(a, ler).verdadeiro() && avalia(b, ler).verdadeiro()),
        Expr::Ou(a, b) => Valor::Bool(avalia(a, ler).verdadeiro() || avalia(b, ler).verdadeiro()),
        Expr::Cmp(op, a, b) => {
            let (a, b) = (avalia(a, ler), avalia(b, ler));
            Valor::Bool(match op {
                Op::Igual => iguais(&a, &b),
                Op::Diferente => !iguais(&a, &b),
                _ => match (a.num(), b.num()) {
                    (Some(x), Some(y)) => match op {
                        Op::Menor => x < y,
                        Op::MenorIgual => x <= y,
                        Op::Maior => x > y,
                        _ => x >= y,
                    },
                    _ => false,
                },
            })
        }
        Expr::Em(item, lista) => {
            let item = avalia(item, ler);
            let lista = avalia(lista, ler);
            Valor::Bool(lista.itens().iter().any(|v| iguais(v, &item)))
        }
        Expr::Vazio(x, negado) => Valor::Bool(avalia(x, ler).vazio() != *negado),
        Expr::Se(c, a, b) => {
            if avalia(c, ler).verdadeiro() {
                avalia(a, ler)
            } else {
                match b {
                    Some(b) => avalia(b, ler),
                    None => Valor::Texto(String::new()),
                }
            }
        }
        Expr::Interp(partes) => {
            let mut out = String::new();
            for p in partes {
                match p {
                    Parte::Texto(t) => out.push_str(t),
                    Parte::Chave(k) => out.push_str(&ler(k).unwrap_or_default()),
                }
            }
            Valor::Texto(out)
        }
        Expr::Padrao(k, padrao) => match ler(k) {
            Some(s) => Valor::Texto(s),
            None => Valor::Texto(padrao.clone()),
        },
        // `json(a, b)`: os argumentos COM o tipo (literal `true` é booleano, `3` é
        // número; uma chave do contexto chega como texto) num array JSON — é o
        // que a ação `nome(a, b)` do `.gv` carrega até o handler Luau.
        Expr::Chamada(nome, pos, _) if nome == "json" => {
            let itens: Vec<serde_json::Value> = pos.iter().map(|p| avalia(p, ler).json()).collect();
            Valor::Texto(serde_json::Value::Array(itens).to_string())
        }
        Expr::Chamada(nome, pos, nomeados) => {
            let pos: Vec<String> = pos.iter().map(|p| avalia(p, ler).texto().into_owned()).collect();
            let nomeados: Vec<(String, String)> = nomeados
                .iter()
                .map(|(k, v)| (k.clone(), avalia(v, ler).texto().into_owned()))
                .collect();
            chama(nome, &pos, &nomeados, ler)
        }
    }
}

/// As funções da linguagem. `t`/`num`/`date` são do i18n; as outras, daqui.
fn chama(
    nome: &str,
    pos: &[String],
    nomeados: &[(String, String)],
    ler: &mut dyn FnMut(&str) -> Option<String>,
) -> Valor {
    let primeiro = pos.first().map(String::as_str).unwrap_or("");
    match nome {
        "t" | "num" | "date" => {
            Valor::Texto(crate::i18n::chama(nome, pos, nomeados, ler).unwrap_or_default())
        }
        "len" => Valor::Num(match Valor::Texto(primeiro.to_string()) {
            v if primeiro.trim_start().starts_with('[') => v.itens().len() as f64,
            _ => primeiro.chars().count() as f64,
        }),
        "lower" => Valor::Texto(primeiro.to_lowercase()),
        "upper" => Valor::Texto(primeiro.to_uppercase()),
        "trim" => Valor::Texto(primeiro.trim().to_string()),
        _ => Valor::Nulo,
    }
}

/// As funções que a linguagem conhece — o resto de `nome(` não é chamada.
pub const FUNCOES: &[&str] = &["t", "num", "date", "len", "lower", "upper", "trim", "json"];

// ───────────────────────────── parse ─────────────────────────────

/// Erro de sintaxe: posição (em caracteres, a partir de 0) e mensagem.
#[derive(Debug, Clone, PartialEq)]
pub struct ErroExpr {
    pub pos: usize,
    pub msg: String,
}

impl std::fmt::Display for ErroExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (coluna {})", self.msg, self.pos + 1)
    }
}

/// Lê uma expressão inteira (sobrar texto é erro).
pub fn parse(src: &str) -> Result<Expr, ErroExpr> {
    let mut p = Leitor::novo(src);
    p.espaco();
    if p.fim() {
        return Err(p.erro("expressão vazia"));
    }
    // `{chave|padrão}`: a forma antiga, com o padrão em texto livre. Só quando o
    // `|` não é `||` e o lado esquerdo é um nome simples.
    if let Some((k, d)) = src.split_once('|')
        && !src.contains("||")
        && eh_caminho(k.trim())
    {
        return Ok(Expr::Padrao(k.trim().to_string(), d.trim().to_string()));
    }
    let e = p.ternario()?;
    p.espaco();
    if !p.fim() {
        return Err(p.erro(format!("não esperava `{}` aqui", p.resto_curto())));
    }
    Ok(e)
}

/// `true` se `s` é só um nome de chave (`x`, `item.campo`, `__dialog.message`) —
/// o caminho rápido do `process_tpl`, que nem passa pelo parser.
pub fn eh_caminho(s: &str) -> bool {
    let s = s.strip_prefix('@').unwrap_or(s);
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '-'))
        && !matches!(s, "true" | "false" | "null")
}

thread_local! {
    static CACHE: RefCell<HashMap<Box<str>, Result<Rc<Expr>, ErroExpr>>> =
        RefCell::new(HashMap::new());
}

/// [`parse`] com cache por thread: o mesmo texto de expressão é lido uma vez.
pub fn parse_cached(src: &str) -> Result<Rc<Expr>, ErroExpr> {
    CACHE.with(|c| {
        if let Some(r) = c.borrow().get(src) {
            return r.clone();
        }
        let r = parse(src).map(Rc::new);
        c.borrow_mut().insert(src.into(), r.clone());
        r
    })
}

struct Leitor<'s> {
    cs: Vec<char>,
    i: usize,
    _src: &'s str,
}

impl<'s> Leitor<'s> {
    fn novo(src: &'s str) -> Self {
        Self { cs: src.chars().collect(), i: 0, _src: src }
    }

    fn fim(&self) -> bool {
        self.i >= self.cs.len()
    }

    fn peek(&self) -> Option<char> {
        self.cs.get(self.i).copied()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.cs.get(self.i + n).copied()
    }

    fn espaco(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.i += 1;
        }
    }

    fn erro(&self, msg: impl Into<String>) -> ErroExpr {
        ErroExpr { pos: self.i, msg: msg.into() }
    }

    fn resto_curto(&self) -> String {
        self.cs[self.i..].iter().take(12).collect()
    }

    fn come(&mut self, s: &str) -> bool {
        let n = s.chars().count();
        if self.cs[self.i..].iter().take(n).copied().eq(s.chars()) {
            self.i += n;
            true
        } else {
            false
        }
    }

    /// Uma palavra-chave inteira (não o prefixo de um nome: `inside` não é `in`).
    fn palavra(&mut self, w: &str) -> bool {
        let n = w.chars().count();
        let casa = self.cs[self.i..].iter().take(n).copied().eq(w.chars())
            && !self.peek_at(n).is_some_and(|c| c.is_alphanumeric() || c == '_');
        if casa {
            self.i += n;
        }
        casa
    }

    fn ternario(&mut self) -> Result<Expr, ErroExpr> {
        let c = self.ou()?;
        self.espaco();
        if !self.come("?") {
            return Ok(c);
        }
        self.espaco();
        let a = self.ternario()?;
        self.espaco();
        let b = if self.come(":") {
            self.espaco();
            Some(Box::new(self.ternario()?))
        } else {
            None
        };
        Ok(Expr::Se(Box::new(c), Box::new(a), b))
    }

    fn ou(&mut self) -> Result<Expr, ErroExpr> {
        let mut e = self.e()?;
        loop {
            self.espaco();
            if self.come("||") {
                self.espaco();
                e = Expr::Ou(Box::new(e), Box::new(self.e()?));
            } else {
                return Ok(e);
            }
        }
    }

    fn e(&mut self) -> Result<Expr, ErroExpr> {
        let mut e = self.nao()?;
        loop {
            self.espaco();
            if self.come("&&") {
                self.espaco();
                e = Expr::E(Box::new(e), Box::new(self.nao()?));
            } else {
                return Ok(e);
            }
        }
    }

    fn nao(&mut self) -> Result<Expr, ErroExpr> {
        self.espaco();
        if self.peek() == Some('!') && self.peek_at(1) != Some('=') {
            self.i += 1;
            return Ok(Expr::Nao(Box::new(self.nao()?)));
        }
        self.cmp()
    }

    fn cmp(&mut self) -> Result<Expr, ErroExpr> {
        let a = self.primario()?;
        self.espaco();
        if self.palavra("in") {
            self.espaco();
            let b = self.primario()?;
            return Ok(Expr::Em(Box::new(a), Box::new(b)));
        }
        if self.palavra("is") {
            self.espaco();
            let negado = self.palavra("not");
            self.espaco();
            if !self.palavra("empty") {
                return Err(self.erro("depois de `is` só existe `empty` (ou `not empty`)"));
            }
            return Ok(Expr::Vazio(Box::new(a), negado));
        }
        let op = if self.come("==") {
            Op::Igual
        } else if self.come("!=") {
            Op::Diferente
        } else if self.come("<=") {
            Op::MenorIgual
        } else if self.come(">=") {
            Op::MaiorIgual
        } else if self.come("<") {
            Op::Menor
        } else if self.come(">") {
            Op::Maior
        } else {
            return Ok(a);
        };
        self.espaco();
        let b = self.primario()?;
        Ok(Expr::Cmp(op, Box::new(a), Box::new(b)))
    }

    fn primario(&mut self) -> Result<Expr, ErroExpr> {
        self.espaco();
        match self.peek() {
            None => Err(self.erro("a expressão acabou no meio")),
            Some('(') => {
                self.i += 1;
                let e = self.ternario()?;
                self.espaco();
                if !self.come(")") {
                    return Err(self.erro("faltou o `)`"));
                }
                Ok(e)
            }
            Some('[') => {
                self.i += 1;
                let mut itens = Vec::new();
                self.espaco();
                if self.come("]") {
                    return Ok(Expr::Lista(itens));
                }
                loop {
                    itens.push(self.ternario()?);
                    self.espaco();
                    if self.come(",") {
                        self.espaco();
                        if self.come("]") {
                            return Ok(Expr::Lista(itens));
                        }
                        continue;
                    }
                    if self.come("]") {
                        return Ok(Expr::Lista(itens));
                    }
                    return Err(self.erro("esperava `,` ou `]` na lista"));
                }
            }
            Some(q @ ('"' | '\'')) => {
                self.i += 1;
                let mut s = String::new();
                loop {
                    match self.peek() {
                        None => return Err(self.erro("texto nunca fechado")),
                        Some('\\') if self.peek_at(1).is_some_and(|c| c == q || c == '\\') => {
                            s.push(self.peek_at(1).unwrap());
                            self.i += 2;
                        }
                        Some(c) if c == q => {
                            self.i += 1;
                            return Ok(match interpola(&s) {
                                Ok(e) => e,
                                Err(texto) => Expr::Lit(Valor::Texto(texto)),
                            });
                        }
                        Some(c) => {
                            s.push(c);
                            self.i += 1;
                        }
                    }
                }
            }
            Some(c) if c.is_ascii_digit() || (c == '-' && self.peek_at(1).is_some_and(|d| d.is_ascii_digit())) => {
                let ini = self.i;
                self.i += 1;
                while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '.') {
                    self.i += 1;
                }
                let txt: String = self.cs[ini..self.i].iter().collect();
                txt.parse::<f64>()
                    .map(|n| Expr::Lit(Valor::Num(n)))
                    .map_err(|_| ErroExpr { pos: ini, msg: format!("número inválido `{txt}`") })
            }
            Some('@') => {
                self.i += 1;
                let nome = self.caminho();
                if nome.is_empty() {
                    return Err(self.erro("esperava o nome de uma chave depois do `@`"));
                }
                Ok(Expr::Ref(nome))
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let nome = self.caminho();
                match nome.as_str() {
                    "true" => return Ok(Expr::Lit(Valor::Bool(true))),
                    "false" => return Ok(Expr::Lit(Valor::Bool(false))),
                    "null" => return Ok(Expr::Lit(Valor::Nulo)),
                    _ => {}
                }
                if self.peek() == Some('(') {
                    if !FUNCOES.contains(&nome.as_str()) {
                        return Err(self.erro(format!(
                            "função desconhecida `{nome}` (existem: {})",
                            FUNCOES.join(", ")
                        )));
                    }
                    self.i += 1;
                    return self.chamada(nome);
                }
                Ok(Expr::Ref(nome))
            }
            Some(c) => Err(self.erro(format!("não esperava `{c}` aqui"))),
        }
    }

    fn chamada(&mut self, nome: String) -> Result<Expr, ErroExpr> {
        let mut pos = Vec::new();
        let mut nomeados = Vec::new();
        self.espaco();
        if self.come(")") {
            return Ok(Expr::Chamada(nome, pos, nomeados));
        }
        loop {
            self.espaco();
            // `nome = expr` (argumento nomeado) — olha adiante sem consumir.
            let marca = self.i;
            let k = if self.peek().is_some_and(|c| c.is_alphabetic() || c == '_') {
                self.caminho()
            } else {
                String::new()
            };
            self.espaco();
            if !k.is_empty() && self.peek() == Some('=') && self.peek_at(1) != Some('=') {
                self.i += 1;
                self.espaco();
                nomeados.push((k, self.ternario()?));
            } else {
                self.i = marca;
                pos.push(self.ternario()?);
            }
            self.espaco();
            if self.come(",") {
                continue;
            }
            if self.come(")") {
                return Ok(Expr::Chamada(nome, pos, nomeados));
            }
            return Err(self.erro(format!("esperava `,` ou `)` na chamada de `{nome}`")));
        }
    }

    /// Nome de chave: letras, dígitos, `_`, `-` no meio e `.` quando vem mais
    /// nome depois (`@item.campo`, mas não o ponto final de uma frase).
    fn caminho(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            let continua = self.peek_at(1).is_some_and(|n| n.is_alphanumeric() || n == '_');
            let ok = c.is_alphanumeric() || c == '_' || ((c == '.' || c == '-') && continua && !s.is_empty());
            if !ok {
                break;
            }
            s.push(c);
            self.i += 1;
        }
        s
    }
}

/// Onde a expressão que começa em `cs[ini]` termina, num atributo do `.gv`: no
/// primeiro `,` ou `)` fora de aspas, parênteses e colchetes, ou numa quebra de
/// linha / `//` no nível de fora. Devolve o índice (exclusivo).
pub fn fim_no_atributo(cs: &[char], ini: usize) -> usize {
    let mut i = ini;
    let mut prof = 0i32;
    let mut aspas: Option<char> = None;
    while i < cs.len() {
        let c = cs[i];
        if let Some(q) = aspas {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == q {
                aspas = None;
            }
            i += 1;
            continue;
        }
        match c {
            '"' | '\'' => aspas = Some(c),
            '(' | '[' => prof += 1,
            ')' | ']' if prof > 0 => prof -= 1,
            ',' | ')' if prof == 0 => return i,
            '\n' if prof == 0 => return i,
            '/' if prof == 0 && cs.get(i + 1) == Some(&'/') => return i,
            _ => {}
        }
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pares: &[(&str, &str)]) -> impl FnMut(&str) -> Option<String> {
        let m: HashMap<String, String> =
            pares.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |k| m.get(k).cloned()
    }

    fn ev(src: &str, pares: &[(&str, &str)]) -> String {
        let e = parse(src).unwrap_or_else(|e| panic!("{src}: {e}"));
        let mut l = ctx(pares);
        avalia(&e, &mut l).texto().into_owned()
    }

    #[test]
    fn texto_entre_aspas_interpola_chave() {
        // O `TabButton` do rustploy: `if @current == "@target"` compara valores.
        assert_eq!(ev(r#"@current == "@target""#, &[("current", "logs"), ("target", "logs")]), "true");
        assert_eq!(ev(r#"@current == "@target""#, &[("current", "logs"), ("target", "env")]), "false");
        assert_eq!(ev(r#"@aba == "@tab.id""#, &[("aba", "web"), ("tab.id", "web")]), "true");
        assert_eq!(ev(r#""v@versao.""#, &[("versao", "1.2")]), "v1.2.");
        assert_eq!(ev(r#"'@@admin'"#, &[("admin", "x")]), "@admin");
        assert_eq!(ev(r#""sem chave""#, &[]), "sem chave");
        assert_eq!(ev(r#""@ausente""#, &[]), "");
        assert_eq!(ev(r#""@3""#, &[]), "@3");
    }

    #[test]
    fn precedencia_e_ou_nao() {
        assert_eq!(ev("@a || @b && @c", &[("a", "false"), ("b", "true"), ("c", "false")]), "false");
        assert_eq!(ev("(@a || @b) && !@c", &[("a", "true"), ("c", "false")]), "true");
        assert_eq!(ev("!@a", &[]), "true");
    }

    #[test]
    fn comparacao_numerica_e_de_texto() {
        assert_eq!(ev("@n > 80", &[("n", "81")]), "true");
        assert_eq!(ev("@n >= 80", &[("n", "79.5")]), "false");
        assert_eq!(ev("@n == 1", &[("n", "1.0")]), "true");
        assert_eq!(ev("@s == \"api\"", &[("s", "api")]), "true");
        assert_eq!(ev("@s != 'api'", &[("s", "web")]), "true");
        // Texto contra número em `<`: falso, nunca comparação lexicográfica.
        assert_eq!(ev("@s < 3", &[("s", "abc")]), "false");
        // `"true"` do contexto é igual ao literal `true`.
        assert_eq!(ev("@b == true", &[("b", "true")]), "true");
    }

    #[test]
    fn verdade_de_texto() {
        for falso in ["", "false", "0", "no", "off", "null", "[]", "{}", "0.0"] {
            assert_eq!(ev("@x ? 'v' : 'f'", &[("x", falso)]), "f", "{falso:?}");
        }
        for verdadeiro in ["true", "1", "abc", "[1]", "yes"] {
            assert_eq!(ev("@x ? 'v' : 'f'", &[("x", verdadeiro)]), "v", "{verdadeiro:?}");
        }
        assert_eq!(ev("@ausente ? 'v' : 'f'", &[]), "f");
    }

    #[test]
    fn is_empty_em_cada_tipo() {
        assert_eq!(ev("@x is empty", &[("x", "")]), "true");
        assert_eq!(ev("@x is empty", &[("x", "[]")]), "true");
        assert_eq!(ev("@x is empty", &[]), "true");
        // A armadilha antiga: um texto comum contava como vazio.
        assert_eq!(ev("@x is empty", &[("x", "olá")]), "false");
        assert_eq!(ev("@x is not empty", &[("x", "[1,2]")]), "true");
    }

    #[test]
    fn in_com_os_tres_formatos_de_lista() {
        assert_eq!(ev("@k in [\"int\", \"double\"]", &[("k", "double")]), "true");
        assert_eq!(ev("@k in @l", &[("k", "b"), ("l", "[\"a\",\"b\"]")]), "true");
        assert_eq!(ev("@k in @l", &[("k", "b"), ("l", "a, b c")]), "true");
        assert_eq!(ev("\"x\" in @l", &[("l", "a b")]), "false");
        assert_eq!(ev("3 in [1, 2, 3]", &[]), "true");
    }

    #[test]
    fn ternario_e_padrao() {
        assert_eq!(ev("@n > 80 ? '⚠ ' : ''", &[("n", "90")]), "⚠ ");
        assert_eq!(ev("@ok ? 'sim'", &[("ok", "false")]), "");
        assert_eq!(ev("largura|fill", &[]), "fill");
        assert_eq!(ev("largura|fill", &[("largura", "200")]), "200");
        assert_eq!(ev("@a ? @b ? 1 : 2 : 3", &[("a", "true"), ("b", "false")]), "2");
    }

    #[test]
    fn nome_nu_e_arroba_sao_a_mesma_leitura() {
        assert_eq!(ev("projeto", &[("projeto", "rp")]), "rp");
        assert_eq!(ev("@item.nome", &[("item.nome", "x")]), "x");
        assert_eq!(ev("__dialog.message", &[("__dialog.message", "m")]), "m");
    }

    #[test]
    fn funcoes() {
        assert_eq!(ev("len(@l)", &[("l", "[1,2,3]")]), "3");
        assert_eq!(ev("len(@s) > 2", &[("s", "abc")]), "true");
        assert_eq!(ev("upper(@s)", &[("s", "abc")]), "ABC");
    }

    #[test]
    fn json_guarda_o_tipo_dos_literais() {
        assert_eq!(ev("json(@k, true, 3, 'x')", &[("k", "abc")]), r#"["abc",true,3,"x"]"#);
        assert_eq!(ev("json()", &[]), "[]");
    }

    #[test]
    fn curto_circuito_nao_le_o_outro_lado() {
        let e = parse("@a && @b").unwrap();
        let mut lidas = Vec::new();
        let mut ler = |k: &str| {
            lidas.push(k.to_string());
            if k == "a" { Some("false".to_string()) } else { Some("true".to_string()) }
        };
        assert!(!avalia(&e, &mut ler).verdadeiro());
        assert_eq!(lidas, vec!["a"]);
    }

    #[test]
    fn erros_com_posicao() {
        let e = parse("@a && ").unwrap_err();
        assert!(e.msg.contains("acabou"), "{e}");
        let e = parse("@a ==").unwrap_err();
        assert!(e.msg.contains("acabou"), "{e}");
        let e = parse("@a is cheio").unwrap_err();
        assert!(e.msg.contains("empty"), "{e}");
        let e = parse("foo(@a)").unwrap_err();
        assert!(e.msg.contains("função desconhecida"), "{e}");
        let e = parse("@a @b").unwrap_err();
        assert_eq!(e.pos, 3);
    }

    #[test]
    fn fim_do_atributo_respeita_aspas_e_parenteses() {
        let cs: Vec<char> = "@a ? \"x, y\" : t('k', n = @b), outro = 1".chars().collect();
        let fim = fim_no_atributo(&cs, 0);
        assert_eq!(cs[..fim].iter().collect::<String>(), "@a ? \"x, y\" : t('k', n = @b)");
    }

    #[test]
    fn eh_caminho_reconhece_so_nomes() {
        assert!(eh_caminho("projeto"));
        assert!(eh_caminho("item.nome"));
        assert!(eh_caminho("@x"));
        assert!(!eh_caminho("a && b"));
        assert!(!eh_caminho("true"));
        assert!(!eh_caminho("t('x')"));
    }
}
