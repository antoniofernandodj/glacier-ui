//! O `examples/web_contador` carregado por `EmbeddedAssets` — o caminho que o
//! alvo web usa, já que o navegador não tem disco.
//!
//! Olha a ÁRVORE AVALIADA, não a chave de contexto (ver CLAUDE.md, "Antes de
//! dizer que funciona"): o `.gv` embutido virou nós, o `.gss` embutido chegou
//! aos botões com o `var()` resolvido, e um clique muda o texto na tela.

use std::sync::Arc;

use glacier_ui::{Component, Context, EngineMessage, GlacierUI, NodeType, Template, UiNode};

/// O mesmo componente de `examples/web_contador/main.rs`.
struct Contador {
    valor: i32,
}

impl Component for Contador {
    fn name(&self) -> &str {
        "contador"
    }

    fn template(&self) -> Template {
        Template::File("examples/web_contador/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("contador", self.valor.to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            "zerar" => self.valor = 0,
            _ => return,
        }
        ctx.set("contador", self.valor.to_string());
    }
}

fn motor() -> GlacierUI {
    let assets = glacier_ui::embed_assets![
        "examples/web_contador/app.gv",
        "examples/web_contador/app.gss",
        "examples/web_contador/theme.json",
    ];
    let mut m = GlacierUI::new().with_asset_source(Arc::new(assets));
    m.register(Box::new(Contador { valor: 0 })).unwrap();
    m.set_initial_screen("contador");
    m
}

fn textos(node: &UiNode, out: &mut Vec<String>) {
    if let NodeType::Text { content, .. } = &node.kind {
        out.push(content.clone());
    }
    for c in &node.children {
        textos(c, out);
    }
}

/// `(on_click, color)` de cada botão, na ordem da tela.
fn botoes(node: &UiNode, out: &mut Vec<(Option<String>, Option<String>)>) {
    if let NodeType::Button { on_click, color, .. } = &node.kind {
        out.push((on_click.clone(), color.clone()));
    }
    for c in &node.children {
        botoes(c, out);
    }
}

fn valor_na_tela(m: &mut GlacierUI) -> Vec<String> {
    let mut out = Vec::new();
    textos(m.evaluated("contador").unwrap(), &mut out);
    out
}

#[test]
fn a_tela_embutida_tem_o_valor_e_os_tres_botoes_estilizados() {
    let mut m = motor();
    let t = valor_na_tela(&mut m);
    assert!(t.contains(&"0".to_string()), "o valor não chegou à tela: {t:?}");

    let mut b = Vec::new();
    botoes(m.evaluated("contador").unwrap(), &mut b);
    let acoes: Vec<_> = b.iter().map(|(a, _)| a.as_deref()).collect();
    assert_eq!(acoes, [Some("decrementar"), Some("zerar"), Some("incrementar")]);

    // A cor vem de `.perigo { color: var(--perigo) }` no .gss EMBUTIDO. Se a
    // folha não tivesse sido achada pela fonte, isto seria `None` — e a tela
    // abriria sem erro nenhum, só sem estilo.
    let cores: Vec<_> = b.iter().map(|(_, c)| c.as_deref()).collect();
    assert_eq!(cores, [Some("#BF616A"), Some("#88C0D0"), Some("#A3BE8C")]);
}

#[test]
fn clicar_muda_o_texto_na_arvore() {
    let mut m = motor();
    m.dispatch(&EngineMessage::UiClick("incrementar".into()));
    m.dispatch(&EngineMessage::UiClick("incrementar".into()));
    assert!(valor_na_tela(&mut m).contains(&"2".to_string()));

    m.dispatch(&EngineMessage::UiClick("zerar".into()));
    assert!(valor_na_tela(&mut m).contains(&"0".to_string()));
}
