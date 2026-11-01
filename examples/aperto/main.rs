use glacier_ui::{Component, Context, GlacierDaemon, Template};

/// Componente que encapsula UI (template XML) + comportamento + estado.
struct Aperto {
    valor: i32,
}

impl Aperto {
    fn new() -> Self {
        Self { valor: 0 }
    }
}

impl Component for Aperto {
    fn name(&self) -> &str {
        "aperto"
    }

    fn template(&self) -> Template {
        Template::File("examples/aperto/aperto.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("v", String::new());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            _ => return,
        }
        ctx.set("v", String::new());
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Aperto::new())) {
                eprintln!("Error registering component: {}", e);
            }
            motor.set_initial_screen("aperto");
        })
        .run()
}
