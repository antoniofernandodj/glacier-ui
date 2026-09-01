//! `<dateedit>`, `<timeedit>` e `<datetimeedit>` — o `QDateEdit`, o
//! `QTimeEdit` e o `QDateTimeEdit` do Qt.
//!
//! Rode com: `cargo run --example timepicker`
//!
//! As três são a **mesma primitiva**, com seções diferentes: clicar numa seção
//! a seleciona (realce da paleta, como o `2001` destacado de um `QDateEdit`) e
//! as setas ▴▾ mexem naquela seção. Um controle cobre o valor inteiro — não há
//! prop de passo nem um widget por campo.
//!
//! **Não há nenhum `update` e nenhum script.** A aritmética (virar dentro da
//! seção, dias no mês, bissexto) roda no motor; o widget grava na chave que
//! cada instância nomeia na prop `value`, sempre em **ISO** — `format="br"` só
//! muda a ordem das seções na tela.
//!
//! Até a 0.67 isto era um builtin delegante e este exemplo tinha ~40 linhas de
//! Luau montando um seletor à mão. Sumiram todas.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - TimePicker")
        .main(|motor| {
            // A tela NÃO pode se chamar `timepicker`: o registro do app vence o
            // builtin de mesmo nome (a regra de override), então a tag
            // `<timepicker/>` de dentro do template passaria a apontar para a
            // própria tela — recursão infinita. O motor agora acusa isso com um
            // erro em vez de estourar a pilha, mas o nome certo evita o assunto.
            motor
                .register_component("tela_hora", "examples/timepicker/app.gv")
                .expect("registrar a tela do exemplo");
            // Uma chamada por chave, com a chave literal: é o que a extensão do
            // VS Code procura (`define_data("…"`) para linkar o `value="inicio"`
            // do template até aqui. Num laço sobre uma lista de tuplas o link
            // some.
            motor.define_data("data", "2026-09-01");
            motor.define_data("hora", "13:45:02");
            motor.define_data("quando", "2026-09-01 09:00");
            motor.define_data("data_br", "2026-09-01");
            motor.define_data("de", "09:00");
            motor.define_data("ate", "18:00");
            motor.set_initial_screen("tela_hora");
        })
        .run()
}
