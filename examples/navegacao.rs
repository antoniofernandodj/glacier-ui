use xml_ui::{UiEngine, EngineMessage};
use iced::{Element, Task};
use std::time::Duration;

/// Demonstra navegação entre telas: cada tela é um componente registrado,
/// e os botões declaram o destino no próprio XML via `navigateTo`/`navigateBack`.
/// O estado (`user_name`) é compartilhado entre todas as telas.
struct AppNav {
    motor: UiEngine,
}

impl AppNav {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = UiEngine::new();

        for (nome, caminho) in [
            ("home", "templates/nav_home.xml"),
            ("perfil", "templates/nav_perfil.xml"),
            ("config", "templates/nav_config.xml"),
        ] {
            if let Err(e) = motor.register_component(nome, caminho) {
                eprintln!("Erro ao registrar '{}': {}", nome, e);
            }
        }

        // Estado compartilhado entre as telas.
        motor.define_data("user_name", "Clara Silva");
        motor.define_data("user_role", "Engenheira de Software");

        // Tela inicial.
        motor.set_initial_screen("home");

        (Self { motor }, Task::none())
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        match message {
            EngineMessage::Navigate(destino) => self.motor.navigate_to(&destino),
            EngineMessage::NavigateBack => self.motor.navigate_back(),
            EngineMessage::XmlInputChanged { action, value } => {
                if action == "mudar_nome" {
                    self.motor.define_data("user_name", &value);
                }
            }
            EngineMessage::XmlClick(_) => {}
            EngineMessage::FileChanged(_) => {
                let _ = self.motor.check_reload();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, EngineMessage> {
        self.motor.render_current().unwrap_or_else(|e| {
            iced::widget::text(format!("Erro ao render: {}", e))
                .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                .into()
        })
    }

    fn subscription(&self) -> iced::Subscription<EngineMessage> {
        UiEngine::reload_subscription(Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application("XML UI - Navegação", AppNav::update, AppNav::view)
        .subscription(AppNav::subscription)
        .run_with(|| AppNav::new())
}
