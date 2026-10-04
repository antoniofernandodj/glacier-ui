//! Regras de leitura do `.gvb` que não podem regredir.

use glacier_ui::gvb::desugar;

/// Com ou sem linha em branco entre os atributos e os filhos, a árvore é a mesma.
#[test]
fn linha_em_branco_entre_atributos_e_filhos_e_opcional() {
    let com = "row {\n  class: linha\n\n  Card { n: A }\n  Card { n: B }\n}\n";
    let sem = "row {\n  class: linha\n  Card { n: A }\n  Card { n: B }\n}\n";
    let xml = |s: &str| desugar(s).unwrap().split_whitespace().collect::<String>();
    assert_eq!(xml(com), xml(sem));
    assert_eq!(xml(sem), r#"<rowclass="linha"><Cardn="A"/><Cardn="B"/></row>"#);
}

/// O texto vale antes e depois do bloco, em uma linha e em `"""`.
#[test]
fn texto_antes_ou_depois_do_bloco() {
    let xml = |s: &str| desugar(s).unwrap().split_whitespace().collect::<String>();
    let a = xml(r#"text "parado" { class: tag_off else: "" }"#);
    let b = xml(r#"text { class: tag_off else: "" } "parado""#);
    assert_eq!(a, b);
    let c = xml("text \"\"\"\n  parado\n\"\"\" { class: tag_off }");
    let d = xml("text { class: tag_off } \"\"\"\n  parado\n\"\"\"");
    assert_eq!(c, d);
}
