//! Runner de testes da camada Luau — o que está por trás de `glacier test`.
//!
//! Um teste é um arquivo `*.test.luau` (ou `*_test.luau`). Ele roda numa VM com
//! TODOS os globais do motor e as extensões do app (`lua_extension`), mais:
//!
//! - cada função global `test_*` é um teste (descoberta pelo nome, na ordem em
//!   que aparece no arquivo); `setup()`, se existir, roda antes de cada um;
//! - `assert_eq / assert_ne / assert_nil / assert_not_nil / assert_true /
//!   assert_false / assert_has / assert_match / assert_fails / assert_complete`,
//!   sempre `(obtido, esperado)`, e `each(casos, fn)` para tabela de casos;
//! - `fail(msg)`;
//! - o comentário `---` logo acima de um `test_*` é a documentação dele (ver
//!   [`doc_above`]): aparece com `-vv` e **sempre** quando o teste falha;
//! - `component(caminho, ctx_inicial?)`: carrega um `.gv`/`.gvb` com `<script>`
//!   como o motor carregaria e devolve um objeto com `:call(acao, valor?)`,
//!   `:get(chave)`, `:set(chave, valor)`, `:data()` e `:toasts()`. Um handler
//!   que falha **lança** o erro (em produção ele viraria um toast); `:missing()`
//!   avalia o template contra o `ctx` de agora e devolve as chaves que o markup
//!   **lê e ninguém publicou** (ver [`missing_keys`]).
//!
//! `require` funciona como no app (relativo ao arquivo de teste), então módulos
//! Luau puros testam-se só com `require` + `assert_*`.
//!
//! Chamadas que suspendem (`fetch`, `confirm`, diálogos) não são resolvidas aqui:
//! o handler roda até a suspensão e para.

use std::cell::RefCell;
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
        total.passed,
        total.failed,
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
    let results: mlua::Table = match run.call(filter) {
        Ok(t) => t,
        Err(e) => {
            println!("{}\n  ✗ {e}", path.display());
            summary.broken_files = 1;
            return summary;
        }
    };
    let outcomes: Vec<Outcome> = results
        .sequence_values::<mlua::Table>()
        .flatten()
        .map(|r| Outcome::from_lua(&r))
        .collect();
    summary.passed = outcomes.iter().filter(|o| o.ok).count();
    summary.failed = outcomes.len() - summary.passed;

    if opts.verbosity == 0 {
        println!(
            "{}  {} ✓{}",
            path.display(),
            summary.passed,
            if summary.failed > 0 {
                format!("  {} ✗", summary.failed)
            } else {
                String::new()
            }
        );
    } else {
        println!("{}", path.display());
    }
    for dup in duplicate_tests(&script) {
        println!("  aviso: test_{dup} é definida mais de uma vez; só a última roda");
    }
    for o in &outcomes {
        print_outcome(path, &script, o, opts);
    }
    summary
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
            println!("  ✓ {}{timing}", o.name);
            if opts.verbosity >= 2 {
                for l in first_paragraph(&doc) {
                    println!("      {l}");
                }
            }
        } else if logs {
            println!("  · {}", o.name);
        }
    } else {
        let onde = o
            .line
            .map(|l| format!("   {}:{l}", path.display()))
            .unwrap_or_default();
        println!("  ✗ {}{onde}{timing}", o.name);
        for l in &doc {
            println!("{}", format!("      {l}").trim_end());
        }
        if !doc.is_empty() {
            println!();
        }
        let em = o
            .assert_line
            .map(|l| format!(" (linha {l})"))
            .unwrap_or_default();
        match &o.kind {
            Some(kind) => println!("      {kind}{em}"),
            None if !em.is_empty() => println!("      erro{em}"),
            None => {}
        }
        for l in o.detail.lines() {
            // `detail` das asserções já vem com 2 espaços de recuo.
            println!("      {l}");
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
        methods.add_method("get", |_, this, key: String| {
            Ok(this.data.borrow().get(&key).cloned())
        });
        methods.add_method("set", |_, this, (key, value): (String, String)| {
            this.data.borrow_mut().insert(key, value);
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

    // O contexto do teste substitui o que o registro semeou, e o que foi
    // avaliado com o outro sai do cache.
    ui.context_data = data.clone();
    ui.evaluated_templates.clear();
    ui.eval_deps.clear();
    ui.tree_bindings.clear();
    ui.eval_cache = Default::default();
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

fn install_component(lua: &Lua) -> mlua::Result<()> {
    let component = lua.create_function(|_, (path, initial): (String, Option<mlua::Table>)| {
        let comp = LuauComponent::from_file(path.clone(), path.clone())
            .map_err(|e| mlua::Error::runtime(e.to_string()))?;
        let mut data = ContextMap::default();
        if let Some(initial) = initial {
            for pair in initial.pairs::<String, Value>() {
                let (k, v) = pair?;
                let s = match v {
                    Value::String(s) => s.to_string_lossy().to_string(),
                    Value::Integer(i) => i.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    _ => continue,
                };
                data.insert(k, s);
            }
        }
        Ok(TestComponent {
            comp,
            path,
            data: RefCell::new(data),
            toasts: RefCell::new(Vec::new()),
        })
    })?;
    lua.globals().set("component", component)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let src = "-- nada\n--- Primeira.\n---\n--- Segunda.\n---- régua\nfunction test_x() end";
        assert_eq!(doc_above(src, 6), vec!["", "Segunda."]);
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
