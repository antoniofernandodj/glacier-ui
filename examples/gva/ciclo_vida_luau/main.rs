//! Ciclo de vida de tela em Luau: `on_enter`, `on_leave` e streams com
//! `scope = "screen"`, contra um servidor SSE + WebSocket **real** (uma thread
//! deste processo — ver `servidor.rs`).
//!
//! - `inicio`  : mostra o estado dos dois streams da outra tela.
//! - `monitor` : abre SSE e WS em `on_enter` com `scope = "screen"`; ao voltar,
//!   o motor roda `on_leave` e fecha os dois, entregando `on_close`.
//!
//! O terminal imprime as conexões abrindo/fechando do lado do servidor.
//!
//! Rode com: `cargo run --example ciclo_vida_luau`

mod servidor;

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    let porta = servidor::iniciar();

    GlacierDaemon::new()
        .main(move |motor| {
            // A URL do servidor chega ao script pelo contexto.
            motor.define_data("sse_url", &format!("http://127.0.0.1:{porta}/sse"));
            motor.define_data("ws_url", &format!("ws://127.0.0.1:{porta}/ws"));
            if let Err(e) = motor.register_component("inicio", "examples/gva/ciclo_vida_luau/inicio.gva") {
                eprintln!("Erro ao registrar 'inicio': {e}");
            }
            motor.set_initial_screen("inicio");
        })
        .run()
}
