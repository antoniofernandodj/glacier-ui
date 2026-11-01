//! Menu bar ancorada (File/Edit) + menu de contexto (botão direito), com
//! submenus aninhados a profundidade arbitrária (ver `src/menu.rs`). UI +
//! comportamento no mesmo arquivo, em Lua — ver `examples/menus/menus.gv`.
//!
//! Rode com: `cargo run --example menus`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/menus/menus.gv")
        .run()
}
