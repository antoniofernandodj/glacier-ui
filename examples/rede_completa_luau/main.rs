//! **Toda a rede do Glacier, em Luau** (.gv): o estado vive em `scripts/app.luau`
//! (`fetch`, o cliente `http()`, cookies, multipart e `abort`). Ver `docs/REDE.md`.
//!
//!   (o exemplo liga e desliga o servidor de teste sozinho)
//!   WGPU_BACKEND=gl cargo run --example rede_completa_luau

use glacier_ui::GlacierDaemon;

#[path = "../rede_completa/servidor_teste.rs"]
mod servidor_teste;

fn main() -> iced::Result {
    let _servidor = servidor_teste::ligar();
    GlacierDaemon::new()
        .main_template("examples/rede_completa_luau/app.gv")
        .run()
}
