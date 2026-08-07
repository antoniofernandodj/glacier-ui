//! `<ComboEdit>`: um dropdown editável (`combo_box` do iced) — dá pra digitar
//! um valor novo (não precisa estar na lista) ou escolher um já conhecido.
//! Este exemplo é o caso de uso que motivou o widget: uma lista de
//! servidor/token salvos, onde escolher um servidor da lista preenche o
//! token sozinho (`servidor_selecionado`), mas digitar um servidor novo
//! deixa o campo livre pra um par ainda não salvo.
//!
//! Rode com: `cargo run --example combo_edit`

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Servidores {
    /// (url, token), na ordem em que foram salvos.
    pares: Vec<(String, String)>,
}

impl Servidores {
    fn new() -> Self {
        Self {
            // Já nasce com dois pares salvos, pra mostrar o autopreenchimento
            // sem precisar salvar nada primeiro.
            pares: vec![
                ("https://prod.exemplo.com".into(), "tok_prod_abc123".into()),
                ("https://staging.exemplo.com".into(), "tok_staging_xyz789".into()),
            ],
        }
    }

    /// Publica a lista de URLs salvas (pro `options` do `<ComboEdit>`) e o
    /// texto de depuração exibido embaixo.
    fn sincronizar(&self, ctx: &mut Context) {
        let urls: Vec<String> = self
            .pares
            .iter()
            .map(|(url, _)| format!("\"{}\"", url.replace('"', "\\\"")))
            .collect();
        ctx.set("servidores", format!("[{}]", urls.join(",")));

        let texto = if self.pares.is_empty() {
            "(nenhum ainda)".to_string()
        } else {
            self.pares
                .iter()
                .map(|(url, token)| format!("{url} -> {token}"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        ctx.set("pares_txt", texto);
    }

    fn token_de(&self, url: &str) -> Option<&str> {
        self.pares
            .iter()
            .find(|(u, _)| u == url)
            .map(|(_, t)| t.as_str())
    }
}

impl Component for Servidores {
    fn name(&self) -> &str {
        "servidores"
    }

    fn template(&self) -> Template {
        Template::File("examples/combo_edit/combo_edit.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("url", "");
        ctx.set("token", "");
        self.sincronizar(ctx);
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            // Disparado a cada tecla digitada no combo (o motor já escreveu o
            // texto em `ctx.url` sozinho — ver `UiComboInput`). Se o texto não
            // bate com nenhum servidor salvo, limpa o token: é um par novo.
            "url_changed" => {
                let url = value.unwrap_or_default();
                match self.token_de(url) {
                    Some(t) => ctx.set("token", t.to_string()),
                    None => ctx.set("token", ""),
                }
            }
            // Disparado só quando um item JÁ SALVO é escolhido na lista —
            // aqui mora o autopreenchimento pedido.
            "servidor_selecionado" => {
                if let Some(url) = value
                    && let Some(t) = self.token_de(url)
                {
                    ctx.set("token", t.to_string());
                }
            }
            "token_changed" => {
                ctx.set("token", value.unwrap_or_default().to_string());
            }
            "salvar" => {
                let url = ctx.get("url").cloned().unwrap_or_default();
                let token = ctx.get("token").cloned().unwrap_or_default();
                if !url.is_empty() {
                    match self.pares.iter_mut().find(|(u, _)| *u == url) {
                        Some(par) => par.1 = token,
                        None => self.pares.push((url, token)),
                    }
                    self.sincronizar(ctx);
                }
            }
            _ => {}
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - ComboEdit (servidores salvos)")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Servidores::new())) {
                eprintln!("Erro ao registrar 'servidores': {}", e);
            }
            motor.set_initial_screen("servidores");
        })
        .run()
}
