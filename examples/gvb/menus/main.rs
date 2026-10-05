//! Menu bar ancorada (File/Edit) + menu de contexto (botão direito), com
//! submenus aninhados a profundidade arbitrária (ver `src/menu.rs`). UI +
//! comportamento no mesmo arquivo, em Lua — ver `examples/gvb/menus/menus.gvb`.
//!
//! Rode com: `cargo run --example gvb_menus`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gvb/menus/menus.gvb").run()
}
