//! **Onda 5, em Luau**: a mesma tela do `examples/onda5`, sem um `Component`
//! em Rust.
//!
//! Rode com: `cargo run --example onda5_luau`
//!
//! Os dois exemplos compartilham o markup — `app.gv` é o mesmo arquivo, linha
//! por linha, tirando o `<script src>` e o título. O que muda é onde mora o
//! comportamento:
//!
//! ```text
//!                     examples/onda5          examples/onda5_luau
//! estado inicial      Component::init         function init()
//! as três ações       Component::update       uma função global por ação
//! a data de hoje      fn hoje_iso() (20 li.)  date.today()  (uma)
//! as duas listas      const CIDADES/SERVICOS  local CIDADES/SERVICOS
//! registro            motor.register(Box…)    motor.register_component(…)
//! ```
//!
//! # O que o exemplo mostra, e que a versão Rust não mostra sozinha
//!
//! **Que os seis widgets desta onda não pedem script nenhum.** É fácil ler o
//! `update` da versão Rust e supor que ele é o motor da tela; ele não é. Aqui,
//! com o Rust reduzido a este arquivo, sobra a prova: abrir e fechar painéis,
//! trocar a página de uma aba, filtrar sugestões enquanto se digita, navegar a
//! lista com ▲▼, aceitar com Enter, desistir com Esc, deslizar a gaveta e abrir
//! a grade ancorada ao campo continuam funcionando com as **mesmas** três
//! funções de Luau que a versão Rust tem de Rust.
//!
//! **E que a fronteira não é uma linguagem, é uma pergunta.** O que ficou no
//! script é o que nenhum widget podia saber: que dia é hoje, o que significa
//! clicar em "Sair", e para qual aba um atalho da gaveta leva.
//!
//! # A diferença que aparece no arquivo
//!
//! Uma, e ela é o achado desta dupla: **`date.today()` é uma linha.** A versão
//! Rust precisa do algoritmo `civil_from_days` inteiro para semear `hoje`,
//! porque o motor não tem uma crate de data — e não tem de propósito (§4 do
//! `PLANO_WIDGETS.md`: o realce de hoje é *prop, não relógio*). O prelúdio Luau
//! tem. É a mesma observação que o `examples/onda3` registrou, e ela vale para
//! todo o `calendarPopup` desta onda.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 5 (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            // Sem `register(Box::new(...))`: não há `impl Component` neste
            // exemplo. O `.gv` traz o `<script src>`, e o motor cuida do resto.
            if let Err(e) = motor.register_component("onda5_luau", "examples/onda5_luau/app.gv") {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda5_luau");
        })
        .run()
}
