//! **Toda a rede do Glacier num exemplo só**: `fetch` (GET/POST, `timeout`, `file://`,
//! `download_file`), cookies por host, `multipart/form-data`, `abort_controller` e o cliente
//! `http()` com `base_url`, cabeçalhos, interceptors, `retries`, `on_error`, `extend`,
//! cookies, multipart e abort. Ver `docs/REDE.md`.
//!
//!   python3 tools/servidor_rede.py &
//!   WGPU_BACKEND=gl cargo run --example rede_completa

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gva/rede_completa/app.gva")
        .run()
}
