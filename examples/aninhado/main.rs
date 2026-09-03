use glacier_ui::{Component, Context, GlacierDaemon, Template};

/// Demonstra um `Component` registrado DENTRO de outro `Component`.
///
/// O pai (`Painel`) possui o filho (`CartaoContador`) via `children()`, e cada
/// um trata as ações que saem da sua própria UI:
///   - `+` / `-`          (UI do filho) -> `CartaoContador::update`
///   - "Trocar destaque"  (UI do pai)   -> `Painel::update`
///
/// O motor faz isso namespaceando as ações da subárvore do filho
/// (`incrementar` -> `CartaoContador::incrementar`) e roteando no `dispatch`.
///
/// Filho com comportamento e estado próprios.
struct CartaoContador {
    valor: i32,
}

impl Component for CartaoContador {
    fn name(&self) -> &str {
        "CartaoContador"
    }

    fn template(&self) -> Template {
        Template::File("examples/aninhado/cartao_contador.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("valor", self.valor.to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            _ => return,
        }
        ctx.set("valor", self.valor.to_string());
    }
}

/// Pai: tem uma ação própria e possui o `CartaoContador`.
///
/// O botão do pai já trocava o **fundo da janela** por uma chave de contexto
/// (`painel_bg`), pintando a raiz `fill/fill`. Era a camada mais cara da tela —
/// um retângulo sombreado por pixel em toda a janela, por cima do que o tema já
/// pinta. O fundo agora é do `theme.json`, e o que sobra aqui é o que o exemplo
/// existe para mostrar: uma ação que sai da UI do **pai** e cai no `update` do
/// pai. Trocar a cor da janela em runtime é a ação builtin `style:<nome>`, que
/// não passa por componente nenhum (ver `examples/galeria_estilos`).
struct Painel {
    quente: bool,
}

impl Painel {
    fn aplicar_destaque(&self, ctx: &mut Context) {
        if self.quente {
            ctx.set("destaque", "quente");
            ctx.set("cor_texto", "#F5E0DC");
        } else {
            ctx.set("destaque", "frio");
            ctx.set("cor_texto", "#89B4FA");
        }
    }
}

impl Component for Painel {
    fn name(&self) -> &str {
        "painel"
    }

    fn template(&self) -> Template {
        Template::File("examples/aninhado/painel.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        self.aplicar_destaque(ctx);
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        if action == "trocar_destaque" {
            self.quente = !self.quente;
            self.aplicar_destaque(ctx);
        }
    }

    fn children(&self) -> Vec<Box<dyn Component>> {
        vec![Box::new(CartaoContador { valor: 0 })]
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Componentes Aninhados")
        .main(|motor| {
            // Registra só o pai; o filho entra em cascata via children().
            if let Err(e) = motor.register(Box::new(Painel { quente: false })) {
                eprintln!("Erro ao registrar 'painel': {}", e);
            }
            motor.set_initial_screen("painel");
        })
        .run()
}
