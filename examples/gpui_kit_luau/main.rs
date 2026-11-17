//! **Fase gpui-kit, em Luau**: os widgets que a comparação com o
//! [gpui-kit](https://gpui-kit.com) trouxe a partir da 0.161 — `alert`, `kbd`,
//! `descriptionlist`, `command`, `field`, `dropdownbutton` e `inputgroup` — com
//! o estado no script.
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
//! - `dropdownbutton` (0.163): "Deploy" é o split button (a ação principal +
//!   o "▾" com as alternativas do JSON e do markup); "Ver" abre o menu inteiro.
//! - `field` + `inputgroup` (0.162/0.163): o domínio com `https://` colado e um
//!   "Copiar" dentro da mesma moldura; o script valida e o `field` mostra o erro.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gpui_kit_luau/app.gv")
        .run()
}
