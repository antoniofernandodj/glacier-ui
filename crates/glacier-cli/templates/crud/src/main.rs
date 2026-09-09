//! Lista editável (model/view) — uma `<tableview>` ligada a um array do
//! contexto, com Novo/Editar/Excluir via `prompt{}` e `confirm{}` da camada
//! Luau. Os dados vivem em memória (`views/scripts/state.luau`).
//! `src/main.rs` é só a casca: registra `views/app.gv` e abre a janela.

use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(erro) = motor.register_component("app", "views/app.gv") {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("app");
        })
        .run()
}
