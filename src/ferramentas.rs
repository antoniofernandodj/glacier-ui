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
/// `glacier check --tipos`: escreve o `ctx.d.luau` com o tipo de cada chave do
/// contexto (além dos achados). `1` usa `<raiz>/scripts/ctx.d.luau`; um caminho
/// absolute escreve ali.
pub const ENV_CHECK_TIPOS: &str = "GLACIER_CHECK_TIPOS";

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
    // ── o que as Frentes 1, 6, 8 e 11 do `TODO.md` acrescentam ──────────────
    let pan = panorama(raiz)?;
    achados.extend(verificar_handlers(&gvs, &pan));
    achados.extend(verificar_verdade(&gvs, &pan));
    achados.extend(verificar_imports(&gvs, &pan));
    achados.extend(verificar_sombra(&gvs, &pan));
    achados.extend(verificar_cor(&gvs, &gsss));

    crate::stylesheet::set_estrito(false);
    Ok(achados)
}

// ─────────────────────────── o panorama da árvore ───────────────────────────

/// O que o `check` sabe sobre a árvore de views: os handlers de cada script, o
/// tipo de cada chave do contexto e os componentes registrados.Montado uma vez
/// por `check`, lido por todas as verificações.
pub(crate) struct Panorama {
    /// As funções globais declaradas por qualquer `.luau` da árvore.
    pub funcoes: Vec<crate::handlers::Funcao>,
    /// `chave → tipo` do que os scripts gravam no `ctx` e do que `props`/`state`
    /// declaram.
    pub chaves: crate::handlers::Chaves,
    /// Os handlers referenciados por algum `.gv`.
    pub chamados: std::collections::HashSet<String>,
    /// As chaves que alguém grava como **texto** (de um `textinput`, de um
    /// `ctx.x = "…"`): com `app(truthy = typed)` um teste nelas muda de valor.
    pub textos: std::collections::HashSet<String>,
    /// As chaves escritas no contexto global por script ou widget nenhum `.gv` de
    /// componente — o que um `state`/`prop` de mesmo nome esconderia.
    pub globais: std::collections::HashSet<String>,
    /// Os nomes de componente registrados na árvore (por `component(name = …)` ou
    /// por arquivo `.component.gv`).
    pub componentes: std::collections::HashSet<String>,
    /// `(arquivo, handler, linha)` dos scripts de um diretório de handler — onde
    /// "definido e nunca chamado por `.gv` nenhum" é um aviso honesto.
    pub nunca_chamados: Vec<(String, String, u32)>,
}

