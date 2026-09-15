//! **`<foreach fallback>`** — uma cozinha com três etapas, cada uma um
//! `<foreach>` com o seu componente de lista vazia.
//!
//! Rode com: `cargo run --example foreach_fallback` (com `WGPU_BACKEND=gl`
//! nesta máquina). O par em Luau, com o mesmo markup, é
//! `foreach_fallback_luau`.
//!
//! # O que este exemplo mostra
//!
//! - `<foreach items="…" fallback="Componente">`: a tag de repetição e o que
//!   aparece no lugar da lista quando ela está vazia. A lista de entregues usa
//!   um nome literal (`fallback="SemEntregas"`).
//! - O fallback chegando por prop: o componente `Etapa` recebe
//!   `vazio="FilaVazia"` e o repassa como `fallback="{vazio}"`. A mesma `Etapa`
//!   desenha as três colunas, cada uma com o seu "vazio".
//!
//! # O que o app precisa saber
//!
//! Nada sobre os fallbacks: este `main.rs` só move pedidos entre quatro listas e
//! as publica como JSON. Uma lista vazia vira `[]`, e é isso que faz o
//! `fallback` entrar.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

#[derive(Clone)]
struct Pedido {
    id: u32,
    mesa: u32,
    prato: &'static str,
}

/// Pratos oferecidos em rodízio pelo botão "Novo pedido".
const CARDAPIO: [&str; 6] = [
    "Risoto de cogumelos",
    "Moqueca",
    "Feijoada",
    "Lasanha",
    "Salada caprese",
    "Picanha na chapa",
];

#[derive(Default)]
struct Cozinha {
    fila: Vec<Pedido>,
    preparo: Vec<Pedido>,
    prontos: Vec<Pedido>,
    entregues: Vec<Pedido>,
    proximo: u32,
}

impl Cozinha {
    fn novo_pedido(&mut self) -> Pedido {
        self.proximo += 1;
        let n = self.proximo;
        let pedido = Pedido {
            id: n,
            mesa: n * 7 % 12 + 1,
            prato: CARDAPIO[n as usize % CARDAPIO.len()],
        };
        self.fila.push(pedido.clone());
        pedido
    }

    /// Projeta as quatro listas no contexto, com os totais já prontos.
    fn publicar(&self, ctx: &mut Context) {
        ctx.set("fila", json_de(&self.fila));
        ctx.set("preparo", json_de(&self.preparo));
        ctx.set("prontos", json_de(&self.prontos));
        ctx.set("entregues", json_de(&self.entregues));
        ctx.set("total_fila", self.fila.len().to_string());
        ctx.set("total_preparo", self.preparo.len().to_string());
        ctx.set("total_prontos", self.prontos.len().to_string());
    }
}

fn json_de(pedidos: &[Pedido]) -> String {
    serde_json::Value::Array(
        pedidos
            .iter()
            .map(|p| serde_json::json!({ "id": p.id, "mesa": p.mesa, "prato": p.prato }))
            .collect(),
    )
    .to_string()
}

/// Tira o pedido `id` de `de` e o põe no fim de `para`.
fn mover(de: &mut Vec<Pedido>, para: &mut Vec<Pedido>, id: u32) -> Option<Pedido> {
    let pos = de.iter().position(|p| p.id == id)?;
    let pedido = de.remove(pos);
    para.push(pedido.clone());
    Some(pedido)
}

impl Component for Cozinha {
    fn name(&self) -> &str {
        "foreach_fallback"
    }

    fn template(&self) -> Template {
        Template::File("examples/foreach_fallback/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // Começa com a fila e o fogo ocupados, e "Prontos"/"Entregues" vazios:
        // os dois fallbacks já aparecem na primeira tela.
        self.novo_pedido();
        self.novo_pedido();
        let primeiro = self.novo_pedido();
        mover(&mut self.fila, &mut self.preparo, primeiro.id);
        ctx.set("status", "Pronto".to_string());
        self.publicar(ctx);
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // O id viaja dentro da ação (`preparar:3`): um clique não carrega valor.
        let (nome, alvo) = action.split_once(':').unwrap_or((action, ""));
        let id = alvo.parse::<u32>().ok();

        let status = match (nome, id) {
            ("novo", _) => {
                let p = self.novo_pedido();
                format!("Pedido #{} — mesa {}", p.id, p.mesa)
            }
            ("preparar", Some(id)) => match mover(&mut self.fila, &mut self.preparo, id) {
                Some(p) => format!("No fogo: {}", p.prato),
                None => return,
            },
            ("finalizar", Some(id)) => match mover(&mut self.preparo, &mut self.prontos, id) {
                Some(p) => format!("Pronto: {}", p.prato),
                None => return,
            },
            ("entregar", Some(id)) => match mover(&mut self.prontos, &mut self.entregues, id) {
                Some(p) => format!("Entregue na mesa {}", p.mesa),
                None => return,
            },
            ("limpar", _) => {
                self.fila.clear();
                self.preparo.clear();
                self.prontos.clear();
                self.entregues.clear();
                "Tudo limpo — cada lista mostra o seu fallback".to_string()
            }
            _ => return,
        };
        ctx.set("status", status);
        self.publicar(ctx);
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Pedidos (foreach + fallback)")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Cozinha::default())) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("foreach_fallback");
        })
        .run()
}
