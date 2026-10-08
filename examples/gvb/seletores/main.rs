//! **Seletores compostos no `.gss`**, com o comportamento em Rust.
//!
//! Rode com: `cargo run --example gvb_seletores`
//!
//! A tela é uma lista de serviços, e o ponto do exemplo está no `app.gss`: a
//! linha, o nome, o estado e o botão de cada serviço são estilizados pelo
//! **lugar** na árvore (`row.servico > text.nome`), e não por uma classe de
//! propósito em cada nó. O markup só nomeia papéis (`servico`, `nome`,
//! `estado`) e deixa duas classes variarem:
//!
//! - `{s.estado}` na linha (`ativo`/`parado`) vem do **dado**, e
//!   `row.servico.parado > text.nome` vence `row.servico > text.nome` por
//!   especificidade, sem nenhum `if` no markup;
//! - `{densidade}` na raiz vem de um **botão**, e `.compacta .lista >
//!   row.servico` reestiliza as linhas — que são itens de lista em cache —
//!   porque a classe de um ancestral entra na chave do cache delas.
//!
//! O `examples/gvb/seletores_luau` é a mesma tela com este `impl Component`
//! trocado por um script.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Seletores {
    /// `(id, nome, ativo)`. Fica no struct; a tela só vê o JSON publicado.
    servicos: Vec<(&'static str, &'static str, bool)>,
    compacta: bool,
}

impl Seletores {
    fn new() -> Self {
        Seletores {
            servicos: vec![
                ("api", "API pública", true),
                ("db", "Banco primário", true),
                ("cache", "Redis", false),
                ("fila", "Fila de eventos", true),
                ("cron", "Agendador", false),
            ],
            compacta: false,
        }
    }

    /// Grava o estado nas chaves que o markup lê.
    fn publicar(&self, ctx: &mut Context) {
        let itens: Vec<serde_json::Value> = self
            .servicos
            .iter()
            .map(|(id, nome, ativo)| {
                serde_json::json!({
                    "id": id,
                    "nome": nome,
                    "estado": if *ativo { "ativo" } else { "parado" },
                    "acao": if *ativo { "Parar" } else { "Ligar" },
                })
            })
            .collect();
        ctx.set("servicos", serde_json::Value::Array(itens).to_string());
        let densidade = if self.compacta {
            "compacta"
        } else {
            "confortavel"
        };
        ctx.set("densidade", densidade.to_string());
    }
}

impl Component for Seletores {
    fn name(&self) -> &str {
        "seletores"
    }

    fn template(&self) -> Template {
        Template::File("examples/gvb/seletores/app.gvb".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        self.publicar(ctx);
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        let (nome, alvo) = action.split_once(':').unwrap_or((action, ""));
        match nome {
            "densidade" => self.compacta = !self.compacta,
            "parar_todos" => self.servicos.iter_mut().for_each(|s| s.2 = false),
            "ligar_todos" => self.servicos.iter_mut().for_each(|s| s.2 = true),
            "alternar" => {
                if let Some(s) = self.servicos.iter_mut().find(|s| s.0 == alvo) {
                    s.2 = !s.2;
                }
            }
            _ => return,
        }
        self.publicar(ctx);
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Seletores::new())) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("seletores");
        })
        .run()
}
