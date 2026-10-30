//! **Toda a rede do Glacier, em Luau** (.gvb): o estado vive em `scripts/app.luau`
//! (`fetch`, o cliente `http()`, cookies, multipart e `abort`). Ver `docs/REDE.md`.
//!
//!   python3 tools/servidor_rede.py &
//!   WGPU_BACKEND=gl cargo run --example gvb_rede_completa_luau

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/rede_completa_luau/app.gvb")
        .run()
}
