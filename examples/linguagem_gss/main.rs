//! O `.gss` da 0.169: aninhamento com `&`, `:not()`, `:is()`, `font_weight`,
//! `font_style`, `line_height`, `box_shadow`, `@theme`, modo estrito e a cor do CSS
//! (`gss_color = css`). Ver `docs/LINGUAGEM.md` e `docs/GSS_MODERNO.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example linguagem_gss`
//! (a sombra pode acumular em drivers Vulkan incompletos — ver `TROUBLESHOOTING.md`).

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/linguagem_gss/app.gv")
        .run()
}
