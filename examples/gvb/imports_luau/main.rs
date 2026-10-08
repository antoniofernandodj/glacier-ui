//! Imports na camada Lua: o `<script>` de `app.gvb` divide a lógica em
//! bibliotecas e as importa com `require`, mantendo tudo encapsulado:
//!
//! ```luau
//! local http    = require("net.http_client")  -- net/http_client.luau (client de rede)
//! local strings = require("util.strings")      -- util/strings.luau    (lógica pura)
//! ```
//!
//! `require("a.b")` procura `a/b.luau` (e `a/b/init.luau`) relativo ao diretório
//! do template, depois em `<dir>/lib`, depois nos caminhos de `GLACIER_LUA_PATH`.
//! Os módulos rodam no mesmo interpretador do componente, então o client de
//! rede pode usar `fetch` (async/await via corrotina) por baixo dos panos.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/imports_luau/app.gvb")
        .run()
}
