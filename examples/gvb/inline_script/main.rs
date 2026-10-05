//! Teste de realce e de leitura: um `script` e um `style` com o código escrito
//! no próprio `.gvb`, sem Rust do app.
//!
//!     WGPU_BACKEND=gl cargo run --example gvb_inline_script
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/inline_script/app.gvb")
        .run()
}
