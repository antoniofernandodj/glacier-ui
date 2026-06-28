use glacier_ui::{GlacierUI, EngineMessage, component};
use iced::{Element, Task, widget::text, Color, Subscription};
use std::time::Duration;

/// UI + comportamento no MESMO arquivo: a markup e os métodos (`incrementar`/
/// `decrementar`) vivem em `templates/contador_macro.xml`, dentro de `<script>`.
///
/// O `#[component]` lê o XML em tempo de compilação, transforma cada `fn` do
/// `<script>` numa ação, e sincroniza o campo `contador` com o contexto — então
/// `{contador}` na markup reflete `self.contador`.
#[component(path = "templates/contador_macro.xml", name = "contador")]
#[derive(Default)]
struct Contador {
    contador: i32,
}

struct App {
    motor: GlacierUI,
}

impl App {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = GlacierUI::new();
        if let Err(e) = motor.register(Box::new(Contador::default())) {
            eprintln!("Erro ao registrar: {}", e);
        }
        motor.set_initial_screen("contador");

        (Self { motor }, Task::none())
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        self.motor.dispatch(&message)
    }

    fn view(&self) -> Element<'_, EngineMessage> {
        self.motor.render_current().unwrap_or_else(|e| {
            text(format!("Erro ao renderizar: {}", e))
                .color(Color::from_rgb(1.0, 0.0, 0.0))
                .into()
        })
    }

    fn subscription(&self) -> Subscription<EngineMessage> {
        GlacierUI::reload_subscription(Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application("Glacier - Contador (script)", App::update, App::view)
        .subscription(App::subscription)
        .run_with(|| App::new())
}
