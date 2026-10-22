//! Runner de testes da camada Luau — o que está por trás de `glacier test`.
//!
//! Um teste é um arquivo `*.test.luau` (ou `*_test.luau`). Ele roda numa VM com
//! TODOS os globais do motor e as extensões do app (`lua_extension`), mais:
//!
//! - cada função global `test_*` é um teste (descoberta pelo nome, na ordem em
//!   que aparece no arquivo; aparece no relatório como um caminho Rust,
//!   `pasta::arquivo::test_x`, e cada um sai assim que termina); `setup()`, se existir, roda antes de cada um;
//! - `assert_eq / assert_ne / assert_nil / assert_not_nil / assert_true /
//!   assert_false / assert_has / assert_match / assert_fails / assert_complete`,
//!   sempre `(obtido, esperado)`, e `each(casos, fn)` para tabela de casos;
//! - `fail(msg)`;
//! - o comentário `---` logo acima de um `test_*` é a documentação dele (ver
//!   [`doc_above`]): aparece com `-vv` e **sempre** quando o teste falha;
//! - `component(caminho, ctx_inicial?)`: carrega um `.gv`/`.gvb` com `<script>`
//!   como o motor carregaria e devolve um objeto com `:call(acao, valor?)`,
//!   `:get(chave)`, `:set(chave, valor)`, `:data()` e `:toasts()`. O `ctx` inicial e o
//!   `:set` aceitam tabelas (viram JSON, como `ctx.x = {...}` num script). Um handler
//!   que falha **lança** o erro (em produção ele viraria um toast); `:missing()`
//!   avalia o template contra o `ctx` de agora e devolve as chaves que o markup
//!   **lê e ninguém publicou** (ver [`missing_keys`]);
//! - a **árvore avaliada**: `:render()`, `:texts()`, `:buttons()`, `:find(q)`,
//!   `:find_all(q)`, `:set_viewport(w, h)` e `:style(no)` (ver [`no_para_lua`]),
//!   mais `assert_renders(caminho)` e `assert_has_text`/`assert_has_button`
//!   (e os `assert_no_*`). Cada chamada reavalia a tela do zero contra o `ctx`
//!   de agora.
//!
//! `require` funciona como no app (relativo ao arquivo de teste), então módulos
//! Luau puros testam-se só com `require` + `assert_*`.
//!
//! Chamadas que suspendem (`fetch`, `confirm`, diálogos) não são resolvidas aqui:
//! o handler roda até a suspensão e para.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mlua::{Lua, UserData, UserDataMethods, Value};

use super::LuauComponent;
use crate::ContextMap;
use crate::asset_source::DiskAssets;
use crate::component::Context;

const TEST_PRELUDE: &str = include_str!("testing.luau");

/// Variável de ambiente que faz o app rodar os testes em vez de abrir janela
/// (lida por `GlacierDaemon::run`). Vazia = todos os testes; senão, o caminho
/// de um arquivo ou diretório. `GLACIER_TEST_FILTER` filtra pelo nome do teste.
pub const ENV_VAR: &str = "GLACIER_TEST";
/// Filtro opcional por trecho do nome do teste.
pub const ENV_FILTER: &str = "GLACIER_TEST_FILTER";
/// Nível de verbosidade (`-v` = 1, `-vv` = 2, `-vvv` = 3).
pub const ENV_VERBOSE: &str = "GLACIER_TEST_VERBOSE";
/// `1` esconde os logs (`print`, `console.*`) dos testes (`-s`).
pub const ENV_SILENT: &str = "GLACIER_TEST_SILENT";

/// Como o relatório é impresso.
///
/// | nível | mostra |
/// |---|---|
/// | 0 | uma linha por arquivo; as falhas por inteiro |
/// | 1 (`-v`) | o nome de cada teste |
/// | 2 (`-vv`) | + a documentação (`---`) de cada teste |
/// | 3 (`-vvv`) | + o tempo de cada teste e o stacktrace das falhas |
///
/// Os logs que o teste escreveu (`print`, `console.*`) aparecem em todos os
/// níveis, a não ser com `silent` (`-s`). A falha mostra sempre a documentação,
/// a linha e o obtido/esperado.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub verbosity: u8,
    pub silent: bool,
}

impl Options {
    fn from_env() -> Self {
        Self {
            verbosity: std::env::var(ENV_VERBOSE)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            silent: std::env::var(ENV_SILENT).is_ok_and(|v| v == "1"),
        }
    }
}

/// Resultado agregado de uma execução.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub passed: usize,
    pub failed: usize,
    /// Arquivos que nem carregaram (erro de sintaxe, `require` quebrado…).
    pub broken_files: usize,
}

impl Summary {
    pub fn ok(&self) -> bool {
        self.failed == 0 && self.broken_files == 0
    }
}

/// Acha os arquivos de teste: o próprio `target` se for arquivo; senão,
/// recursivamente, tudo que termina em `.test.luau`/`_test.luau`. Sem `target`,
/// procura em `views/` e `tests/`.
pub fn discover(target: Option<&str>) -> Vec<PathBuf> {
    let roots: Vec<PathBuf> = match target {
        Some(t) if !t.is_empty() => vec![PathBuf::from(t)],
        _ => vec![PathBuf::from("views"), PathBuf::from("tests")],
    };
    let mut found = Vec::new();
    for root in roots {
        walk(&root, &mut found);
    }
    found.sort();
    found
}

fn is_test_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with(".test.luau") || n.ends_with("_test.luau"))
}

