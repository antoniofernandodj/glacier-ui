//! `opacity`, `transition`, `calc()` e `var()` em campo numérico no `.gss` — Fase 5 do
//! `PLANO_SUBSTITUIBILIDADE.md`. Ver `docs/GSS_MODERNO.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example css_moderno`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gva/css_moderno/app.gva")
        .run()
}
