use xml_ui::{UiEngine, EngineMessage, Component, Context, Template};
use iced::{Element, Task, widget::text, Color, Subscription};
use std::time::Duration;

/// Componente que encapsula UI (template XML) + comportamento + estado.
struct Contador {
    valor: i32,
}

impl Component for Contador {
    fn name(&self) -> &str {
        "contador"
    }

    fn template(&self) -> Template {
        Template::File("templates/contador.xml".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("contador", self.valor.to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            _ => return,
        }
        ctx.set("contador", self.valor.to_string());
    }
}

struct AppContador {
    motor: UiEngine,
}

impl AppContador {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = UiEngine::new();
        if let Err(e) = motor.register(Box::new(Contador { valor: 0 })) {
            eprintln!("Error registering component: {}", e);
        }
        motor.set_initial_screen("contador");

        (Self { motor }, Task::none())
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        if let Err(e) = self.motor.dispatch(&message) {
            eprintln!("Error dispatching message: {}", e);
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, EngineMessage> {
        match self.motor.render_current() {
            Ok(elem) => elem,
            Err(e) => text(format!("Error rendering UI: {}", e))
                .color(Color::from_rgb(1.0, 0.0, 0.0))
                .into(),
        }
    }

    fn subscription(&self) -> Subscription<EngineMessage> {
        UiEngine::reload_subscription(Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application("XML UI - Contador", AppContador::update, AppContador::view)
        .subscription(AppContador::subscription)
        .run_with(|| AppContador::new())
}
