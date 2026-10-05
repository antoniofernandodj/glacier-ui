//! **`<foreach fallback>`, em Luau** — o par do exemplo `foreach_fallback`,
//! com o mesmo markup e a lógica num `<script>`.
//!
//! Rode com: `cargo run --example gvb_foreach_fallback_luau` (com
//! `WGPU_BACKEND=gl` nesta máquina).
//!
//! # O que este exemplo mostra
//!
//! Cada etapa da cozinha é um `<foreach items="…" fallback="{vazio}">` dentro
//! do componente `Etapa`, que recebe o componente de "vazio" por atributo
//! (`vazio="FilaVazia"`). A lista de entregues usa o `fallback` literal.
//!
//! O script (`script.luau`) não sabe nada sobre os fallbacks: ele só publica
//! quatro listas com `json.encode(json.array(...))`. Uma lista vazia vira `[]`,
//! e é isso que faz o componente de "vazio" entrar.
//!
//! Nada disto é do lado Rust: este `main.rs` só registra a tela.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gvb/foreach_fallback_luau/app.gvb").run()
}