fn walk(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        // Um arquivo dado explicitamente roda mesmo sem o sufixo de convenção.
        out.push(path.to_path_buf());
    } else if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if is_test_file(&p) {
                out.push(p);
            }
        }
    }
}

/// Roda os testes e imprime o relatório. Devolve o código de saída do processo.
pub fn run_cli(target: Option<&str>, filter: Option<&str>, opts: Options) -> i32 {
    let files = discover(target);
    if files.is_empty() {
        eprintln!("nenhum arquivo *.test.luau encontrado (procurei em views/ e tests/)");
        return 1;
    }
    let mut total = Summary::default();
    for file in &files {
        let s = run_file(file, filter, opts);
        total.passed += s.passed;
        total.failed += s.failed;
        total.broken_files += s.broken_files;
    }
    println!(
        "\n{} passaram, {} falharam{}",
        paint(Green, &total.passed.to_string()),
        paint(if total.failed > 0 { Red } else { Green }, &total.failed.to_string()),
        if total.broken_files > 0 {
            format!(", {} arquivo(s) não carregaram", total.broken_files)
        } else {
            String::new()
        }
    );
    if total.ok() { 0 } else { 1 }
}

/// Entrada para o gancho do `GlacierDaemon::run`: lê as variáveis de ambiente.
pub fn run_from_env() -> Option<i32> {
    let target = std::env::var(ENV_VAR).ok()?;
    let filter = std::env::var(ENV_FILTER).ok().filter(|f| !f.is_empty());
    Some(run_cli(Some(&target), filter.as_deref(), Options::from_env()))
}

/// O que o runner Luau devolve por teste.
#[derive(Debug, Default)]
struct Outcome {
    name: String,
    ok: bool,
    /// Linha da definição do `test_*`.
    line: Option<i64>,
    /// A asserção que falhou (`assert_eq`…), se foi uma.
    kind: Option<String>,
    detail: String,
    /// Linha do `assert_*` que falhou.
    assert_line: Option<i64>,
    trace: String,
    ms: f64,
    logs: Vec<String>,
}

impl Outcome {
    fn from_lua(r: &mlua::Table) -> Self {
        Self {
            name: r.get("name").unwrap_or_default(),
            ok: r.get("ok").unwrap_or(false),
            line: r.get("line").ok(),
            kind: r.get("kind").ok(),
            detail: r.get("detail").unwrap_or_default(),
            assert_line: r.get("assert_line").ok(),
            trace: r.get("trace").unwrap_or_default(),
            ms: r.get("ms").unwrap_or_default(),
            logs: r
                .get::<mlua::Table>("logs")
                .map(|t| t.sequence_values::<String>().flatten().collect())
                .unwrap_or_default(),
        }
    }
}

/// A documentação de um teste: os `---` consecutivos imediatamente acima da
/// linha `line` (1-based) de `source`, sem o marcador. Vazia se não houver.
///
/// O Luau não tem docstring; o comentário `---` é a convenção que o `luau-lsp`
/// já mostra no hover. O runner lê o texto do arquivo, então não custa nada em
/// execução.
pub fn doc_above(source: &str, line: usize) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut doc = Vec::new();
    let mut i = line.saturating_sub(1);
    while i > 0 {
        let t = lines[i - 1].trim_start();
        // `----` é régua de comentário, não documentação.
        match t.strip_prefix("---") {
            Some(rest) if !rest.starts_with('-') => {
                doc.push(rest.strip_prefix(' ').unwrap_or(rest).trim_end().to_string());
            }
            _ => break,
        }
        i -= 1;
    }
    doc.reverse();
    doc
}

/// O primeiro parágrafo de uma documentação (até a primeira linha em branco).
fn first_paragraph(doc: &[String]) -> &[String] {
    let end = doc.iter().position(|l| l.is_empty()).unwrap_or(doc.len());
    &doc[..end]
}

/// Os `test_*` definidos mais de uma vez no arquivo: a segunda definição
/// sobrescreve a primeira em silêncio, e quem lê o texto consegue avisar.
fn duplicate_tests(source: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut dups: Vec<String> = Vec::new();
    for line in source.lines() {
        let Some(rest) = line.trim_start().strip_prefix("function test_") else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if seen.contains(&name) {
            if !dups.contains(&name) {
                dups.push(name);
            }
        } else {
            seen.push(name);
        }
    }
    dups
}

