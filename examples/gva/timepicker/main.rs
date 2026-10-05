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
    GlacierDaemon::new().main_template("examples/gva/timepicker/app.gva").run()
}
