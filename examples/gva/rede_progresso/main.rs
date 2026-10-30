//! `on_progress`: uma barra que anda enquanto 50 MB sobem (POST) ou descem (GET).
//!
//!   python3 tools/servidor_rede.py &
//!   WGPU_BACKEND=gl cargo run --example rede_progresso
//!
//! O `fetch(url, { on_progress = function(p) … end })` recebe `{ phase, done, total?, fraction? }`
//! ("upload" ou "download"); o `http()` repassa a mesma opção. Ver `docs/REDE.md`.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gva/rede_progresso/app.gva")
        .run()
}
