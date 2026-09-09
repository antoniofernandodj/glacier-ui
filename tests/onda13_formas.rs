//! Onda 13 — o vocabulário de formas do `<canvas>` e o `QWhatsThis`.
//!
//! O que estes testes guardam, além da feature: **a árvore avaliada** (a
//! checagem que o `CLAUDE.md` manda fazer). Um `<circle r="{raio}">` cujo `geo`
//! não interpola, ou não reinterpola quando `raio` muda, desenharia um círculo
//! parado — e passaria num teste que só olhasse a chave de contexto.

use glacier_ui::{EngineMessage, GlacierUI, NodeType};

/// Um `.gv` (e um `.gss` irmão) temporário, com nome ÚNICO por teste — os
/// testes rodam em paralelo e um nome fixo faria um sobrescrever o arquivo do
/// outro no meio do `register_component`. `{GSS}` no markup vira o caminho do
/// `.gss`.
fn tela(nome: &str, markup: &str, gss: &str) -> GlacierUI {
    std::fs::create_dir_all("templates").ok();
    let gss_path = format!("templates/{nome}.gss");
    std::fs::write(&gss_path, gss).unwrap();
    let gv = format!("templates/{nome}.gv");
    std::fs::write(&gv, markup.replace("{GSS}", &gss_path)).unwrap();
    let mut motor = GlacierUI::new();
    motor.register_component("tela", &gv).unwrap();
    motor.navigate_to("tela");
    motor
}

fn semeia(motor: &mut GlacierUI, pares: &[(&str, &str)]) {
    let _ = motor.dispatch(&EngineMessage::ContextPatch(
        pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    ));
}

fn acha<'a>(node: &'a glacier_ui::UiNode, tag: &str) -> Option<&'a glacier_ui::UiNode> {
    if node.kind.tag_name() == Some(tag) {
        return Some(node);
    }
    node.children.iter().find_map(|c| acha(c, tag))
}

fn geo<'a>(node: &'a glacier_ui::UiNode, chave: &str) -> Option<&'a str> {
    match &node.kind {
        NodeType::Shape { geo, .. } => geo
            .iter()
            .find(|(k, _)| k == chave)
            .map(|(_, v)| v.as_str()),
        _ => None,
    }
}

const MARKUP: &str = r#"<screen>
  <resources>
    <link rel="stylesheet" href="{GSS}" />
  </resources>
  <canvas class="tela-desenho">
    <circle cx="10" cy="10" r="{raio}" class="cheia" />
    <path d="{tracado}" class="traco" />
    <rect x="0" y="0" w="20" h="10" />
    <text x="4" y="30" class="rot">ângulo {raio}</text>
  </canvas>
</screen>"#;

