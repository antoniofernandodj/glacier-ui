//! Runner de testes da camada Luau — o que está por trás de `glacier test`.
//!
//! Um teste é um arquivo `*.test.luau` (ou `*_test.luau`). Ele roda numa VM com
//! TODOS os globais do motor e as extensões do app (`lua_extension`), mais:
//!
//! - `describe(nome, fn)`, `test(nome, fn)` (`it`), `before_each(fn)`;
//! - `expect(v):toBe / toEqual / toBeNil / toBeTruthy / toBeFalsy / toContain /
//!   toMatch / toThrow`, e `.never` para negar (`expect(v).never:toBe(1)`);
//! - `fail(msg)`;
//! - `component(caminho, ctx_inicial?)`: carrega um `.gv`/`.gvb` com `<script>`
//!   como o motor carregaria e devolve um objeto com `:call(acao, valor?)`,
//!   `:get(chave)`, `:set(chave, valor)`, `:data()` e `:toasts()`. Um handler
//!   que falha **lança** o erro (em produção ele viraria um toast).
//!
//! `require` funciona como no app (relativo ao arquivo de teste), então módulos
//! Luau puros testam-se só com `require` + `expect`.
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
pub fn run_cli(target: Option<&str>, filter: Option<&str>) -> i32 {
    let files = discover(target);
    if files.is_empty() {
        eprintln!("nenhum arquivo *.test.luau encontrado (procurei em views/ e tests/)");
        return 1;
    }
    let mut total = Summary::default();
    for file in &files {
        let s = run_file(file, filter);
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
    Some(run_cli(Some(&target), filter.as_deref()))
}

/// Roda um arquivo de teste e imprime ✓/✗ por teste.
pub fn run_file(path: &Path, filter: Option<&str>) -> Summary {
    println!("{}", path.display());
    let mut summary = Summary::default();
    let script = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            println!("  ✗ não consegui ler: {e}");
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
        path_str,
        path,
        Arc::new(DiskAssets),
        Some(&install),
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("  ✗ não carregou: {e}");
            summary.broken_files = 1;
            return summary;
        }
    };
    let run: mlua::Function = match comp.luau.globals().get("__glacier_run") {
        Ok(f) => f,
        Err(e) => {
            println!("  ✗ {e}");
            summary.broken_files = 1;
            return summary;
        }
    };
    let results: mlua::Table = match run.call(filter) {
        Ok(t) => t,
        Err(e) => {
            println!("  ✗ {e}");
            summary.broken_files = 1;
            return summary;
        }
    };
    for r in results.sequence_values::<mlua::Table>().flatten() {
        let name: String = r.get("name").unwrap_or_default();
        if r.get::<bool>("ok").unwrap_or(false) {
            println!("  ✓ {name}");
            summary.passed += 1;
        } else {
            let err: String = r.get("err").unwrap_or_default();
            println!("  ✗ {name}\n    {err}");
            summary.failed += 1;
        }
    }
    summary
}

/// Um componente carregado por `component(...)`, com o contexto que ele muta.
struct TestComponent {
    comp: LuauComponent,
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
            "describe('soma', function()\n\
               test('ok', function() expect(1 + 1):toBe(2) end)\n\
               test('quebra', function() expect({a = 1}):toEqual({a = 2}) end)\n\
               test('nega', function() expect(1).never:toBe(2) end)\n\
               test('lanca', function() expect(function() error('boom') end):toThrow('boom') end)\n\
             end)",
        )
        .unwrap();
        let s = run_file(&file, None);
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
                "test('inc', function()\n\
                   local c = component({:?}, {{ n = 1 }})\n\
                   c:call('inc')\n\
                   expect(c:get('n')):toBe('2')\n\
                   expect(function() c:call('nao_existe') end):toThrow('nao_existe')\n\
                 end)",
                tela.to_str().unwrap()
            ),
        )
        .unwrap();
        let s = run_file(&file, None);
        assert_eq!((s.passed, s.failed, s.broken_files), (1, 0, 0), "{s:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
