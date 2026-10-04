//! Abre qualquer template numa janela — `.gvb`, `.gva` ou `.gv`.
//!
//!     WGPU_BACKEND=gl cargo run --example gvb_run -- examples/gvb/onda9_luau/app.gvb
//!
//! Serve para os exemplos que não dependem de código Rust do app: os `*_luau`,
//! os `*_lua` e as telas estáticas. Um exemplo cujo `main.rs` registra um
//! `Component` com `update` em Rust (o `contador`, o `onda9`) continua precisando
//! do `main.rs` dele — as ações dele não existem aqui.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    let Some(caminho) = std::env::args().nth(1) else {
        eprintln!("uso: cargo run --example gvb_run -- <arquivo.gvb|.gva|.gv>");
        std::process::exit(2);
    };
    GlacierDaemon::new()
        .title("Glacier — gvb_run")
        .main_template(caminho)
        .run()
}
