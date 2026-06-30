//! Estilos `.gss` COM ESCOPO escritos INLINE num template KDL.
//!
//! Igual a `examples/estilos_inline.rs`, mas o template é KDL
//! (`templates/estilos_inline.kdl`). Como KDL não tem tag de fechamento, o corpo
//! GSS vai numa string multilinha (`""" ... """`) como argumento do nó `style`.
//! O motor seleciona o parser pela extensão `.kdl`; o resto é idêntico.
//!
//! Rode com: `cargo run --example estilos_inline_kdl`

use glacier_ui::{GlacierUI, EngineMessage, Component, Context, Template};
use iced::{Element, Task};
use std::time::Duration;

struct Estilos {
    valor: i32,
}

impl Component for Estilos {
    fn name(&self) -> &str { "estilos_inline_kdl" }

    fn template(&self) -> Template {
        // Basta apontar para um `.kdl`: o motor seleciona o parser pela extensão.
        Template::File("templates/estilos_inline.kdl".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("valor", self.valor.to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            _ => return,
        }
        ctx.set("valor", self.valor.to_string());
    }
}

struct App {
    motor: GlacierUI,
}

impl App {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = GlacierUI::new();
        if let Err(e) = motor.register(Box::new(Estilos { valor: 0 })) {
            eprintln!("Erro ao registrar 'estilos_inline_kdl': {}", e);
        }
        motor.set_initial_screen("estilos_inline_kdl");
        (Self { motor }, Task::none())
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        self.motor.dispatch(&message)
    }

    fn view(&self) -> Element<'_, EngineMessage> {
        self.motor.render_current().unwrap_or_else(|e| {
            iced::widget::text(format!("Erro ao renderizar: {}", e))
                .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                .into()
        })
    }

    fn subscription(&self) -> iced::Subscription<EngineMessage> {
        GlacierUI::reload_subscription(Duration::from_millis(500))
    }

    fn theme(&self) -> iced::Theme {
        self.motor.theme()
    }
}

fn main() -> iced::Result {
    iced::application(|| App::new(), App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Glacier - Estilos inline (KDL)")
        .run()
}
