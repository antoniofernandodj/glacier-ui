//! **Script do app**: o `app.gv` declara um `script` no `resources` do `app(...)`,
//! que semeia `usuario` e `visitas` e expõe `registrar_visita` (chamada do markup
//! como `app::registrar_visita`). As duas telas não têm script — ver
//! `docs/PLANO_SCRIPT_APP.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example app_script`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/app_script/app.gv")
        .run()
}
