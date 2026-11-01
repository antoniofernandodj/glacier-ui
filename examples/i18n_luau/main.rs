//! i18n em Luau — Fase 7 do `PLANO_SUBSTITUIBILIDADE.md`. Ver `docs/I18N.md`.
//!
//!   WGPU_BACKEND=gl cargo run --example i18n_luau

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .locales_dir("examples/i18n/locales")
        .default_locale("pt-BR")
        .main_template("examples/i18n_luau/app.gv")
        .run()
}
