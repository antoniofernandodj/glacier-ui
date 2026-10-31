//! Painel de referência da Fase 0 — ver `docs/MEDICOES.md`.
//!
//!   (cd examples/gvb/painel && python3 -m http.server 8098) &
//!   WGPU_BACKEND=gl cargo run --release --example painel

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .locales_dir("examples/gvb/painel/locales")
        .default_locale("pt-BR")
        .main_template("examples/gvb/painel/app.gvb")
        .run()
}
