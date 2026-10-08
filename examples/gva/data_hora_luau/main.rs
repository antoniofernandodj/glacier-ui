//! Os campos de data/hora **inteiramente controlados por Luau** — sem um
//! `impl Component`, sem `define_data`, sem uma linha de lógica em Rust.
//!
//! Rode com: `cargo run --example data_hora_luau`
//!
//! Compare com `examples/gva/timepicker`, que é a outra ponta: lá os campos gravam
//! a chave sozinhos e o app não escreve nada. Aqui cada campo tem `onChange`, e
//! isso **inverte quem manda**: o widget passa a só avisar, e quem decide se o
//! valor entra é o script. É o mesmo contrato do `<textinput>`.
//!
//! É essa inversão que permite as regras deste exemplo — recusar uma saída
//! anterior à entrada, avisar com um `toast`, recalcular o resumo a cada
//! alteração. Nenhuma delas caberia no widget, porque nenhuma é sobre datas: são
//! sobre o *negócio* de quem usa.
//!
//! O `main` só registra a tela. Tudo o mais — inclusive os valores iniciais —
//! está em `app.luau`.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gva/data_hora_luau/app.gva")
        .run()
}
