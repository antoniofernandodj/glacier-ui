//! Handlers de script Luau e ações de `.gv`, conferidos um contra o outro
//! (Frente 6 do `TODO.md`).
//!
//! O `.gv` resolve o handler **pelo nome, na hora do clique**: `on_click =
//! salvar()` procura a função global `salvar` no script do dono da ação. Nada,
//! antes do clique, conferia que ela existe, quantos argumentos recebe ou de que
//! tipo — `on_click = salvr()` carregava, desenhava e não fazia nada.
//!
//! Este módulo faz a parte estática desse trabalho, lendo texto:
//!
//! - [`assinaturas`] extrai as funções GLOAIS de um `.luau` (nome, aridade e os
//!   tipos anotados no `--!strict`) — globais porque é só nelas que o despacho
//!   resolve (`globals.get(nome)`);
//! - [`acoes`] extrai de um `.gv` cada ação chamada, com o arquivo‑linha;
//! - [`Chaves`] monta `chave → tipo` do que os scripts gravam no `ctx` e do que
//!   `props`/`state` declaram, que é o que permite conferir um `@chave` passada de
//!   argumento contra a anotação do parâmetro;
//! - [`conferir`] decide se uma ação bate com uma assinatura.
//!
//! Quem consome é o `glacier check` ([`crate::ferramentas`]) — e é isso que
//! substitui o `tools/busy_actions.py --check` de um app na parte "o handler do
//! template existe no script".

use std::collections::HashMap;

/// Um tipo anotado num parâmetro (`--!strict`: `function f(k: string)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Str,
    Num,
    Bool,
    Tabela,
    /// União, opcional, um nome de tipo do app, `any`: nada a conferir.
    Outro,
}

impl Tipo {
    /// O tipo de um literal do `.gv` (`salvar("a", 3, true)`).
    pub fn do_literal(v: &str) -> Option<Tipo> {
        let t = v.trim();
        if t == "true" || t == "false" {
            return Some(Tipo::Bool);
        }
        if t.starts_with('"') || t.starts_with('\'') {
            return Some(Tipo::Str);
        }
        if !t.is_empty() && t.parse::<f64>().is_ok() {
            return Some(Tipo::Num);
        }
        if t.starts_with('[') || t.starts_with('{') {
            return Some(Tipo::Tabela);
        }
        None
    }

    pub fn nome(self) -> &'static str {
        match self {
            Tipo::Str => "string",
            Tipo::Num => "number",
            Tipo::Bool => "boolean",
            Tipo::Tabela => "table",
            Tipo::Outro => "qualquer",
        }
    }

    /// O tipo de uma anotação Luau.
    fn da_anotacao(s: &str) -> Option<Tipo> {
        let t = s.trim().trim_end_matches('?');
        if t.contains('|') || t.contains(' ') || t.contains('.') || t.starts_with('{') {
            return None;
        }
        match t {
            "string" => Some(Tipo::Str),
            "number" => Some(Tipo::Num),
            "boolean" => Some(Tipo::Bool),
            "table" | "Table" => Some(Tipo::Tabela),
            _ => None,
        }
    }
}

/// Uma função global de um script.
#[derive(Debug, Clone)]
pub struct Funcao {
    pub nome: String,
    /// (nome do parâmetro, tipo anotado) na ordem. `None` = sem anotação.
    pub params: Vec<(String, Option<Tipo>)>,
    /// `function f(a, ...)` — aceita quantos argumentos vierem.
    pub variadica: bool,
    /// quantos dos parâmetros anotados são opcionais (`a: string?`).
    pub opcionais: usize,
    pub linha: u32,
}

impl Funcao {
    /// Argumentos obrigatórios: os que podem faltar sem erro.
    pub fn obrigatorios(&self) -> usize {
        self.params.len().saturating_sub(self.opcionais)
    }
}

