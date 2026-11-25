//! `glacier check` e `glacier fmt --migrate` (seção 9 do
//! `docs/PLANO_LINGUAGEM_GV_GSS.md`).
//!
//! Rodam **dentro do app** (o CLI faz `cargo run` com uma variável de ambiente,
//! como o `glacier test`), para usar exatamente o motor da versão do projeto.
//!
//! - **check**: parse de todo `.gv` e `.gss`, formas depreciadas, classes usadas
//!   e nunca definidas (e o contrário), e regra que pinta a cor do tema. Com
//!   `--strict`, qualquer achado reprova.
//! - **migrate**: reescreve as formas depreciadas no próprio texto — trocas
//!   pontuais, que preservam comentários, indentação e o resto do arquivo. Uma
//!   reimpressão a partir da árvore perderia os comentários (o parser os
//!   descarta), e por isso não é assim que se faz aqui.

use std::path::{Path, PathBuf};

use regex::Regex;

/// `glacier check`: a variável com o diretório das views.
pub const ENV_CHECK: &str = "GLACIER_CHECK";
/// `glacier check --strict`.
pub const ENV_CHECK_STRICT: &str = "GLACIER_CHECK_STRICT";
/// `glacier fmt`: a variável com o diretório das views.
pub const ENV_FMT: &str = "GLACIER_FMT";
/// `glacier fmt --migrate`: reescreve as formas depreciadas.
pub const ENV_FMT_MIGRATE: &str = "GLACIER_FMT_MIGRATE";
/// `glacier fmt --migrate-color`: passa o `.gss` para a semântica de cor do CSS
/// e liga `gss_color = css` no `app`.
pub const ENV_FMT_COLOR: &str = "GLACIER_FMT_COLOR";
/// `glacier fmt --check`: só diz o que mudaria, sem gravar.
pub const ENV_FMT_DRY: &str = "GLACIER_FMT_DRY";

/// Um achado do `check`, já no formato `arquivo:linha: mensagem`.
#[derive(Debug, Clone, PartialEq)]
pub struct Achado {
    pub arquivo: String,
    pub linha: u32,
    pub msg: String,
}

impl std::fmt::Display for Achado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.arquivo, self.linha, self.msg)
    }
}

/// Todos os arquivos com uma das extensões, recursivamente, em ordem.
fn arquivos(raiz: &Path, ext: &[&str]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn anda(d: &Path, ext: &[&str], out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        let mut es: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        es.sort();
        for p in es {
            if p.is_dir() {
                anda(&p, ext, out);
            } else if p.extension().and_then(|x| x.to_str()).is_some_and(|x| ext.contains(&x)) {
                out.push(p);
            }
        }
    }
    anda(raiz, ext, &mut out);
    out
}

// ───────────────────────────── check ─────────────────────────────

