//! Demonstra as lacunas fechadas em `PLANO_LUAU_ROBUSTEZ.md`: temporizadores
//! (`after`/`cancel`), persistência local (`storage`), leitura de viewport,
//! tabelas aceitas em `ctx` (serializadas via `json.encode`) e erros de
//! script visíveis ao usuário (`on_error`, com fallback automático em toast).
//!
//! Toda a lógica está em `robustez.luau` — este `main.rs` só registra o
//! componente. Hot-reload e expiração de toasts já vêm ligados pelo
//! `GlacierDaemon`.
//!
//! Rode com: `cargo run --example gvb_robustez_luau`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/robustez_luau/robustez.gvb")
        .run()
}
