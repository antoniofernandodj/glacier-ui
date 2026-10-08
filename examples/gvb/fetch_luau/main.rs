//! Chamadas de rede a partir do `<script>` Lua, sem bloquear a UI.
//!
//! No template `fetch_luau.gvb`, a função `buscar` faz:
//! ```luau
//! local res = fetch("https://api.ipify.org?format=json")
//! ```
//! `fetch` **suspende a corrotina** da ação (via `coroutine.yield`); o motor
//! dispara a requisição HTTP no executor async do iced (hyper + rustls) e, ao
//! receber a resposta, **retoma a corrotina** no ponto do `fetch` com a tabela
//! `{ ok, status, body, error }`. Do lado do Lua, parece `await` — mas a thread
//! de UI nunca trava (o "carregando..." aparece enquanto a rede responde).

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/fetch_luau/fetch_luau.gvb")
        .run()
}