/// Os blocos `style """…"""` de um `.gv`: (linha do corpo, corpo).
fn styles_do_gv(src: &str) -> Vec<(u32, String)> {
    let re = Regex::new(r#"(?s)style(?:\([^)]*\))?\s*"""(.*?)""""#).unwrap();
    re.captures_iter(src)
        .map(|c| {
            let m = c.get(1).unwrap();
            let linha = src[..m.start()].matches('\n').count() as u32 + 1;
            (linha, m.as_str().to_string())
        })
        .collect()
}

/// As classes ESTÁTICAS que um `.gv` usa: os nomes de `class = …` (sem as partes
/// `{…}`, que são dinâmicas) e os de `class:nome = …`, com a linha.
fn classes_usadas(src: &str) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    let attr = Regex::new(r#"\bclass\s*=\s*(?:"([^"]*)"|([^\s,)"]+))"#).unwrap();
    let cond = Regex::new(r"\bclass:([A-Za-z0-9_-]+)\s*=").unwrap();
    let chaves = Regex::new(r"\{[^}]*\}").unwrap();
    for c in attr.captures_iter(src) {
        let m = c.get(1).or(c.get(2)).unwrap();
        let linha = src[..m.start()].matches('\n').count() as u32 + 1;
        // Uma expressão `{…}` inteira pode ter espaço dentro: tira-a primeiro
        // (marcando o lugar), e o token que tocava nela (`b--{@x}`) é dinâmico.
        let fixo = chaves.replace_all(m.as_str(), "\u{1}");
        for nome in fixo.split_whitespace() {
            if !nome.contains(['\u{1}', '@', '{', '}']) {
                out.push((nome.to_string(), linha));
            }
        }
    }
    for c in cond.captures_iter(src) {
        let m = c.get(1).unwrap();
        let linha = src[..m.start()].matches('\n').count() as u32 + 1;
        out.push((m.as_str().to_string(), linha));
    }
    out
}

/// As classes que um texto `.gss` define (em qualquer posição de um seletor),
/// com a linha (relativa ao texto).
fn classes_definidas(gss: &str) -> Vec<(String, u32)> {
    let sem_coment = Regex::new(r"(?s)/\*.*?\*/").unwrap().replace_all(gss, |c: &regex::Captures| {
        c[0].chars().map(|ch| if ch == '\n' { '\n' } else { ' ' }).collect::<String>()
    });
    let seletor = Regex::new(r"([^{};]*)\{").unwrap();
    let classe = Regex::new(r"\.([A-Za-z_][A-Za-z0-9_-]*)").unwrap();
    let mut out = Vec::new();
    for c in seletor.captures_iter(&sem_coment) {
        let s = c.get(1).unwrap();
        if s.as_str().trim_start().starts_with('@') {
            continue;
        }
        let linha = sem_coment[..s.start()].matches('\n').count() as u32 + 1;
        // `&--x` (aninhamento) não define uma classe solta; o nome completo
        // depende do pai — e o pai já foi contado.
        for k in classe.captures_iter(s.as_str()) {
            out.push((k[1].to_string(), linha));
        }
    }
    out
}

/// O `glacier check` de um diretório de views. Devolve os achados; `Err` só para
/// erro de E/S.
pub fn check(raiz: &Path, estrito: bool) -> std::io::Result<Vec<Achado>> {
    crate::stylesheet::set_estrito(estrito);
    let mut achados = Vec::new();
    let gvs = arquivos(raiz, &["gv"]);
    let gsss = arquivos(raiz, &["gss"]);
    let mut usadas: Vec<(String, String, u32)> = Vec::new();
    let mut definidas: Vec<(String, String, u32)> = Vec::new();
    let mut tema_fundo: Option<String> = None;

    for p in &gvs {
        let nome = p.display().to_string();
        let src = std::fs::read_to_string(p)?;
        match crate::gv::formas_depreciadas(&src) {
            Ok(formas) => {
                for (linha, msg) in formas {
                    achados.push(Achado { arquivo: nome.clone(), linha, msg: msg.to_string() });
                }
            }
            Err(d) => achados.push(Achado { arquivo: nome.clone(), linha: d.line, msg: d.message }),
        }
        for (c, l) in classes_usadas(&src) {
            usadas.push((c, nome.clone(), l));
        }
        for (linha0, corpo) in styles_do_gv(&src) {
            if let Err(e) = crate::stylesheet::StyleSheet::parse_in(&corpo, Some(&nome), linha0) {
                achados.push(Achado { arquivo: nome.clone(), linha: linha0, msg: e.to_string() });
            }
            for (c, l) in classes_definidas(&corpo) {
                definidas.push((c, nome.clone(), linha0 + l - 1));
            }
        }
    }
    let mut regras_com_fundo: Vec<(String, u32, String, String)> = Vec::new();
    for p in &gsss {
        let nome = p.display().to_string();
        let src = std::fs::read_to_string(p)?;
        match crate::stylesheet::StyleSheet::parse_in(&src, Some(&nome), 1) {
            Ok(sheet) => {
                if let Some(t) = &sheet.theme
                    && let Some(bg) = t.get("background")
                {
                    tema_fundo = Some(crate::stylesheet::substitute_vars(bg, &sheet.variables));
                }
                for (classe, regra) in &sheet.rules {
                    if let Some(bg) = &regra.background {
                        let bg = crate::stylesheet::substitute_vars(bg, &sheet.variables);
                        regras_com_fundo.push((nome.clone(), 0, format!(".{classe}"), bg));
                    }
                }
            }
            Err(e) => achados.push(Achado { arquivo: nome.clone(), linha: 1, msg: e.to_string() }),
        }
        for (c, l) in classes_definidas(&src) {
            definidas.push((c, nome.clone(), l));
        }
    }
    // O fundo do tema também pode vir de um `theme.json` ao lado.
    if tema_fundo.is_none() {
        for p in arquivos(raiz, &["json"]) {
            if p.file_name().is_some_and(|n| n == "theme.json")
                && let Ok(t) = std::fs::read_to_string(&p)
                && let Ok(v) = serde_json::from_str::<serde_json::Value>(&t)
                && let Some(bg) = v.get("background").and_then(|b| b.as_str())
            {
                tema_fundo = Some(bg.to_string());
            }
        }
    }
    if let Some(tema) = &tema_fundo {
        for (arq, linha, sel, bg) in &regras_com_fundo {
            if bg.trim().eq_ignore_ascii_case(tema.trim()) {
                achados.push(Achado {
                    arquivo: arq.clone(),
                    linha: (*linha).max(1),
                    msg: format!(
                        "`{sel}` pinta a cor do tema ({bg}) — o tema já pinta a janela; numa caixa grande isso é uma camada redobrada por quadro (PRIMITIVAS.md, \"O que custa num quadro\")"
                    ),
                });
            }
        }
    }
    // Classes: usada e nunca definida; definida e nunca usada.
    let def: std::collections::HashSet<&str> = definidas.iter().map(|d| d.0.as_str()).collect();
    let uso: std::collections::HashSet<&str> = usadas.iter().map(|u| u.0.as_str()).collect();
    let mut vistas = std::collections::HashSet::new();
    for (c, arq, l) in &usadas {
        if !def.contains(c.as_str()) && vistas.insert(("u", c.clone())) {
            achados.push(Achado {
                arquivo: arq.clone(),
                linha: *l,
                msg: format!("a classe `{c}` é usada e nenhum `.gss` nem `style` a define"),
            });
        }
    }
    for (c, arq, l) in &definidas {
        // Utilitários (`u-*`) e modificadores podem existir para uso futuro ou
        // por interpolação (`x--{@estado}`); o aviso só vale para o resto.
        let dinamico_possivel = c.contains("--");
        if !uso.contains(c.as_str()) && !c.starts_with("u-") && !dinamico_possivel && vistas.insert(("d", c.clone())) {
            achados.push(Achado {
                arquivo: arq.clone(),
                linha: *l,
                msg: format!("a classe `{c}` é definida e nenhum `.gv` a usa"),
            });
        }
    }
    crate::stylesheet::set_estrito(false);
    Ok(achados)
}

/// `GLACIER_CHECK=dir` (o `glacier check`): roda o [`check`] e devolve o código
/// de saída do processo.
pub fn check_from_env() -> Option<i32> {
    let raiz = std::env::var(ENV_CHECK).ok()?;
    let estrito = std::env::var(ENV_CHECK_STRICT).is_ok_and(|v| v == "1");
    match check(Path::new(&raiz), estrito) {
        Ok(achados) => {
            for a in &achados {
                println!("{a}");
            }
            println!("{} achado(s) em {raiz}", achados.len());
            Some(if estrito && !achados.is_empty() { 1 } else { 0 })
        }
        Err(e) => {
            eprintln!("glacier check: {e}");
            Some(2)
        }
    }
}

// ───────────────────────────── migrate ─────────────────────────────

/// Os prefixos de ação que o motor trata em Rust (ou o prelúdio Luau, por
/// nome): não viram a forma de chamada `nome(…)`.
const ACOES_DO_MOTOR: &[&str] = &[
    "clipboard", "open", "window", "style", "dialog", "app", "tray", "notifications", "audio",
    "field", "whatsthis", "textarea_end", "textarea_top",
];

/// Um valor de atributo como literal de expressão: `"…"` fica, `@x` fica, o resto
/// ganha aspas.
fn como_literal(v: &str) -> String {
    let v = v.trim();
    // `"@tab.id"` / `"{tab.id}"` sozinhos entre aspas eram a chave interpolada:
    // na expressão, a forma clara é a própria chave.
    let dentro = v.strip_prefix('"').and_then(|x| x.strip_suffix('"')).unwrap_or("");
    let chave = dentro
        .strip_prefix('@')
        .or_else(|| dentro.strip_prefix('{').and_then(|x| x.strip_suffix('}')));
    if let Some(k) = chave
        && !k.is_empty()
        && k.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        return format!("@{k}");
    }
    if v.starts_with('"') || v.starts_with('@') {
        v.to_string()
    } else {
        format!("\"{v}\"")
    }
}