const GSS: &str = r#".tela-desenho { width: 200; height: 120; }
.cheia { fill: #ff0000; }
.traco { stroke: #00ff00; stroke-width: 3; }
.rot { size: 11; }
"#;

#[test]
fn geometria_interpola_e_reinterpola_quando_a_chave_muda() {
    let mut motor = tela("onda13_geo", MARKUP, GSS);
    semeia(&mut motor, &[("raio", "20"), ("tracado", "M0 0 L10 10")]);

    let arvore = motor.evaluated("tela").unwrap();
    let circulo = acha(arvore, "circle").expect("o <circle> está na árvore");
    assert_eq!(geo(circulo, "r"), Some("20"), "r interpolado de {{raio}}");
    assert_eq!(geo(circulo, "cx"), Some("10"), "cx literal preservado");

    let path = acha(arvore, "path").expect("o <path> está na árvore");
    assert_eq!(geo(path, "d"), Some("M0 0 L10 10"));

    // A mudança de chave: o canvas TEM de reinterpolar (o `geo` é resolvido no
    // eval, não no render — a diferença do `items=` de um gráfico).
    semeia(&mut motor, &[("raio", "40")]);
    let arvore = motor.evaluated("tela").unwrap();
    let circulo = acha(arvore, "circle").unwrap();
    assert_eq!(geo(circulo, "r"), Some("40"), "r seguiu {{raio}} para 40");
}

#[test]
fn fill_e_stroke_vem_da_classe_gss() {
    let mut motor = tela("onda13_estilo", MARKUP, GSS);
    semeia(&mut motor, &[("raio", "10"), ("tracado", "M0 0")]);
    let arvore = motor.evaluated("tela").unwrap();

    // `.cheia { fill: #ff0000 }` → o preenchimento é o `background` do nó.
    let circulo = acha(arvore, "circle").unwrap();
    assert_eq!(circulo.background.as_deref(), Some("#ff0000"));

    // `.traco { stroke: #00ff00; stroke-width: 3 }` → borda.
    let path = acha(arvore, "path").unwrap();
    assert_eq!(path.border_color(), Some("#00ff00"));
    assert_eq!(path.border_width, Some(3.0));
}

#[test]
fn text_dentro_do_canvas_continua_um_text_com_posicao_em_x_y() {
    let mut motor = tela("onda13_texto", MARKUP, GSS);
    semeia(&mut motor, &[("raio", "7"), ("tracado", "M0 0")]);
    let arvore = motor.evaluated("tela").unwrap();

    let txt = acha(arvore, "text").expect("o <text> do canvas está na árvore");
    match &txt.kind {
        NodeType::Text { content, .. } => {
            assert_eq!(content, "ângulo 7", "o texto interpola como um <text> normal");
        }
        outro => panic!("o <text> do canvas deveria ser NodeType::Text, é {outro:?}"),
    }
    // A posição vem dos atributos universais `x`/`y` (o `pin` do <stack>).
    assert_eq!(txt.pin_x(), Some("4"));
    assert_eq!(txt.pin_y(), Some("30"));
}

/// A ponta-a-ponta: um `<script>` liga um `every()` num handler, cada tique
/// escreve `ponta_x`, e o `<line x2="{ponta_x}">` do `<canvas>` **segue** —
/// prova que a geometria de uma forma reavalia quando a chave muda, mesmo
/// dirigida por um timer do Luau. (O `every` num handler é encaminhado pelo
/// motor; o do `init`, não — só streams.)
#[test]
fn linha_do_canvas_segue_um_every_do_luau() {
    let markup = r#"<screen>
  <resources>
    <script src="onda13_agulha.luau"></script>
  </resources>
  <canvas>
    <line x1="10" y1="10" x2="{ponta_x}" y2="10" />
  </canvas>
  <button text="ir" on_click="alternar" />
</screen>"#;
    // O id do primeiro `after`/`every` de um componente é 1 (`next_id` começa
    // em 1) — é o que este teste retoma à mão, no lugar do `tokio::sleep`.
    std::fs::create_dir_all("templates").ok();
    std::fs::write(
        "templates/onda13_agulha.luau",
        "local x = 10\n\
         local t = nil\n\
         local function passo() x = x + 5 ctx.ponta_x = tostring(x) end\n\
         function alternar() if t then t:cancel() t = nil else t = every(50, passo) end end\n\
         function init() ctx.ponta_x = tostring(x) end\n",
    )
    .unwrap();
    std::fs::write("templates/onda13_agulha.gv", markup).unwrap();

    let mut motor = GlacierUI::new();
    motor
        .register_component("tela", "templates/onda13_agulha.gv")
        .unwrap();
    motor.navigate_to("tela");

    let g0 = geo(acha(motor.evaluated("tela").unwrap(), "line").unwrap(), "x2")
        .unwrap()
        .to_string();
    assert_eq!(g0, "10", "estado inicial de x2 = {{ponta_x}}");

    // Liga o timer (num handler — o motor encaminha).
    let _ = motor.dispatch(&EngineMessage::UiClick("alternar".into()));
    // Simula o disparo do primeiro tique (id 1).
    let _ = motor.dispatch(&EngineMessage::LuauTimer {
        owner: "tela".into(),
        id: 1,
    });

    let g1 = geo(acha(motor.evaluated("tela").unwrap(), "line").unwrap(), "x2")
        .unwrap()
        .to_string();
    assert_eq!(g1, "15", "o tique moveu ponta_x e o <line> seguiu");
}

#[test]
fn whatsthis_liga_e_desliga_a_chave_do_motor() {
    let mut motor = tela(
        "onda13_whatsthis",
        r#"<screen>
  <column>
    <button text="ajuda" on_click="whatsthis:toggle" whats_this="isto é X" />
  </column>
</screen>"#,
        "",
    );

    assert_eq!(motor.context().get("__whatsthis"), None, "começa desligado");

    let _ = motor.dispatch(&EngineMessage::UiClick("whatsthis:toggle".into()));
    assert_eq!(
        motor.context().get("__whatsthis").map(String::as_str),
        Some("1"),
        "toggle liga"
    );

    let _ = motor.dispatch(&EngineMessage::UiClick("whatsthis:off".into()));
    assert_eq!(motor.context().get("__whatsthis"), None, "off desliga");

    // O atributo chegou à árvore avaliada.
    let arvore = motor.evaluated("tela").unwrap();
    let botao = acha(arvore, "button").unwrap();
    assert_eq!(botao.whats_this(), Some("isto é X"));
}
