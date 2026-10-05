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
    GlacierDaemon::new().main_template("examples/gva/series_multiplas_luau/app.gva").run()
}
