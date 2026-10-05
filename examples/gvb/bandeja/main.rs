//! **Ícone de bandeja** (system tray) + app que **sobrevive à última janela**,
//! tudo declarado no markup: a `<tray>` e o `<script>` moram no `painel.gvb`, e o
//! Rust só diz qual arquivo abre.
//!
//! Com a `<tray>` no `<resources>`, fechar a janela **não encerra** o app: ele
//! recolhe para a bandeja. O menu do ícone controla o ciclo de vida, e o runner
//! trata as três ações sozinho:
//!
//! - **Abrir** (`tray:open`) — reabre (ou foca) a janela principal.
//! - **Notificações** (`notifications:toggle`) — liga/desliga as notificações do
//!   SO; o `<check>` acompanha o estado (`{{__notifications}}`).
//! - **Sair** (`tray:quit`) — encerra o app de vez.
//!
//! No Windows o clique **esquerdo** no ícone também reabre a janela; no Linux não
//! há evento de clique no ícone (o clique abre o menu) — use o item "Abrir".
//!
//! Rode com: `cargo run --example gvb_bandeja --features tray`
//! (sem a feature `tray` a bandeja não sobe e o app encerra na última janela.)

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gvb/bandeja/painel.gvb").run()
}