/// Roda um arquivo de teste e imprime o relatório (ver [`Options`]).
pub fn run_file(path: &Path, filter: Option<&str>, opts: Options) -> Summary {
    let mut summary = Summary::default();
    let script = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            println!("{}\n  ✗ não consegui ler: {e}", path.display());
            summary.broken_files = 1;
            return summary;
        }
    };
    let path_str = path.to_string_lossy().into_owned();
    let install = |lua: &Lua| -> mlua::Result<()> {
        lua.load(TEST_PRELUDE).set_name("<glacier test>").exec()?;
        install_component(lua)
    };
    let comp = match LuauComponent::build_with(
        &script,
        path_str.clone(),
        path_str.clone(),
        path,
        Arc::new(DiskAssets),
        Some(&install),
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("{}\n  ✗ não carregou: {e}", path.display());
            summary.broken_files = 1;
            return summary;
        }
    };
    let run: mlua::Function = match comp.luau.globals().get("__glacier_run") {
        Ok(f) => f,
        Err(e) => {
            println!("{}\n  ✗ {e}", path.display());
            summary.broken_files = 1;
            return summary;
        }
    };
    // O cabeçalho sai antes do primeiro teste; cada resultado, assim que termina.
    println!("{}", path.display());
    flush();
    for dup in duplicate_tests(&script) {
        println!("  aviso: test_{dup} é definida mais de uma vez; só a última roda");
    }
    let tally = Rc::new(Cell::new((0usize, 0usize)));
    let emit = {
        let tally = Rc::clone(&tally);
        let script = script.clone();
        let path = path.to_path_buf();
        comp.luau.create_function(move |_, r: mlua::Table| {
            let o = Outcome::from_lua(&r);
            let (p, f) = tally.get();
            tally.set(if o.ok { (p + 1, f) } else { (p, f + 1) });
            print_outcome(&path, &script, &o, opts);
            flush();
            Ok(())
        })
    };
    let emit = match emit {
        Ok(f) => f,
        Err(e) => {
            println!("  {} {e}", paint(Red, "✗"));
            summary.broken_files = 1;
            return summary;
        }
    };
    if let Err(e) = run.call::<()>((filter, module_prefix(path), emit)) {
        println!("  {} {e}", paint(Red, "✗"));
        summary.broken_files = 1;
    }
    (summary.passed, summary.failed) = tally.get();

    if opts.verbosity == 0 {
        println!(
            "  {} {}{}",
            summary.passed,
            paint(Green, "✓"),
            if summary.failed > 0 {
                format!("  {} {}", summary.failed, paint(Red, "✗"))
            } else {
                String::new()
            }
        );
    }
    summary
}

/// O caminho de módulos de um arquivo de teste, no estilo Rust: os diretórios
/// acima e o nome do arquivo sem o sufixo (`tests/format/dashboard.test.luau` →
/// `format::dashboard`). A raiz `tests/`/`views/` não conta.
fn module_prefix(path: &Path) -> String {
    let rel = std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok().map(Path::to_path_buf))
        .unwrap_or_else(|| path.to_path_buf());
    let mut parts: Vec<String> = rel
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    if parts.len() > 1 && matches!(parts[0].as_str(), "tests" | "views") {
        parts.remove(0);
    }
    if let Some(last) = parts.last_mut() {
        for suffix in [".test.luau", "_test.luau", ".luau"] {
            if let Some(stem) = last.strip_suffix(suffix) {
                *last = stem.to_string();
                break;
            }
        }
    }
    parts.join("::")
}

#[derive(Clone, Copy)]
enum Tint {
    Green,
    Red,
    Yellow,
    Gray,
}
use Tint::*;

/// Cores só em terminal (ou com `FORCE_COLOR`); `NO_COLOR` desliga.
fn color_enabled() -> bool {
    use std::io::IsTerminal;
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| {
        if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
            return false;
        }
        std::env::var_os("FORCE_COLOR").is_some_and(|v| !v.is_empty() && v != "0")
            || std::io::stdout().is_terminal()
    })
}

fn paint(tint: Tint, text: &str) -> String {
    if !color_enabled() {
        return text.to_string();
    }
    let code = match tint {
        Green => "32",
        Red => "31",
        Yellow => "33",
        Gray => "38;5;250",
    };
    format!("\x1b[{code}m{text}\x1b[0m")
}

/// Sem isto o relatório fica retido no buffer quando a saída é um pipe.
fn flush() {
    use std::io::Write;
    let _ = std::io::stdout().flush();
}

/// Imprime um teste conforme o nível de verbosidade.
fn print_outcome(path: &Path, script: &str, o: &Outcome, opts: Options) {
    let doc = o
        .line
        .map(|l| doc_above(script, l as usize))
        .unwrap_or_default();
    let logs = !opts.silent && !o.logs.is_empty();
    let timing = if opts.verbosity >= 3 {
        format!("  ({:.1} ms)", o.ms)
    } else {
        String::new()
    };

    if o.ok {
        if opts.verbosity >= 1 {
            println!("  {} {}{timing}", paint(Green, "✓"), paint(Yellow, &o.name));
            if opts.verbosity >= 2 {
                for l in first_paragraph(&doc) {
                    println!("      {}", paint(Gray, l));
                }
            }
        } else if logs {
            println!("  · {}", paint(Yellow, &o.name));
        }
    } else {
        let onde = o
            .line
            .map(|l| format!("   {}:{l}", path.display()))
            .unwrap_or_default();
        println!("  {} {}{onde}{timing}", paint(Red, "✗"), paint(Yellow, &o.name));
        for l in &doc {
            println!("      {}", paint(Gray, l.trim_end()));
        }
        if !doc.is_empty() {
            println!();
        }
        let em = o
            .assert_line
            .map(|l| format!(" (linha {l})"))
            .unwrap_or_default();
        match &o.kind {
            Some(kind) => println!("      {}", paint(Red, &format!("{kind}{em}"))),
            None if !em.is_empty() => println!("      {}", paint(Red, &format!("erro{em}"))),
            None => {}
        }
        for l in o.detail.lines() {
            // `detail` das asserções já vem com 2 espaços de recuo.
            println!("      {}", paint(Red, l));
        }
        if opts.verbosity >= 3 {
            let trace = o.trace.trim();
            if !trace.is_empty() {
                println!("      stacktrace:");
                // Os quadros do próprio runner (`<glacier test>`) não ajudam ninguém.
                for l in trace.lines().filter(|l| !l.contains("<glacier test>")) {
                    println!("        {}", l.trim());
                }
            }
        }
    }
    if logs {
        for entry in &o.logs {
            for l in entry.lines() {
                println!("      │ {l}");
            }
        }
    }
}