fn eh_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Tira comentários e strings de um `.luau`, **mantendo as quebras de linha**: é
/// assim que a linha de uma função bate com a linha do arquivo.
fn limpo(src: &str) -> String {
    let cs: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '-' && cs.get(i + 1) == Some(&'-') && cs.get(i + 2) == Some(&'[')
            && cs.get(i + 3) == Some(&'[')
        {
            let mut j = i + 4;
            while j + 1 < cs.len() && !(cs[j] == ']' && cs[j + 1] == ']') {
                if cs[j] == '\n' {
                    out.push('\n');
                }
                j += 1;
            }
            i = (j + 2).min(cs.len());
            continue;
        }
        if cs[i] == '-' && cs.get(i + 1) == Some(&'-') {
            while i < cs.len() && cs[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if cs[i] == '[' && cs.get(i + 1) == Some(&'[') {
            let mut j = i + 2;
            while j + 1 < cs.len() && !(cs[j] == ']' && cs[j + 1] == ']') {
                if cs[j] == '\n' {
                    out.push('\n');
                }
                j += 1;
            }
            i = (j + 2).min(cs.len());
            out.push(' ');
            continue;
        }
        if cs[i] == '"' || cs[i] == '\'' {
            let q = cs[i];
            out.push(' ');
            let mut j = i + 1;
            while j < cs.len() && cs[j] != q {
                if cs[j] == '\\' {
                    j += 1;
                }
                if cs[j] == '\n' {
                    out.push('\n');
                }
                j += 1;
            }
            i = (j + 1).min(cs.len());
            continue;
        }
        out.push(cs[i]);
        i += 1;
    }
    out
}

/// O índice, em `cs`, de cada palavra-chave `kw` que é uma palavra só.
fn palavras<'s>(cs: &'s [char], kw: &str) -> Vec<usize> {
    let k: Vec<char> = kw.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + k.len() <= cs.len() {
        if cs[i..i + k.len()] == k[..] {
            let antes = i == 0 || !eh_ident(cs[i - 1]);
            let depois = cs.get(i + k.len()).is_none_or(|c| !eh_ident(*c));
            if antes && depois {
                out.push(i);
            }
        }
        i += 1;
    }
    out
}

/// A linha de um índice de caractere.
fn linha_de(cs: &[char], i: usize) -> u32 {
    cs[..i.min(cs.len())].iter().filter(|&&c| c == '\n').count() as u32 + 1
}

/// As funções globais de um script: `function f(…)` (sem o `local`),
/// `function M.f(…)`, `M.f = function(…)` e `f = function(…)`. Um `local
/// function` não é alcançável pelo despacho e por isso não entra.
pub fn assinaturas(src: &str) -> Vec<Funcao> {
    let texto = limpo(src);
    let cs: Vec<char> = texto.chars().collect();
    let mut out = Vec::new();
    for f in palavras(&cs, "function") {
        let antes = &cs[..f];
        let ultimo = antes.iter().rev().find(|c| !c.is_whitespace());
        let depois = &cs[f + "function".len()..];
        let ini = depois.iter().position(|c| !c.is_whitespace()).unwrap_or(depois.len());
        let corpo = &depois[ini..];
        let Some(abre) = corpo.iter().position(|&c| c == '(') else { continue };
        let nome: String = corpo[..abre].iter().collect::<String>().trim().to_string();
        let ponto = nome.rfind(['.', ':']);
        let ultimo_token = ponto.map_or(nome.as_str(), |i| &nome[i + 1..]);
        let valido = !nome.is_empty()
            && nome
                .chars()
                .all(|c| eh_ident(c) || c == '.' || c == ':')
            && !nome.starts_with(|c: char| c.is_ascii_digit());
        let Some(fechar) = fecha_parenteses(corpo, abre) else { continue };
        let params = &corpo[abre + 1..fechar];
        let (params, variadica, opcionais) = params_da_lista(&params.iter().collect::<String>());
        let pos = f + ini + abre;
        let eh_atribuicao = ultimo.is_some_and(|c| *c == '=');
        // `local function f(…)` não é alcançável pelo despacho (que lê globais):
        // registrar como global seria esconder o bug que o check existe para achar.
        let declarada_local = antes
            .iter()
            .rev()
            .skip_while(|c| c.is_whitespace())
            .collect::<String>()
            .starts_with("lacol" /* "local" ao contrário */);
        if declarada_local && !eh_atribuicao {
            continue;
        }
        let alvo = if eh_atribuicao {
            nome_antes_do_eq(antes)
        } else if valido {
            nome.clone()
        } else {
            continue;
        };
        if alvo.is_empty() || keyword(&alvo) {
            continue;
        }
        // `function M.f(…)` é a função `f` do módulo `M`; o despacho só vê globais,
        // mas um app que faz `salvar = M.salvar` no próprio script chega aqui pelo
        // ramo de atribuição. Guarda os dois nomes.
        let mut nomes = vec![alvo.clone()];
        if !eh_atribuicao && ultimo_token != alvo && !ultimo_token.is_empty() {
            nomes.push(ultimo_token.to_string());
        }
        for n in nomes {
            if keyword(&n) {
                continue;
            }
            out.push(Funcao {
                nome: n,
                params: params.clone(),
                variadica,
                opcionais,
                linha: linha_de(&cs, pos),
            });
        }
    }
    out.sort_by_key(|f| (f.nome.clone(), f.linha));
    out.dedup_by_key(|f| (f.nome.clone(), f.linha));
    out
}

