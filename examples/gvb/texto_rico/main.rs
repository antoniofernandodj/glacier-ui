//! Texto inline misto, `<a>` e `<markdown>` — Fase 1 do `PLANO_SUBSTITUIBILIDADE.md`.
//!
//!   WGPU_BACKEND=gl cargo run --example gvb_texto_rico
//!   cargo run --example gvb_texto_rico -- --arvore     (imprime a árvore avaliada)
//!
//! `--arvore` existe porque este repositório já teve widget que passava em todos os
//! testes e aparecia vazio: aqui se confere o que o motor AVALIOU, não só o que
//! renderiza.

use glacier_ui::{Component, Context, GlacierDaemon, GlacierUI, NodeType, Template, UiNode};

struct Detalhes;

impl Component for Detalhes {
    fn name(&self) -> &str {
        "detalhes"
    }
    fn template(&self) -> Template {
        Template::File("examples/gvb/texto_rico/detalhes.gvb".into())
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {}
}

struct Sobre;

impl Component for Sobre {
    fn name(&self) -> &str {
        "texto_rico"
    }
    fn template(&self) -> Template {
        Template::File("examples/gvb/texto_rico/app.gvb".into())
    }
    fn init(&mut self, ctx: &mut Context) {
        ctx.set("usuario", "Ana".to_string());
        ctx.set("cliques", "0".to_string());
    }
    fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
        if action == "contar" {
            let n: u32 = ctx.get("cliques").and_then(|c| c.parse().ok()).unwrap_or(0);
            ctx.set("cliques", (n + 1).to_string());
        }
    }
}

fn imprimir(n: &UiNode, nivel: usize) {
    let recuo = "  ".repeat(nivel);
    match &n.kind {
        NodeType::Prose { blocos, size, .. } => {
            println!("{recuo}Prose size={size:?} ({} blocos)", blocos.len());
            for b in blocos {
                println!("{recuo}  {b:?}");
            }
        }
        NodeType::Markdown { content, .. } => {
            println!("{recuo}Markdown ({} bytes):\n{content}", content.len());
        }
        NodeType::Text { content, .. } => println!("{recuo}Text {content:?}"),
        NodeType::TextArea { value_var, readonly, fixo: Some(f), .. } => {
            println!("{recuo}TextArea selecionável chave={value_var} readonly={readonly} size={:?} texto={:?}", f.size, f.content)
        }
        _ => println!("{recuo}{:?}", std::mem::discriminant(&n.kind)),
    }
    for c in n.children.iter() {
        imprimir(c, nivel + 1);
    }
}

fn main() -> iced::Result {
    if std::env::args().any(|a| a == "--arvore") {
        let mut motor = GlacierUI::new();
        motor.register(Box::new(Detalhes)).expect("registrar detalhes");
        motor.register(Box::new(Sobre)).expect("registrar");
        match motor.evaluated("texto_rico") {
            Ok(raiz) => imprimir(raiz, 0),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    GlacierDaemon::new()
        .main(|motor| {
            let _ = motor.register(Box::new(Sobre));
            let _ = motor.register(Box::new(Detalhes));
            motor.set_initial_screen("texto_rico");
        })
        .run()
}
