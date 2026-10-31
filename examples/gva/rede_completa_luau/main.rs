//! **Toda a rede do Glacier num exemplo só**: `fetch` (GET/POST, `timeout`, `file://`,
//! `download_file`), cookies por host, `multipart/form-data`, `abort_controller` e o cliente
//! `http()` com `base_url`, cabeçalhos, interceptors, `retries`, `on_error`, `extend`,
//! cookies, multipart e abort. Ver `docs/REDE.md`.
//!
//!   (o exemplo liga e desliga o servidor de teste sozinho)
//!   WGPU_BACKEND=gl cargo run --example rede_completa_luau

use glacier_ui::GlacierDaemon;

#[path = "../../comum/servidor_teste.rs"]
mod servidor_teste;

fn main() -> iced::Result {
    let _servidor = servidor_teste::ligar();
    GlacierDaemon::new()
        .main_template("examples/gva/rede_completa_luau/app.gva")
        .run()
}
