//! Fase 7 do `docs/PLANO_LINGUAGEM_GV_GSS.md`: a semântica de cor do CSS
//! (`app(gss_color = css)`). Num binário próprio: o modo é uma chave global.

use glacier_ui::{GlacierUI, NodeType, UiNode};

fn botao<'a>(no: &'a UiNode, rotulo: &str) -> Option<&'a UiNode> {
    if let NodeType::Button { text, .. } = &no.kind
        && text == rotulo
    {
        return Some(no);
    }
    no.children.iter().find_map(|c| botao(c, rotulo))
}

#[test]
fn modo_css_background_e_fundo_color_e_rotulo() {
    glacier_ui::eval::set_cor_css(true);
    let dir = std::env::temp_dir().join("glacier_linguagem_cor");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cor.gv");
    std::fs::write(
        &path,
        r##"component {
  resources {
    style """
    .btn { background: #112233; color: #FFFFFF; }
    .btn:hover { background: #445566; color: #000000; }
    """
  }
  column {
    button("Classe", class = btn, on_click = x)
    button("Inline", background = "#ABCDEF", color = "#123456", on_click = x)
  }
}
"##,
    )
    .unwrap();
    let mut m = GlacierUI::new();
    m.register_component("cor", path.to_str().unwrap()).unwrap();
    let ev = m.evaluated("cor").unwrap().clone();

    let b = botao(&ev, "Classe").unwrap();
    let NodeType::Button { color, .. } = &b.kind else { unreachable!() };
    assert_eq!(color.as_deref(), Some("#112233"), "fundo do botão");
    assert_eq!(b.text_color(), Some("#FFFFFF"), "cor do rótulo");
    assert_eq!(b.background, None, "o embrulho genérico não pinta um segundo fundo");
    let h = b.hover_style().unwrap();
    assert_eq!(h.background.as_deref(), Some("#445566"));
    assert_eq!(h.text_color.as_deref(), Some("#000000"));

    let b = botao(&ev, "Inline").unwrap();
    let NodeType::Button { color, .. } = &b.kind else { unreachable!() };
    assert_eq!(color.as_deref(), Some("#ABCDEF"));
    assert_eq!(b.text_color(), Some("#123456"));

    // Os estilos embutidos (escritos no sentido antigo) são traduzidos.
    let frost = glacier_ui::style::by_name("frost").unwrap().sheet().unwrap();
    let r = &frost.tags["button"];
    assert_eq!(r.background.as_deref(), Some("#fdfdfd"));
    assert_eq!(r.color.as_deref(), Some("#1a1a1a"));
    glacier_ui::eval::set_cor_css(false);
}
