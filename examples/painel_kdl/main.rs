//! Painel de equipe em KDL com `import` de componente e folha `.gss` escopada.
//!
//! Demonstra dois recursos declarados no topo do template KDL
//! (`examples/painel_kdl/painel_kdl.kdl`):
//!
//!   - `import "CartaoKdl" from="examples/painel_kdl/cartao_kdl.kdl"` — importa outro
//!     componente, que também é KDL. O motor escolhe o parser pela extensão de
//!     cada arquivo, então um `.kdl` pode importar outro `.kdl` livremente.
//!   - `style "examples/painel_kdl/painel_kdl.gss"` — carrega uma stylesheet COM ESCOPO:
//!     as classes (`.painel`, `.titulo`, `.lista`) valem só no que ESTE
//!     componente renderiza diretamente, por cima de quaisquer globais. O
//!     CartaoKdl, por ser importado, é avaliado no seu próprio escopo e se
//!     estiliza por atributos inline.
//!
//! Não há estado nem ações: o componente só monta a árvore declarativa. Tudo
//! tem hot-reload — edite o `.kdl` ou o `.gss` com a app rodando.
//!
//! Rode com: `cargo run --example painel_kdl`

use glacier_ui::{GlacierUI, EngineMessage, Component, Context, Template};
use iced::{Element, Task, widget::text, Color, Subscription};
use std::time::Duration;

/// Componente de entrada. O `import` e o `style` ficam no próprio template, então
/// aqui não é preciso registrar o CartaoKdl nem carregar a stylesheet à mão.
struct Painel;

impl Component for Painel {
    fn name(&self) -> &str {
        "painel_kdl"
    }

    fn template(&self) -> Template {
        Template::File("examples/painel_kdl/painel_kdl.kdl".into())
    }

    fn init(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {}
}

struct AppPainel {
    motor: GlacierUI,
}

impl AppPainel {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = GlacierUI::new();
        if let Err(e) = motor.register(Box::new(Painel)) {
            eprintln!("Erro ao registrar 'painel_kdl': {}", e);
        }
        motor.set_initial_screen("painel_kdl");

        (Self { motor }, Task::none())
    }

    fn update(&mut self, message: EngineMessage) -> Task<EngineMessage> {
        self.motor.dispatch(&message)
    }

    fn view(&self) -> Element<'_, EngineMessage> {
        match self.motor.render_current() {
            Ok(elem) => elem,
            Err(e) => text(format!("Erro ao renderizar: {}", e))
                .color(Color::from_rgb(1.0, 0.0, 0.0))
                .into(),
        }
    }

    fn subscription(&self) -> Subscription<EngineMessage> {
        GlacierUI::reload_subscription(Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application(|| AppPainel::new(), AppPainel::update, AppPainel::view)
        .subscription(AppPainel::subscription)
        .title("Glacier - Painel (KDL + .gss)")
        .run()
}
