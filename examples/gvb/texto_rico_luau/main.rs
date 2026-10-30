//! **Texto rico em `.gvb`, em Luau**: a mesma tela do `examples/gvb/texto_rico`, sem um
//! `Component` em Rust. Ver `docs/TEXTO.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example gvb_texto_rico_luau`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/texto_rico_luau/app.gvb")
        .run()
}
