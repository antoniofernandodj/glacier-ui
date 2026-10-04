//! Catálogo de widgets — uma janela com uma barra lateral de categorias, cada
//! uma abrindo uma tela cheia de widgets do motor com um exemplo mínimo e vivo.
//!
//! `src/main.rs` é só a casca: sobe o runner e registra `views/app.gvb`
//! automaticamente. Título, tamanho, tema e todo o resto moram nos
//! `.gvb`/`.gss`/`.luau` e recarregam a quente. As telas de categoria
//! entram por `<link rel="import">` em cascata a partir de `views/app.gvb`
//! — nenhuma precisa ser registrada aqui.

use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .run()
}
