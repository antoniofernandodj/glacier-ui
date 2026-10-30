//! **Texto rico, em Luau**: a mesma tela do `examples/gva/texto_rico`, sem um
//! `Component` em Rust. Ver `docs/TEXTO.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example texto_rico_luau`
//!
//! ```text
//!                     texto_rico (Rust)           texto_rico_luau
//! estado inicial      Component::init             function init()
//! a ação do link      Component::update           function contar()
//! as duas telas       dois `register`             duas <screen> do <app>
//! ```

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gva/texto_rico_luau/app.gva")
        .run()
}
