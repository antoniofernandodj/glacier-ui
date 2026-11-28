//! As frentes do `TODO.md` num app real: o exemplo `linguagem_frentes` é a prova
//! de que a linguagem nova dessugara, parseia e é conferida pelo `check`.
//!
//! **Escrito nesta entrega e não executado**: rodar com `make test` (ou
//! `cargo test --test linguagem_frentes`) antes de publicar a versão — ver o
//! registro no fim do `TODO.md`.

use glacier_ui::gv;

fn ler(rel: &str) -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

const APP: &str = "examples/linguagem_frentes/app.gv";
const CARTAO: &str = "examples/linguagem_frentes/cartao.component.gv";

#[test]
fn o_exemplo_dessugara_e_parseia() {
    for rel in [APP, CARTAO] {
        let src = ler(rel);
        let xml = gv::desugar(&src).unwrap_or_else(|d| panic!("{rel}: {} (linha {})", d.message, d.line));
        glacier_ui::parser::UiNode::parse_gv_in(&src, Some(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        // O `state` virou contrato do componente, não nó de layout.
        if rel == CARTAO {
            assert!(xml.contains("<props>"), "{xml}");
            assert!(xml.contains("estado=\"\""), "{xml}");
            assert!(!xml.contains("<state>"), "{xml}");
        }
    }
}

#[test]
fn as_formas_novas_chegam_ao_xml_interno() {
    let x = gv::desugar(&ler(APP)).unwrap();
    // Uma via: `badge(:text = @ativo ? "tipado" : "textual")`.
    assert!(x.contains(r#"text="{(@ativo ? "tipado" : "textual")}"#) || x.contains(":text=\""), "{x}");
    // Vínculo nos dois sentidos: a ligação interna pelo NOME da chave.
    assert!(x.contains(":value=\"resposta\""), "{x}");
    assert!(x.contains(":checked=\"ativo\""), "{x}");
    // Classe condicional com ponto e lista de classes.
    assert!(x.contains(":class.ativa") || x.contains("class=\"{(@ativo) ? 'ativa' : ''}\""), "{x}");
    assert!(x.contains("class=\"tela tela--apertada\""), "{x}");
    // Par numérico virou a string que o atributo sempre esperou.
    assert!(x.contains("size=\"940 900\""), "{x}");
    // Toda ação está na forma de chamada.
    assert!(x.contains("on_click=\"salvar_lento()"), "{x}");
    assert!(!x.contains("on_click=\"testar_verdade\""), "{x}");
}

#[test]
fn o_exemplo_nao_usa_nenhuma_das_formas_que_o_plano_mata() {
    let src = format!("{}\n{}", ler(APP), ler(CARTAO));
    for proibido in [
        "== \"true\"",
        "!= \"true\"",
        "== \"false\"",
        "json.encode",
        "form_control",
        "text_color",
        "window:",
        "field:",
        "|}", // `{chave|padrão}`
    ] {
        assert!(!src.contains(proibido), "sobrou a forma antiga `{proibido}` no exemplo");
    }
}

#[test]
fn o_check_da_arvore_do_exemplo_nao_acusa_handler_faltando() {
    // A Frente 6: cada ação do `.gv` tem de existir no `.luau` com aquela aridade.
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/linguagem_frentes");
    let acoes: Vec<_> = glacier_ui::handlers::acoes(&ler(APP))
        .into_iter()
        .map(|a| a.handler)
        .collect();
    assert!(!acoes.is_empty(), "nenhuma ação lida do exemplo");
    // Os scripts do exemplo estão embutidos no `.gv` (`script """…"""`): o
    // verificador de árvore os lê do próprio arquivo, e aqui conferimos à mão.
    let corpo = glacier_ui::gv::script_do_gv(&ler(APP));
    let funcoes = glacier_ui::handlers::assinaturas(&corpo);
    for a in glacier_ui::handlers::acoes(&ler(APP)) {
        if glacier_ui::handlers::EMBUTIDAS.contains(&a.handler.as_str()) {
            continue;
        }
        let existe = funcoes.iter().any(|f| f.nome == a.handler);
        assert!(existe, "o handler `{}` do exemplo não está no script", a.handler);
    }
}

#[test]
fn migrar_e_reindentar_sao_idempotentes_no_exemplo() {
    for rel in [APP, CARTAO] {
        let src = ler(rel);
        let (m1, _) = glacier_ui::ferramentas::migrar_gv(&src);
        let (m2, k) = glacier_ui::ferramentas::migrar_gv(&m1);
        assert_eq!(m1, m2, "{rel}: fmt --migrate duas vezes != uma");
        assert_eq!(k, 0, "{rel}: a segunda passada ainda achou troca");
        let r1 = glacier_ui::ferramentas::reindentar_gv(&src);
        let r2 = glacier_ui::ferramentas::reindentar_gv(&r1);
        assert_eq!(r1, r2, "{rel}: fmt duas vezes != uma");
        assert_eq!(
            src.matches('\n').count(),
            r1.matches('\n').count(),
            "{rel}: a reindentação mudou o número de linhas"
        );
    }
}
