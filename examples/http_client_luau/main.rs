//! O cliente `http(...)` do prelúdio — um wrapper "orientado a objeto" sobre o
//! `fetch`, com base_url, headers e timeout de base, interceptors e error
//! handlers.
//!
//! Rode com: `cargo run --example http_client_luau` (com `WGPU_BACKEND=gl` nesta
//! máquina). Precisa de rede — bate em `jsonplaceholder.typicode.com`.
//!
//! Tudo o que este exemplo mostra é MARKUP + `scripts/app.luau`. Nenhuma linha
//! de Rust além do bootstrap: o `http` é global do prelúdio, disponível em
//! qualquer `<script>`.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — http client (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) = motor
                .register_component("http_client_luau", "examples/http_client_luau/app.gv")
            {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("http_client_luau");
        })
        .run()
}
