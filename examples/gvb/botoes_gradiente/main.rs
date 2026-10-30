use glacier_ui::{Component, Context, GlacierDaemon, Template};

/// Botões grandes e largos com gradiente, só com `.gss`: um `button` com
/// `gradient: <ângulo> #a #b` e `border_radius`.
struct Botoes;

impl Component for Botoes {
    fn name(&self) -> &str {
        "botoes"
    }

    fn template(&self) -> Template {
        Template::File("examples/gvb/botoes_gradiente/botoes.gvb".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("ultimo", "nenhum");
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        ctx.set("ultimo", action);
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Botoes)) {
                eprintln!("Erro ao registrar 'botoes': {}", e);
            }
            motor.set_initial_screen("botoes");
        })
        .run()
}