/// Um componente carregado por `component(...)`, com o contexto que ele muta.
struct TestComponent {
    comp: LuauComponent,
    /// O arquivo de onde veio, para o `:missing()` reavaliar o template.
    path: String,
    data: RefCell<ContextMap>,
    toasts: RefCell<Vec<(String, String)>>,
    /// O viewport de `:set_viewport(w, h)`; `None` = o do motor (o padrão).
    viewport: Cell<Option<(f32, f32)>>,
}

impl TestComponent {
    /// Carrega, avalia contra o contexto de agora e entrega a árvore a `f`.
    fn arvore<R>(
        &self,
        f: impl FnOnce(&crate::parser::UiNode) -> mlua::Result<R>,
    ) -> mlua::Result<R> {
        com_arvore(&self.path, &self.data.borrow(), self.viewport.get(), f)
    }
}

impl UserData for TestComponent {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("call", |_, this, (action, value): (String, Option<String>)| {
            let mut data = this.data.borrow_mut();
            let mut ctx = Context::new(&mut data);
            let ran = this
                .comp
                .run_inner(&action, value.as_deref(), &mut ctx)
                .map_err(|e| mlua::Error::runtime(format!("{action}: {e}")))?;
            for t in std::mem::take(&mut ctx.toasts) {
                this.toasts
                    .borrow_mut()
                    .push((format!("{:?}", t.kind).to_lowercase(), t.message));
            }
            if !ran {
                return Err(mlua::Error::runtime(format!(
                    "o componente não tem a função '{action}'"
                )));
            }
            Ok(())
        });
        methods.add_method("missing", |lua, this, ()| {
            let data = this.data.borrow();
            let keys = missing_keys(&this.path, &data)
                .map_err(|e| mlua::Error::runtime(format!("missing: {e}")))?;
            lua.create_sequence_from(keys)
        });
        // ── a árvore avaliada (ver `no_para_lua`) ───────────────────────────
        methods.add_method("render", |lua, this, ()| {
            this.arvore(|raiz| no_para_lua(lua, raiz))
        });
        methods.add_method("texts", |lua, this, ()| {
            let mut out = Vec::new();
            this.arvore(|raiz| Ok(coletar_textos(raiz, false, &mut out)))?;
            lua.create_sequence_from(out)
        });
        methods.add_method("buttons", |lua, this, ()| {
            let mut out = Vec::new();
            this.arvore(|raiz| Ok(coletar_textos(raiz, true, &mut out)))?;
            lua.create_sequence_from(out)
        });
        methods.add_method("find", |lua, this, criterio: mlua::Table| {
            let c = Criterio::de_lua(&criterio)?;
            this.arvore(|raiz| {
                let mut achados = Vec::new();
                achar(raiz, &c, true, &mut achados);
                achados
                    .first()
                    .map(|n| no_para_lua(lua, n))
                    .transpose()
            })
        });
        methods.add_method("find_all", |lua, this, criterio: mlua::Table| {
            let c = Criterio::de_lua(&criterio)?;
            this.arvore(|raiz| {
                let mut achados = Vec::new();
                achar(raiz, &c, false, &mut achados);
                let lista = lua.create_table()?;
                for n in achados {
                    lista.push(no_para_lua(lua, n)?)?;
                }
                Ok(lista)
            })
        });
        methods.add_method("set_viewport", |_, this, (w, h): (f32, f32)| {
            this.viewport.set(Some((w, h)));
            Ok(())
        });
        // `:style(no)` — as propriedades resolvidas de um nó devolvido por
        // `find`/`render`; também aceita o `id` do nó, como atalho.
        methods.add_method("style", |lua, this, alvo: Value| match alvo {
            Value::Table(t) => t.get::<Value>("style"),
            Value::String(id) => {
                let id = id.to_str()?.to_string();
                let c = Criterio {
                    id: Some(id.clone()),
                    ..Default::default()
                };
                this.arvore(|raiz| {
                    let mut achados = Vec::new();
                    achar(raiz, &c, true, &mut achados);
                    match achados.first() {
                        Some(n) => no_para_lua(lua, n)?.get::<Value>("style"),
                        None => Err(mlua::Error::runtime(format!(
                            "style: nenhum nó com id '{id}'"
                        ))),
                    }
                })
            }
            _ => Err(mlua::Error::runtime(
                "style: esperava um nó (de find/render) ou um id",
            )),
        });
        methods.add_method("get", |_, this, key: String| {
            Ok(this.data.borrow().get(&key).cloned())
        });
        // `:set(chave, valor)` grava como o `ctx.chave = valor` de um script:
        // texto, número e booleano viram texto, e uma **tabela** vira JSON (uma
        // lista de objetos é uma lista de objetos, sem escrever o JSON à mão).
        // `nil` remove a chave. Para uma lista vazia, `json.array({})` — `{}`
        // sozinho é um objeto.
        methods.add_method("set", |lua, this, (key, value): (String, Value)| {
            if matches!(value, Value::Nil) {
                this.data.borrow_mut().remove(&key);
                return Ok(());
            }
            let texto = ctx_text(lua, &key, &value)?;
            this.data.borrow_mut().insert(key, texto);
            Ok(())
        });
        methods.add_method("data", |lua, this, ()| {
            let t = lua.create_table()?;
            for (k, v) in this.data.borrow().iter() {
                t.set(k.as_str(), v.as_str())?;
            }
            Ok(t)
        });
        methods.add_method("toasts", |lua, this, ()| {
            let list = lua.create_table()?;
            for (kind, message) in this.toasts.borrow().iter() {
                let t = lua.create_table()?;
                t.set("kind", kind.as_str())?;
                t.set("message", message.as_str())?;
                list.push(t)?;
            }
            Ok(list)
        });
    }
}

