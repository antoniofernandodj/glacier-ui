//! Diálogo de arquivo/pasta nativo do SO — arquivo único, múltiplos
//! arquivos, diretório e salvar (ver `src/file_dialog.rs`). UI +
//! comportamento no mesmo arquivo, em Lua — ver
//! `examples/gva/file_dialog/file_dialog.gva`.
//!
//! Rode com: `cargo run --example file_dialog`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gva/file_dialog/file_dialog.gva").run()
}
