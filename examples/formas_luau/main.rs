//! **Onda 13, em Luau**: o vocabulário de formas do lado do script.
//!
//! Rode com: `cargo run --example formas_luau` (com `WGPU_BACKEND=gl` nesta
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
    GlacierDaemon::new()
        .title("Glacier — Onda 13 (formas, Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) =
                motor.register_component("formas_luau", "examples/formas_luau/app.gv")
            {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("formas_luau");
        })
        .run()
}
