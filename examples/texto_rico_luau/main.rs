//! **Texto rico em `.gv`, em Luau**: a mesma tela do `examples/texto_rico`, sem um
//! `Component` em Rust. Ver `docs/TEXTO.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example texto_rico_luau`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/texto_rico_luau/app.gv")
        .run()
}
