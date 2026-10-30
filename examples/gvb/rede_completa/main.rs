//! **Toda a rede do Glacier, em Rust** (.gvb): o estado vive num `Component` que usa a API
//! pública `glacier_ui::net` — a mesma camada de rede (jar de cookies, multipart, cancelamento)
//! que o `fetch` do Luau. Ver `docs/REDE.md` e `examples/comum/rede_rust.rs`.
//!
//!   python3 tools/servidor_rede.py &
//!   WGPU_BACKEND=gl cargo run --example gvb_rede_completa

#[path = "../../comum/rede_rust.rs"]
mod rede_rust;

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(rede_rust::RedeCompleta::novo("examples/gvb/rede_completa/app.gvb"))) {
                eprintln!("{e}");
            }
            motor.set_initial_screen("rede_completa");
        })
        .run()
}
