//! **Onda 6, em Luau**: a mesma tela do `examples/onda6`, sem um `Component`
//! em Rust.
//!
//! Rode com: `cargo run --example onda6_luau`
//!
//! Os dois exemplos compartilham o markup — `app.gv` é o mesmo arquivo, linha
//! por linha, tirando o `<script src>` e o título. O que muda é onde mora o
//! comportamento:
//!
//! ```text
//!                     examples/onda6          examples/onda6_luau
//! estado inicial      Component::init         function init()
//! as duas ações       Component::update       uma função global por ação
//! as linhas           const SERVICOS + json!  local SERVICOS (tabela)
//! as colunas          uma string JSON crua    local COLUNAS (tabela)
//! a árvore            fn arvore_json()        local ARVORE (tabela)
//! registro            motor.register(Box…)    motor.register_component(…)
//! ```
//!
//! # O que o exemplo mostra, e que a versão Rust não mostra sozinha
//!
//! **Que os seis widgets desta onda não pedem script nenhum.** Com o Rust
//! reduzido a este arquivo, sobra a prova: ordenar uma coluna (e inverter ao
//! clicar de novo), escolher uma linha, marcar várias, arrastar a alça de uma
//! coluna, abrir e fechar um nó da árvore e descer um nível no Miller
//! continuam funcionando com as **mesmas** duas funções de Luau que a versão
//! Rust tem de Rust.
//!
//! # A diferença que aparece no arquivo
//!
//! Esta é a onda em que ela mais aparece, e é uma só: **as três estruturas.**
//! A tela passa aos widgets as linhas da tabela, a definição das colunas e uma
//! árvore aninhada — e do lado Rust as três exigem `serde_json::json!`
//! construído à mão, incluindo uma função de trinta linhas só para a árvore.
//! Aqui são tabelas literais, e o `ctx` as codifica em JSON ao atribuí-las.
//!
//! É a mesma observação que o `examples/onda4_luau` fez sobre uma lista de
//! serviços, agora com uma estrutura três níveis mais funda — que é onde a
//! diferença deixa de ser cosmética.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 6 (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            // Sem `register(Box::new(...))`: não há `impl Component` neste
            // exemplo. O `.gv` traz o `<script src>`, e o motor cuida do resto.
            if let Err(e) = motor.register_component("onda6_luau", "examples/onda6_luau/app.gv") {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda6_luau");
        })
        .run()
}
