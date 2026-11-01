//! i18n — Fase 7 do `PLANO_SUBSTITUIBILIDADE.md`: `{t("chave")}`, plural, número e data
//! por locale, e troca de idioma ao vivo.
//!
//!   WGPU_BACKEND=gl cargo run --example i18n
//!   cargo run --example i18n -- --arvore       (imprime o texto avaliado, por locale)

use glacier_ui::{Component, Context, GlacierDaemon, GlacierUI, NodeType, Template, UiNode};

/// `locale` e `contagem` só servem ao `--arvore`; no app normal vêm vazios.
#[derive(Default)]
struct Painel {
    locale: String,
    contagem: String,
}

impl Component for Painel {
    fn name(&self) -> &str {
        "i18n"
    }
    fn template(&self) -> Template {
        Template::File("examples/i18n/app.gv".into())
    }
    fn init(&mut self, ctx: &mut Context) {
        ctx.set("usuario", "Ana".to_string());
        let contagem = if self.contagem.is_empty() { "1" } else { &self.contagem };
        ctx.set("contagem", contagem.to_string());
        if !self.locale.is_empty() {
            ctx.set("__locale", self.locale.clone());
        }
        ctx.set("preco", "1234567.891".to_string());
        ctx.set("quando", "2026-10-31".to_string());
    }
    fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
        let n: i64 = ctx.get("contagem").and_then(|c| c.parse().ok()).unwrap_or(0);
        match action {
            "mais" => ctx.set("contagem", (n + 1).to_string()),
            "menos" => ctx.set("contagem", (n - 1).max(0).to_string()),
            "idioma_pt" => ctx.set("__locale", "pt-br".to_string()),
            "idioma_en" => ctx.set("__locale", "en".to_string()),
            "idioma_ru" => ctx.set("__locale", "ru".to_string()),
            "idioma_ar" => ctx.set("__locale", "ar".to_string()),
            _ => {}
        }
    }
}

fn catalogos() -> GlacierDaemon {
    GlacierDaemon::new().locales_dir("examples/i18n/locales").default_locale("pt-BR")
}

fn textos(n: &UiNode, out: &mut Vec<String>) {
    if let NodeType::Text { content, .. } = &n.kind {
        out.push(content.clone());
    }
    for c in n.children.iter() {
        textos(c, out);
    }
}

fn main() -> iced::Result {
    if std::env::args().any(|a| a == "--arvore") {
        let _ = catalogos();
        for loc in ["pt-br", "en", "ru", "ar"] {
            for contagem in ["1", "2", "5", "21"] {
                let mut motor = GlacierUI::new();
                let painel = Painel { locale: loc.into(), contagem: contagem.into() };
                motor.register(Box::new(painel)).expect("registrar");
                let raiz = motor.evaluated("i18n").expect("avaliar");
                let mut t = Vec::new();
                textos(raiz, &mut t);
                println!("[{loc:5} n={contagem:2}] {}", t.join(" | "));
            }
        }
        return Ok(());
    }
    catalogos()
        .main(|motor| {
            let _ = motor.register(Box::new(Painel::default()));
            motor.set_initial_screen("i18n");
        })
        .run()
}
