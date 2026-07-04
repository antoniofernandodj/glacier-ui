//! Contador escrito em KDL em vez de XML.
//!
//! O único ponto que muda em relação ao exemplo `contador` é a extensão do
//! arquivo de template: `examples/contador_kdl/contador.kdl`. O motor detecta a extensão
//! e usa o parser KDL automaticamente — qualquer extensão desconhecida cai no
//! parser XML. O resto (componente, estado, dispatch, hot-reload) é idêntico.
//!
//! Rode com: `cargo run --example contador_kdl`

use glacier_ui::{GlacierUI, EngineMessage, Component, Context, Template};
use iced::{Element, Task, widget::text, Color, Subscription};
use std::time::Duration;

/// Componente que encapsula UI (template KDL) + comportamento + estado.
struct Contador {
    valor: i32,
}

impl Contador {
    fn new() -> Self {
        Self { valor: 0 }
    }
}

impl Component for Contador {
    fn name(&self) -> &str {
        "contador"
    }

    fn template(&self) -> Template {
        // Basta apontar para um `.kdl`: o motor seleciona o parser pela extensão.
        Template::File("examples/contador_kdl/contador.kdl".into())
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
    motor: GlacierUI,
}

impl AppContador {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = GlacierUI::new();
        if let Err(e) = motor.register(Box::new(Contador::new())) {
            eprintln!("Error registering component: {}", e);
        }
        motor.set_initial_screen("contador");

        (Self { motor }, Task::none())
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        self.motor.dispatch(&message)
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
        GlacierUI::reload_subscription(Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application(|| AppContador::new(), AppContador::update, AppContador::view)
        .subscription(AppContador::subscription)
        .title("Glacier - Contador (KDL)")
        .run()
}
