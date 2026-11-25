//! Botão ocupado automático: com `app(busy_buttons = true)`, o botão fica
//! desabilitado enquanto o handler do próprio `on_click` está suspenso (num
//! `confirm`, num `fetch`), sem `disabled = …` à mão. Ver `docs/LINGUAGEM.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example linguagem_ocupado`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/linguagem_ocupado/app.gv")
        .run()
}
