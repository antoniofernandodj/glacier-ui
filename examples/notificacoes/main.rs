//! Notificações NATIVAS DO SISTEMA OPERACIONAL: a função global `notify` na
//! camada Luau (ver o prelúdio) e `Context::notify` na API Rust. Diferente do
//! exemplo `toasts` — que desenha o aviso dentro da própria janela —, estas são
//! entregues à central de notificações do SO (freedesktop/D-Bus no Linux, WinRT
//! no Windows, NSUserNotification no macOS) e aparecem mesmo com o app
//! minimizado ou em outro workspace.
//!
//! Toda a lógica está em `notificacoes.luau`; este `main.rs` só registra o
//! componente.
//!
//! Rode com: `cargo run --example notificacoes`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - notificações do SO")
        .main(|motor| {
            if let Err(e) =
                motor.register_component("notificacoes", "examples/notificacoes/notificacoes.gv")
            {
                eprintln!("Erro ao registrar: {}", e);
            }
            motor.set_initial_screen("notificacoes");
        })
        .run()
}
