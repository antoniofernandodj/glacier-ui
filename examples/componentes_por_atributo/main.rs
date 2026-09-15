//! **Componentes por atributo** — um componente que recebe outros componentes,
//! e o `fallback` de uma lista vazia.
//!
//! Rode com: `cargo run --example componentes_por_atributo` (com
//! `WGPU_BACKEND=gl` nesta máquina).
//!
//! # Componente como atributo
//!
//! A `<Vitrine>` (em `vitrine.gv`) não sabe desenhar cabeçalho, linha nem lista
//! vazia. Quem usa diz quais componentes fazem cada papel, pelo **nome**:
//!
//! ```xml
//! <Vitrine cabecalho="TituloDestaque" linha="LinhaTarefa" vazio="ListaVazia" … />
//! ```
//!
//! e a Vitrine os desenha com `<render>`, que resolve um nome vindo de prop:
//!
//! ```xml
//! <render component="{cabecalho}" titulo="{titulo}" total="{total}" />
//! ```
//!
//! # `fallback`
//!
//! `<foreach>` aceita `fallback="Nome"` — no `<template foreach>`, que também é
//! `if`/`else`, o atributo se chama `foreach_fallback`: o componente
//! desenhado quando a lista está vazia. O nome pode ser literal
//! (`fallback="ListaVazia"`) ou chegar por prop (`fallback="{vazio}"`). Tem de
//! ser um componente já declarado no `<resources>`, importado ou registrado — um
//! nome errado é `UnknownComponent` já na primeira avaliação, mesmo com a lista
//! cheia.
//!
//! # O que o app precisa saber
//!
//! Nada sobre os componentes: este `main.rs` só mantém três listas e as publica
//! como JSON. As ações escritas dentro dos componentes (`concluir:{id}`) chegam
//! aqui sem prefixo, porque nenhum deles tem comportamento próprio.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Item {
    id: u32,
    texto: String,
}

/// Tarefas oferecidas em rodízio pelo botão "Adicionar tarefa".
const SUGESTOES: [&str; 5] = [
    "Revisar o pull request",
    "Atualizar o CHANGELOG",
    "Responder a issue #42",
    "Rodar o exemplo com WGPU_BACKEND=gl",
    "Escrever a documentação do fallback",
];

struct ComponentesPorAtributo {
    tarefas: Vec<Item>,
    concluidas: Vec<Item>,
    compras: Vec<Item>,
    filtro: String,
    proximo: u32,
}

impl ComponentesPorAtributo {
    fn novo_id(&mut self) -> u32 {
        self.proximo += 1;
        self.proximo
    }

    /// Serializa as listas para o contexto. Uma lista vazia vira `[]`, que é o
    /// que faz o `fallback` do `<foreach>` entrar.
    fn sincronizar(&self, ctx: &mut Context) {
        let filtro = self.filtro.to_lowercase();
        let compras: Vec<&Item> = self
            .compras
            .iter()
            .filter(|c| c.texto.to_lowercase().contains(&filtro))
            .collect();

        ctx.set("tarefas", json_de(self.tarefas.iter()));
        ctx.set("concluidas", json_de(self.concluidas.iter()));
        ctx.set("compras_filtradas", json_de(compras.iter().copied()));
        ctx.set("total_tarefas", self.tarefas.len().to_string());
        ctx.set("total_compras", compras.len().to_string());
    }
}

fn json_de<'a>(itens: impl Iterator<Item = &'a Item>) -> String {
    serde_json::Value::Array(
        itens
            .map(|i| serde_json::json!({ "id": i.id, "texto": i.texto }))
            .collect(),
    )
    .to_string()
}

/// Tira o item `id` de `de` e o põe no fim de `para`. Devolve o texto dele.
fn mover(de: &mut Vec<Item>, para: &mut Vec<Item>, id: u32) -> Option<String> {
    let pos = de.iter().position(|i| i.id == id)?;
    let item = de.remove(pos);
    let texto = item.texto.clone();
    para.push(item);
    Some(texto)
}

impl Component for ComponentesPorAtributo {
    fn name(&self) -> &str {
        "componentes_por_atributo"
    }

    fn template(&self) -> Template {
        Template::File("examples/componentes_por_atributo/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        for texto in ["Leite", "Pão", "Café", "Queijo", "Maçã"] {
            let id = self.novo_id();
            self.compras.push(Item {
                id,
                texto: texto.into(),
            });
        }
        let id = self.novo_id();
        self.tarefas.push(Item {
            id,
            texto: SUGESTOES[0].into(),
        });
        ctx.set("filtro", String::new());
        ctx.set("status", "Pronto".to_string());
        self.sincronizar(ctx);
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // O id viaja dentro da ação (`concluir:7`): um clique não carrega valor.
        let (nome, alvo) = action.split_once(':').unwrap_or((action, ""));
        let id = alvo.parse::<u32>().ok();

        let status = match (nome, id) {
            ("adicionar", _) => {
                let texto = SUGESTOES[self.tarefas.len() % SUGESTOES.len()];
                let id = self.novo_id();
                self.tarefas.push(Item {
                    id,
                    texto: texto.into(),
                });
                format!("Adicionada: {texto}")
            }
            ("limpar", _) => {
                self.tarefas.clear();
                "Tarefas limpas — o fallback entra no lugar da lista".to_string()
            }
            ("concluir", Some(id)) => match mover(&mut self.tarefas, &mut self.concluidas, id) {
                Some(texto) => format!("Concluída: {texto}"),
                None => return,
            },
            ("reabrir", Some(id)) => match mover(&mut self.concluidas, &mut self.tarefas, id) {
                Some(texto) => format!("Reaberta: {texto}"),
                None => return,
            },
            ("filtrar", _) => {
                self.filtro = value.unwrap_or_default().to_string();
                ctx.set("filtro", self.filtro.clone());
                format!("Filtro: “{}”", self.filtro)
            }
            _ => return,
        };
        ctx.set("status", status);
        self.sincronizar(ctx);
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Componentes por atributo")
        .main(|motor| {
            let tela = ComponentesPorAtributo {
                tarefas: Vec::new(),
                concluidas: Vec::new(),
                compras: Vec::new(),
                filtro: String::new(),
                proximo: 0,
            };
            if let Err(e) = motor.register(Box::new(tela)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("componentes_por_atributo");
        })
        .run()
}
