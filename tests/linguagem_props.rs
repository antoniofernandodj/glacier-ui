//! Fase 3 do `docs/PLANO_LINGUAGEM_GV_GSS.md`: props tipados.

use glacier_ui::{GlacierUI, NodeType, UiNode};

fn textos(no: &UiNode) -> Vec<String> {
    let mut out = Vec::new();
    fn anda(no: &UiNode, out: &mut Vec<String>) {
        if let NodeType::Text { content, .. } = &no.kind {
            out.push(content.clone());
        }
        for c in &no.children {
            anda(c, out);
        }
    }
    anda(no, &mut out);
    out
}

fn tela(nome: &str, uso: &str) -> Result<GlacierUI, String> {
    let d = std::env::temp_dir().join("glacier_linguagem_props");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(
        d.join("selo.component.gv"),
        r#"component {
  props {
    prop(name = rotulo, type = string)
    prop(name = ativo, type = bool, default = false)
    prop(name = total, type = number, default = 0)
  }
  column {
    text("@rotulo", if = @ativo)
    text("{@total > 9 ? '9+' : @total}")
  }
}
"#,
    )
    .unwrap();
    std::fs::write(
        d.join(format!("{nome}.gv")),
        format!(
            "component {{\n  resources {{\n    link(rel = import, href = \"selo.component.gv\", as = Selo)\n  }}\n  column {{\n    {uso}\n  }}\n}}\n"
        ),
    )
    .unwrap();
    let mut m = GlacierUI::new();
    m.register_component(nome, d.join(format!("{nome}.gv")).to_str().unwrap())
        .map_err(|e| e.to_string())?;
    m.evaluated(nome).map_err(|e| e.to_string())?;
    Ok(m)
}

#[test]
fn prop_bool_funciona_sem_comparar_com_texto() {
    let mut m = tela("props_ok", r#"Selo(rotulo = "Novo", ativo = true, total = 12)"#).unwrap();
    assert_eq!(textos(&m.evaluated("props_ok").unwrap()), ["Novo", "9+"]);
}

#[test]
fn literal_do_tipo_errado_e_erro() {
    let e = tela("props_ruim", r#"Selo(rotulo = "x", ativo = talvez)"#).err().unwrap();
    assert!(e.contains("tipo bool") && e.contains("talvez"), "{e}");
    let e = tela("props_ruim2", r#"Selo(rotulo = "x", total = muitos)"#).err().unwrap();
    assert!(e.contains("tipo number"), "{e}");
}

#[test]
fn valor_do_contexto_do_tipo_errado_so_avisa() {
    // Dado de tempo de execução: avisa no stderr, não derruba a tela.
    let m = tela("props_ctx", r#"Selo(rotulo = "x", total = @n)"#);
    assert!(m.is_ok(), "{:?}", m.err());
}

#[test]
fn tipo_desconhecido_e_erro_de_parse() {
    let src = "component {\n  props {\n    prop(name = x, type = inteiro)\n  }\n  column\n}\n";
    let e = UiNode::parse_gv_in(src, Some("t.gv")).unwrap_err().to_string();
    assert!(e.contains("tipo 'inteiro' desconhecido"), "{e}");
}
