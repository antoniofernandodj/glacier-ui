use xml_ui::{MotorUI, MensagemMotor};
use iced::{Element, Task};
use std::time::Duration;

/// Demonstra um componente (`CartaoUsuario`) que recebe props e é instanciado
/// dentro de um loop `<ForEach>`, um cartão por item da lista.
struct Membro {
    nome: String,
    cargo: String,
    cor: String,
}

struct AppLista {
    motor: MotorUI,
    membros: Vec<Membro>,
    proximo: usize,
}

/// Cores de avatar usadas em rodízio conforme a lista cresce.
const PALETA: [&str; 5] = ["#89B4FA", "#F5C2E7", "#A6E3A1", "#FAB387", "#CBA6F7"];

/// Membros candidatos adicionados ao clicar no botão.
const CANDIDATOS: [(&str, &str); 4] = [
    ("Marina Costa", "Product Manager"),
    ("Rafael Lima", "Engenheiro de Dados"),
    ("Beatriz Souza", "QA Engineer"),
    ("Diego Alves", "DevOps"),
];

impl AppLista {
    fn new() -> (Self, Task<MensagemMotor>) {
        let mut motor = MotorUI::new();

        // Componente principal (com o ForEach) e o componente reutilizável.
        if let Err(e) = motor.registrar_componente("lista", "templates/lista_usuarios.xml") {
            eprintln!("Erro ao registrar 'lista': {}", e);
        }
        // O nome registrado precisa bater com a tag <CartaoUsuario> usada no XML.
        if let Err(e) = motor.registrar_componente("CartaoUsuario", "templates/cartao_usuario.xml") {
            eprintln!("Erro ao registrar 'CartaoUsuario': {}", e);
        }

        let membros = vec![
            Membro { nome: "Clara Silva".into(), cargo: "Engenheira de Software".into(), cor: PALETA[0].into() },
            Membro { nome: "Sophia Martins".into(), cargo: "Designer UI/UX".into(), cor: PALETA[1].into() },
        ];

        let mut app = Self { motor, membros, proximo: 0 };
        app.sincronizar();

        (app, Task::none())
    }

    /// Serializa a lista de membros para JSON e publica no contexto do motor.
    /// O `<ForEach items="usuarios">` consome esse array.
    fn sincronizar(&mut self) {
        let arr: Vec<serde_json::Value> = self
            .membros
            .iter()
            .map(|m| {
                let inicial = m.nome.chars().next().map(|c| c.to_string()).unwrap_or_default();
                serde_json::json!({
                    "nome": m.nome,
                    "cargo": m.cargo,
                    "inicial": inicial,
                    "cor": m.cor,
                })
            })
            .collect();

        let json = serde_json::Value::Array(arr).to_string();
        self.motor.definir_dado("usuarios", &json);
        self.motor.definir_dado("total", &self.membros.len().to_string());
    }

    fn update(&mut self, message: MensagemMotor) -> Task<MensagemMotor> {
        match message {
            MensagemMotor::XmlClick(acao) if acao == "adicionar" => {
                let (nome, cargo) = CANDIDATOS[self.proximo % CANDIDATOS.len()];
                let cor = PALETA[self.membros.len() % PALETA.len()];
                self.membros.push(Membro {
                    nome: nome.into(),
                    cargo: cargo.into(),
                    cor: cor.into(),
                });
                self.proximo += 1;
                self.sincronizar();
            }
            MensagemMotor::FileChanged(_) => {
                let reloaded = self.motor.verificar_recarregamento();
                if !reloaded.is_empty() {
                    println!("Componentes recarregados: {:?}", reloaded);
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, MensagemMotor> {
        match self.motor.renderizar("lista") {
            Ok(elem) => elem,
            Err(e) => iced::widget::text(format!("Erro ao renderizar: {}", e))
                .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                .into(),
        }
    }

    fn subscription(&self) -> iced::Subscription<MensagemMotor> {
        MotorUI::subscricao_recarregamento(Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application("XML UI - Lista de Membros", AppLista::update, AppLista::view)
        .subscription(AppLista::subscription)
        .run_with(|| AppLista::new())
}
