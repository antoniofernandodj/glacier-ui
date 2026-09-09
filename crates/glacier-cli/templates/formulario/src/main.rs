//! Formulário validado — entradas, máscaras, um `<buttonbox>` e validação em
//! Luau que trava o "Salvar" enquanto houver erro. `src/main.rs` é só a casca:
//! registra `views/app.gv` e abre a janela.

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
