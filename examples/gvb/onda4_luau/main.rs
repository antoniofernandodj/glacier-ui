//! **Onda 4, em Luau**: a mesma tela do `examples/gvb/onda4`, sem um `Component`
//! em Rust.
//!
//! Rode com: `cargo run --example gvb_onda4_luau`
//!
//! Os dois exemplos compartilham o markup — `app.gvb` é o mesmo arquivo, linha
//! por linha, tirando o `<script src>` e o título. O que muda é onde mora o
//! comportamento:
//!
//! ```text
//!                     examples/gvb/onda4          examples/gvb/onda4_luau
//! estado inicial      Component::init         function init()
//! as cinco ações      Component::update       uma função global por ação
//! recorte da página   fn recortar(&self, …)   local function recortar()
//! a lista completa    const SERVICOS          local SERVICOS
//! registro            motor.register(Box…)    motor.register_component(…)
//! ```
//!
//! # O que o exemplo mostra, e que a versão Rust não mostra sozinha
//!
//! **Que os sete widgets não pedem script nenhum.** É fácil ler o `update` da
//! versão Rust e supor que ele é o motor da tela; ele não é. Aqui, com o Rust
//! reduzido a este arquivo, sobra a prova: paginação, seleção simples e
//! múltipla, seções, estrelas, máscara e o clamp com duas casas continuam
//! funcionando com as **mesmas** quatro linhas de Luau que a versão Rust tem de
//! Rust. Tudo o que os widgets fazem, eles fazem escrevendo nas chaves que o
//! app nomeou.
//!
//! **E que a fronteira não é uma linguagem, é uma pergunta.** O que ficou no
//! script é o que nenhum widget podia saber: quantos itens cabem numa página, e
//! o que fazer quando alguém clica em "Excluir".
//!
//! # A diferença que aparece no arquivo
//!
//! O `Onda4` de Rust é um struct, e podia guardar estado em `self`. Um script
//! não tem `self`: o que sobrevive entre ações mora no `ctx` (o que a tela
//! precisa ver) ou num `local` do módulo (o que ela não precisa). A lista
//! completa de serviços é o segundo caso — ela nunca entra no contexto, só a
//! página recortada.
//!
//! A outra: `ctx` guarda texto, e uma tabela atribuída a uma chave vira JSON —
//! que é o formato que um `for-each` lê. `ctx.servicos = { … }` faz o que
//! `serde_json::Value::Array(itens).to_string()` faz do outro lado.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/onda4_luau/app.gvb")
        .run()
}
