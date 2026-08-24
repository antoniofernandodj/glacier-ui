//! Menu bar ancorada (File/Edit) + menu de contexto (botão direito), com
//! submenus aninhados a profundidade arbitrária (ver `src/menu.rs`). UI +
//! comportamento no mesmo arquivo, em Lua — ver `examples/menus/menus.gv`.
//!
//! Rode com: `cargo run --example menus`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Menus")
        .main(|motor| {
            if let Err(e) = motor.register_component("menus", "examples/menus/menus.gv") {
                eprintln!("Erro ao registrar: {}", e);
            }
            motor.set_initial_screen("menus");
        })
        .run()
}
