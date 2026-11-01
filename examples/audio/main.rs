//! `<audio>` — Fase 6 do `PLANO_SUBSTITUIBILIDADE.md` (só o áudio; `<video>` não foi feito).
//!
//!   WGPU_BACKEND=gl cargo run --example audio --features audio
//!
//! Precisa da feature `audio` (o `rodio` pede `libasound2-dev` no Linux). Sem ela o player
//! mostra "erro: feature `audio` desligada".

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Painel;

impl Component for Painel {
    fn name(&self) -> &str {
        "audio"
    }
    fn template(&self) -> Template {
        Template::File("examples/audio/app.gv".into())
    }
    fn init(&mut self, ctx: &mut Context) {
        ctx.set("fins", "0".to_string());
    }
    fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
        if action == "terminou" {
            let n: u32 = ctx.get("fins").and_then(|c| c.parse().ok()).unwrap_or(0);
            ctx.set("fins", (n + 1).to_string());
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            let _ = motor.register(Box::new(Painel));
            motor.set_initial_screen("audio");
        })
        .run()
}
