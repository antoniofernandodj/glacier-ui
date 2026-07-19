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
//! De quebra, a seção "Indicadores" mostra o `<ProgressBar>` (formalizado
//! como primitiva) e o `<Spinner>` (indicador indeterminado/"busy" — gira
//! sozinho, sem precisar de estado no contexto; ver `src/spinner.rs`).
//!
//! `Galeria` também mostra `<script>` Lua coexistindo com um `Component`
//! Rust no mesmo `register()`: `galeria.gv` define `avancar_progresso()` em
//! Lua, que vence sobre o `update()` Rust para essa ação; todo o resto
//! (checkbox, combos, textarea, ...) não tem função Lua e cai no `update()`
//! Rust normalmente — a camada por-ação que `GlacierUI::register` liga
//! automaticamente quando o template carrega `<script>`.
//!
//! Rode com: `cargo run --example galeria_estilos`
use glacier_ui::{Component, Context, GlacierDaemon, Template, style};

struct Galeria;

impl Galeria {
    fn boxed() -> Box<Galeria> {
        Box::new(Galeria)
    }
}

impl Component for Galeria {
    fn name(&self) -> &str {
        "galeria"
    }

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
        // `<ProgressBar>`: valor no range 0..1 (widget clampa, mas mantemos
        // a fonte da verdade já dentro do range configurado no template).
        ctx.set("progresso", "0.3");
        ctx.set("marcado", "true");
        ctx.set("tres_estados", "mixed"); // parcial, como no gallery do Qt
        ctx.set("ligado", "false");
        ctx.set("texto", "");
        ctx.set("senha", "");
        ctx.set(
            "notas",
            "Twinkle, twinkle, little star,\nHow I wonder what you are.",
        );
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // A maioria dos controles liga `on_change`/`on_toggle` direto à sua
        // própria variável de contexto (ramo genérico abaixo); a troca de
        // estilo nem passa por aqui (é a ação builtin `style:set`). O botão
        // do `<ProgressBar>` ("avancar_progresso") tem função Lua homônima em
        // `galeria.gv` — ela vence e este `update()` nunca vê essa ação.
        if let Some(v) = value {
            ctx.set(action, v.to_string());
        }
    }
}

fn main() -> iced::Result {
    let app = GlacierDaemon::new()
        .title("Glacier — Galeria de Estilos")
        .main_size(560.0, 640.0)
        .style(style::FUSION)
        .main(|motor| {
            if let Err(e) = motor.register(Galeria::boxed()) {
                eprintln!("erro ao registrar: {e}");
            }
            motor.set_initial_screen("galeria");
        });

    app.run()
}
