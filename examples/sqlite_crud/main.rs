//! Registrar funções Rust na camada Lua — um cliente **SQLite** completo,
//! usado por um mini-CRUD escrito inteiramente no `<script>`.
//!
//! # A ideia
//!
//! O motor expõe alguns globais ao `<script>` (`fetch`, `json`, `storage`, …),
//! mas não um banco de dados — e não deveria: cada app quer o seu. A ponte é
//! [`GlacierDaemon::lua_extension`]: um closure `Fn(&mlua::Lua) -> Result<()>`
//! que roda em cada VM Luau nova e instala nela o que quiser. Aqui ele instala
//! um global `sqlite`:
//!
//! ```lua
//! local db = sqlite.connect("tarefas.db")   -- nome simples → diretório temporário do SO
//! db:execute("CREATE TABLE IF NOT EXISTS t (id INTEGER PRIMARY KEY, nome TEXT)")
//! db:begin()
//! db:execute("INSERT INTO t (nome) VALUES (?)", { "café" })
//! db:commit()
//! local linhas = db:query("SELECT id, nome FROM t ORDER BY id")  -- array de tabelas
//! db:close()
//! ```
//!
//! Métodos da conexão: `execute(sql, params?) -> nº de linhas`,
//! `query(sql, params?) -> {linha…}`, `begin()` / `commit()` / `rollback()`,
//! `last_insert_id()`, `close()`. `params` é um array Lua (`nil`/bool/número/
//! string), ligado por `?` posicional.
//!
//! `WGPU_BACKEND=gl cargo run --example sqlite_crud` nesta máquina (Vulkan da
//! GPU integrada quebrado).

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use glacier_ui::mlua::{self, Lua, Table, UserData, UserDataMethods, Value};
use glacier_ui::GlacierDaemon;
use rusqlite::Connection;
use rusqlite::types::{Value as SqlValue, ValueRef};

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .title("Glacier - CRUD com SQLite")
        // A ÚNICA linha que liga o banco: a partir daqui, todo `<script>` deste
        // app enxerga o global `sqlite`.
        .lua_extension(instalar_sqlite)
        .main(|motor| {
            if let Err(e) = motor.register_component("crud", "examples/sqlite_crud/crud.gv") {
                eprintln!("Erro ao registrar: {e}");
            }
            motor.set_initial_screen("crud");
        })
        .run()
}

// ── A ponte ────────────────────────────────────────────────────────────────

/// Instala o global `sqlite` na VM. Assinatura de [`glacier_ui::LuaExtension`]
/// (um closure `Fn(&Lua) -> mlua::Result<()>` já a satisfaz).
fn instalar_sqlite(lua: &Lua) -> mlua::Result<()> {
    let sqlite = lua.create_table()?;

    let connect = lua.create_function(|lua, path: Option<String>| {
        let alvo = resolver_caminho(path.as_deref().unwrap_or(":memory:"));
        let conn = Connection::open(&alvo).map_err(mlua::Error::external)?;
        // Conveniências de app desktop; ignoradas de propósito se falharem
        // (`:memory:` não aceita WAL, por exemplo).
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;");
        lua.create_userdata(Conexao(RefCell::new(Some(conn))))
    })?;
    sqlite.set("connect", connect)?;

    lua.globals().set("sqlite", sqlite)?;
    Ok(())
}

/// Um nome simples (`tarefas.db`) vai para o diretório temporário do SO — o
/// exemplo é re-executável sem sujar o repositório e os dados sobrevivem entre
/// execuções. Um caminho com barra, ou `:memory:`, é usado como está.
fn resolver_caminho(bruto: &str) -> PathBuf {
    if bruto == ":memory:" || Path::new(bruto).is_absolute() || bruto.contains(std::path::MAIN_SEPARATOR)
    {
        PathBuf::from(bruto)
    } else {
        std::env::temp_dir().join(bruto)
    }
}

/// A conexão vista pelo Lua. `Option` para o `close()` poder soltá-la antes do
/// GC; os métodos erram se ela já foi fechada. `RefCell` porque `add_method`
/// entrega `&self` — o app é single-thread (thread da UI), então não há disputa.
struct Conexao(RefCell<Option<Connection>>);

