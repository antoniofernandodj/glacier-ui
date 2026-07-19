//! Galeria de widgets no espírito do "Widget Gallery" do Qt, para demonstrar os
//! **estilos builtin** (`glacier_ui::style`):
//!
//! - `GlacierDaemon::style(style::FUSION)` define o estilo default do app —
//!   o análogo do `QApplication::setStyle` (paleta + regras de tag, aplicadas
//!   a todas as janelas, abaixo de qualquer `.gss` do próprio app);
//! - o combo "Style:" troca o estilo em runtime via a ação builtin
//!   `onChange="style:set"` (nenhum código de componente envolvido); um botão
//!   com `on_click="style:<nome>"` faria o mesmo;
//! - o nome do estilo ativo fica no contexto em `glacier_style`, que é o
//!   `value` que o próprio `<Select>` exibe.
//!
//! Rode com: `cargo run --example galeria_estilos`
use glacier_ui::{style, Component, Context, GlacierDaemon, Template};

struct Galeria;

impl Component for Galeria {
    fn name(&self) -> &str { "galeria" }

    fn template(&self) -> Template {
        Template::File("examples/galeria_estilos/galeria.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // Opções do combo "Style:" — os nomes dos quatro estilos embutidos.
        let nomes: Vec<String> = style::all()
            .iter()
            .map(|s| format!("\"{}\"", s.name))
            .collect();

        ctx.set("estilos", format!("[{}]", nomes.join(",")));
        ctx.set("marcado", "true");
        ctx.set("tres_estados", "mixed"); // parcial, como no gallery do Qt
        ctx.set("ligado", "false");
        ctx.set("texto", "");
        ctx.set("senha", "");
        ctx.set("notas", "Twinkle, twinkle, little star,\nHow I wonder what you are.");
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // Todos os controles ligam `onChange`/`onToggle` direto à sua própria
        // variável de contexto; a troca de estilo nem passa por aqui (é a ação
        // builtin `style:set`).
        if let Some(v) = value {
            ctx.set(action, v.to_string());
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — Galeria de Estilos")
        .main_size(560.0, 560.0)
        .style(style::FUSION)
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Galeria)) {
                eprintln!("erro ao registrar: {e}");
            }
            motor.set_initial_screen("galeria");
        })
        .run()
}
