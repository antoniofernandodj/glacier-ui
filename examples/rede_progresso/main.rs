//! `on_progress`: uma barra que anda enquanto 50 MB sobem (POST) ou descem (GET).
//!
//!   (o exemplo liga e desliga o servidor de teste sozinho)
//!   WGPU_BACKEND=gl cargo run --example rede_progresso
//!
//! O `fetch(url, { on_progress = function(p) … end })` recebe `{ phase, done, total?, fraction? }`
//! ("upload" ou "download"); o `http()` repassa a mesma opção. Ver `docs/REDE.md`.

use glacier_ui::GlacierDaemon;

#[path = "../rede_completa/servidor_teste.rs"]
mod servidor_teste;

fn main() -> iced::Result {
    let _servidor = servidor_teste::ligar();
    GlacierDaemon::new()
        .main_template("examples/rede_progresso/app.gv")
        .run()
}
