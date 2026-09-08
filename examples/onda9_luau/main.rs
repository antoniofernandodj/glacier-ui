//! **Onda 9, em Luau**: o ponteiro preso, do lado do script.
//!
//! Rode com: `cargo run --example onda9_luau`
//!
//! # O que este exemplo mostra e a versão Rust não mostra
//!
//! Nada de API nova — e **é essa a observação**.
//!
//! A Onda 8 precisou de quatro globais novos do lado Luau (`prompt`,
//! `progress`, `pick_color` e o retorno generalizado de `DialogOutcome`),
//! porque um diálogo **suspende**: a corrotina para no modal e volta com a
//! resposta, e sem um global não haveria como parar. Foi metade do custo
//! daquela onda.
//!
//! Um arrasto não suspende nada. Ele escreve numa chave, e o Luau já lê e
//! escreve chaves desde a 0.60 — `ctx.painel_h = "300 fill"` é a mesma linha
//! que configuraria um `<slider>`. Os nove widgets desta onda chegam ao script
//! prontos, sem uma linha em `src/luau/`.
//!
//! Isso vale registrar porque é o **terceiro regime de custo** que o
//! `PLANO_WIDGETS.md` cataloga, agora medido do lado do script: um habilitador
//! de motor que rende meia dúzia de widgets e ainda assim custa **zero** à
//! camada de cima, porque ele fala a língua que ela já falava (a chave nomeada).
//!
//! # O que sobra para o script fazer, então
//!
//! Exatamente três coisas, e todas as três já eram possíveis antes:
//!
//! 1. **semear** as chaves (`init`) — inclusive a coleção do `<tumbler>`, que
//!    do lado Luau é uma tabela e vira JSON sozinha;
//! 2. **receber o `on_change`** de quem delega — aqui só o `<swipeview>`, porque
//!    os outros oito gravam a chave sozinhos;
//! 3. **reagir a um atalho** — e um `<shortcut>` despacha uma função comum, não
//!    um evento de tipo novo.
//!
//! # A aritmética é do script, e continua sendo
//!
//! Os alvos do `<rubberband>` são posicionados por uma conta em `caixas()`.
//! Podia parecer trabalho para o markup, e não é: um template não calcula, que
//! é a descoberta que a Onda 8 escreveu ao partir o `<wizard>` em builtin (os
//! slots) mais primitiva (as contas).

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 9 (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) = motor.register_component("onda9_luau", "examples/onda9_luau/app.gv") {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda9_luau");
        })
        .run()
}
