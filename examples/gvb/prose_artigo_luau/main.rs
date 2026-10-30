//! **Um artigo, quatro sintaxes, em Luau**: a mesma tela do `examples/gvb_prose_artigo_luauRUST`, sem um
//! `Component` em Rust — o texto vem de arquivos, lidos pelo `fetch("file://…")` do
//! `scripts/app.luau`. Ver `docs/TEXTO.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example gvb_prose_artigo_luau`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/prose_artigo_luau/app.gvb")
        .run()
}
