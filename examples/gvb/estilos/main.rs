use glacier_ui::{Component, Context, GlacierDaemon, Template};

/// Demonstra stylesheets `.gss` e `<link>` no template:
/// - GLOBAL: `examples/gvb/estilos/app.gss`, carregada via `motor.load_stylesheet(...)`,
///   vale para todos os componentes (card, title, subtitle, stack, actions).
/// - ESCOPADA: `<link rel="stylesheet" href="examples/gvb/estilos/estilos.gss">`, válida só
///   neste componente (as classes `.btn*`).
/// - TEMA: `<link rel="theme" href="examples/gvb/estilos/theme.json">` define a paleta do
///   `iced`; o `GlacierDaemon` aplica o tema da janela automaticamente (via
///   `motor.theme()`), sem wiring manual.
///
/// (`<link>` também aceita `rel="import"`/`"component"` e `rel="data"`.)
/// Atributos inline no nó sempre vencem a classe. Todos os arquivos têm
/// hot-reload: edite-os com a app rodando.
struct Estilos;

impl Component for Estilos {
    fn name(&self) -> &str {
        "estilos"
    }

    fn template(&self) -> Template {
        Template::File("examples/gvb/estilos/estilos.gvb".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("valor", "0");
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        let atual: i32 = ctx
            .get("valor")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);

        match action {
            "incrementar" => ctx.set("valor", (atual + 1).to_string()),
            "decrementar" => ctx.set("valor", (atual - 1).to_string()),
            _ => {}
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Estilos)) {
                eprintln!("Erro ao registrar 'estilos': {}", e);
            }
            motor.set_initial_screen("estilos");
        })
        .run()
}
