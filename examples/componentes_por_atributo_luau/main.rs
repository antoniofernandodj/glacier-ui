//! **Componentes por atributo, em Luau** — o par do exemplo
//! `componentes_por_atributo`, com a lógica num `<script>`.
//!
//! Rode com: `cargo run --example componentes_por_atributo_luau` (com
//! `WGPU_BACKEND=gl` nesta máquina).
//!
//! # O que este exemplo mostra
//!
//! A `<Vitrine>` recebe três componentes por atributo — `cabecalho`, `linha` e
//! `vazio` — e dois deles chegam de chaves que o **script** escreve:
//!
//! ```xml
//! <Vitrine cabecalho="{cabecalho}" linha="LinhaContato" vazio="{vazio}" … />
//! ```
//!
//! O nome de um componente é um dado como outro qualquer. O botão "Trocar
//! cabeçalho" só grava outra string em `ctx.cabecalho`; a busca troca
//! `ctx.vazio` entre `NenhumContato` e `BuscaSemResultado` conforme o motivo de
//! a lista estar vazia.
//!
//! A lista de favoritos usa a tag `<foreach>` com um `fallback` literal.
//!
//! Nada disto é do lado Rust: este `main.rs` só registra a tela.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Componentes por atributo (Luau)")
        .main(|motor| {
            if let Err(e) = motor.register_component(
                "componentes_por_atributo_luau",
                "examples/componentes_por_atributo_luau/app.gv",
            ) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("componentes_por_atributo_luau");
        })
        .run()
}
