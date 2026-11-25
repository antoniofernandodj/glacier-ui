//! Fase 4 do `docs/PLANO_LINGUAGEM_GV_GSS.md`: duas formas por conceito (a curta,
//! num nó, e a de bloco), uma semântica — e as falhas caladas dessas formas
//! viram erro ou passam a funcionar.

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

fn dir() -> std::path::PathBuf {
    let d = std::env::temp_dir().join("glacier_linguagem_formas");
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// G14: `if =` na raiz ÚNICA de um componente era ignorado (o componente
/// aparecia sempre). Com duas raízes funcionava — agora as duas formas valem.
#[test]
fn if_na_raiz_unica_do_componente_vale() {
    let d = dir();
    std::fs::write(
        d.join("painel.component.gv"),
        "component {\n  column(if = @aba == \"logs\") {\n    text(\"PAINEL\")\n  }\n}\n",
    )
    .unwrap();
    std::fs::write(
        d.join("tela_g14.gv"),
        "component {\n  resources {\n    link(rel = import, href = \"painel.component.gv\", as = Painel)\n  }\n  column {\n    Painel\n    text(\"fim\")\n  }\n}\n",
    )
    .unwrap();
    let mut m = GlacierUI::new();
    m.register_component("tela_g14", d.join("tela_g14.gv").to_str().unwrap())
        .unwrap();
    m.define_data("aba", "outra");
    assert_eq!(textos(&m.evaluated("tela_g14").unwrap()), ["fim"]);
    m.define_data("aba", "logs");
    assert_eq!(textos(&m.evaluated("tela_g14").unwrap()), ["PAINEL", "fim"]);
}

/// O `else` como atributo logo depois do `if =` continua valendo.
#[test]
fn else_como_atributo_no_lugar_certo() {
    let src = "component {\n  column {\n    text(\"A\", if = @x)\n    text(\"B\", else)\n  }\n}\n";
    let d = dir();
    std::fs::write(d.join("else_ok.gv"), src).unwrap();
    let mut m = GlacierUI::new();
    m.register_component("else_ok", d.join("else_ok.gv").to_str().unwrap())
        .unwrap();
    m.define_data("x", "");
    assert_eq!(textos(&m.evaluated("else_ok").unwrap()), ["B"]);
}

/// Um nó entre o `if =` e o `else` muda o sentido do `else` — agora é erro, com a
/// linha do `else`, a do irmão e a dica.
#[test]
fn else_longe_do_if_e_erro_de_posicao() {
    let src = "component {\n  column {\n    text(\"A\", if = @x)\n    text(\"meio\")\n    text(\"B\", else)\n  }\n}\n";
    let err = UiNode::parse_gv_in(src, Some("else_ruim.gv")).unwrap_err().to_string();
    assert!(err.contains(":5"), "devia apontar a linha 5: {err}");
    assert!(err.contains("linha 4"), "devia citar o irmão da linha 4: {err}");
    assert!(err.contains("bloco"), "devia sugerir o bloco: {err}");

    let src = "component {\n  column {\n    text(\"B\", else)\n  }\n}\n";
    let err = UiNode::parse_gv_in(src, Some("else_primeiro.gv")).unwrap_err().to_string();
    assert!(err.contains("primeiro do bloco"), "{err}");
}

/// O `else` de um bloco `if … { } else { }` não é afetado pela checagem.
#[test]
fn else_de_bloco_nao_e_afetado() {
    let src = "component {\n  column {\n    if @x {\n      text(\"A\")\n    } else if @y {\n      text(\"B\")\n    } else {\n      text(\"C\")\n    }\n  }\n}\n";
    UiNode::parse_gv_in(src, Some("bloco.gv")).unwrap();
}

/// `each = @lista as v` repete o nó — a forma curta do laço.
#[test]
fn each_como_atributo_repete_o_no() {
    let d = dir();
    std::fs::write(
        d.join("each_curto.gv"),
        "component {\n  column {\n    text(\"@s.nome\", each = @servicos as s)\n  }\n}\n",
    )
    .unwrap();
    let mut m = GlacierUI::new();
    m.register_component("each_curto", d.join("each_curto.gv").to_str().unwrap())
        .unwrap();
    m.define_data("servicos", r#"[{"nome":"api"},{"nome":"web"}]"#);
    assert_eq!(textos(&m.evaluated("each_curto").unwrap()), ["api", "web"]);
}

/// As formas que saem são listadas com a linha e a forma nova.
#[test]
fn formas_depreciadas_sao_listadas() {
    let src = "component {\n  column {\n    text(\"a\", if = @x, equals = y)\n    template(if = @z) {\n      text(\"b\")\n    }\n    text(\"c\", :for_each = l, var = v)\n    text(\"d\", if = @x == \"y\")\n  }\n}\n";
    let f = glacier_ui::gv::formas_depreciadas(src).unwrap();
    let linhas: Vec<u32> = f.iter().map(|(l, _)| *l).collect();
    assert_eq!(linhas, [3, 4, 7], "{f:#?}");
    assert!(f[0].1.contains("if = @x == \\\"y\\\"") || f[0].1.contains("equals"), "{}", f[0].1);
}

/// A migração (`glacier fmt --migrate`) não muda o que a tela mostra: o mesmo
/// contexto, o `.gv` antigo e o migrado, os mesmos textos.
#[test]
fn migracao_preserva_a_tela() {
    let antigo = r#"component {
  column {
    text("admin", if = @papel, equals = admin)
    text("comum", else)
    template(if = @aba, one_of = "logs eventos") {
      text("aba de registro")
    }
    template(else) {
      text("outra aba")
    }
    text("@s.nome", :for_each = servicos, var = s)
    text("vazio", if = @lista, empty)
  }
}
"#;
    let (novo, n) = glacier_ui::ferramentas::migrar_gv(antigo);
    assert!(n >= 5, "{novo}");
    let restantes = glacier_ui::gv::formas_depreciadas(&novo).unwrap();
    assert!(restantes.is_empty(), "{restantes:?}\n{novo}");
    let d = dir();
    std::fs::write(d.join("antigo.gv"), antigo).unwrap();
    std::fs::write(d.join("migrado.gv"), &novo).unwrap();
    for ctx in [
        [("papel", "admin"), ("aba", "logs"), ("lista", "[]")],
        [("papel", "user"), ("aba", "config"), ("lista", "[1]")],
    ] {
        let mut vistos = Vec::new();
        for nome in ["antigo", "migrado"] {
            let mut m = GlacierUI::new();
            m.register_component(nome, d.join(format!("{nome}.gv")).to_str().unwrap())
                .unwrap();
            m.define_data("servicos", r#"[{"nome":"api"},{"nome":"web"}]"#);
            for (k, v) in ctx {
                m.define_data(k, v);
            }
            vistos.push(textos(&m.evaluated(nome).unwrap()));
        }
        assert_eq!(vistos[0], vistos[1], "migrado:\n{novo}");
    }
}
