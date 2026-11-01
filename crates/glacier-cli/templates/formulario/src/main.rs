//! Formulário validado — entradas, máscaras e validação **declarada no
//! `<form>`** (`rules="…"` nos campos); o motor valida ao enviar e chama
//! `salvar` ou `apontar`. `src/main.rs` é só a casca: registra `views/app.gv`
//! automaticamente e abre a janela.

use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .run()
}