/// Reescreve as formas depreciadas de um `.gv`. Devolve o texto novo e quantas
/// trocas fez.
pub fn migrar_gv(src: &str) -> (String, usize) {
    let mut s = src.to_string();
    let mut n = 0usize;
    let valor = r#"("[^"]*"|@[\w.]+|[^\s,)"]+)"#;
    let cond = r"(if|else_if)\s*=\s*(@[\w.]+)";

    // 1. Comparadores ao lado de `if =`/`else_if =` → expressão.
    let pares: [(&str, &str); 4] = [
        ("(?:equals|eq)", "=="),
        ("(?:not_equals|ne)", "!="),
        ("(?:one_of|equals_any)", "in"),
        ("(?:contains|has)", "contains"),
    ];
    for (nome, op) in pares {
        let re = Regex::new(&format!(r"{cond}\s*,\s*{nome}\s*=\s*{valor}")).unwrap();
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                n += 1;
                let (k, x, v) = (&c[1], &c[2], &c[3]);
                match op {
                    // `one_of = @alvo`: a lista vem da chave — `in @alvo`, não a
                    // lista literal `["@alvo"]` (que compararia com o texto).
                    "in" if v.trim().starts_with('@') => format!("{k} = {x} in {}", v.trim()),
                    "in" => {
                        let itens: Vec<String> = v
                            .trim_matches('"')
                            .split_whitespace()
                            .map(|i| format!("\"{i}\""))
                            .collect();
                        format!("{k} = {x} in [{}]", itens.join(", "))
                    }
                    "contains" => format!("{k} = {} in {x}", como_literal(v)),
                    _ => format!("{k} = {x} {op} {}", como_literal(v)),
                }
            })
            .into_owned();
    }
    for (nome, forma) in [("not_empty", "is not empty"), ("empty", "is empty")] {
        // `empty`, `empty = true` e `empty = ""` (o marcador escrito com valor
        // vazio) são a mesma coisa.
        let re = Regex::new(&format!(r#"{cond}\s*,\s*{nome}(?:\s*=\s*(?:true|"")|\b)"#)).unwrap();
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                n += 1;
                format!("{} = {} {forma}", &c[1], &c[2])
            })
            .into_owned();
    }

    // 2. `template(…)` → bloco.
    let mut out = String::with_capacity(s.len());
    let mut resto = s.as_str();
    while let Some(i) = resto.find("template(") {
        out.push_str(&resto[..i]);
        let depois = &resto[i + "template(".len()..];
        let Some(fim) = fecha_parentese(depois) else {
            out.push_str(&resto[i..]);
            resto = "";
            break;
        };
        let dentro = &depois[..fim];
        let apos = &depois[fim + 1..];
        let attrs = partes(dentro);
        let troca = match attrs.as_slice() {
            [a] if a == "else" => Some("else".to_string()),
            [a] if a.starts_with("if") && a[2..].trim_start().starts_with('=') => {
                Some(format!("if {}", a[2..].trim_start()[1..].trim()))
            }
            [a] if a.starts_with("else_if") => Some(format!("else if {}", a[7..].trim_start()[1..].trim())),
            _ => {
                let fe = attrs.iter().find_map(|a| a.strip_prefix(":for_each").map(|r| r.trim_start_matches([' ', '=']).trim().to_string()));
                let var = attrs.iter().find_map(|a| a.strip_prefix("var").filter(|r| r.trim_start().starts_with('=')).map(|r| r.trim_start_matches([' ', '=']).trim().to_string()));
                match (fe, var) {
                    (Some(l), Some(v)) => {
                        let outros: Vec<String> = attrs
                            .iter()
                            .filter(|a| !a.starts_with(":for_each") && !(a.starts_with("var") && a[3..].trim_start().starts_with('=')))
                            .map(|a| a.replacen("reorder_key", "key", 1))
                            .collect();
                        if outros.is_empty() {
                            Some(format!("each @{l} as {v}"))
                        } else {
                            Some(format!("each @{l} as {v} ({})", outros.join(", ")))
                        }
                    }
                    _ => None,
                }
            }
        };
        match troca {
            Some(t) => {
                n += 1;
                out.push_str(&t);
            }
            None => {
                out.push_str("template(");
                out.push_str(dentro);
                out.push(')');
            }
        }
        resto = apos;
    }
    out.push_str(resto);
    s = out;

    // 3. `:for_each = x, var = v` num nó → `each = @x as v`.
    let re = Regex::new(r":for_each\s*=\s*([\w.]+)\s*,(\s*)var\s*=\s*(\w+)").unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            n += 1;
            format!("each = @{} as {}", &c[1], &c[3])
        })
        .into_owned();

    // 4. `"{t('chave')}"` sozinho → `t("chave")`; com argumentos também.
    let re = Regex::new(r#""\{t\('([^']*)'((?:,\s*[\w.]+\s*=\s*[\w.@]+)*)\)\}""#).unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            n += 1;
            let args = c[2]
                .split(',')
                .filter(|a| !a.trim().is_empty())
                .map(|a| {
                    let (k, v) = a.split_once('=').unwrap_or((a, ""));
                    format!(", {} = {}", k.trim(), v.trim())
                })
                .collect::<String>();
            format!("t(\"{}\"{args})", &c[1])
        })
        .into_owned();

    // 5. `on_x = handler:arg` → `on_x = handler(arg)` (fora os do motor).
    let re = Regex::new(r#"(on_\w+\s*=\s*)([A-Za-z_]\w*):(@[\w.]+|[\w.-]+)(\s*[,)\n])"#).unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            if ACOES_DO_MOTOR.contains(&&c[2]) {
                return c[0].to_string();
            }
            n += 1;
            format!("{}{}({}){}", &c[1], &c[2], como_literal(&c[3]), &c[4])
        })
        .into_owned();

    // 6. A mesma coisa entre aspas: `on_x = "remover:{c.id}"` → `remover(@c.id)`,
    //    `on_x = "ir:home"` → `ir("home")`. Só o sufixo de UMA interpolação
    //    inteira ou de uma palavra: `"f:a-{x}"` (texto montado) fica como está.
    let re = Regex::new(r#"(on_\w+\s*=\s*)"([A-Za-z_]\w*):(?:\{([A-Za-z_][\w.]*)\}|([\w.-]+))""#).unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            if ACOES_DO_MOTOR.contains(&&c[2]) {
                return c[0].to_string();
            }
            n += 1;
            let arg = match (c.get(3), c.get(4)) {
                (Some(chave), _) => format!("@{}", chave.as_str()),
                (_, Some(lit)) => como_literal(lit.as_str()),
                _ => unreachable!(),
            };
            format!("{}{}({})", &c[1], &c[2], arg)
        })
        .into_owned();

    (s, n)
}

