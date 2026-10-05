//! **Onda 13, em Luau**: o vocabulário de formas do lado do script.
//!
//! Rode com: `cargo run --example gvb_onda13_luau` (com `WGPU_BACKEND=gl` nesta
//! máquina).
//!
//! # O que este exemplo mostra
//!
//! A mesma observação das ondas anteriores: o `<canvas>` e as formas são
//! MARKUP, e a geometria dirigida por dado é só uma chave de contexto. O que o
//! Luau faz é semear e mexer nos números — aqui, um relógio que gira uma
//! agulha desenhada com `<line>` e `<arc>`.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gvb/onda13_luau/app.gvb").run()
}
