//! **Seletores compostos no `.gss`**, com o comportamento em Luau.
//!
//! Rode com: `cargo run --example gvb_seletores_luau`
//!
//! A mesma tela do `examples/gvb/seletores`, com o `impl Component` de Rust
//! trocado por `scripts/app.luau`. O markup e o `app.gss` são os mesmos — o que
//! o exemplo mostra está no `.gss`: estilo decidido pelo lugar do nó na árvore
//! (`row.servico > text.nome`), por uma classe vinda do dado
//! (`row.servico.parado > …`) e por uma classe dinâmica num ancestral
//! (`.compacta .lista > row.servico`).

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/seletores_luau/app.gvb")
        .run()
}