/// Uma tela carregada como o motor a carrega, pronta para ser avaliada contra
/// um contexto: o que `:missing()`, `:render()` e `assert_renders` compartilham.
struct Carregada {
    ui: crate::GlacierUI,
    name: String,
    is_app: bool,
    content: String,
}

/// Carrega `path` (um `app(...)` com seus `import`s, ou uma tela solta), troca o
/// contexto por `data` e esvazia tudo o que foi avaliado com o contexto do
/// registro. Com `viewport`, as `@media` são resolvidas para aquela largura.
///
/// Falha com o erro do parser/avaliador (arquivo e linha) se qualquer view
/// carregada estiver quebrada — inclusive uma que nenhum teste chama.
fn carregar(
    path: &str,
    data: &ContextMap,
    viewport: Option<(f32, f32)>,
) -> crate::Result<Carregada> {
    use crate::{GlacierError, GlacierUI, parser};

    let content = crate::asset_source::read_markup(&DiskAssets, path)
        .map_err(|e| GlacierError::io("template", path, e))?;
    let is_app = parser::UiNode::parse_app_xml(&content, Some(path))?.is_some();

    let mut ui = GlacierUI::new();
    let name = if is_app {
        ui.register_app(path)?;
        ui.current_screen_name()
            .map(str::to_string)
            .ok_or(GlacierError::NoActiveScreen)?
    } else {
        ui.register_component("tela", path)?;
        "tela".to_string()
    };

    if let Some(v) = viewport {
        ui.inputs.set_viewport(v);
    }
    // O contexto do teste substitui o que o registro semeou, e o que foi
    // avaliado com o outro sai do cache.
    ui.context_data = data.clone();
    ui.evaluated_templates.clear();
    ui.eval_deps.clear();
    ui.tree_bindings.clear();
    ui.eval_cache = Default::default();
    Ok(Carregada {
        ui,
        name,
        is_app,
        content: content.to_string(),
    })
}

/// Carrega, avalia e entrega a árvore avaliada a `f`.
fn com_arvore<R>(
    path: &str,
    data: &ContextMap,
    viewport: Option<(f32, f32)>,
    f: impl FnOnce(&crate::parser::UiNode) -> mlua::Result<R>,
) -> mlua::Result<R> {
    let rt = |e: crate::GlacierError| mlua::Error::runtime(e.to_string());
    let mut c = carregar(path, data, viewport).map_err(rt)?;
    crate::eval::manter_identidade(|| c.ui.evaluate_into_cache(&c.name)).map_err(rt)?;
    let raiz = c
        .ui
        .evaluated_templates
        .get(&c.name)
        .ok_or_else(|| mlua::Error::runtime("a tela não produziu árvore"))?;
    f(raiz)
}

// ── a árvore avaliada, em tabelas Luau ───────────────────────────────────────

/// O texto "principal" de um nó: o conteúdo de um `text`, o rótulo de um
/// `button`/`checkbox`/`toggle`. `None` nos que não têm um.
fn texto_do_no(n: &crate::parser::UiNode) -> Option<&str> {
    use crate::parser::NodeType as T;
    match &n.kind {
        T::Text { content, .. } => Some(content),
        T::Button { text, .. } => Some(text),
        T::Checkbox { label, .. } | T::Toggle { label, .. } => Some(label),
        _ => None,
    }
}

fn nome_do_no(n: &crate::parser::UiNode) -> &'static str {
    n.kind.tag_name().unwrap_or("node")
}

fn escondido(n: &crate::parser::UiNode) -> bool {
    n.hidden == Some(true)
}

fn set_some<V: mlua::IntoLua>(t: &mlua::Table, k: &str, v: Option<V>) -> mlua::Result<()> {
    match v {
        Some(v) => t.set(k, v),
        None => Ok(()),
    }
}