/// Lê a árvore inteira uma vez: scripts, `.gv`, imports.
fn panorama(raiz: &Path) -> std::io::Result<Panorama> {
    use std::collections::HashSet;
    let mut p = Panorama {
        funcoes: Vec::new(),
        chaves: crate::handlers::Chaves::default(),
        chamados: HashSet::new(),
        textos: HashSet::new(),
        globais: HashSet::new(),
        componentes: HashSet::new(),
        nunca_chamados: Vec::new(),
    };
    for f in arquivos(raiz, &["luau", "lua"]) {
        let src = std::fs::read_to_string(&f)?;
        let funcoes = crate::handlers::assinaturas(&src);
        // Um script de módulo (`utils.luau`) é chamado de dentro de outro Luau; o
        // aviso só vale onde a convenção do app põe handler de template.
        let de_handler = f.display().to_string().replace('\\', "/").contains("handler");
        for fu in &funcoes {
            if de_handler {
                p.nunca_chamados.push((
                    f.display().to_string(),
                    fu.nome.clone(),
                    fu.linha,
                ));
            }
        }
        p.funcoes.extend(funcoes);
        p.chaves.do_script(&src);
    }
    for f in arquivos(raiz, &["gv"]) {
        let src = std::fs::read_to_string(&f)?;
        p.chaves.do_gv(&src);
        for a in crate::handlers::acoes(&src) {
            p.chamados.insert(a.handler.clone());
        }
        // `component(name = X)` e o nome do arquivo `.component.gv`.
        for c in Regex::new(r"(?m)^\s*component\s*\([^)]*name\s*=\s*([\w]+)")
            .unwrap()
            .captures_iter(&src)
        {
            p.componentes.insert(c[1].to_string());
        }
        if let Some(n) = f.file_name().and_then(|n| n.to_str()) {
            if let Some(stem) = n.strip_suffix(".component.gv") {
                p.componentes.insert(nome_de_arquivo(stem));
            }
        }
        // Chaves que recebem TEXTO de um widget, e o que viaja no global.
        for c in Regex::new(r"(?mi)(?:^|[(,\s])(?:textinput|input|textarea|texteditor|maskedinput|checkbox|toggle|toggler|comboedit|select|calendar|colorwheel|rating|spinbox|slider)\s*\([^)]*?::?[a-z_]+\s*=\s*@?([\w.]+)")
            .unwrap()
            .captures_iter(&src)
        {
            p.textos.insert(c[1].to_string());
            p.globais.insert(c[1].to_string());
        }
        for c in Regex::new(r#"(?mi)\b(?:disabled|hidden|busy|if|else_if)\s*=\s*[@{(]*\s*!?\s*@([\w.]+)"#)
            .unwrap()
            .captures_iter(&src)
        {
            p.globais.insert(c[1].to_string());
        }
    }
    for f in arquivos(raiz, &["md"]) {
        let src = std::fs::read_to_string(&f)?;
        p.chaves.do_gv(&src);
    }
    // As chaves que um script grava com tipo também são globais.
    for (k, _) in &p.chaves.tipos {
        p.globais.insert(k.clone());
    }
    p.componentes.remove("");
    Ok(p)
}

/// `service_card` → `ServiceCard` (o nome com que um arquivo `.component.gv` é
/// usado no markup, quando ninguém dá `as =`).
fn nome_de_arquivo(stem: &str) -> String {
    let mut out = String::new();
    let mut maiuscula = true;
    for c in stem.chars() {
        if c == '_' || c == '-' {
            maiuscula = true;
            continue;
        }
        out.push(if maiuscula { c.to_ascii_uppercase() } else { c });
        maiuscula = false;
    }
    out
}

// ─────────────────────── Frente 6: handler ↔ template ───────────────────────

/// Confere cada ação de cada `.gv` contra os scripts da árvore: existe, aridade,
/// tipo de argumento. É o que substitui o `tools/busy_actions.py --check` de um
/// app na parte "o handler do template existe no script".
fn verificar_handlers(gvs: &[PathBuf], pan: &Panorama) -> Vec<Achado> {
    use crate::handlers as h;
    let mut out = Vec::new();
    for p in gvs {
        let nome = p.display().to_string();
        let Ok(src) = std::fs::read_to_string(p) else { continue };
        for a in h::acoes(&src) {
            if h::EMBUTIDAS.contains(&a.handler.as_str()) {
                continue;
            }
            let linha = a.linha;
            if a.nua {
                out.push(Achado {
                    arquivo: nome.clone(),
                    linha,
                    msg: format!(
                        "ação `{}` no `{}` está na forma nua — ação é sempre chamada: `{}()`",
                        a.handler, a.attr, a.handler
                    ),
                });
            }
            let problemas = h::conferir(&a, &pan.funcoes, &pan.chaves);
            match problemas {
                Some(h::Problema::Ausente) => {
                    let sug = h::sugerir(&a.handler, &pan.funcoes);
                    let dica = sug.map_or_else(String::new, |s| format!("; o mais próximo é `{s}`"));
                    out.push(Achado {
                        arquivo: nome.clone(),
                        linha,
                        msg: format!(
                            "o handler `{}` de `{}` não existe em nenhum script da árvore{}",
                            a.handler, a.attr, dica
                        ),
                    });
                }
                Some(h::Problema::Aride { dada, esperada }) => out.push(Achado {
                    arquivo: nome.clone(),
                    linha,
                    msg: format!(
                        "`{}` recebe {dada} argumento(s); a assinatura espera {esperada}",
                        a.handler
                    ),
                }),
                Some(h::Problema::Tipo { pos, esperado, dado }) => out.push(Achado {
                    arquivo: nome.clone(),
                    linha,
                    msg: format!(
                        "o argumento {pos} de `{}` é {dado}; a assinatura anota `{esperado}`",
                        a.handler
                    ),
                }),
                None => {}
            }
        }
    }
    // Handler definido e nunca chamado por `.gv` nenhum — aviso, e só nos
    // diretórios de handler do app (um módulo é chamado de dentro do Luau).
    for (arquivo, nome, linha) in &pan.nunca_chamados {
        if !pan.chamados.contains(nome) {
            out.push(Achado {
                arquivo: arquivo.clone(),
                linha: *linha,
                msg: format!(
                    "o handler `{nome}` não é chamado por nenhum `.gv` (se só o Luau o chama, não tem conserto aqui)"
                ),
            });
        }
    }
    out
}

// ────────────────── Frente 1: o que a regra tipada muda ──────────────────

/// Os testes de verdade cuja chave pode receber **texto**: hoje `"não"`, `"no"` e
/// `"off"` são falsos; com `app(truthy = typed)` passam a ser texto, e texto é
/// verdadeiro. A marca só é ligada pela migração quando esta lista está vazia.
fn verificar_verdade(gvs: &[PathBuf], pan: &Panorama) -> Vec<Achado> {
    let mut out = Vec::new();
    let re =
        Regex::new(r"(?mi)\b(if|else_if|hidden|disabled|busy)\s*=\s*!?\s*@([\w.]+)\b").unwrap();
    for p in gvs {
        let nome = p.display().to_string();
        let Ok(src) = std::fs::read_to_string(p) else { continue };
        let mut vistas = std::collections::HashSet::new();
        for c in re.captures_iter(&src) {
            let chave = c[2].to_string();
            if !pan.textos.contains(&chave) || !vistas.insert(chave.clone()) {
                continue;
            }
            out.push(Achado {
                arquivo: nome.clone(),
                linha: linha_do(&src, &c[0]),
                msg: format!(
                    "`{} = @{chave}` depende da regra de verdade: `{chave}` recebe texto, e com `app(truthy = typed)` um texto é sempre verdadeiro — teste `@{chave} != \"\"` ou dê o tipo à chave",
                    &c[1]
                ),
            });
        }
    }
    out
}

/// A linha (1-based) de um trecho dentro do arquivo.
fn linha_do(src: &str, trecho: &str) -> u32 {
    match src.find(trecho) {
        Some(i) => src[..i].matches('\n').count() as u32 + 1,
        None => 1,
    }
}

// ───────────────── Frente 11: import e escopo local ─────────────────

/// O nome usado sem `link(rel = import)` no arquivo. Hoje o registro é global e
/// vence quem importou primeiro; com o escopo local por padrão, usar sem importar
/// é erro com a linha e o `link` a acrescentar.
fn verificar_imports(gvs: &[PathBuf], pan: &Panorama) -> Vec<Achado> {
    let mut out = Vec::new();
    if pan.componentes.is_empty() {
        return out;
    }
    let globais = imports_globais(gvs);
    let re_uso = Regex::new(r"(?m)^\s*([A-Z][A-Za-z0-9_]*)\s*[({]").unwrap();
    for p in gvs {
        let nome = p.display().to_string();
        if nome.ends_with("app.gv") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(p) else { continue };
        let importados = imports_do_arquivo(&src);
        let declarados = declares_do_arquivo(&src);
        let mut vistas = std::collections::HashSet::new();
        for c in re_uso.captures_iter(&src) {
            let t = &c[1];
            if !pan.componentes.contains(t)
                || importados.contains(t)
                || declarados.contains(t)
                || globais.contains(t)
                || !vistas.insert(t.to_string())
            {
                continue;
            }
            out.push(Achado {
                arquivo: nome.clone(),
                linha: linha_do(&src, &c[0]),
                msg: format!(
                    "`{t}` é usado sem `link(rel = import, href = \"…\", as = {t})` neste arquivo — o import é local ao arquivo",
                ),
            });
        }
    }
    out
}

/// Os nomes que um `.gv` importa (`link(rel = import, …, as = Nome)`), e os que o
/// arquivo declara por si (`component(name = …)`).
fn imports_do_arquivo(src: &str) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let re = Regex::new(r#"(?mi)link\s*\([^)]*(?:rel\s*=\s*import|rel\s*=\s*\"import\")[^)]*\)"#).unwrap();
    for c in re.captures_iter(src) {
        let bloco = &c[0];
        if let Some(n) = Regex::new(r#"as\s*=\s*\"?([\w]+)"#).unwrap().captures(bloco) {
            out.insert(n[1].to_string());
        } else if let Some(h) = Regex::new(r#"(?:href|src|from)\s*=\s*\"?([^\" ,)]+)"#).unwrap().captures(bloco) {
            let stem = h[1].rsplit('/').next().unwrap_or("").strip_suffix(".gv").unwrap_or("");
            let stem = stem.strip_suffix(".component").unwrap_or(stem);
            if !stem.is_empty() {
                out.insert(nome_de_arquivo(stem));
            }
        }
    }
    out
}

fn declares_do_arquivo(src: &str) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for c in Regex::new(r"(?m)^\s*component\s*\([^)]*name\s*=\s*([\w]+)").unwrap().captures_iter(src) {
        out.insert(c[1].to_string());
    }
    out
}

