//! O builtin `<spinbox/>` — campo numérico com os degraus ▴▾, o `QSpinBox` do Qt.
//!
//! Rode com: `cargo run --example gvb_spinbox`
//!
//! O app registra a tela e semeia o valor inicial de cada chave. `SpinBox` não
//! é registrado aqui: a lib o registra sozinha em `GlacierUI::new()` (ver
//! `src/builtins/mod.rs`), então a tag funciona como uma primitiva. E não há nenhum `update` do lado do app —
//! somar, subtrair e saturar é comportamento do próprio widget; ele escreve na
//! chave de contexto que cada instância nomeia na prop `value`, que é o que os
//! `{quantidade}`/`{preco}` do template exibem de volta.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gvb/spinbox/app.gvb").run()
}
