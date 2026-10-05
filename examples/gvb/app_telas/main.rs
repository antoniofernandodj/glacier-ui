//! **App com várias telas**, declarado por inteiro no `app.gvb`: a raiz é o
//! `app(...)` e as telas são filhos dele. Escrito **antes** da funcionalidade
//! (TDD) — ver `docs/PLANO_APP_TELAS.md`; hoje o `run` falha ao ler o `app`.
//!
//! O Rust só liga o que o markup não expressa: o `impl Component` que serve a
//! tela `relogio` (declarada no manifesto sem corpo e sem `src`).
//!
//! Rode com: `cargo run --example gvb_app_telas --features tray`

use std::time::{SystemTime, UNIX_EPOCH};

use glacier_ui::{Component, Context, GlacierDaemon, Template};

/// Tela `relogio`: o nome é o que o manifesto declara em `screen(name = relogio)`.
struct Relogio;

impl Relogio {
    /// `HH:MM:SS` em UTC. Sem dependência de fuso: o exemplo é sobre telas.
    fn agora() -> String {
        let s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            % 86_400;
        format!("{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
    }
}

impl Component for Relogio {
    fn name(&self) -> &str {
        "relogio"
    }

    fn template(&self) -> Template {
        Template::File("examples/gvb/app_telas/relogio.gvb".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("hora", Self::agora());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        if action == "atualizar" {
            ctx.set("hora", Self::agora());
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main_template("examples/gvb/app_telas/app.gvb")
        // Plano, 1.6: o `.main` deixa de ser o dono do template principal (quem
        // o escolhe é o `main_template`/o padrão) e só registra o que tem lógica
        // em Rust. O nome registrado casa com o `screen(name = relogio)` do
        // manifesto; sem registro, o app não sobe.
        .main(|motor| {
            if let Err(erro) = motor.register(Box::new(Relogio)) {
                eprintln!("{erro}");
            }
        })
        .run()
}
