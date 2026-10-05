//! **App com várias telas**, declarado por inteiro no `app.gvb`: a raiz é o
//! `app(...)` e as telas são filhos dele. Escrito **antes** da funcionalidade
//! (TDD) — ver `docs/PLANO_APP_TELAS.md`; hoje o `run` falha ao ler o `app`.
//!
//! O Rust só liga o que o markup não expressa: o `impl Component` que serve a
//! tela `relogio` (declarada no manifesto sem corpo).
//!
//! Rode com: `cargo run --example gvb_app_telas --features tray`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/app_telas/app.gvb")
        // TODO(plano, 1.6): `.main(|motor| motor.register(Box::new(Relogio)))`
        // quando a tela `relogio` ganhar o `impl Component` — o `.main` deixa de
        // ser o dono do template principal e só registra componentes.
        .run()
}
