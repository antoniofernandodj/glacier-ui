//! i18n em Luau — Fase 7 do `PLANO_SUBSTITUIBILIDADE.md`. Ver `docs/I18N.md`.
//!
//!   WGPU_BACKEND=gl cargo run --example gvb_i18n_luau

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .locales_dir("examples/comum/i18n")
        .default_locale("pt-BR")
        .main_template("examples/gvb/i18n_luau/app.gvb")
        .run()
}