/// `{ kind, text, class, id, hidden, attrs, style, children }`.
///
/// - `attrs`: o que o autor escreveu e o widget usa (`on_click`, `value_var`,
///   `checked_var`, `placeholder`, `source`, `disabled`…), com `{chave}` já
///   resolvido contra o contexto;
/// - `style`: as propriedades de caixa **depois** de seletor, combinador e
///   `@media` (`width`, `padding`, `background`, `color`, `size`…).
///
/// Só aparece o que o nó tem: um campo ausente é `nil`, não `""`.
fn no_para_lua(lua: &Lua, n: &crate::parser::UiNode) -> mlua::Result<mlua::Table> {
    use crate::parser::NodeType as T;
    let t = lua.create_table()?;
    t.set("kind", nome_do_no(n))?;
    set_some(&t, "text", texto_do_no(n))?;
    set_some(&t, "class", n.class.as_deref())?;
    set_some(&t, "id", n.id.as_deref())?;
    t.set("hidden", escondido(n))?;

    let attrs = lua.create_table()?;
    set_some(&attrs, "disabled", n.disabled)?;
    let style = lua.create_table()?;
    set_some(&style, "width", n.width.as_deref())?;
    set_some(&style, "height", n.height.as_deref())?;
    set_some(&style, "padding", n.padding.as_deref())?;
    set_some(&style, "spacing", n.spacing)?;
    set_some(&style, "background", n.background.as_deref())?;
    set_some(&style, "border_radius", n.border_radius)?;
    set_some(&style, "border_width", n.border_width)?;
    set_some(&style, "max_width", n.max_width)?;
    set_some(&style, "max_height", n.max_height)?;
    match &n.kind {
        T::Text {
            size, bold, color, ..
        } => {
            set_some(&style, "size", *size)?;
            if *bold {
                style.set("bold", true)?;
            }
            set_some(&style, "color", color.as_deref())?;
        }
        T::Button {
            on_click,
            navigate_to,
            color,
            ..
        } => {
            set_some(&attrs, "on_click", on_click.as_deref())?;
            set_some(&attrs, "navigate_to", navigate_to.as_deref())?;
            set_some(&style, "color", color.as_deref())?;
        }
        T::TextInput {
            placeholder,
            value_var,
            on_change,
            secure,
            ..
        } => {
            attrs.set("placeholder", placeholder.as_str())?;
            attrs.set("value_var", value_var.as_str())?;
            attrs.set("on_change", on_change.as_str())?;
            attrs.set("secure", *secure)?;
        }
        T::TextArea {
            placeholder,
            value_var,
            on_change,
            readonly,
            ..
        } => {
            attrs.set("placeholder", placeholder.as_str())?;
            attrs.set("value_var", value_var.as_str())?;
            attrs.set("on_change", on_change.as_str())?;
            attrs.set("readonly", *readonly)?;
        }
        T::Checkbox {
            checked_var,
            on_toggle,
            ..
        }
        | T::Toggle {
            checked_var,
            on_toggle,
            ..
        } => {
            attrs.set("checked_var", checked_var.as_str())?;
            attrs.set("on_toggle", on_toggle.as_str())?;
        }
        T::Image { source, .. } | T::Svg { source, .. } => {
            attrs.set("source", source.as_str())?;
        }
        T::ProgressBar { value_var, .. } => {
            attrs.set("value_var", value_var.as_str())?;
        }
        _ => {}
    }
    t.set("attrs", attrs)?;
    t.set("style", style)?;

    let filhos = lua.create_table()?;
    for f in n.children.iter() {
        filhos.push(no_para_lua(lua, f)?)?;
    }
    t.set("children", filhos)?;
    Ok(t)
}

/// Os textos (ou rótulos de botão) **visíveis**, em ordem. Um nó `hidden` leva a
/// subárvore inteira junto, como no desenho.
fn coletar_textos(
    n: &crate::parser::UiNode,
    so_botoes: bool,
    out: &mut Vec<String>,
) {
    use crate::parser::NodeType as T;
    if escondido(n) {
        return;
    }
    let casa = if so_botoes {
        matches!(n.kind, T::Button { .. })
    } else {
        matches!(n.kind, T::Text { .. })
    };
    if casa && let Some(t) = texto_do_no(n) {
        out.push(t.to_string());
    }
    for f in n.children.iter() {
        coletar_textos(f, so_botoes, out);
    }
}

/// O que `find`/`find_all` aceitam: todo campo dado precisa casar.
#[derive(Default)]
struct Criterio {
    kind: Option<String>,
    text: Option<String>,
    class: Option<String>,
    id: Option<String>,
}

impl Criterio {
    fn de_lua(t: &mlua::Table) -> mlua::Result<Self> {
        Ok(Self {
            kind: t.get("kind")?,
            text: t.get("text")?,
            class: t.get("class")?,
            id: t.get("id")?,
        })
    }

    fn casa(&self, n: &crate::parser::UiNode) -> bool {
        self.kind.as_deref().is_none_or(|k| k.eq_ignore_ascii_case(nome_do_no(n)))
            && self.text.as_deref().is_none_or(|x| texto_do_no(n) == Some(x))
            && self.id.as_deref().is_none_or(|x| n.id.as_deref() == Some(x))
            && self.class.as_deref().is_none_or(|c| {
                n.class
                    .as_deref()
                    .is_some_and(|cs| cs.split_whitespace().any(|x| x == c))
            })
    }
}

fn achar<'a>(
    n: &'a crate::parser::UiNode,
    c: &Criterio,
    primeiro: bool,
    out: &mut Vec<&'a crate::parser::UiNode>,
) {
    if primeiro && !out.is_empty() {
        return;
    }
    if c.casa(n) {
        out.push(n);
    }
    for f in n.children.iter() {
        achar(f, c, primeiro, out);
    }
}

