//! **Onda 13** do `PLANO_WIDGETS.md`: o desenho que o `.gv` escreve.
//!
//! Rode com: `cargo run --example formas` (nesta máquina, com
//! `WGPU_BACKEND=gl`).
//!
//! ```text
//! O habilitador — o vocabulário de formas
//!
//!   <canvas> como pai; sob ele <path>/<arc>/<circle>/<rect>/<line>/
//!   <polyline>/<polygon> (cada um um NodeType::Shape) e <text> (que continua
//!   um NodeType::Text, lido como forma de texto pela posição x/y).
//!
//!   Geometria (cx, d, points) é DADO — inline no .gv, interpola no eval.
//!   Traço/preenchimento (fill, stroke, stroke-width) é ESTILO — classe .gss,
//!   reusando os campos background/border-color/border-width.
//!
//!   A caixa de ferramentas (arcos, béziers) é a mesma do src/canvas.rs da
//!   Onda 7. É a condicional daquela onda executada: "se o Canvas sair, sai
//!   como vocabulário declarativo".
//! ```
//!
//! # QWhatsThis de carona
//!
//! `whats_this="…"` num nó qualquer + a ação `whatsthis:toggle` ligam o modo
//! pegajoso: com ele ligado, pairar mostra a ajuda `whats_this` em vez do
//! `tooltip` transiente.
use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Formas;

impl Component for Formas {
    fn name(&self) -> &str {
        "formas"
    }

    fn template(&self) -> Template {
        Template::File("examples/formas/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // Geometria dirigida por dado: o raio do círculo e um traçado de path.
        ctx.set("raio", "34".to_string());
        ctx.set("onda", "M10 90 Q 40 20 70 90 T 130 90".to_string());
        ctx.set("status", "raio = 34".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            "maior" | "menor" => {
                let r: f32 = ctx.get("raio").and_then(|s| s.parse().ok()).unwrap_or(34.0);
                let novo = if action == "maior" { r + 8.0 } else { r - 8.0 };
                let novo = novo.clamp(8.0, 70.0);
                ctx.set("raio", format!("{novo:.0}"));
                ctx.set("status", format!("raio = {novo:.0}"));
            }
            // O braço genérico: a ação É o nome da chave (o `<textinput>` do
            // traçado).
            chave if value.is_some() => ctx.set(chave, value.unwrap_or_default().to_string()),
            _ => {}
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — Onda 13 (formas)")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Formas)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("formas");
        })
        .run()
}
