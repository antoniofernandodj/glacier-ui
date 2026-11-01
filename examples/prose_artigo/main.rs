//! **Um artigo, três sintaxes**: o texto vem de ARQUIVOS (`examples/conteudo_prose`),
//! lidos aqui em Rust, e entra nos `prose(syntax, content)` da tela. É o mesmo caminho de
//! um texto que viesse de um banco ou de uma API — o `prose` só vê a chave do contexto.
//! Ver `docs/TEXTO.md`.
//!
//! Rode com: `WGPU_BACKEND=gl cargo run --example prose_artigo`

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Artigo;

fn ler(caminho: &str) -> String {
    std::fs::read_to_string(caminho).unwrap_or_else(|e| format!("erro ao ler {caminho}: {e}"))
}

impl Component for Artigo {
    fn name(&self) -> &str {
        "artigo"
    }

    fn template(&self) -> Template {
        Template::File("examples/prose_artigo/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("artigo_html", ler("examples/conteudo_prose/artigo.html"));
        ctx.set("artigo_gv", ler("examples/conteudo_prose/artigo.gv.txt"));
        ctx.set("artigo_md", ler("examples/conteudo_prose/artigo.md"));
        // Conteúdo hostil: um link que tenta disparar uma ação do app. É RECUSADO.
        ctx.set(
            "artigo_ruim",
            "p { span(\"Veja \") a(\"apagar tudo\", on_click = \"app::apagar\") }".to_string(),
        );
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {}
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Artigo)) {
                eprintln!("{e}");
            }
            motor.set_initial_screen("artigo");
        })
        .run()
}