/// As chaves que o **markup** de `path` lê do contexto e que `data` não tem.
///
/// É o cruzamento que um teste de handler não faz: o handler escreve a chave A,
/// o `.gvb` lê a chave B, e a tela aparece VAZIA sem erro nenhum. Aqui o
/// template é carregado como o motor o carrega (um `app(...)` ou uma tela solta,
/// com seus `import`s), avaliado contra `data`, e o rastreamento de leituras do
/// próprio avaliador — o mesmo que decide o cache — diz o que foi consultado.
///
/// Cobre o que o avaliador lê (`{chave}`, `@chave`, `if=`, `for-each=` e afins)
/// e as **ligações** do markup (`:value`, `:items`, `:columns`…), que o widget
/// lê só ao desenhar e o avaliador nunca vê. Fica de fora o que o motor publica sozinho em tempo de
/// execução (`__*`) e o que é variável local de um `for-each`.
///
/// Não contam: a chave lida com valor padrão (`{k|padrão}`), a lida dentro do
/// template de um builtin (as props de estilo de um `<card>`, opcionais por
/// construção) e o `erro_<campo>` de um `<form>` com
/// `rules`, que o motor só publica ao enviar. Se ainda sobrar uma chave que é
/// legitimamente tardia (um `if="{status}"` que um handler publica depois), o
/// teste a **nomeia**, com o porquê; o resto da lista é a falha.
fn missing_keys(path: &str, data: &ContextMap) -> crate::Result<Vec<String>> {
    use crate::{GlacierError, parser};

    let Carregada {
        mut ui,
        name,
        is_app,
        content,
    } = carregar(path, data, None)?;
    let builtins = ui.builtin_component_names.clone();
    let (avaliado, ignoradas) =
        crate::eval::rastrear_ausentes(builtins, || ui.evaluate_into_cache(&name));
    avaliado?;

    let mut missing: Vec<String> = ui
        .eval_deps
        .get(&name)
        .into_iter()
        .flatten()
        .filter(|(key, value)| value.is_none() && !ignoradas.contains(key))
        .map(|(key, _)| key.clone())
        .collect();
    // As ligações (`:value`, `:items`, `:columns`…): o widget lê a chave só ao
    // desenhar, então o avaliador acima não as viu.
    let tela = is_app.then_some(name.as_str());
    let ligadas = parser::binding_keys(&content, tela)
        .map_err(|e| GlacierError::io("template", path, std::io::Error::other(e)))?;
    missing.extend(ligadas.into_iter().filter(|key| !data.contains_key(key)));
    // O `<form>` publica `erro_<campo>` sozinho, ao enviar: antes disso a chave
    // não existe, e não é esquecimento de ninguém.
    let erros_do_form: Vec<String> = ui
        .tree_bindings
        .get(&name)
        .into_iter()
        .flat_map(|b| b.form_fields.iter().map(|f| f.error_key.clone()))
        .collect();
    missing.retain(|key| !key.starts_with("__") && !erros_do_form.contains(key));
    missing.sort();
    missing.dedup();
    Ok(missing)
}

/// O texto que o contexto guarda para `valor` — a mesma conversão de um
/// `ctx.chave = valor` num script. Falha (em vez de descartar em silêncio) com um
/// tipo que não cabe no contexto, como uma função.
fn ctx_text(lua: &Lua, chave: &str, valor: &Value) -> mlua::Result<String> {
    super::luau_value_to_string(lua, valor).ok_or_else(|| {
        mlua::Error::runtime(format!(
            "ctx '{chave}': um valor do tipo '{}' não cabe no contexto \
             (texto, número, booleano ou tabela de dados)",
            valor.type_name()
        ))
    })
}

