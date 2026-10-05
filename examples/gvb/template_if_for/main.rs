//! **`<template>` com `if` e com `for-each`** — uma lista de tarefas escrita só
//! com a tag `template`, sem o açúcar `if @x { }` / `each @l as i { }`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example gvb_template_if_for`.
//!
//! # O que este exemplo mostra
//!
//! - `template(if = …)` / `template(else = "")`: o condicional, em dois ramos.
//! - `template(:for-each = tarefas, var = t)`: a repetição. `:for-each` é uma
//!   **ligação** — o NOME da chave que guarda o array JSON —, e `var` batiza o
//!   item.
//! - Um `template` com `if` **dentro** do corpo de um com `for-each`: as duas
//!   diretivas não compõem numa tag só, então uma vai dentro da outra.
//!
//! # O que o app precisa saber
//!
//! Nada de markup: este `main.rs` guarda as tarefas, as publica como JSON e
//! trata três ações (`alternar:<id>`, `filtro` e `nova`).

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Tarefa {
    id: u32,
    titulo: String,
    feita: bool,
}

struct Tarefas {
    itens: Vec<Tarefa>,
    proximo: u32,
    so_pendentes: bool,
}

impl Tarefas {
    fn new() -> Self {
        let titulos = ["Escrever o exemplo", "Rodar o exemplo", "Conferir o if", "Conferir o for"];
        Self {
            itens: titulos
                .iter()
                .enumerate()
                .map(|(i, t)| Tarefa { id: i as u32 + 1, titulo: t.to_string(), feita: i == 0 })
                .collect(),
            proximo: titulos.len() as u32 + 1,
            so_pendentes: false,
        }
    }

    /// Projeta o estado no contexto: o array já filtrado e os rótulos prontos.
    fn publicar(&self, ctx: &mut Context) {
        let visiveis: Vec<serde_json::Value> = self
            .itens
            .iter()
            .filter(|t| !(self.so_pendentes && t.feita))
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "titulo": t.titulo,
                    // texto, não booleano: o `equals` do template compara texto
                    "estado": if t.feita { "feita" } else { "pendente" },
                })
            })
            .collect();
        ctx.set("tarefas", serde_json::Value::Array(visiveis.clone()).to_string());
        ctx.set("so_pendentes", self.so_pendentes.to_string());
        ctx.set("sem_tarefas", visiveis.is_empty().to_string());
        let pendentes = self.itens.iter().filter(|t| !t.feita).count();
        ctx.set("resumo", format!("{pendentes} pendente(s) de {}", self.itens.len()));
    }
}

impl Component for Tarefas {
    fn name(&self) -> &str {
        "template_if_for"
    }

    fn template(&self) -> Template {
        Template::File("examples/gvb/template_if_for/app.gvb".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        self.publicar(ctx);
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // O id viaja dentro da ação (`alternar:3`): um clique não carrega valor.
        let (nome, alvo) = action.split_once(':').unwrap_or((action, ""));
        match nome {
            "alternar" => {
                if let Some(t) = alvo.parse::<u32>().ok().and_then(|id| self.itens.iter_mut().find(|t| t.id == id)) {
                    t.feita = !t.feita;
                }
            }
            "filtro" => self.so_pendentes = !self.so_pendentes,
            "nova" => {
                let n = self.proximo;
                self.proximo += 1;
                self.itens.push(Tarefa { id: n, titulo: format!("Tarefa nova #{n}"), feita: false });
            }
            _ => return,
        }
        self.publicar(ctx);
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - template com if e for")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Tarefas::new())) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("template_if_for");
        })
        .run()
}