/// O nome à esquerda de um `= function`: o último identificador (com pontos).
fn nome_antes_do_eq(antes: &[char]) -> String {
    let sem_eq: Vec<char> = {
        let mut i = antes.len();
        while i > 0 && (antes[i - 1].is_whitespace() || antes[i - 1] == '=') {
            i -= 1;
        }
        antes[..i].to_vec()
    };
    let fim = sem_eq.len();
    let mut ini = fim;
    while ini > 0 && (eh_ident(sem_eq[ini - 1]) || sem_eq[ini - 1] == '.') {
        ini -= 1;
    }
    sem_eq[ini..fim].iter().collect()
}

/// O fim do `(` que abre em `abre` (índice em `cs`), com aspas ignoradas.
fn fecha_parenteses(cs: &[char], abre: usize) -> Option<usize> {
    let mut prof = 0i32;
    let mut aspas: Option<char> = None;
    for i in abre..cs.len() {
        if let Some(q) = aspas {
            if cs[i] == q {
                aspas = None;
            }
            continue;
        }
        match cs[i] {
            '"' | '\'' => aspas = Some(cs[i]),
            '(' => prof += 1,
            ')' => {
                prof -= 1;
                if prof == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn keyword(nome: &str) -> bool {
    matches!(
        nome,
        "if" | "then" | "else" | "elseif" | "end" | "function" | "local" | "return" | "for"
            | "while" | "do" | "repeat" | "until" | "in" | "and" | "or" | "not" | "true" | "false"
            | "nil" | "continue" | "break" | "self"
    )
}

/// `(a: string, b: number?, ...)`: os parâmetros, se é variádica e quantos são
/// opcionais.
fn params_da_lista(s: &str) -> (Vec<(String, Option<Tipo>)>, bool, usize) {
    let mut out = Vec::new();
    let mut variadica = false;
    let mut opcionais = 0;
    for p in split_top(s, ',') {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        if p.starts_with("...") {
            variadica = true;
            continue;
        }
        match p.split_once(':') {
            Some((n, t)) => {
                let t = t.trim();
                if t.ends_with('?') || t.contains('?') {
                    opcionais += 1;
                }
                out.push((n.trim().to_string(), Tipo::da_anotacao(t)));
            }
            None => out.push((p.to_string(), None)),
        }
    }
    (out, variadica, opcionais)
}

/// Divide por `sep` fora de `<…>`, `(…)` e `{…}`.
fn split_top(s: &str, sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut atual = String::new();
    let mut ang = 0i32;
    let mut par = 0i32;
    for c in s.chars() {
        match c {
            '<' => ang += 1,
            '>' => ang -= 1,
            '(' | '{' => par += 1,
            ')' | '}' => par -= 1,
            c if c == sep && ang <= 0 && par == 0 => {
                out.push(std::mem::take(&mut atual));
                continue;
            }
            _ => {}
        }
        atual.push(c);
    }
    out.push(atual);
    out
}

/// Uma ação chamada por um `.gv`.
#[derive(Debug, Clone, PartialEq)]
pub struct Acao {
    /// O atributo (`on_click`, `on_submit`…).
    pub attr: String,
    /// O nome do handler, sem o prefixo de dono.
    pub handler: String,
    /// Os argumentos crus, na ordem.
    pub args: Vec<String>,
    pub linha: u32,
    /// A forma de sempre, sem os parênteses (`on_click = salvar`) — depreciada.
    pub nua: bool,
}

/// As ações de um `.gv`: todo `on_* = …`, com a linha do arquivo.
pub fn acoes(src: &str) -> Vec<Acao> {
    let mut out = Vec::new();
    for (n, linha) in src.lines().enumerate() {
        let limpa = sem_comentario(linha);
        let cs: Vec<char> = limpa.chars().collect();
        for i in indices_de_on(&cs) {
            let resto: String = cs[i..].iter().collect();
            let Some(eq) = resto.find('=') else { continue };
            let attr = resto[..eq].trim().to_string();
            if !attr.starts_with("on_") {
                continue;
            }
            let valor = resto[eq + 1..].trim_start();
            let vcs: Vec<char> = valor.chars().collect();
            let fim = fim_do_valor(&vcs);
            let v = valor[..fim].trim();
            // Interpolada ou textual (`on_click = "{acao}"`, `"salvar:{id}"`): o
            // nome não é estático, e o `check` de uma string montada é chute.
            if v.is_empty() || v.starts_with('{') || v.starts_with('"') || v.starts_with('@') {
                continue;
            }
            let (handler, args, nua) = desfazer_chamada(v);
            if handler.is_empty() || !handler.chars().all(|c| eh_ident(c)) {
                continue;
            }
            out.push(Acao { attr, handler, args, linha: (n + 1) as u32, nua });
        }
    }
    out
}

/// `salvar(@id, true)` → (`salvar`, [`@id`, `true`], falso); `salvar()` →
/// (`salvar`, [], falso); `salvar` → (`salvar`, [], **verdadeiro** — a forma
/// nua); `Comp::salvar(x)` → (`salvar`, …) com o prefixo de dono removido.
fn desfazer_chamada(v: &str) -> (String, Vec<String>, bool) {
    let sem_dono = v.rsplit_once("::").map_or(v, |(_, h)| h);
    // `window:minimize`, `field:nome`: ação do motor, resolvida em Rust.
    let base = sem_dono.split([':']).next().unwrap_or(sem_dono);
    match base.find('(') {
        Some(i) => {
            let nome = base[..i].trim().to_string();
            let dentro = base[i + 1..].trim_end_matches(')').trim();
            let args = if dentro.is_empty() {
                Vec::new()
            } else {
                split_top(dentro, ',').into_iter().map(|a| a.trim().to_string()).collect()
            };
            (nome, args, false)
        }
        None => (base.trim().to_string(), Vec::new(), true),
    }
}

/// Onde o valor de um atributo termina na linha: a `,` ou o `)` de fora.
pub(crate) fn fim_do_valor(cs: &[char]) -> usize {
    let mut par = 0i32;
    let mut col = 0i32;
    let mut aspas: Option<char> = None;
    for (i, c) in cs.iter().enumerate() {
        if let Some(q) = aspas {
            if c == &q {
                aspas = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => aspas = Some(*c),
            '(' => par += 1,
            '[' => col += 1,
            ']' => col -= 1,
            ')' => {
                if par == 0 && col == 0 {
                    return i;
                }
                par -= 1;
            }
            ',' if par == 0 && col == 0 => return i,
            _ => {}
        }
    }
    cs.len()
}

/// Os índices de um `on_` que começa atributo (não dentro de string).
fn indices_de_on(cs: &[char]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut aspas: Option<char> = None;
    let mut i = 0;
    while i < cs.len() {
        if let Some(q) = aspas {
            if cs[i] == q {
                aspas = None;
            }
            i += 1;
            continue;
        }
        if cs[i] == '"' || cs[i] == '\'' {
            aspas = Some(cs[i]);
            i += 1;
            continue;
        }
        let eh = cs[i] == 'o'
            && cs.get(i + 1) == Some(&'n')
            && cs.get(i + 2) == Some(&'_')
            && (i == 0 || !(eh_ident(cs[i - 1]) || cs[i - 1] == '.'));
        if eh {
            out.push(i);
        }
        i += 1;
    }
    out
}

/// A linha sem o comentário `//` de fora de string (o `.gv` comenta com `//`).
fn sem_comentario(linha: &str) -> String {
    let cs: Vec<char> = linha.chars().collect();
    let mut out = String::new();
    let mut aspas: Option<char> = None;
    let mut i = 0;
    while i < cs.len() {
        if let Some(q) = aspas {
            if cs[i] == q {
                aspas = None;
            }
            out.push(cs[i]);
            i += 1;
            continue;
        }
        if cs[i] == '"' || cs[i] == '\'' {
            aspas = Some(cs[i]);
            out.push(cs[i]);
            i += 1;
            continue;
        }
        if cs[i] == '/' && cs.get(i + 1) == Some(&'/') {
            break;
        }
        out.push(cs[i]);
        i += 1;
    }
    out
}

/// `chave → tipo`: o que os scripts gravam no `ctx` e o que `props`/`state`
/// declaram. É o que dá ao `check` o tipo de um argumento `@chave`.
#[derive(Debug, Default)]
pub struct Chaves {
    pub tipos: HashMap<String, Tipo>,
    /// De onde a chave veio: `"ctx"` (escrita por um script), `"prop"` ou
    /// `"estado"` (declarada pelo componente). É o comentário do `ctx.d.luau`.
    pub origem: HashMap<String, &'static str>,
}

impl Chaves {
    pub fn ver(&self, chave: &str) -> Option<Tipo> {
        self.tipos.get(chave).copied()
    }

    /// `ctx.flag = true`, `ctx.n = 3`, `ctx.lista = {}` num script Luau.
    pub fn do_script(&mut self, src: &str) {
        for linha in src.lines() {
            let l = linha.trim();
            let Some(resto) = l
                .strip_prefix("ctx.")
                .or_else(|| l.strip_prefix("state."))
                .or_else(|| l.strip_prefix("self."))
            else {
                continue;
            };
            let Some(eq) = resto.find('=') else { continue };
            let chave = resto[..eq].trim();
            if chave.is_empty() || !chave.chars().all(|c| eh_ident(c) || c == '.') {
                continue;
            }
            let valor = resto[eq + 1..].split("//").next().unwrap_or("").trim();
            let t = match valor.chars().next() {
                Some('{') | Some('[') => Some(Tipo::Tabela),
                Some('"') | Some('\'') => Some(Tipo::Str),
                _ => Tipo::do_literal(valor),
            };
            if let Some(t) = t {
                self.tipos.insert(chave.to_string(), t);
                self.origem.insert(chave.to_string(), "ctx");
            }
        }
    }

    /// `prop(name = x, type = bool)` — inclusive os vindos de um `state { … }`.
    pub fn do_gv(&mut self, src: &str) {
        let re = regex::Regex::new(r"(?s)prop\s*\(([^)]*)\)").unwrap();
        let nome = regex::Regex::new(r"name\s*=\s*([\w.\-]+)").unwrap();
        let tipo = regex::Regex::new(r"type\s*=\s*(\w+)").unwrap();
        for c in re.captures_iter(src) {
            let dentro = &c[1];
            let Some(n) = nome.captures(dentro).map(|k| k[1].to_string()) else { continue };
            let Some(t) = tipo.captures(dentro).map(|k| k[1].to_string()) else { continue };
            let t = match t.as_str() {
                "string" | "text" | "texto" => Tipo::Str,
                "number" | "numero" => Tipo::Num,
                "bool" | "boolean" => Tipo::Bool,
                "list" | "lista" | "object" | "objeto" | "array" => Tipo::Tabela,
                _ => continue,
            };
            self.tipos.entry(n.clone()).or_insert(t);
            self.origem.entry(n).or_insert("prop");
        }
        // `state { aberto = false }`: o tipo é o do literal inicial.
        let re = regex::Regex::new(r"(?m)^\s*([\w.\-]+)\s*=\s*([^,\n}]+)").unwrap();
        for bloco in blocos_state(src) {
            for c in re.captures_iter(&bloco) {
                let n = c[1].to_string();
                let v = c[2].trim();
                if let Some(t) = Tipo::do_literal(v) {
                    self.tipos.entry(n.clone()).or_insert(t);
                    self.origem.insert(n, "estado");
                }
            }
        }
    }

    pub fn vazio(&self) -> bool {
        self.tipos.is_empty()
    }
}

/// Os corpos dos blocos `state { … }` de um `.gv` (texto cru, para o `check`).
pub fn blocos_state(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut resto = src;
    let mut desloc = 0;
    while let Some(i) = resto.find("state") {
        let depois = &resto[i + "state".len()..];
        let Some(abre) = depois.find('{') else { break };
        if !depois[..abre].trim().is_empty() {
            resto = &resto[i + "state".len()..];
            desloc += i + "state".len();
            continue;
        }
        let mut prof = 0i32;
        let mut fim = None;
        for (j, c) in depois[abre..].char_indices() {
            match c {
                '{' => prof += 1,
                '}' => {
                    prof -= 1;
                    if prof == 0 {
                        fim = Some(abre + j);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(f) = fim else { break };
        out.push(depois[abre + 1..f].to_string());
        let avanco = i + f + 1;
        desloc += avanco;
        resto = &resto[avanco..];
        let _ = desloc;
    }
    out
}

/// O que o `check` acusa sobre uma ação.
#[derive(Debug, Clone, PartialEq)]
pub enum Problema {
    /// Nenhuma função global com este nome em script nenhum.
    Ausente,
    /// O número de argumentos não bate com a assinatura.
    Aride { dada: usize, esperada: String },
    /// Um literal, ou uma `@chave` tipada, contra o tipo anotado do parâmetro.
    Tipo { pos: usize, esperado: &'static str, dado: String },
}

/// Confere uma ação contra as funções de um conjunto de scripts. Uma ação que casa
/// com QUALQUER das declarações do nome passa (um handler pode estar em módulo).
pub fn conferir(acao: &Acao, funcoes: &[Funcao], chaves: &Chaves) -> Option<Problema> {
    let candidatas: Vec<&Funcao> = funcoes.iter().filter(|f| f.nome == acao.handler).collect();
    if candidatas.is_empty() {
        return Some(Problema::Ausente);
    }
    let dada = acao.args.len();
    let boa = candidatas.iter().any(|f| {
        let max = if f.variadica { usize::MAX } else { f.params.len() + 1 };
        dada >= f.obrigatorios() && dada <= max
    });
    if !boa {
        let f = &candidatas[0];
        let esperada = if f.variadica {
            format!("{} ou mais", f.obrigatorios())
        } else if f.params.is_empty() {
            "0 ou 1 (o value do evento)".to_string()
        } else {
            format!("{} a {}", f.obrigatorios(), f.params.len() + 1)
        };
        return Some(Problema::Aride { dada, esperada });
    }
    for (i, arg) in acao.args.iter().enumerate() {
        // `+1` de folga: num `on_change` o motor acrescenta o `value` depois dos
        // argumentos do markup — o parâmetro conferido é o da posição dada.
        let Some(f) = candidatas.iter().find(|f| {
            let max = if f.variadica { usize::MAX } else { f.params.len() + 1 };
            dada >= f.obrigatorios() && dada <= max
        }) else {
            continue;
        };
        let Some((_, Some(esperado))) = f.params.get(i).cloned() else { continue };
        if esperado == Tipo::Outro {
            continue;
        }
        let dado = if let Some(k) = arg.strip_prefix('@') {
            let k = k.trim_start_matches('{').trim_end_matches('}');
            chaves.ver(k)?
        } else {
            Tipo::do_literal(arg)?
        };
        if dado != esperado {
            return Some(Problema::Tipo {
                pos: i + 1,
                esperado: esperado.nome(),
                dado: format!("{} (`{}`)", dado.nome(), arg.trim()),
            });
        }
    }
    None
}

/// O nome mais próximo de um handler inexistente — a sugestão do erro.
pub fn sugerir(nome: &str, funcoes: &[Funcao]) -> Option<String> {
    let teto = (nome.chars().count() / 3).max(1);
    funcoes
        .iter()
        .map(|f| (distancia(nome, &f.nome), f.nome.clone()))
        .filter(|(d, _)| *d <= teto)
        .min_by_key(|(d, _)| *d)
        .map(|(_, n)| n)
}

/// Levenshtein, para a sugestão de nome.
fn distancia(a: &str, b: &str) -> usize {
    let ta: Vec<char> = a.chars().collect();
    let tb: Vec<char> = b.chars().collect();
    let mut linhas: Vec<usize> = (0..=tb.len()).collect();
    for i in 1..=ta.len() {
        let mut anterior = linhas[0];
        linhas[0] = i;
        for j in 1..=tb.len() {
            let custo = if ta[i - 1] == tb[j - 1] { anterior } else { anterior + 1 };
            anterior = linhas[j];
            linhas[j] = custo.min(linhas[j] + 1).min(anterior + 1);
        }
    }
    linhas[tb.len()]
}

/// As ações embutidas que o motor resolve em Rust — não são handler de script.
pub const EMBUTIDAS: &[&str] = &[
    "window",
    "field",
    "tray",
    "notifications",
    "dialog",
    "style",
    "open",
    "clipboard",
    "textarea_end",
    "textarea_top",
    "whatsthis",
    "audio",
    "app",
    "rules",
    "form",
];

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = r#"--!strict
local utils = require("./utils")

export type Item = { id: string }

function salvar()
    print("oi")
end

function exportar(key: string, on: boolean)
end

local function ajuda() end

function painel:fechar() end

campo = function(v: number) end
"#;

    fn funcoes() -> Vec<Funcao> {
        assinaturas(SCRIPT)
    }

    #[test]
    fn assinatura_dos_quatro_modos_de_declarar() {
        let f = funcoes();
        let nomes: Vec<&str> = f.iter().map(|x| x.nome.as_str()).collect();
        assert!(nomes.contains(&"salvar"), "{nomes:?}");
        assert!(nomes.contains(&"exportar"), "{nomes:?}");
        assert!(nomes.contains(&"campo"), "{nomes:?}");
        // `local function` não é alcançável pelo despacho: não entra.
        assert!(!nomes.contains(&"ajuda"), "{nomes:?}");
    }

    #[test]
    fn aridade_e_tipos() {
        let f = funcoes();
        let e = f.iter().find(|x| x.nome == "exportar").unwrap();
        assert_eq!(e.params.len(), 2);
        assert_eq!(e.params[0].1, Some(Tipo::Str));
        assert_eq!(e.params[1].1, Some(Tipo::Bool));
        assert_eq!(e.obrigatorios(), 2);
        assert_eq!(f.iter().find(|x| x.nome == "salvar").unwrap().params.len(), 0);
    }

    #[test]
    fn comentario_e_string_nao_declaram_funcao() {
        let f = assinaturas("-- function fantasma() end\nlocal s = 'function falsa()'\n");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn linha_bate_com_o_arquivo() {
        let f = funcoes();
        let salvar = f.iter().find(|x| x.nome == "salvar").unwrap();
        let linha = (salvar.linha as usize) - 1;
        assert!(
            SCRIPT.lines().nth(linha).unwrap().contains("function salvar"),
            "linha {}: {:?}",
            salvar.linha,
            SCRIPT.lines().nth(linha)
        );
    }

    #[test]
    fn acoes_de_um_gv() {
        let a = acoes(
            r#"row {
  button("S", on_click = salvar)
  button("E", on_click = exportar(@item.key, true))
  textinput(on_change = field:nome)
  // on_click = nada_decomentario
  text("on_click = dentro_de_string")
}"#,
        );
        let nomes: Vec<&str> = a.iter().map(|x| x.handler.as_str()).collect();
        assert_eq!(nomes, vec!["salvar", "exportar", "field"], "{a:?}");
        assert!(a[0].nua, "{:?}", a[0]);
        assert!(!a[1].nua);
        assert_eq!(a[1].args, vec!["@item.key", "true"], "{:?}", a[1]);
        assert_eq!(a[0].linha, 2);
    }

    fn chaves() -> Chaves {
        let mut c = Chaves::default();
        c.do_script("ctx.flag = true\nctx.n = 3\nctx.nome = \"x\"\nctx.l = {}\n");
        c
    }

    #[test]
    fn conferir_existe_aridade_e_tipo() {
        let f = funcoes();
        let c = chaves();
        let acao = |attr: &str, handler: &str, args: &[&str]| Acao {
            attr: attr.to_string(),
            handler: handler.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            linha: 1,
            nua: false,
        };
        assert_eq!(conferir(&acao("on_click", "salvar", &[]), &f, &c), None);
        assert_eq!(conferir(&acao("on_click", "salvr", &[]), &f, &c), Some(Problema::Ausente));
        assert!(matches!(
            conferir(&acao("on_click", "exportar", &["\"a\"".into()]), &f, &c),
            None
        ));
        assert!(matches!(
            conferir(&acao("on_click", "exportar", &["\"a\"".into(), "3".into()]), &f, &c),
            Some(Problema::Tipo { pos: 2, .. })
        ));
        assert!(matches!(
            conferir(&acao("on_click", "exportar", &["\"a\"".into(), "@n".into()]), &f, &c),
            Some(Problema::Tipo { pos: 2, .. })
        ));
        // `@flag` é bool pela árvore, e o parâmetro anota boolean: passa.
        assert_eq!(
            conferir(&acao("on_click", "exportar", &["\"a\"".into(), "@flag".into()]), &f, &c),
            None
        );
        assert!(matches!(
            conferir(&acao("on_click", "exportar", &[]), &f, &c),
            Some(Problema::Aride { .. })
        ));
    }

    #[test]
    fn sugere_o_nome_mais_proximo() {
        assert_eq!(sugerir("salvr", &funcoes()).as_deref(), Some("salvar"));
        assert_eq!(sugerir("umnomequeninguemtem", &funcoes()), None);
    }

    #[test]
    fn chaves_tipadas_de_prop_e_state() {
        let mut c = Chaves::default();
        c.do_gv(
            r#"component(name = C) {
  props {
    prop(name = rotulo, type = string)
  }
  state {
    aberto = false,
    total = 0,
  }
}"#,
        );
        assert_eq!(c.ver("rotulo"), Some(Tipo::Str));
        assert_eq!(c.ver("aberto"), Some(Tipo::Bool));
        assert_eq!(c.ver("total"), Some(Tipo::Num));
    }
}
