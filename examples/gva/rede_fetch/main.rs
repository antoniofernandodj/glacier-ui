//! A rede do `fetch`: cookies por host, multipart e cancelamento. Ver `docs/REDE.md`.
//!
//!   python3 tools/servidor_rede.py &
//!   WGPU_BACKEND=gl cargo run --example rede_fetch

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gva/rede_fetch/app.gva")
        .run()
}
