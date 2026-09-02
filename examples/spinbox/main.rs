//! O builtin `<spinbox/>` — campo numérico com os degraus ▴▾, o `QSpinBox` do Qt.
//!
//! Rode com: `cargo run --example spinbox`
//!
//! O app registra a tela e semeia o valor inicial de cada chave. `SpinBox` não
//! é registrado aqui: a lib o registra sozinha em `GlacierUI::new()` (ver
//! `src/builtins/mod.rs`), então a tag funciona como uma primitiva. E não há nenhum `update` do lado do app —
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
            // Valor inicial de cada chave. É a única coisa que o app faz além
            // de registrar a tela: um `<spinbox/>` cuja chave nunca foi escrita
            // nasce em branco (o primeiro clique num degrau o inicializa no
            // `min`), e um campo numérico vazio parece um campo quebrado.
            //
            // Uma chamada por chave, com a chave literal: é o que a extensão do
            // VS Code procura (`define_data("…"`) para linkar o `value="preco"`
            // do template até aqui. Num laço sobre uma lista de tuplas o link
            // some.
            motor.define_data("quantidade", "1");
            motor.define_data("preco", "4.50");
            motor.define_data("zoom", "100");
            motor.define_data("adultos", "2");
            motor.define_data("criancas", "0");
            // A dupla da 0.69: `class` na Row, `field_class` no campo.
            motor.define_data("mesas", "2");
            motor.define_data("obs", "");
            motor.set_initial_screen("spinbox");
        })
        .run()
}
