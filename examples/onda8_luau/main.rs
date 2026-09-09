//! **Onda 8, em Luau**: os quatro diálogos que a onda entrega à camada de
//! script.
//!
//! Rode com: `cargo run --example onda8_luau`
//!
//! # O que este exemplo mostra
//!
//! **Item 1 — o `<dialog>` declarativo** aparece nos dois exemplos, por
//! paridade: a mesma tag do `onda8`, um corpo em markup, aberto por
//! `dialog:editar_rotulo`. Ele NÃO é API de script — o botão "Salvar" despacha
//! `salvar_rotulo`, um handler Luau comum, que lê as chaves `__dialog.*`.
//!
//! **Itens 2, 3 e 4** (`InputDialog`, `ProgressDialog`, `ColorDialog`), esses
//! sim, só a camada de script mostra, e a razão é o **suspensivo**. `prompt` e
//! `pick_color` param a corrotina no diálogo e voltam com a resposta:
//!
//! ```lua
//! local nome = prompt({ title = "Renomear", value = atual })
//! if nome then ctx.servico = nome end
//! ```
//!
//! Do lado Rust, o mesmo pedido é um `ctx.show_dialog` mais um braço no
//! `update` para tratar o botão — correto, e três lugares em vez de um. É a
//! mesma diferença que o `fetch` tem desde sempre, e o motivo de o habilitador
//! B desta onda existir: até a 0.93 a resposta de um diálogo suspensivo era um
//! `bool`, porque o único suspensivo era o `confirm()`.
//!
//! # O `nil` da desistência
//!
//! Um `prompt` cancelado devolve `nil`, não `false` — a convenção que o
//! `open_file()` já usava. Parecem a mesma coisa (em Luau os dois são falsos) e
//! não são: `""` é "respondeu vazio" e `nil` é "não respondeu".
//!
//! # E o que NÃO mudou
//!
//! O `confirm{}` continua sem corpo e continua devolvendo booleano. É o teste
//! de regressão do habilitador B, escrito como exemplo: generalizar o retorno
//! não podia custar a forma que já existia.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 8 (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) = motor.register_component("onda8_luau", "examples/onda8_luau/app.gv") {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda8_luau");
        })
        .run()
}
