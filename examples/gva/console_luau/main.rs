//! O `console.*` do prelúdio — logs coloridos e decorados para o terminal,
//! filtráveis por nível.
//!
//! Rode com: `cargo run --example console_luau` (com `WGPU_BACKEND=gl` nesta
//! máquina) e **olhe o terminal**: é lá que o `console` escreve. A janela só
//! tem os botões.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gva/console_luau/app.gva").run()
}
