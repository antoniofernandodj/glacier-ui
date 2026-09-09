//! **Onda 11, habilitador C, em Luau**: séries múltiplas do lado do script.
//!
//! Rode com: `cargo run --example series_multiplas_luau` (com `WGPU_BACKEND=gl`
//! nesta máquina).
//!
//! # O que este exemplo mostra
//!
//! Nada de API nova — a mesma observação das ondas 9 a 11. O `series="chave"`
//! é uma convenção de DADOS: o script escreve uma tabela de
//! `{ name, points, color? }` na chave, e o `<linechart>` a lê no render. O que
//! sobra para o Luau é semear e, opcionalmente, mexer nos números — as duas
//! coisas de sempre.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — séries múltiplas (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) = motor.register_component(
                "series_multiplas_luau",
                "examples/series_multiplas_luau/app.gv",
            ) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("series_multiplas_luau");
        })
        .run()
}
