//! **Fase gpui-kit, em Luau**: os quatro widgets que a comparação com o
//! [gpui-kit](https://gpui-kit.com) trouxe na 0.161 — `alert`, `kbd`,
//! `descriptionlist` e `command` — com o estado no script.
//!
//! Rode com: `cargo run --example gpui_kit_luau` (com `WGPU_BACKEND=gl` nesta
//! máquina).
//!
//! # O que este exemplo mostra
//!
//! - `command`: a paleta abre com **Ctrl+K** (um `shortcut` no layout), filtra
//!   enquanto se digita (o widget grava a chave sozinho) e entrega o `id`
//!   escolhido a `executar(id)`. Esc fecha.
//! - `alert`: um aviso que **fica** — o script decide a variante e o texto, e
//!   o "×" chama `fechar_aviso`, que apaga a chave que o mantinha visível.
//! - `descriptionlist`: a ficha do serviço selecionado, vinda de um array
//!   JSON que o script monta.
//! - `kbd`: os atalhos desenhados, no topo e dentro de cada item da paleta.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gpui_kit_luau/app.gv")
        .run()
}
