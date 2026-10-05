//! **Onda 3** do `PLANO_WIDGETS.md`: o calendário — uma primitiva, três tags.
//!
//! Rode com: `cargo run --example onda3`
//!
//! `<calendar>`, `<monthyearpicker>` e `<daterangepicker>` são o **mesmo**
//! `NodeType::Calendar`, exatamente como `<dateedit>`/`<timeedit>` são o mesmo
//! `NodeType::DateTimeEdit`. O que muda entre elas é o que um clique grava (um
//! dia, um mês, duas datas) e quantas grades aparecem — nada mais.
//!
//! # Por que este exemplo é todo em Luau
//!
//! Por causa de uma prop: `today`. O realce de hoje é **prop, não relógio** —
//! o motor não lê a hora do sistema em lugar nenhum, de propósito (ver
//! `PLANO_WIDGETS.md` §4), e quem sabe que dia é hoje é o app. Do lado do
//! script isso é `date.today()`, uma linha; do lado do Rust exigiria uma crate
//! de data que o motor recusou duas vezes.
//!
//! O resto do exemplo aproveita a mesma carona: as regras de intervalo
//! (quantas noites, recusar uma saída antes da entrada) são de **negócio**, não
//! do widget, e é o `onChange` que as torna possíveis — o mesmo contrato do
//! `<textinput>` e do `<dateedit>`.
//!
//! Compare com `examples/gva/data_hora_luau`, que é a mesma inversão sobre os
//! campos por seções: lá se **digita** a data, aqui se **aponta** para ela.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gva/onda3/app.gva").run()
}
