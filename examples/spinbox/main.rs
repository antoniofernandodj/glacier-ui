//! O builtin `<SpinBox/>` — campo numérico com as setas ▼▲, o `QSpinBox` do Qt.
//!
//! Rode com: `cargo run --example spinbox`
//!
//! O app registra **só a tela**. `SpinBox` não é registrado aqui: a lib o
//! registra sozinha em `GlacierUI::new()` (ver `src/builtins/mod.rs`), então a
//! tag funciona como uma primitiva. E não há nenhum `update` do lado do app —
//! somar, subtrair e saturar é comportamento do próprio widget; ele escreve na
//! chave de contexto que cada instância nomeia na prop `value`, que é o que os
//! `{quantidade}`/`{preco}` do template exibem de volta.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - SpinBox")
        .main(|motor| {
            motor
                .register_component("spinbox", "examples/spinbox/app.gv")
                .expect("registrar a tela do exemplo");
            motor.set_initial_screen("spinbox");
        })
        .run()
}
