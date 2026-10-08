//! **Onda 7, em Luau**: a mesma tela do `examples/gvb/onda7`, sem um `Component`
//! em Rust.
//!
//! Rode com: `cargo run --example gvb_onda7_luau`
//!
//! Os dois exemplos compartilham o markup — `app.gvb` é o mesmo arquivo, linha
//! por linha, tirando o `<script src>` e o título. O que muda é onde mora o
//! comportamento:
//!
//! ```text
//!                     examples/gvb/onda7          examples/gvb/onda7_luau
//! estado inicial      Component::init         function init()
//! as séries           const + serde_json!     tabelas literais
//! as faixas do gauge  uma string JSON crua    local LIMITES_DISCO
//! o p95               fn p95()                local function p95()
//! o relógio           fn hora_local()         date.time()
//! registro            motor.register(Box…)    motor.register_component(…)
//! ```
//!
//! # O que este exemplo mostra e a versão Rust não mostra
//!
//! **O painel anda sozinho.** O script liga um `every(1000, …)` e passa a
//! escrever `relogio`, `cpu`, `lat_api` e `rede` a cada segundo. Os sete
//! widgets acompanham sem saber que existe um temporizador — eles redesenham a
//! partir da **chave**, que é o contrato de todo widget deste motor.
//!
//! Vale dizer o que isso **não** significa: nenhum dos sete precisa do `every`
//! para funcionar. Girar o `<dial>`, ler um `<gauge>`, desenhar as séries — o
//! `init` sozinho já entrega tudo isso, exatamente como na versão Rust. O
//! temporizador está aqui para provar a outra metade.
//!
//! # A diferença que aparece no arquivo
//!
//! As **séries**. Esta é a onda em que a tela passa mais dado estruturado aos
//! widgets: três séries de 24 pontos, duas de rótulo e valor, e uma tabela de
//! faixas coloridas. Do lado Rust cada uma exige um `serde_json::json!`
//! construído à mão; aqui são tabelas literais, e o `ctx` as codifica ao
//! atribuí-las.
//!
//! É a mesma observação que o `examples/gvb/onda6_luau` fez sobre a árvore
//! aninhada, agora com séries numéricas — que é onde ela deixa de ser
//! cosmética e vira legibilidade.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/onda7_luau/app.gvb")
        .run()
}
