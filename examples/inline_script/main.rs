//! Teste de realce e de leitura: um `script` e um `style` com o código escrito
//! no próprio `.gv`, sem Rust do app.
//!
//!     WGPU_BACKEND=gl cargo run --example inline_script
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/inline_script/app.gv")
        .run()
}
