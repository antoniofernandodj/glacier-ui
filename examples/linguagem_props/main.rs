//! Props com tipo: `prop(name = x, type = bool | number | string | list | …)`.
//! Um literal do tipo errado no uso do componente é erro ao carregar a tela; um
//! valor do contexto do tipo errado só avisa. Ver `docs/LINGUAGEM.md`, "Props".
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example linguagem_props`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/linguagem_props/app.gv")
        .run()
}