/// Os `global = true` do `app.gv` — a única forma de um nome valer para o app.
fn imports_globais(gvs: &[PathBuf]) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for p in gvs {
        if !p.display().to_string().ends_with("app.gv") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(p) else { continue };
        let re =
            Regex::new(r"(?mi)link\s*\([^)]*global\s*=\s*(?:true|1)[^)]*\)").unwrap();
        for c in re.captures_iter(&src) {
            if let Some(n) = Regex::new(r#"as\s*=\s*\"?([\w]+)"#).unwrap().captures(&c[0]) {
                out.insert(n[1].to_string());
            }
        }
    }
    out
}

/// Frente 11: uma chave de `state`/`prop` com o mesmo nome de uma chave global que
/// o app escreve esconde a global em silêncio.
fn verificar_sombra(gvs: &[PathBuf], pan: &Panorama) -> Vec<Achado> {
    let mut out = Vec::new();
    let re = Regex::new(r"(?m)^\s*([\w\-]+)\s*=\s*").unwrap();
    for p in gvs {
        let nome = p.display().to_string();
        let Ok(src) = std::fs::read_to_string(p) else { continue };
        for bloco in crate::handlers::blocos_state(&src) {
            for c in re.captures_iter(&bloco) {
                let chave = &c[1];
                if pan.globais.contains(chave) {
                    out.push(Achado {
                        arquivo: nome.clone(),
                        linha: linha_do(&src, chave),
                        msg: format!(
                            "o estado `{chave}` esconde a chave global `{chave}`, que o app também escreve — renomeie ou declare `prop(name = {chave}, shadow = true)`",
                        ),
                    });
                }
            }
        }
    }
    out
}

// ───────────────── Frente 8: cor do CSS como padrão ─────────────────

/// `text_color` sobrando num app já migrado para `gss_color = css`, e o aviso de
/// depreiação de quem ainda pede `legacy`.
fn verificar_cor(gvs: &[PathBuf], gsss: &[PathBuf]) -> Vec<Achado> {
    let mut out = Vec::new();
    let mut css = false;
    let mut legacy = None;
    for p in gvs {
        let Ok(src) = std::fs::read_to_string(p) else { continue };
        let n = p.display().to_string();
        if src.contains("gss_color = css") {
            css = true;
        }
        if src.contains("gss_color = legacy") {
            legacy = Some(n);
        }
    }
    if let Some(a) = legacy {
        out.push(Achado {
            arquivo: a,
            linha: 1,
            msg: "`app(gss_color = legacy)` está depreciado — `glacier fmt --migrate-color` passa o `.gss` para a semântica do CSS (`background` é o fundo, `color` é o conteúdo)".to_string(),
        });
    }
    if css {
        let re = Regex::new(r"(?m)^\s*text_color\s*:").unwrap();
        for p in gsss {
            let src = std::fs::read_to_string(p).unwrap_or_default();
            for c in re.captures_iter(&src) {
                out.push(Achado {
                    arquivo: p.display().to_string(),
                    linha: linha_do(&src, &c[0]),
                    msg: "`text_color` é da semântica `legacy`; num app `gss_color = css` o conteúdo é o `color`".to_string(),
                });
                break;
            }
        }
        for p in gvs {
            let src = std::fs::read_to_string(p).unwrap_or_default();
            if src.contains("text_color:") {
                for (linha0, corpo) in styles_do_gv(&src) {
                    if re.is_match(&corpo) {
                        out.push(Achado {
                            arquivo: p.display().to_string(),
                            linha: linha0,
                            msg: "`text_color` num `style` de um app `gss_color = css` sobra da migração".to_string(),
                        });
                    }
                }
            }
        }
    }
    out
}

