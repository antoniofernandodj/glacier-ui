//! Streams de vida longa (SSE e WebSocket) a partir do `<script>` Lua.
//!
//! No template `stream_lua.gvb`:
//! ```luau
//! sse_conn = sse("https://sse.dev/test", { on_message = "sse_recebeu" })
//! ws_conn  = websocket("wss://echo.websocket.org", { on_message = "ws_recebeu" })
//! ws_conn:send("ping")   -- envia pela conexão viva
//! ```
//! Ao contrário do `fetch` (one-shot, que suspende a corrotina), `sse` e
//! `websocket` NÃO bloqueiam: registram o stream e retornam um handle na hora.
//! Cada evento que chega da rede chama de volta o handler nomeado em `opts`
//! (`on_message`, `on_open`, `on_error`, `on_close`), que escreve em `ctx` e a
//! UI reavalia — como qualquer ação.
//!
//! Os streams viram `iced::Subscription`s por janela; o `GlacierDaemon` já as
//! liga automaticamente (junto do hot-reload), sem wiring manual.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    // Por padrão usa os endpoints públicos; aponte para a API local com
    //   GLACIER_STREAM_TEMPLATE=examples/gvb/stream_lua/stream_local.gvb
    let template = std::env::var("GLACIER_STREAM_TEMPLATE")
        .unwrap_or_else(|_| "examples/gvb/stream_lua/stream_lua.gvb".to_string());
    GlacierDaemon::new().main_template(template).run()
}
