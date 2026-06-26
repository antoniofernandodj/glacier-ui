use xml_ui::{UiEngine, EngineMessage};
use iced::{Element, Task, widget::text, Color, Subscription};
use std::time::Duration;

struct AppContador {
    motor: UiEngine,
    contador: i32,
}

impl AppContador {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = UiEngine::new();
        if let Err(e) = motor.register_component("contador", "templates/contador.xml") {
            eprintln!("Error registering component: {}", e);
        }
        
        let contador = 0;
        motor.define_data("contador", &contador.to_string());

        ( Self { motor, contador }, Task::none() )
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        match message {
            EngineMessage::XmlClick(acao) => {
                match acao.as_str() {
                    "incrementar" => {
                        self.contador += 1;
                        self.motor.define_data("contador", &self.contador.to_string());
                    }
                    "decrementar" => {
                        self.contador -= 1;
                        self.motor.define_data("contador", &self.contador.to_string());
                    }
                    _ => {}
                }
            }
            EngineMessage::FileChanged(_) => {
                // Check if any template files changed and reload them
                let reloaded = self.motor.check_reload();
                if !reloaded.is_empty() {
                    println!("Reloaded components: {:?}", reloaded);
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, EngineMessage> {
        match self.motor.render("contador") {
            Ok(elem) => elem,
            Err(e) => {
                text(format!("Error rendering UI: {}", e))
                    .color(Color::from_rgb(1.0, 0.0, 0.0))
                    .into()
            }
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
