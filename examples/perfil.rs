use xml_ui::{MotorUI, MensagemMotor};
use iced::{Element, Task};
use std::time::Duration;

struct AppPerfil {
    motor: MotorUI,
    seguindo: bool,
}

impl AppPerfil {
    fn new() -> (Self, Task<MensagemMotor>) {
        let mut motor = MotorUI::new();
        
        // Register parent dashboard and child profile card components
        if let Err(e) = motor.registrar_componente("perfil", "templates/perfil.xml") {
            eprintln!("Error registering component 'perfil': {}", e);
        }
        if let Err(e) = motor.registrar_componente("PerfilCard", "templates/perfil_card.xml") {
            eprintln!("Error registering component 'PerfilCard': {}", e);
        }

        // Initialize state variables
        motor.definir_dado("user_name", "Clara Silva");
        motor.definir_dado("user_role", "Engenheira de Software Senior");
        motor.definir_dado("texto_botao", "Seguir");
        motor.definir_dado("btn_color", "#313244"); // Sleek base button color

        (
            Self { motor, seguindo: false },
            Task::none(),
        )
    }

    fn update(&mut self, message: MensagemMotor) -> Task<MensagemMotor> {
        match message {
            MensagemMotor::XmlInputChanged { action, value } => {
                match action.as_str() {
                    "mudar_nome" => {
                        self.motor.definir_dado("user_name", &value);
                    }
                    "mudar_cargo" => {
                        self.motor.definir_dado("user_role", &value);
                    }
                    _ => {}
                }
            }
            MensagemMotor::XmlClick(acao) => {
                match acao.as_str() {
                    "seguir_usuario" => {
                        self.seguindo = !self.seguindo;
                        if self.seguindo {
                            self.motor.definir_dado("texto_botao", "Seguindo ✓");
                            self.motor.definir_dado("btn_color", "#A6E3A1"); // Light green for active/following
                        } else {
                            self.motor.definir_dado("texto_botao", "Seguir");
                            self.motor.definir_dado("btn_color", "#313244"); // Back to default dark
                        }
                    }
                    "set_dev" => {
                        self.motor.definir_dado("user_name", "Clara Silva");
                        self.motor.definir_dado("user_role", "Engenheira de Software Senior");
                    }
                    "set_designer" => {
                        self.motor.definir_dado("user_name", "Sophia Martins");
                        self.motor.definir_dado("user_role", "Designer de Interface (UI/UX)");
                    }
                    _ => println!("Action clicked: {}", acao),
                }
            }
            MensagemMotor::FileChanged(_) => {
                // Hot reloading check
                let reloaded = self.motor.verificar_recarregamento();
                if !reloaded.is_empty() {
                    println!("Reloaded components: {:?}", reloaded);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, MensagemMotor> {
        match self.motor.renderizar("perfil") {
            Ok(elem) => elem,
            Err(e) => {
                iced::widget::text(format!("Error rendering UI: {}", e))
                    .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                    .into()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<MensagemMotor> {
        MotorUI::subscricao_recarregamento(Duration::from_millis(250))
    }
}

fn main() -> iced::Result {
    iced::application("XML UI - Painel de Perfil", AppPerfil::update, AppPerfil::view)
        .subscription(AppPerfil::subscription)
        .run_with(|| AppPerfil::new())
}
