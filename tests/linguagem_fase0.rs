//! Fase 0 do `docs/PLANO_LINGUAGEM_GV_GSS.md`: o que a análise apontou como
//! armadilha e que precisa ficar travado por teste — tag dentro de comentário
//! no `.gv` e vírgula de seletor dentro de `@media` no `.gss`.

use glacier_ui::stylesheet::parse_gss;
use glacier_ui::UiNode;

/// Uma tag literal num comentário `//`, num comentário `/* */` dentro de
/// `style """…"""` e num comentário entre atributos não quebra o parse.
#[test]
fn tag_dentro_de_comentario_nao_quebra_o_gv() {
    let src = r#"// Um <column> no comentário do topo, e um </text> solto.
component {
  resources {
    style """
    /* <text> e <button class="x"> num comentário do style */
    .a { size: 12; }
    """
  }

  column(
    // <b>negrito</b> num comentário entre atributos
    class = a,
  ) {
    text("ok") // <row> no fim da linha
  }
}
"#;
    UiNode::parse_gv_in(src, Some("comentario.gv")).expect("comentário com tag deve parsear");
}

/// Com a tag no comentário, um erro DEPOIS dele aponta a linha certa do `.gv`.
#[test]
fn erro_depois_de_comentario_com_tag_aponta_a_linha_certa() {
    let src = "// <column> aqui\ncomponent {\n  column {\n    text(\"a\") \"solto\"\n  }\n}\n";
    let err = UiNode::parse_gv_in(src, Some("x.gv")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(":4"), "o erro devia citar a linha 4: {msg}");
}

/// `.a, .b { }` dentro de `@media` aplica a regra às duas partes.
#[test]
fn virgula_dentro_de_media_aplica_a_cada_parte() {
    let sheet = parse_gss("@media (max-width: 600) {\n  .a, .b { size: 9; }\n}\n").unwrap();
    assert_eq!(sheet.media.len(), 1);
    let m = &sheet.media[0];
    assert_eq!(m.rules.get("a").and_then(|r| r.size), Some(9.0));
    assert_eq!(m.rules.get("b").and_then(|r| r.size), Some(9.0));
}

/// Vírgula misturando estado e combinador no mesmo seletor.
#[test]
fn virgula_com_estado_e_combinador() {
    let sheet = parse_gss(".a:hover, .b > .c { size: 7; }\n").unwrap();
    assert!(!sheet.states.is_empty(), "o `.a:hover` devia virar regra de estado");
    assert!(!sheet.complex.is_empty(), "o `.b > .c` devia virar regra composta");
}

// ── Fase 8: aninhamento no `.gss` ────────────────────────────────────────────

/// `&:estado`, `&--modificador` (BEM) e descendente sem `&`, como regras planas.
#[test]
fn aninhamento_vira_regras_planas() {
    let sheet = parse_gss(
        ".btn {\n  size: 13;\n  &:hover { size: 14; }\n  &--primary { size: 15; &:disabled { size: 16; } }\n  .icone { size: 12; }\n}\n",
    )
    .unwrap();
    assert_eq!(sheet.rules["btn"].size, Some(13.0));
    assert_eq!(sheet.rules["btn--primary"].size, Some(15.0));
    let hover = &sheet.states["btn"];
    assert_eq!(hover.values().next().and_then(|r| r.size), Some(14.0));
    let prim = &sheet.states["btn--primary"];
    assert_eq!(prim.values().next().and_then(|r| r.size), Some(16.0));
    // `.btn .icone` é composto (descendente).
    assert_eq!(sheet.complex.len(), 1);
}

/// O pai com vírgula multiplica o filho; a vírgula no filho também.
#[test]
fn aninhamento_com_virgula() {
    let sheet = parse_gss(".a, .b {\n  &--x, &--y { size: 9; }\n}\n").unwrap();
    for k in ["a--x", "a--y", "b--x", "b--y"] {
        assert_eq!(sheet.rules[k].size, Some(9.0), "{k}");
    }
}

/// Um erro dentro do bloco aninhado aponta a linha dele, não a do pai.
#[test]
fn erro_no_aninhado_aponta_a_linha_certa() {
    let e = parse_gss(".btn {\n  size: 13;\n  &:hover {\n    size: grande;\n  }\n}\n")
        .unwrap_err()
        .to_string();
    assert!(e.contains(":4"), "devia citar a linha 4: {e}");
}

/// A regra sem fechar continua erro (o aninhamento não engole o resto do arquivo).
#[test]
fn regra_sem_fechar_continua_erro() {
    let e = parse_gss(".a { size: 1;\n.b { size: 2; }\n").unwrap_err().to_string();
    assert!(e.contains("nunca é fechada"), "{e}");
}

/// `:is(:hover, :focus)` vira duas regras de estado; a vírgula de dentro não
/// divide a lista de seletores.
#[test]
fn is_com_estados_vira_duas_regras() {
    let sheet = parse_gss(".campo:is(:hover, :focus) { size: 9; }\n").unwrap();
    assert_eq!(sheet.states["campo"].len(), 2);
}

/// `:not` só aceita composto simples, com erro que diz o porquê.
#[test]
fn not_com_combinador_e_erro() {
    let e = parse_gss(".a:not(.b > .c) { size: 1; }\n").unwrap_err().to_string();
    assert!(e.contains("só compostos simples"), "{e}");
}