impl Conexao {
    fn com<R>(&self, f: impl FnOnce(&Connection) -> rusqlite::Result<R>) -> mlua::Result<R> {
        let guarda = self.0.borrow();
        let conn = guarda
            .as_ref()
            .ok_or_else(|| mlua::Error::runtime("conexão sqlite já fechada"))?;
        f(conn).map_err(mlua::Error::external)
    }
}

impl UserData for Conexao {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        // INSERT/UPDATE/DELETE/DDL de uma instrução. Devolve o nº de linhas
        // afetadas.
        m.add_method("execute", |_, this, (sql, params): (String, Option<Table>)| {
            let vals = ligar_params(params)?;
            let n = this.com(|c| c.execute(&sql, rusqlite::params_from_iter(vals)))?;
            Ok(n as i64)
        });

        // SELECT. Devolve um array de tabelas (coluna → valor); `Text`/`Blob`
        // chegam como string Lua, `NULL` como `nil`.
        m.add_method("query", |lua, this, (sql, params): (String, Option<Table>)| {
            let vals = ligar_params(params)?;
            let guarda = this.0.borrow();
            let conn = guarda
                .as_ref()
                .ok_or_else(|| mlua::Error::runtime("conexão sqlite já fechada"))?;
            let mut stmt = conn.prepare(&sql).map_err(mlua::Error::external)?;
            let colunas: Vec<String> =
                stmt.column_names().iter().map(|s| s.to_string()).collect();
            let mut rows = stmt
                .query(rusqlite::params_from_iter(vals))
                .map_err(mlua::Error::external)?;

            let saida = lua.create_table()?;
            let mut i = 1i64;
            while let Some(row) = rows.next().map_err(mlua::Error::external)? {
                let registro = lua.create_table()?;
                for (idx, nome) in colunas.iter().enumerate() {
                    let valor = match row.get_ref(idx).map_err(mlua::Error::external)? {
                        ValueRef::Null => Value::Nil,
                        ValueRef::Integer(n) => Value::Integer(n),
                        ValueRef::Real(f) => Value::Number(f),
                        ValueRef::Text(b) | ValueRef::Blob(b) => {
                            Value::String(lua.create_string(b)?)
                        }
                    };
                    registro.set(nome.as_str(), valor)?;
                }
                saida.set(i, registro)?;
                i += 1;
            }
            Ok(saida)
        });

        m.add_method("begin", |_, this, ()| this.com(|c| c.execute_batch("BEGIN")));
        m.add_method("commit", |_, this, ()| this.com(|c| c.execute_batch("COMMIT")));
        m.add_method("rollback", |_, this, ()| {
            this.com(|c| c.execute_batch("ROLLBACK"))
        });
        m.add_method("last_insert_id", |_, this, ()| {
            this.com(|c| Ok(c.last_insert_rowid()))
        });
        m.add_method("close", |_, this, ()| {
            *this.0.borrow_mut() = None;
            Ok(())
        });
    }
}

/// Array Lua → parâmetros posicionais (`?`). `nil`/bool/número/string; qualquer
/// outro tipo é erro em vez de virar `NULL` em silêncio.
fn ligar_params(params: Option<Table>) -> mlua::Result<Vec<SqlValue>> {
    let mut saida = Vec::new();
    let Some(t) = params else { return Ok(saida) };
    for item in t.sequence_values::<Value>() {
        saida.push(match item? {
            Value::Nil => SqlValue::Null,
            Value::Boolean(b) => SqlValue::Integer(b as i64),
            Value::Integer(n) => SqlValue::Integer(n),
            Value::Number(f) => SqlValue::Real(f),
            Value::String(s) => SqlValue::Text(s.to_str()?.to_owned()),
            outro => {
                return Err(mlua::Error::runtime(format!(
                    "parâmetro sqlite de tipo não suportado: {}",
                    outro.type_name()
                )));
            }
        });
    }
    Ok(saida)
}
