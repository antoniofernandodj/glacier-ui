//! Os campos de data/hora **inteiramente controlados por Luau** — sem um
//! `impl Component`, sem `define_data`, sem uma linha de lógica em Rust.
//!
//! Rode com: `cargo run --example data_hora_luau`
//!
//! Compare com `examples/timepicker`, que é a outra ponta: lá os campos gravam
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
        .title("Glacier - Data e Hora (Luau)")
        .main(|motor| {
            // A tela não pode se chamar como uma tag de widget: o registro do
            // app vence o builtin/primitiva de mesmo nome e a tag apontaria
            // para ela mesma.
            motor
                .register_component("reserva", "examples/data_hora_luau/app.gv")
                .expect("registrar a tela do exemplo");
            motor.set_initial_screen("reserva");
        })
        .run()
}