/// [`migrar_gv`] aplicado só aos blocos ` ```gv ` de um Markdown — a
/// documentação de um app (um `AGENTS.md`, um README) anda junto com o código.
/// A prosa fica como está: ela pode estar explicando justamente a forma antiga.
pub fn migrar_markdown(src: &str) -> (String, usize) {
    let mut out = String::with_capacity(src.len());
    let mut n = 0;
    let mut resto = src;
    while let Some(i) = resto.find("```gv\n") {
        let ini = i + "```gv\n".len();
        out.push_str(&resto[..ini]);
        let Some(fim) = resto[ini..].find("```") else {
            out.push_str(&resto[ini..]);
            resto = "";
            break;
        };
        let (bloco, k) = migrar_gv(&resto[ini..ini + fim]);
        n += k;
        out.push_str(&bloco);
        resto = &resto[ini + fim..];
    }
    out.push_str(resto);
    (out, n)
}

/// O índice do `)` que fecha o `(` já consumido, respeitando aspas e aninhamento.
fn fecha_parentese(s: &str) -> Option<usize> {
    let mut prof = 0i32;
    let mut aspas = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => aspas = !aspas,
            '(' if !aspas => prof += 1,
            ')' if !aspas => {
                if prof == 0 {
                    return Some(i);
                }
                prof -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Os atributos de uma lista `a = 1, b = "x, y"`, pela vírgula de fora.
fn partes(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut atual = String::new();
    let mut aspas = false;
    let mut prof = 0i32;
    for c in s.chars() {
        match c {
            '"' => aspas = !aspas,
            '(' | '[' if !aspas => prof += 1,
            ')' | ']' if !aspas => prof -= 1,
            ',' if !aspas && prof == 0 => {
                let t = atual.trim().to_string();
                if !t.is_empty() {
                    out.push(t);
                }
                atual.clear();
                continue;
            }
            _ => {}
        }
        atual.push(c);
    }
    let t = atual.trim().to_string();
    if !t.is_empty() {
        out.push(t);
    }
    out
}

/// Reescreve as formas depreciadas de um `.gss` (`hidden: true` → `display: none`).
pub fn migrar_gss(src: &str) -> (String, usize) {
    let mut n = 0;
    let re = Regex::new(r"\bhidden\s*:\s*(true|false|1|0)\b").unwrap();
    let s = re
        .replace_all(src, |c: &regex::Captures| {
            n += 1;
            if matches!(&c[1], "true" | "1") { "display: none".to_string() } else { "display: flex".to_string() }
        })
        .into_owned();
    (s, n)
}

/// `--migrate-color`: nas regras cujo seletor é só `.classe` (ou `.classe:estado`)
/// de uma classe usada num `button`/`select`/`comboedit`/`menu`, troca o sentido
/// antigo do `color` (fundo) pelo do CSS. As regras de outras classes não mudam
/// (lá o `color` já era a cor do conteúdo). Uma classe usada nos dois tipos de
/// nó é ambígua: ganha um comentário `TODO glacier-migrate` e fica como está.
pub fn migrar_cor_gss(src: &str, de_botao: &std::collections::HashSet<String>, ambiguas: &std::collections::HashSet<String>) -> (String, usize) {
    let re = Regex::new(r"(?m)^(\s*)\.([A-Za-z_][\w-]*)(:[a-z]+)?\s*\{([^{}]*)\}").unwrap();
    let mut n = 0;
    let s = re
        .replace_all(src, |c: &regex::Captures| {
            let (ind, classe, estado, corpo) = (&c[1], &c[2], c.get(3).map_or("", |m| m.as_str()), &c[4]);
            if ambiguas.contains(classe) {
                n += 1;
                return format!("{ind}/* TODO glacier-migrate: `.{classe}` é usada em botão e em outro nó — revise o `color` */\n{}", &c[0].trim_start_matches(['\n']));
            }
            if !de_botao.contains(classe) {
                return c[0].to_string();
            }
            n += 1;
            let novo = if estado.is_empty() {
                // `color` (fundo) → `background`; `text_color` (rótulo) → `color`.
                let t = Regex::new(r"\bcolor\s*:").unwrap().replace_all(corpo, "__FUNDO__:");
                let t = Regex::new(r"\btext_color\s*:").unwrap().replace_all(&t, "color:");
                t.replace("__FUNDO__:", "background:")
            } else {
                Regex::new(r"\btext_color\s*:").unwrap().replace_all(corpo, "color:").into_owned()
            };
            format!("{ind}.{classe}{estado} {{{novo}}}")
        })
        .into_owned();
    (s, n)
}

/// As classes estáticas usadas em nós `button`/`select`/`comboedit`/`menu` e as
/// usadas em qualquer outro nó, de todos os `.gv`.
fn classes_por_tipo(gvs: &[(PathBuf, String)]) -> (std::collections::HashSet<String>, std::collections::HashSet<String>) {
    let re = Regex::new(r#"\b([a-z][a-z_]*)\s*\(([^()]*(?:\([^()]*\)[^()]*)*)\)"#).unwrap();
    let mut botao = std::collections::HashSet::new();
    let mut outro = std::collections::HashSet::new();
    for (_, src) in gvs {
        for c in re.captures_iter(src) {
            let tag = &c[1];
            let alvo = if crate::stylesheet::TAGS_COM_FUNDO_PROPRIO.contains(&tag) { &mut botao } else { &mut outro };
            for (cl, _) in classes_usadas(&c[2]) {
                alvo.insert(cl);
            }
        }
    }
    (botao, outro)
}

/// `GLACIER_FMT=dir` (o `glacier fmt`): migra e grava (ou, com `--check`, só lista).
pub fn fmt_from_env() -> Option<i32> {
    let raiz = std::env::var(ENV_FMT).ok()?;
    let migrar = std::env::var(ENV_FMT_MIGRATE).is_ok_and(|v| v == "1");
    let cor = std::env::var(ENV_FMT_COLOR).is_ok_and(|v| v == "1");
    let seco = std::env::var(ENV_FMT_DRY).is_ok_and(|v| v == "1");
    Some(match fmt(Path::new(&raiz), migrar, cor, seco) {
        Ok(mudados) => {
            for (p, k) in &mudados {
                println!("{}: {k} troca(s){}", p.display(), if seco { " (não gravado)" } else { "" });
            }
            println!("{} arquivo(s) {}", mudados.len(), if seco { "mudariam" } else { "mudados" });
            if seco && !mudados.is_empty() { 1 } else { 0 }
        }
        Err(e) => {
            eprintln!("glacier fmt: {e}");
            2
        }
    })
}

/// O `glacier fmt` de um diretório: devolve os arquivos que mudaram (ou
/// mudariam, com `seco`) e quantas trocas cada um teve. Sempre normaliza o
/// espaço no fim das linhas e a quebra de linha final. Com `migrar`, os blocos
/// ` ```gv ` de um `.md` também são reescritos ([`migrar_markdown`]).
pub fn fmt(raiz: &Path, migrar: bool, cor: bool, seco: bool) -> std::io::Result<Vec<(PathBuf, usize)>> {
    let mut mudados = Vec::new();
    let gvs: Vec<(PathBuf, String)> = arquivos(raiz, &["gv"])
        .into_iter()
        .map(|p| std::fs::read_to_string(&p).map(|s| (p, s)))
        .collect::<std::io::Result<_>>()?;
    let (botao, outro) = if cor { classes_por_tipo(&gvs) } else { Default::default() };
    let ambiguas: std::collections::HashSet<String> = botao.intersection(&outro).cloned().collect();
    let so_botao: std::collections::HashSet<String> = botao.difference(&ambiguas).cloned().collect();

    let normaliza = |s: &str| -> String {
        let mut o: String = s.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n");
        o.push('\n');
        o
    };
    for (p, src) in &gvs {
        let (mut novo, mut k) = if migrar { migrar_gv(src) } else { (src.clone(), 0) };
        if cor {
            // O `style """…"""` de um `.gv` também é `.gss`.
            let (c, kc) = migrar_cor_gss(&novo, &so_botao, &ambiguas);
            novo = c;
            k += kc;
            if novo.trim_start().starts_with("app(") && !novo.contains("gss_color") {
                novo = novo.replacen("app(", "app(\n  gss_color = css,", 1);
                k += 1;
            }
        }
        if migrar {
            let (g, kg) = migrar_gss(&novo);
            novo = g;
            k += kg;
        }
        let novo = normaliza(&novo);
        if novo != *src {
            if !seco {
                std::fs::write(p, &novo)?;
            }
            mudados.push((p.clone(), k));
        }
    }
    for p in arquivos(raiz, &["gss"]) {
        let src = std::fs::read_to_string(&p)?;
        let (mut novo, mut k) = if migrar { migrar_gss(&src) } else { (src.clone(), 0) };
        if cor {
            let (c, kc) = migrar_cor_gss(&novo, &so_botao, &ambiguas);
            novo = c;
            k += kc;
        }
        let novo = normaliza(&novo);
        if novo != src {
            if !seco {
                std::fs::write(&p, &novo)?;
            }
            mudados.push((p, k));
        }
    }
    if migrar {
        for p in arquivos(raiz, &["md"]) {
            let src = std::fs::read_to_string(&p)?;
            let (novo, k) = migrar_markdown(&src);
            if novo != src {
                if !seco {
                    std::fs::write(&p, &novo)?;
                }
                mudados.push((p, k));
            }
        }
    }
    Ok(mudados)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparadores_viram_expressao() {
        let (s, n) = migrar_gv(r#"text("a", if = @x, equals = admin)
text("b", if = @x, not_equals = "")
text("c", if = @view, one_of = "a b")
text("d", if = @l, empty)
text("e", else_if = @t, equals = logs)
text("f", if = @m, empty = "", class = x)"#);
        assert_eq!(n, 6);
        assert!(s.contains("if = @m is empty, class = x"), "{s}");
        assert!(s.contains(r#"if = @x == "admin""#), "{s}");
        assert!(s.contains(r#"if = @x != """#), "{s}");
        assert!(s.contains(r#"if = @view in ["a", "b"]"#), "{s}");
        assert!(s.contains("if = @l is empty"), "{s}");
        assert!(s.contains(r#"else_if = @t == "logs""#), "{s}");
    }

    #[test]
    fn template_vira_bloco() {
        let (s, _) = migrar_gv("template(if = @a == \"x\") {\n  text(\"1\")\n}\ntemplate(else) {\n  text(\"2\")\n}\ntemplate(:for_each = l, var = v, reorder_key = id) {\n  text(\"@v\")\n}\n");
        assert!(s.starts_with("if @a == \"x\" {"), "{s}");
        assert!(s.contains("\nelse {"), "{s}");
        assert!(s.contains("each @l as v (key = id) {"), "{s}");
    }

    #[test]
    fn for_each_i18n_e_acoes() {
        let (s, _) = migrar_gv(r#"Row(:for_each = itens, var = s, title = @s.name)
text("{t('comum.ok')}")
text("{t('svc.nome', nome=nome)}")
button("x", on_click = abrir:@id, on_change = field:nome)
button("y", on_click = window:drag)"#);
        assert!(s.contains("each = @itens as s"), "{s}");
        assert!(s.contains(r#"text(t("comum.ok"))"#), "{s}");
        assert!(s.contains(r#"t("svc.nome", nome = nome)"#), "{s}");
        assert!(s.contains("on_click = abrir(@id)"), "{s}");
        assert!(s.contains("on_change = field:nome"), "o prefixo do motor fica: {s}");
        assert!(s.contains("on_click = window:drag"), "{s}");
    }

    #[test]
    fn one_of_com_chave_vira_in_da_chave() {
        let (s, n) = migrar_gv("button(\"x\", if = @view, one_of = @target, on_click = ir)\n");
        assert_eq!(n, 1);
        assert!(s.contains("if = @view in @target,"), "{s}");
        let (s, _) = migrar_gv("text(\"x\", if = @aba, one_of = \"a b\")\n");
        assert!(s.contains(r#"if = @aba in ["a", "b"]"#), "{s}");
    }

    #[test]
    fn chave_entre_aspas_vira_a_chave() {
        let (s, _) = migrar_gv("button(\"x\", if = @aba, equals = \"@tab.id\", on_click = ir)\n");
        assert!(s.contains("if = @aba == @tab.id,"), "{s}");
        let (s, _) = migrar_gv("text(\"x\", if = @aba, not_equals = \"{tab.id}\")\n");
        assert!(s.contains("if = @aba != @tab.id"), "{s}");
    }

    #[test]
    fn embutido_textarea_nao_vira_chamada() {
        let src = "button(\"↓\", on_click = textarea_end:log)\nbutton(\"↑\", on_click = \"textarea_top:log\")\n";
        let (s, n) = migrar_gv(src);
        assert_eq!(n, 0, "{s}");
        assert_eq!(s, src);
    }

    #[test]
    fn acao_entre_aspas_vira_chamada() {
        let src = "button(\"x\", on_click = \"remover:{c.id}\")\nbutton(\"y\", on_click = \"ir:home\")\nbutton(\"z\", on_click = \"clipboard:k\")\nbutton(\"w\", on_click = \"f:a-{x}\")\n";
        let (s, n) = migrar_gv(src);
        assert_eq!(n, 2, "{s}");
        assert!(s.contains("on_click = remover(@c.id)"), "{s}");
        assert!(s.contains("on_click = ir(\"home\")"), "{s}");
        assert!(s.contains("on_click = \"clipboard:k\""), "embutido do motor fica: {s}");
        assert!(s.contains("on_click = \"f:a-{x}\""), "texto montado fica: {s}");
        assert_eq!(migrar_gv(&s).1, 0, "idempotente");
    }

    #[test]
    fn migracao_e_idempotente() {
        let src = "text(\"a\", if = @x, equals = y)\n";
        let (uma, _) = migrar_gv(src);
        let (duas, n) = migrar_gv(&uma);
        assert_eq!(uma, duas);
        assert_eq!(n, 0);
    }

    #[test]
    fn markdown_so_migra_os_blocos_gv() {
        let md = "Antes: `if = @x, equals = y` era a forma.\n\n```gv\ntext(\"a\", if = @x, equals = y)\n```\n";
        let (s, n) = migrar_markdown(md);
        assert_eq!(n, 1);
        assert!(s.contains("Antes: `if = @x, equals = y`"), "a prosa fica: {s}");
        assert!(s.contains("text(\"a\", if = @x == \"y\")"), "{s}");
    }

    #[test]
    fn fmt_migra_os_blocos_gv_de_um_markdown() {
        let d = std::env::temp_dir().join("glacier_fmt_markdown");
        std::fs::create_dir_all(&d).unwrap();
        let md = d.join("AGENTS.md");
        std::fs::write(&md, "Prosa `equals = y` fica.\n\n```gv\ntext(\"a\", if = @x, equals = y)\n```\n").unwrap();
        let mudados = fmt(&d, true, false, false).unwrap();
        assert!(mudados.iter().any(|(p, k)| p == &md && *k == 1), "{mudados:?}");
        let s = std::fs::read_to_string(&md).unwrap();
        assert!(s.contains("Prosa `equals = y` fica."), "{s}");
        assert!(s.contains("if = @x == \"y\""), "{s}");
    }

    #[test]
    fn hidden_vira_display() {
        let (s, n) = migrar_gss(".a { hidden: true; }\n.b { hidden: false; }");
        assert_eq!(n, 2);
        assert!(s.contains("display: none") && s.contains("display: flex"), "{s}");
    }

    #[test]
    fn cor_de_botao_troca_de_sentido() {
        let botao: std::collections::HashSet<String> = ["btn".to_string()].into();
        let (s, _) = migrar_cor_gss(
            ".btn { color: #111; text_color: #fff; }\n.btn:hover { background: #222; text_color: #000; }\n.rotulo { color: #333; }",
            &botao,
            &Default::default(),
        );
        assert!(s.contains(".btn { background: #111; color: #fff; }"), "{s}");
        assert!(s.contains(".btn:hover { background: #222; color: #000; }"), "{s}");
        assert!(s.contains(".rotulo { color: #333; }"), "{s}");
    }

    #[test]
    fn classes_usadas_e_definidas() {
        let u = classes_usadas(r#"column(class = "a b--{@x} {@y}", class:c--on = @z) text(class = d)"#);
        let nomes: Vec<&str> = u.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(nomes, ["a", "d", "c--on"]);
        let d = classes_definidas(".a { x: 1; }\n/* .nao */\n.b > .c:hover { y: 2; }\n@media (max-width: 1) { .e { z: 1; } }");
        let nomes: Vec<&str> = d.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(nomes, ["a", "b", "c", "e"]);
    }
}
