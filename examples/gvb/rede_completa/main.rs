//! **Toda a rede do Glacier, em Rust** (.gvb): o estado vive num `Component` que usa a API
//! pública `glacier_ui::net` — a mesma camada de rede (jar de cookies, multipart, cancelamento)
//! que o `fetch` do Luau. Ver `docs/REDE.md` e `examples/comum/rede_rust.rs`.
//!
//!   (o exemplo liga e desliga o servidor de teste sozinho)
//!   WGPU_BACKEND=gl cargo run --example gvb_rede_completa

#[path = "../../comum/rede_rust.rs"]
mod rede_rust;

use glacier_ui::GlacierDaemon;

#[path = "../../comum/servidor_teste.rs"]
mod servidor_teste;

fn main() -> iced::Result {
    let _servidor = servidor_teste::ligar();
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(rede_rust::RedeCompleta::novo("examples/gvb/rede_completa/app.gvb"))) {
                eprintln!("{e}");
            }
            motor.set_initial_screen("rede_completa");
        })
        .run()
}