/// O texto do `ctx.d.luau` da árvore: a declaração do `ctx` com o tipo de cada
/// chave que o `check` reconhece, para o luau-lsp. É gerado, não escrito à mão.
///
/// A forma é um registro **com reserva** (`[string]: any`): o tipo declarado vale
/// para a chave conhecida (`ctx.ativo = "x"` vira erro no editor quando `ativo` é
/// bool), e o resto ainda passa. Para fechar o contrato de vez — toda chave do app
/// declarada, nada fora dela — remova a linha da reserva: é o passo que quebra um
/// app grande (ver o `TODO.md`, "adaptar o rustploy").
pub fn tipos_do_ctx(pan: &Panorama) -> String {
    use crate::handlers::Tipo;
    let mut chaves: Vec<&String> = pan.chaves.tipos.keys().collect();
    for k in &pan.globais {
        if !pan.chaves.tipos.contains_key(k.as_str()) {
            chaves.push(k);
        }
    }
    chaves.sort();
    chaves.dedup();

    let cabecalho = r"--!strict
-- GERADO POR `glacier check --tipos` — não edite à mão: a próxima geração apaga.
--
-- O `ctx` é injetado pelo motor; este arquivo é o que o editor sabe sobre ele.
-- Cada chave vem de um `.luau` (`ctx.x = …`), de um `props { prop(type = …) }` ou
-- de um `state { … }` dos `.gv` da árvore. A reserva `[string]: any`, no fim,
-- deixa passar o que o scanner não conheceu — tirá-la é o contrato fechado.
declare ctx: {
";
    let mut out = String::from(cabecalho);
    let rotulo = |o: &str| match o {
        "estado" => "-- estado local (`state { … }` dos componentes)",
        "prop" => "-- props declaradas (`props { prop(type = …) }`)",
        _ => "-- gravadas por script (`ctx.chave = …`)",
    };
    let mut vistas = 0usize;
    for origem in ["ctx", "prop", "estado"] {
        let itens: Vec<String> = chaves
            .iter()
            .filter(|k| pan.chaves.origem.get(k.as_str()).copied().unwrap_or("ctx") == origem)
            .map(|k| {
                let t = match pan.chaves.tipos.get(k.as_str()) {
                    Some(Tipo::Bool) => "boolean",
                    Some(Tipo::Num) => "number",
                    Some(Tipo::Str) => "string",
                    Some(Tipo::Tabela) => "table",
                    Some(Tipo::Outro) | None => "any",
                };
                format!("    {k}: {t},\n")
            })
            .collect();
        if itens.is_empty() {
            continue;
        }
        vistas += itens.len();
        out.push_str(&format!("    {}\n", rotulo(origem)));
        for i in itens {
            out.push_str(&i);
        }
    }
    // O que o `globais` viu mas nenhum script tipou: existe, tipo desconhecido.
    let resto: Vec<String> = chaves
        .iter()
        .filter(|k| !pan.chaves.tipos.contains_key(k.as_str()))
        .map(|k| format!("    {k}: any,\n"))
        .collect();
    if !resto.is_empty() {
        vistas += resto.len();
        out.push_str("    -- lidas no markup, escritas por caminho nenhum que o scanner visse\n");
        for i in resto {
            out.push_str(&i);
        }
    }
    out.push_str("    [string]: any,\n}\n");
    out.push_str(&format!(
        "\n// {} chave(s) em {} vista(s) pelo scanner da árvore.\n",
        vistas,
        pan.chaves.tipos.len()
    ));
    out
}

/// O `glacier check --tipos`: a árvore lida e o texto do `ctx.d.luau`.
pub fn tipos(raiz: &Path) -> std::io::Result<String> {
    Ok(tipos_do_ctx(&panorama(raiz)?))
}

/// `GLACIER_CHECK=dir` (o `glacier check`): roda o [`check`] e devolve o código
/// de saída do processo.
pub fn check_from_env() -> Option<i32> {
    let raiz = std::env::var(ENV_CHECK).ok()?;
    let estrito = std::env::var(ENV_CHECK_STRICT).is_ok_and(|v| v == "1");
    // `--tipos`: escreve o `ctx.d.luau` antes de qualquer outra coisa — o tipo das
    // chaves é o que o editor precisa, e não depende dos achados abaixo.
    if let Some(destino) = std::env::var(ENV_CHECK_TIPOS).ok() {
        let caminho = if destino == "1" || destino.is_empty() {
            Path::new(&raiz).join("scripts").join("ctx.d.luau")
        } else {
            PathBuf::from(destino)
        };
        match tipos(Path::new(&raiz)) {
            Ok(texto) => {
                let anterior = std::fs::read_to_string(&caminho).unwrap_or_default();
                if let Some(pai) = caminho.parent()
                    && let Err(e) = std::fs::create_dir_all(pai)
                {
                    eprintln!("glacier check --tipos: {e}");
                    return Some(2);
                }
                if let Err(e) = std::fs::write(&caminho, &texto) {
                    eprintln!("glacier check --tipos: {e}");
                    return Some(2);
                }
                println!(
                    "{}: {}",
                    caminho.display(),
                    if anterior == texto { "já em dia" } else { "escrito" }
                );
                // Dois `declare ctx` numa mesma árvore de sources é erro do
                // luau-lsp: o preset declara um em `glacier.d.luau`.
                if let Some(irmao) = caminho.parent().map(|d| d.join("glacier.d.luau")) {
                    if std::fs::read_to_string(&irmao)
                        .is_ok_and(|t| t.contains("declare ctx"))
                    {
                        println!(
                            "{}: tem um `declare ctx` — tire-o de lá, senão o editor vê dois",
                            irmao.display()
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("glacier check --tipos: {e}");
                return Some(2);
            }
        }
    }
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

/// [`migrar_gv`] com o conjunto das chaves que a árvore grava como **bool**: só
/// elas têm o `@x == "true"` trocado pela forma tipada (Frente 1). Sem o
/// conjunto, a troca é só avisada pelo `check`.
pub fn migrar_gv_com(src: &str, bools: Option<&std::collections::HashSet<String>>) -> (String, usize) {
    let (mut s, mut n) = migrar_gv(src);

    // 7. Frente 6: ação é sempre chamada. `on_x = nome` → `nome()`; os embutidos
    //    vão à forma de chamada deles (`window:minimize` → `window.minimize()`,
    //    `field:nome` → `field("nome")`).
    let re_nua = Regex::new(r"(on_\w+\s*=\s*)([A-Za-z_][A-Za-z0-9_.]*)(\s*[,)\n])").unwrap();
    s = re_nua
        .replace_all(&s, |c: &regex::Captures| {
            let nome = &c[2];
            let nova = if nome.contains(':') {
                acao_de_chamada(nome)
            } else if ACOES_DO_MOTOR.contains(&nome.split(['.', ':']).next().unwrap_or(nome)) {
                None
            } else {
                Some(format!("{nome}()"))
            };
            match nova {
                Some(v) => {
                    n += 1;
                    format!("{}{}{}", &c[1], v, &c[3])
                }
                None => c[0].to_string(),
            }
        })
        .into_owned();

    // 8. Frente 1: `@x == "true"` é o bool como texto. Só troca quando a chave é
    //    bool conhecida — senão é texto comparando com texto, e tem razão.
    if let Some(bools) = bools {
        for (de, para) in [
            (r#"([@!]?[\w.]+)\s*==\s*"true""#, "um"),
            (r#"([@!]?[\w.]+)\s*!=\s*"true""#, "nao_um"),
            (r#"([@!]?[\w.]+)\s*==\s*"false""#, "nao_um"),
            (r#"([@!]?[\w.]+)\s*!=\s*"false""#, "um"),
        ] {
            let re = Regex::new(de).unwrap();
            s = re
                .replace_all(&s, |c: &regex::Captures| {
                    let a = &c[1];
                    let Some(chave) = a.strip_prefix('@') else { return c[0].to_string() };
                    if !bools.contains(chave) {
                        return c[0].to_string();
                    }
                    n += 1;
                    match para {
                        "um" => a.to_string(),
                        _ => format!("!{a}"),
                    }
                })
                .into_owned();
        }
    }

    // 9. Frente 10: `class:nome = expr` → `:class.nome = expr`.
    let re = Regex::new(r"(?m)([\s(,])class:([\w\-]+)\s*=\s*([^,)\n]+)").unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            n += 1;
            format!("{}:class.{} = {}", &c[1], &c[2], &c[3])
        })
        .into_owned();

    // 10. Frente 10: a lista de classes. `"a b"` sem interpolação é uma lista.
    let re = Regex::new(r#"(?m)([\s(,])class\s*=\s*"([\w\- ]+)""#).unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            let itens: Vec<&str> = c[2].split_whitespace().collect();
            if itens.len() < 2 {
                return c[0].to_string();
            }
            n += 1;
            format!("{}class = [{}]", &c[1], itens.join(", "))
        })
        .into_owned();

    // 11. Frente 10: `{chave|padrão}` → `{@chave ?? "padrão"}`.
    let re = Regex::new(r#"\{([\w.\-]+)\|([^{}"'`]+)\}"#).unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            n += 1;
            format!("{{@{} ?? \"{}\"}}", &c[1], &c[2].trim())
        })
        .into_owned();

    // 12. Frente 2/10: o vínculo de duas vias. `:value = nome` era a ligação pelo
    //     nome; a forma é `::value = @nome`.
    let re = Regex::new(r"(?m)([\s(,]):(value|checked|items|selected|open|page|step)\s*=\s*([A-Za-z_][\w.\-]*)").unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            n += 1;
            format!("{}::{} = @{}", &c[1], &c[2], &c[3])
        })
        .into_owned();

    // 13. Frente 10: par numérico como lista.
    let re = Regex::new(r#"(?m)([\s(,])(size|min_size|max_size|fixed_size)\s*=\s*"(\d+)\s+(\d+)""#)
        .unwrap();
    s = re
        .replace_all(&s, |c: &regex::Captures| {
            n += 1;
            format!("{}{} = [{}, {}]", &c[1], &c[2], &c[3], &c[4])
        })
        .into_owned();

    // 14. Frente 10: atributo com expressão sem `:` ganha o `:` de uma via.
    let (novo, k) = colonar_expressoes(&s);
    s = novo;
    n += k;

    (s, n)
}

/// A forma de chamada de um embutido escrito à moda antiga: `janela:resize:nw` →
/// `janela.resize("nw")`, `field:nome` → `field("nome")`, `tray:open` →
/// `tray.open()`. `None` quando não é um embutido conhecido (fica como está, e o
/// `check` acusa o handler ausente).
fn acao_de_chamada(nome: &str) -> Option<String> {
    let partes: Vec<&str> = nome.split(':').collect();
    if partes.len() < 2 || partes[0].is_empty() {
        return None;
    }
    let metodo_de_args = matches!(
        partes[0],
        "field" | "textarea_end" | "textarea_top" | "clipboard" | "whatsthis" | "audio" | "open"
    );
    let sem_metodo = partes.len() == 2 && !metodo_de_args;
    if !ACOES_DO_MOTOR.contains(&partes[0]) && !crate::gv::ACOES_EMBUTIDAS_COM_PREFIXO.contains(&partes[0])
    {
        return None;
    }
    Some(if sem_metodo {
        format!("{}.{}()", partes[0], partes[1])
    } else {
        let args = [&partes[1..]].concat().join(":");
        format!("{}({})", partes[0], if partes[0] == "field" || partes[0] == "open" || partes[0] == "whatsthis" || partes[0] == "clipboard" || partes[0] == "textarea_end" || partes[0] == "textarea_top" { format!("\"{args}\"") } else { args })
    })
}

/// Os atributos que o `.gv` já trata como booleanos e que, com um `:`, dizem por
/// escrito que são uma via (Frente 10).
const ATRIBUITOS_DE_EXPRESSAO: &[&str] = &[
    "disabled",
    "hidden",
    "busy",
    "expanded",
    "selected",
    "visible",
    "readonly",
    "open",
    "checked",
    "ativo",
];

/// Troca `disabled = @a && @b` por `:disabled = @a && @b` — só quando o valor é
/// mesmo uma expressão composta (nome nu e `@chave` sozinha não mudam de lado).
fn colonar_expressoes(src: &str) -> (String, usize) {
    let mut n = 0;
    let mut out = String::with_capacity(src.len());
    for linha in src.lines() {
        let mut resto = linha.to_string();
        for attr in ATRIBUITOS_DE_EXPRESSAO {
            let marca = format!("{attr} = ");
            let Some(i) = resto.find(&marca) else { continue };
            let antes = &resto[..i];
            if antes.contains('"') && !antes.matches('"').count().is_multiple_of(2) {
                continue; // dentro de um texto
            }
            let ini = i + marca.len();
            let cs: Vec<char> = resto[ini..].chars().collect();
            let fim = crate::handlers::fim_do_valor(&cs);
            let valor: String = cs[..fim].iter().collect();
            let v = valor.trim();
            let composto = (v.starts_with('@') || v.starts_with('!'))
                && (v.contains("&&")
                    || v.contains("||")
                    || v.contains("??")
                    || v.contains(" ? ")
                    || v.contains(" in ")
                    || v.contains(" is ")
                    || v.contains("==")
                    || v.contains("!="));
            if !composto || antes.trim_end().ends_with(':') {
                continue;
            }
            resto.replace_range(i..ini, &format!(":{attr} = "));
            n += 1;
            break;
        }
        out.push_str(&resto);
        out.push('\n');
    }
    (out, n)
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

// ───────────────────────────── o fmt que reindenta ─────────────────────────────

/// Largura-alvo de uma linha de `.gv`/`.gss` (a convenção do `CLAUDE.md`).
pub const LARGURA: usize = 100;

/// Reindentação de um `.gv` que **não toca em caractere nenhum além do espaço de
/// fim de linha**: um tokenizador de texto, com as strings `"""…"""`, `l"""…"""`
/// e os comentários `//` / `/* */` reconhecidos e atravessados intactos. É assim
/// que o `fmt` reindenta sem perder comentário nenhum (Frente 7 do `TODO.md`):
/// reimprimir a partir da árvore perderia, porque o parser os descarta.
///
/// A regra é a profundidade do aninhamento: `(` abre um nível de atributo, `{`
/// abre um nível de bloco, `)`/`}` fecham. Uma linha dentro de um bloco de string
/// sai byte a byte. Idempotente por construção (o recuo depende só da estrutura).
pub fn reindentar_gv(src: &str) -> String {
    let cs: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut prof: i32 = 0;
    let mut i = 0;
    while i < cs.len() {
        // ── uma linha física: `texto` é o cru, `estrutura` só o que aninha ──
        let mut texto = String::new();
        let mut estrutura = String::new();
        while i < cs.len() && cs[i] != '\n' {
            let c = cs[i];
            if c == '"' && cs.get(i + 1) == Some(&'"') && cs.get(i + 2) == Some(&'"') {
                let ini = i;
                let fim = busca_troca(&cs, i + 3).map(|f| f + 3).unwrap_or(cs.len());
                texto.extend(&cs[ini..fim]);
                i = fim;
                continue;
            }
            if c == '"' || c == '\'' {
                let ini = i;
                let q = c;
                let mut j = i + 1;
                while j < cs.len() && cs[j] != q && cs[j] != '\n' {
                    if cs[j] == '\\' {
                        j += 1;
                    }
                    j += 1;
                }
                let fim = (j + 1).min(cs.len());
                texto.extend(&cs[ini..fim]);
                i = fim;
                continue;
            }
            if c == '/' && cs.get(i + 1) == Some(&'/') {
                while i < cs.len() && cs[i] != '\n' {
                    texto.push(cs[i]);
                    i += 1;
                }
                break;
            }
            if c == '/' && cs.get(i + 1) == Some(&'*') {
                let ini = i;
                let fim = busca_bloco_comentario(&cs, i + 2)
                    .map(|f| f + 2)
                    .unwrap_or(cs.len());
                texto.extend(&cs[ini..fim]);
                // A estrutura vê o que havia antes e depois do comentário, e o
                // que ele tinha dentro não conta.
                i = fim;
                continue;
            }
            texto.push(c);
            estrutura.push(c);
            i += 1;
        }
        if i < cs.len() {
            i += 1; // o `\n`
        }
        // ── o recuo: profundidade corrente, menos o que a própria linha fecha ──
        let nu = texto.trim();
        if nu.is_empty() {
            out.push('\n');
            continue;
        }
        let fecha_na_frente = estrutura
            .chars()
            .take_while(|c| c.is_whitespace() || matches!(c, ')' | '}' | ','))
            .filter(|c| matches!(c, ')' | '}'))
            .count() as i32;
        let recuo = (prof - fecha_na_frente).max(0) as usize * 2;
        out.push_str(&" ".repeat(recuo));
        out.push_str(nu);
        out.push('\n');
        let abre = estrutura.chars().filter(|c| matches!(c, '{' | '(')).count() as i32;
        let fecha = estrutura.chars().filter(|c| matches!(c, '}' | ')')).count() as i32;
        prof = (prof + abre - fecha).max(0);
    }
    out
}

fn busca_troca(cs: &[char], mut i: usize) -> Option<usize> {
    while i + 2 < cs.len() {
        if cs[i] == '"' && cs[i + 1] == '"' && cs[i + 2] == '"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn busca_bloco_comentario(cs: &[char], mut i: usize) -> Option<usize> {
    while i + 1 < cs.len() {
        if cs[i] == '*' && cs[i + 1] == '/' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// [`fmt`] de um arquivo `.gv`: reindentação + as trocas de migração.
fn formatar_gv(src: &str, migrar: bool, bools: Option<&std::collections::HashSet<String>>) -> (String, usize) {
    let (s, mut n) = if migrar {
        migrar_gv_com(src, bools)
    } else {
        (src.to_string(), 0)
    };
    let re = reindentar_gv(&s);
    if re != s {
        n += 1;
    }
    (re, n)
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
    // As chaves que a árvore grava como bool: só nelas o `== "true"` vira teste
    // de verdade (Frente 1). O resto é texto comparando com texto.
    let pan = panorama(raiz).ok();
    let bools: std::collections::HashSet<String> = pan
        .as_ref()
        .map(|p| {
            p.chaves
                .tipos
                .iter()
                .filter(|(_, t)| **t == crate::handlers::Tipo::Bool)
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default();
    // Riscos de verdade por arquivo: enquanto houver, a marca `truthy = typed` não
    // é ligada — a migração só fecha quando a lista está vazia ou revista.
    let riscos = pan
        .as_ref()
        .map(|p| verificar_verdade(&gvs.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(), p))
        .unwrap_or_default();
    let tem_risco = |nome: &str| riscos.iter().any(|a| a.arquivo == nome);
    for (p, src) in &gvs {
        let nome = p.display().to_string();
        let (mut novo, mut k) = formatar_gv(src, migrar, Some(&bools));
        if migrar
            && novo.trim_start().starts_with("app(")
            && !novo.contains("truthy")
            && !tem_risco(&nome)
            && k > 0
        {
            novo = novo.replacen("app(", "app(\n  truthy = typed,", 1);
            k += 1;
        }
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

// ─────────────────── testes das Frentes 6, 7, 10 e 11 ───────────────────
//
// Escrita junto com o código e **não executada nesta entrega**: rodar com
// `make test` (ou `cargo test --lib ferramentas`) antes de publicar. Ver o
// registro no `TODO.md`.
#[cfg(test)]
mod testes_frentes {
    use super::*;

    fn m(src: &str) -> (String, usize) {
        migrar_gv_com(src, None)
    }

    #[test]
    fn acao_nua_vira_chamada() {
        let (s, n) = m(r#"button("S", on_click = salvar)
button("E", on_click = exportar(@item.key, true))"#);
        assert!(s.contains("on_click = salvar()"), "{s}");
        assert!(s.contains("on_click = exportar(@item.key, true)"), "{s}");
        assert!(n >= 1, "{n}");
    }

    #[test]
    fn embutidos_vao_a_forma_de_chamada() {
        let (s, _) = m(r#"button("_", on_click = window:minimize)
button("↖", on_click = window:resize:nw)
textinput(on_change = field:nome)
item(on_click = tray:open)"#);
        assert!(s.contains("on_click = window.minimize()"), "{s}");
        assert!(s.contains(r#"on_click = window.resize("nw")"#), "{s}");
        assert!(s.contains(r#"on_change = field("nome")"#), "{s}");
        assert!(s.contains("on_click = tray.open()"), "{s}");
    }

    #[test]
    fn a_vinculacao_de_campo_vira_dois_pontos() {
        let (s, n) = m(r#"textinput(:value = nome, on_change = field:nome)"#);
        assert!(s.contains("::value = @nome"), "{s}");
        assert!(n >= 1, "{n}");
    }

    #[test]
    fn classe_condicional_e_lista() {
        let (s, _) = m(r#"row(class:ativa = @v == @a, class = "btn btn--lg")"#);
        assert!(s.contains(":class.ativa = @v == @a"), "{s}");
        assert!(s.contains("class = [btn, btn--lg]"), "{s}");
    }

    #[test]
    fn padrao_de_barra_vira_coalescencia() {
        let (s, _) = m(r#"text("total: {n|zero}")"#);
        assert!(s.contains(r#"{@n ?? "zero"}"#), "{s}");
    }

    #[test]
    fn expressao_de_uma_via_ganha_o_dois_pontos() {
        let (s, n) = m(r#"button("S", disabled = @a && @b)"#);
        assert!(s.contains(":disabled = @a && @b"), "{s}");
        assert!(n >= 1, "{n}");
        // Uma chave sozinha não é "expressão composta": fica como está.
        let (t, k) = m(r#"button("S", disabled = @ocupado)"#);
        assert!(t.contains("disabled = @ocupado"), "{t}");
        assert_eq!(k, 0, "{t}");
    }

    #[test]
    fn so_troca_bool_conhecido() {
        let src = r#"text("a", if = @ativo == "true")
text("b", if = @resposta == "true")"#;
        // Sem o conjunto de bools da árvore, nada é reescrito (só o `check` avisa).
        let (s, _) = migrar_gv(src);
        assert!(s.contains(r#"@ativo == "true""#), "{s}");
        let bools: std::collections::HashSet<String> = ["ativo".to_string()].into_iter().collect();
        let (t, n) = migrar_gv_com(src, Some(&bools));
        assert!(t.contains("if = @ativo"), "{t}");
        assert!(t.contains(r#"@resposta == "true""#), "{t}");
        assert!(n >= 1, "{n}");
        // O contrário também.
        let (u, _) = migrar_gv_com(r#"text(if = @ativo != "true")"#, Some(&bools));
        assert!(u.contains("if = !@ativo"), "{u}");
    }

    #[test]
    fn migracao_nova_e_idempotente() {
        let src = r#"row(class:ativa = @v, class = "a b") {
  textinput(:value = nome, on_change = field:nome, disabled = @x || @y)
  button("S", on_click = salvar)
}"#;
        let (um, _) = m(src);
        let (dois, k) = m(&um);
        assert_eq!(um, dois, "segunda passada mudou o texto:\n{um}\n---\n{dois}");
        assert_eq!(k, 0);
    }

    #[test]
    fn reindentar_nao_perde_comentario_nem_string() {
        let baguncado = "app(id = x) {\n\tscreen(s) {\n\t\t\ttext(\"oi\")  // ola\n\t\tscript \"\"\"\n  local  a   = 1\n\t\t\"\"\"\n}\n}\n";
        let s = reindentar_gv(baguncado);
        assert!(s.contains("// ola"), "{s}");
        // O corpo do `script` sai byte a byte: `local  a   = 1` com os espaços dele.
        assert!(s.contains("local  a   = 1"), "{s}");
        assert!(s.contains("  screen(s) {"), "{s}");
        assert!(s.contains("    text(\"oi\") // ola"), "{s}");
        // Nem uma vírgula dentro de string muda o aninhamento.
        let t = reindentar_gv("row {\ntext(\"a, b ) ( c\")\ntext(\"fim\")\n}\n");
        assert!(t.contains("  text(\"a, b ) ( c\")"), "{t}");
    }

    #[test]
    fn reindentar_e_idempotente_e_preserva_as_linhas() {
        for arquivo in ["examples/linguagem_ocupado/app.gv", "examples/linguagem_props/app.gv"] {
            let Ok(src) = std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(arquivo),
            ) else {
                continue;
            };
            let uma = reindentar_gv(&src);
            let duas = reindentar_gv(&uma);
            assert_eq!(uma, duas, "{arquivo}: fmt duas vezes != uma");
            assert_eq!(
                src.matches('\n').count(),
                uma.matches('\n').count(),
                "{arquivo}: mudou o número de linhas"
            );
            // Comentário nenhum se perde.
            for (_, l) in src.lines().enumerate() {
                let nu = l.trim();
                if nu.starts_with("//") {
                    assert!(uma.lines().any(|o| o.trim() == nu), "{arquivo}: perdeu `{nu}`");
                }
            }
        }
    }

    #[test]
    fn estado_declarado_e_lido_pelo_check() {
        let src = "component(name = C) {\n  state {\n    aberto = false,\n    total = 0,\n  }\n  text(\"x\")\n}";
        let blocos = crate::handlers::blocos_state(src);
        assert_eq!(blocos.len(), 1, "{blocos:?}");
        assert!(blocos[0].contains("aberto = false"), "{:?}", blocos);
    }

    #[test]
    fn o_ctx_d_luau_de_uma_arvore() {
        use crate::handlers::{Chaves, Tipo};
        let mut chaves = Chaves::default();
        chaves.do_script("ctx.ativo = true\nctx.limite = 4\nctx.servicos = {}\n");
        chaves.do_gv(
            r#"component(name = C) {
  props { prop(name = titulo, type = string) }
  state { aberto = false, toques = 0 }
}"#,
        );
        let pan = Panorama {
            funcoes: Vec::new(),
            chaves,
            chamados: Default::default(),
            textos: Default::default(),
            globais: ["contagem".to_string()].into_iter().collect(),
            componentes: Default::default(),
            nunca_chamados: Vec::new(),
        };
        let t = tipos_do_ctx(&pan);
        assert!(t.starts_with("--!strict"), "{t}");
        assert!(t.contains("declare ctx: {"), "{t}");
        assert!(t.contains("    ativo: boolean,"), "{t}");
        assert!(t.contains("    limite: number,"), "{t}");
        assert!(t.contains("    servicos: table,"), "{t}");
        assert!(t.contains("    titulo: string,"), "{t}");
        assert!(t.contains("    aberto: boolean,"), "{t}");
        assert!(t.contains("    toques: number,"), "{t}");
        // A chave vista no markup, sem tipo nenhum, entra como `any`.
        assert!(t.contains("    contagem: any,"), "{t}");
        // A reserva está lá — tirar é o contrato fechado (e o que quebra o app).
        assert!(t.contains("    [string]: any,"), "{t}");
        // Estável: a mesma árvore dá o mesmo arquivo (CI pode exigir zero diff).
        assert_eq!(t, tipos_do_ctx(&pan));
        assert_eq!(t, tipos_do_ctx(&pan));
    }
}
