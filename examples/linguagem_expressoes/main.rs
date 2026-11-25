//! A linguagem de expressão do `.gv`: `&&`, `||`, `!`, comparação numérica, `in`,
//! `is empty`, ternário em atributo e em texto, `class:nome = expr`, a forma curta
//! `each = @lista as v`, o `else` como atributo e a ação com argumentos.
//! Ver `docs/LINGUAGEM.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example linguagem_expressoes`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/linguagem_expressoes/app.gv")
        .run()
}
