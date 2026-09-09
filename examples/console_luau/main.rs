//! O `console.*` do prelúdio — logs coloridos e decorados para o terminal,
//! filtráveis por nível.
//!
//! Rode com: `cargo run --example console_luau` (com `WGPU_BACKEND=gl` nesta
//! máquina) e **olhe o terminal**: é lá que o `console` escreve. A janela só
//! tem os botões.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — console (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) =
                motor.register_component("console_luau", "examples/console_luau/app.gv")
            {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("console_luau");
        })
        .run()
}