fn install_component(lua: &Lua) -> mlua::Result<()> {
    let component = lua.create_function(|lua, (path, initial): (String, Option<mlua::Table>)| {
        let comp = LuauComponent::from_file(path.clone(), path.clone())
            .map_err(|e| mlua::Error::runtime(e.to_string()))?;
        let mut data = ContextMap::default();
        if let Some(initial) = initial {
            for pair in initial.pairs::<String, Value>() {
                let (k, v) = pair?;
                if matches!(v, Value::Nil) {
                    continue;
                }
                let s = ctx_text(lua, &k, &v)?;
                data.insert(k, s);
            }
        }
        Ok(TestComponent {
            comp,
            path,
            data: RefCell::new(data),
            toasts: RefCell::new(Vec::new()),
            viewport: Cell::new(None),
        })
    })?;
    lua.globals().set("component", component)?;

    // `assert_renders(path)` (em `testing.luau`) chama isto: carrega o app/tela
    // com TODOS os `import`s e avalia a tela inicial. Devolve a mensagem do
    // parser (arquivo e linha) em vez de lançar, para a asserção formatá-la.
    let render_check = lua.create_function(|_, path: String| {
        Ok(
            match carregar(&path, &ContextMap::default(), None)
                .and_then(|mut c| c.ui.evaluate_into_cache(&c.name))
            {
                Ok(()) => None,
                Err(e) => Some(e.to_string()),
            },
        )
    })?;
    lua.globals().set("__glacier_render_check", render_check)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixo_de_modulo_estilo_rust() {
        let f = |p: &str| module_prefix(Path::new(p));
        assert_eq!(f("tests/format/dashboard.test.luau"), "format::dashboard");
        assert_eq!(f("views/a_test.luau"), "a");
        assert_eq!(f("x.test.luau"), "x");
    }

    #[test]
    fn roda_um_arquivo_com_acertos_e_falhas() {
        let dir = std::env::temp_dir().join(format!("glacier_t_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.test.luau");
        std::fs::write(
            &file,
            "function test_ok() assert_eq(1 + 1, 2) end\n\
             function test_quebra() assert_eq({a = 1}, {a = 2}) end\n\
             function test_nega() assert_ne(1, 2) end\n\
             function test_lanca() assert_fails(function() error('boom') end, 'boom') end",
        )
        .unwrap();
        let s = run_file(&file, None, Options::default());
        assert_eq!((s.passed, s.failed, s.broken_files), (3, 1, 0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn component_chama_handler_e_le_o_contexto() {
        let dir = std::env::temp_dir().join(format!("glacier_tc_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tela = dir.join("tela.gv");
        std::fs::write(
            &tela,
            "<screen><script>function inc() ctx.n = ctx.n + 1 end</script></screen>",
        )
        .unwrap();
        let file = dir.join("t.test.luau");
        std::fs::write(
            &file,
            format!(
                "function test_inc()\n\
                   local c = component({:?}, {{ n = 1 }})\n\
                   c:call('inc')\n\
                   assert_eq(c:get('n'), '2')\n\
                   assert_fails(function() c:call('nao_existe') end, 'nao_existe')\n\
                 end",
                tela.to_str().unwrap()
            ),
        )
        .unwrap();
        let s = run_file(&file, None, Options::default());
        assert_eq!((s.passed, s.failed, s.broken_files), (1, 0, 0), "{s:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// O handler publica `certo`, o markup lê `certo`, `errado` (interpolado) e
    /// `ligada` (um `:value`): só as duas últimas são esquecimento. É a tela
    /// vazia que um teste de handler não pega.
    #[test]
    fn missing_acusa_a_chave_que_o_markup_le_e_ninguem_publicou() {
        let dir = std::env::temp_dir().join(format!("glacier_tm_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tela = dir.join("tela.gv");
        std::fs::write(
            &tela,
            "<screen><script>function init() ctx.certo = 'x' end</script>\
             <Column><Text content=\"{certo} {errado} {opcional|ok}\"/>\
             <ProgressBar :value=\"ligada\"/></Column></screen>",
        )
        .unwrap();
        let file = dir.join("t.test.luau");
        std::fs::write(
            &file,
            format!(
                "function test_missing()\n\
                   local c = component({:?})\n\
                   c:call('init')\n\
                   assert_eq(c:missing(), {{ 'errado', 'ligada' }})\n\
                 end",
                tela.to_str().unwrap()
            ),
        )
        .unwrap();
        let s = run_file(&file, None, Options::default());
        assert_eq!((s.passed, s.failed, s.broken_files), (1, 0, 0), "{s:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A árvore avaliada: texto, botão, `if`, achar por classe/id e o estilo
    /// resolvido numa largura; e `assert_renders` acusando uma view quebrada.
    #[test]
    fn arvore_avaliada_texts_find_style_e_assert_renders() {
        let dir = std::env::temp_dir().join(format!("glacier_ta_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tela = dir.join("tela.gv");
        std::fs::write(
            &tela,
            "<screen><style>.b { width: 240; } @media (max-width: 500) { .b { width: 100; } }</style>\
             <column id=\"raiz\"><text>Oi</text>\
             <button class=\"b\" on_click=\"salvar\" text=\"Salvar\"/></column></screen>",
        )
        .unwrap();
        let quebrada = dir.join("quebrada.gv");
        std::fs::write(&quebrada, "<screen><column>").unwrap();
        let file = dir.join("t.test.luau");
        std::fs::write(
            &file,
            format!(
                "function test_arvore()\n\
                   local t = component({tela:?})\n\
                   assert_eq(t:texts(), {{ 'Oi' }})\n\
                   assert_has_button(t, 'Salvar')\n\
                   assert_no_text(t, 'Erro')\n\
                   assert_eq(t:find({{ id = 'raiz' }}).kind, 'column')\n\
                   assert_eq(t:find({{ class = 'b' }}).attrs.on_click, 'salvar')\n\
                   assert_nil(t:find({{ text = 'nada' }}))\n\
                   assert_eq(t:style(t:find({{ class = 'b' }})).width, '240')\n\
                   t:set_viewport(400, 800)\n\
                   assert_eq(t:style(t:find({{ class = 'b' }})).width, '100')\n\
                 end\n\
                 function test_renders() assert_renders({tela:?}) end\n\
                 function test_quebrada()\n\
                   assert_fails(function() assert_renders({quebrada:?}) end, 'quebrada.gv')\n\
                 end",
                tela = tela.to_str().unwrap(),
                quebrada = quebrada.to_str().unwrap()
            ),
        )
        .unwrap();
        let s = run_file(&file, None, Options::default());
        assert_eq!((s.passed, s.failed, s.broken_files), (3, 0, 0), "{s:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn descobre_test_por_nome_e_roda_asserts_setup_e_each() {
        let dir = std::env::temp_dir().join(format!("glacier_tn_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("n.test.luau");
        std::fs::write(
            &file,
            "local n = 0\n\
             function setup() n = n + 1 end\n\
             --- Soma.\n\
             function test_soma() assert_eq(1 + 1, 2); assert_eq(n, 1) end\n\
             function test_quebra() assert_eq({ a = 1 }, { a = 2 }) end\n\
             function test_tabela()\n\
               each({ { 1, 2 }, { 2, 3 } }, function(a, b) assert_eq(a + 1, b) end)\n\
             end\n\
             function test_tabela_quebra()\n\
               each({ { 1, 2 }, { 2, 9 } }, function(a, b) assert_eq(a + 1, b) end)\n\
             end\n\
             function test_lanca() assert_fails(function() error('boom') end, 'boom') end\n\
             function test_nulos() assert_nil(nil); assert_not_nil(1); assert_ne(1, 2) end",
        )
        .unwrap();
        let s = run_file(&file, None, Options::default());
        assert_eq!((s.passed, s.failed, s.broken_files), (4, 2, 0), "{s:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn doc_above_junta_os_tres_tracos_acima_da_funcao() {
        // `----` é régua: corta a documentação, não entra nela.
        let src = "-- nada\n---- régua\n--- Primeira.\n---\n--- Segunda.\nfunction test_x() end";
        assert_eq!(doc_above(src, 6), vec!["Primeira.", "", "Segunda."]);
        let src = "--- Perdida.\n---- régua\nfunction test_x() end";
        assert!(doc_above(src, 3).is_empty());
        let src = "--- Primeira.\n---\n--- Segunda.\nfunction test_x() end";
        assert_eq!(doc_above(src, 4), vec!["Primeira.", "", "Segunda."]);
        assert_eq!(first_paragraph(&doc_above(src, 4)), ["Primeira."]);
        assert!(doc_above("function test_x() end", 1).is_empty());
    }

    #[test]
    fn acusa_test_definido_duas_vezes() {
        let src = "function test_a() end\nfunction test_b() end\nfunction test_a() end";
        assert_eq!(duplicate_tests(src), vec!["a".to_string()]);
    }
}
