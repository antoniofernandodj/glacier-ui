use xml_ui::{UiEngine, EngineMessage};
use iced::{Element, Task, Color, widget::text};
use std::time::Duration;

struct AppPerfil {
    motor: UiEngine,
    seguindo: bool,
}

impl AppPerfil {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = UiEngine::new();

        // Only the entry component is registered; PerfilCard is pulled in via the
        // <import> declared at the top of perfil.xml.
        if let Err(e) = motor.register_component("perfil", "templates/perfil.xml") {
            eprintln!("Error registering component 'perfil': {}", e);
        }

        // Initialize state variables
        motor.define_data("user_name", "Clara Silva");
        motor.define_data("user_role", "Engenheira de Software Senior");
        motor.define_data("texto_botao", "Seguir");
        motor.define_data("btn_color", "#313244"); // Sleek base button color

        ( Self { motor, seguindo: false }, Task::none())
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        match message {
            EngineMessage::XmlInputChanged { action, value } => {
                match action.as_str() {
                    "mudar_nome" => { self.motor.define_data("user_name", &value); }
                    "mudar_cargo" => { self.motor.define_data("user_role", &value); }
                    _ => {}
                }
            }
            EngineMessage::XmlClick(acao) => {
                match acao.as_str() {
                    "seguir_usuario" => {
                        self.seguindo = !self.seguindo;
                        if self.seguindo {
                            self.motor.define_data("texto_botao", "Seguindo ✓");
                            self.motor.define_data("btn_color", "#A6E3A1"); // Light green for active/following
                        } else {
                            self.motor.define_data("texto_botao", "Seguir");
                            self.motor.define_data("btn_color", "#313244"); // Back to default dark
                        }
                    }
                    "set_dev" => {
                        self.motor.define_data("user_name", "Clara Silva");
                        self.motor.define_data("user_role", "Engenheira de Software Senior");
                    }
                    "set_designer" => {
                        self.motor.define_data("user_name", "Sophia Martins");
                        self.motor.define_data("user_role", "Designer de Interface (UI/UX)");
                    }
                    _ => println!("Action clicked: {}", acao),
                }
            }
            EngineMessage::FileChanged(_) => {
                // Hot reloading check
                let reloaded = self.motor.check_reload();
                if !reloaded.is_empty() {
                    println!("Reloaded components: {:?}", reloaded);
                }
            }
            // Este exemplo não usa navegação entre telas.
            EngineMessage::Navigate(_) | EngineMessage::NavigateBack => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, EngineMessage> {
        match self.motor.render("perfil") {
            Ok(elem) => elem,
            Err(e) => {
                text(format!("Error rendering UI: {}", e))
                    .color(Color::from_rgb(1.0, 0.0, 0.0))
                    .into()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<EngineMessage> {
        UiEngine::reload_subscription(Duration::from_millis(250))
    }
}

fn main() -> iced::Result {
    iced::application("XML UI - Painel de Perfil", AppPerfil::update, AppPerfil::view)
        .subscription(AppPerfil::subscription)
        .run_with(|| AppPerfil::new())
}
