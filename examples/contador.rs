use xml_ui::{MotorUI, MensagemMotor};
use iced::{Element, Task};
use std::time::Duration;

struct AppContador {
    motor: MotorUI,
    contador: i32,
}

impl AppContador {
    fn new() -> (Self, Task<MensagemMotor>) {
        let mut motor = MotorUI::new();
        if let Err(e) = motor.registrar_componente("contador", "templates/contador.xml") {
            eprintln!("Error registering component: {}", e);
        }
        
        let contador = 0;
        motor.definir_dado("contador", &contador.to_string());

        (
            Self { motor, contador },
            Task::none(),
        )
    }

    fn update(&mut self, message: MensagemMotor) -> Task<MensagemMotor> {
        match message {
            MensagemMotor::XmlClick(acao) => {
                match acao.as_str() {
                    "incrementar" => {
                        self.contador += 1;
                        self.motor.definir_dado("contador", &self.contador.to_string());
                    }
                    "decrementar" => {
                        self.contador -= 1;
                        self.motor.definir_dado("contador", &self.contador.to_string());
                    }
                    _ => {}
                }
            }
            MensagemMotor::FileChanged(_) => {
                // Check if any template files changed and reload them
                let reloaded = self.motor.verificar_recarregamento();
                if !reloaded.is_empty() {
                    println!("Reloaded components: {:?}", reloaded);
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, MensagemMotor> {
        match self.motor.renderizar("contador") {
            Ok(elem) => elem,
            Err(e) => {
                iced::widget::text(format!("Error rendering UI: {}", e))
                    .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                    .into()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<MensagemMotor> {
        MotorUI::subscricao_recarregamento(Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application("XML UI - Contador", AppContador::update, AppContador::view)
        .subscription(AppContador::subscription)
        .run_with(|| AppContador::new())
}
