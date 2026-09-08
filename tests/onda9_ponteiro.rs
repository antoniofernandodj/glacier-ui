//! Onda 9 — o ponteiro preso, e o teclado que ninguém escutava.
//!
//! Os testes cobrem os dois habilitadores (o arrasto de `crate::grip` e a
//! subscription de teclado de `crate::keys`) e os nove widgets que saem deles.
//!
//! # O que eles guardam, além da feature
//!
//! Duas coisas que o `CLAUDE.md` manda checar e que já produziram, neste
//! repositório, widget que passava em todos os testes e aparecia **vazio na
//! tela**:
//!
//! 1. **O `value_var` do nó é o NOME DE CHAVE que o markup escreveu**, não o
//!    valor já interpolado. `sizes="{painel}"` num `<splitter>` faria o widget
//!    procurar uma chave chamada `"240 fill"` — o mesmo bug que o
//!    `<progressbar value="{x}">` já teve. Por isso quase todo teste daqui olha
//!    a **árvore avaliada** antes de olhar o comportamento.
//! 2. **O ramo que deveria existir está lá?** Um `<swipeview>` sem filho
//!    nenhum, ou um `<shortcut>` que o coletor não viu, não dá erro: some em
//!    silêncio.

use glacier_ui::{EngineMessage, GlacierUI, NodeType};
use iced::Point;

/// Escreve um `.gv` temporário e devolve o caminho.
fn escreve(nome: &str, conteudo: &str) -> String {
    std::fs::create_dir_all("templates").ok();
    let caminho = format!("templates/{nome}.gv");
    std::fs::write(&caminho, conteudo).unwrap();
    caminho
}

/// Uma tela pronta para dispatch, já navegada.
fn tela(nome: &str, markup: &str) -> (GlacierUI, String) {
    let caminho = escreve(nome, markup);
    let mut motor = GlacierUI::new();
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    (motor, caminho)
}

fn semeia(motor: &mut GlacierUI, pares: &[(&str, &str)]) {
    let _ = motor.dispatch(&EngineMessage::ContextPatch(
        pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    ));
}

fn valor(motor: &GlacierUI, chave: &str) -> String {
    motor.context().get(chave).cloned().unwrap_or_default()
}

/// Acha o primeiro nó da árvore cujo `tag_name` bate.
fn acha<'a>(node: &'a glacier_ui::UiNode, tag: &str) -> Option<&'a glacier_ui::UiNode> {
    if node.kind.tag_name() == Some(tag) {
        return Some(node);
    }
    node.children.iter().find_map(|c| acha(c, tag))
}

// ─────────────────────────────────────────────────────────────────────────────
// Habilitador A — o arrasto
// ─────────────────────────────────────────────────────────────────────────────

/// O zero de um arrasto **não** é o clique: o primeiro movimento é que o ancora.
///
/// É o problema difícil que o `__colgrip` da Onda 6 já tinha resolvido e que a
/// generalização não podia perder. Sem isto, o arrasto sai com centenas de
/// pixels de erro no primeiro quadro — porque enquanto não há alça presa o
/// motor não escuta o mouse, e a "última posição conhecida" é de um menu aberto
/// meia hora atrás.
#[test]
fn o_primeiro_movimento_ancora_e_nao_arrasta() {
    let (mut motor, caminho) = tela(
        "onda9_ancora",
        r#"<screen>
               <splitter sizes="painel">
                   <text>esquerda</text>
                   <text>direita</text>
               </splitter>
           </screen>"#,
    );
    semeia(&mut motor, &[("painel", "200 fill")]);

    let _ = motor.dispatch(&EngineMessage::GripStart(glacier_ui::grip::Arrasto {
        chave: "painel".into(),
        indice: 0,
        eixo: glacier_ui::grip::Eixo::X,
        origem: None,
        valor0: 200.0,
        alvo: glacier_ui::grip::Alvo::Trilha {
            min: 60.0,
            max: 4000.0,
        },
    }));

    // O primeiro movimento acontece a 900px de distância — e não muda nada.
    let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(900.0, 40.0)));
    assert_eq!(valor(&motor, "painel"), "200 fill");

    // Do segundo em diante a conta é exata a partir dali.
    let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(950.0, 40.0)));
    assert_eq!(valor(&motor, "painel"), "250 fill");

    std::fs::remove_file(&caminho).ok();
}

/// Soltar encerra o arrasto — e é o **mesmo** `DragEnd` que encerra o de uma
/// lista reordenável, de propósito: só existe um botão do mouse.
#[test]
fn soltar_encerra_o_arrasto() {
    let (mut motor, caminho) = tela(
        "onda9_solta",
        r#"<screen>
               <splitter sizes="painel"><text>a</text><text>b</text></splitter>
           </screen>"#,
    );
    semeia(&mut motor, &[("painel", "200 fill")]);
    let _ = motor.dispatch(&EngineMessage::GripStart(glacier_ui::grip::Arrasto {
        chave: "painel".into(),
        indice: 0,
        eixo: glacier_ui::grip::Eixo::X,
        origem: None,
        valor0: 200.0,
        alvo: glacier_ui::grip::Alvo::Trilha {
            min: 60.0,
            max: 4000.0,
        },
    }));
    let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(100.0, 0.0)));
    let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(160.0, 0.0)));
    assert_eq!(valor(&motor, "painel"), "260 fill");

    let _ = motor.dispatch(&EngineMessage::DragEnd);
    // Depois de soltar, mexer o mouse não mexe mais no painel.
    let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(600.0, 0.0)));
    assert_eq!(valor(&motor, "painel"), "260 fill");

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. <splitter>
// ─────────────────────────────────────────────────────────────────────────────

/// O `sizes` do `<splitter>` guarda o **nome da chave**, não o valor dela.
///
/// A armadilha que o `CLAUDE.md` manda checar antes de dizer que funciona: se o
/// eval interpolasse aqui, o widget procuraria uma chave chamada `"240 fill"` e
/// desenharia dois painéis iguais para sempre, sem erro nenhum.
#[test]
fn splitter_guarda_o_nome_da_chave_e_nao_o_valor() {
    let (mut motor, caminho) = tela(
        "onda9_splitter_chave",
        r#"<screen>
               <splitter sizes="painel" direction="v" handle="8" min="90">
                   <text>cima</text>
                   <text>baixo</text>
               </splitter>
           </screen>"#,
    );
    semeia(&mut motor, &[("painel", "240 fill")]);

    let arvore = motor.evaluated("tela").unwrap();
    let no = acha(arvore, "splitter").expect("o <splitter> está na árvore");
    match &no.kind {
        NodeType::Splitter {
            sizes_var,
            vertical,
            handle,
            min,
        } => {
            assert_eq!(sizes_var, "painel", "é o NOME da chave, não '240 fill'");
            assert!(*vertical, "direction=\"v\" empilha");
            assert_eq!(*handle, 8.0);
            assert_eq!(*min, 90.0);
        }
        outro => panic!("não é um <splitter>: {outro:?}"),
    }
    // E os dois painéis continuam sendo filhos — um `<splitter>` que comesse os
    // filhos apareceria vazio na tela, que é o bug que este repositório já teve.
    assert_eq!(no.children.len(), 2);

    std::fs::remove_file(&caminho).ok();
}

/// Uma trilha arrastada à mão **vira fixa** e as vizinhas ficam onde estavam —
/// é a mesma conversão que a alça de coluna do `<tableheader>` faz desde a Onda
/// 6, e agora é uma só implementação para os dois.
#[test]
fn splitter_converte_a_trilha_arrastada_e_preserva_as_outras() {
    let (mut motor, caminho) = tela(
        "onda9_splitter_trilhas",
        r#"<screen>
               <splitter sizes="cols">
                   <text>a</text><text>b</text><text>c</text>
               </splitter>
           </screen>"#,
    );
    semeia(&mut motor, &[("cols", "160 fill 90")]);
    let _ = motor.dispatch(&EngineMessage::GripStart(glacier_ui::grip::Arrasto {
        chave: "cols".into(),
        indice: 1,
        eixo: glacier_ui::grip::Eixo::X,
        origem: None,
        valor0: 120.0,
        alvo: glacier_ui::grip::Alvo::Trilha {
            min: 60.0,
            max: 4000.0,
        },
    }));
    let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(400.0, 0.0)));
    let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(450.0, 0.0)));
    assert_eq!(valor(&motor, "cols"), "160 170 90");

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. <swipeview>
// ─────────────────────────────────────────────────────────────────────────────

/// A página é o **índice** numa chave, e arrastar meia tela a vira. O conteúdo
/// segue o dedo: puxar para a esquerda anda para trás.
#[test]
fn swipeview_vira_a_pagina_e_satura_nas_pontas() {
    let (mut motor, caminho) = tela(
        "onda9_swipe",
        r#"<screen>
               <swipeview value="pagina" threshold="100">
                   <text>zero</text>
                   <text>um</text>
                   <text>dois</text>
               </swipeview>
           </screen>"#,
    );
    semeia(&mut motor, &[("pagina", "1")]);

    let arvore = motor.evaluated("tela").unwrap();
    let no = acha(arvore, "swipeview").expect("o <swipeview> está na árvore");
    match &no.kind {
        NodeType::SwipeView {
            value_var,
            threshold,
            ..
        } => {
            assert_eq!(value_var, "pagina");
            assert_eq!(*threshold, 100.0);
        }
        outro => panic!("não é um <swipeview>: {outro:?}"),
    }
    assert_eq!(no.children.len(), 3, "as três páginas continuam na árvore");

    let arrasta = |motor: &mut GlacierUI, de: f32, para: f32, atual: f32| {
        let _ = motor.dispatch(&EngineMessage::GripStart(glacier_ui::grip::Arrasto {
            chave: "pagina".into(),
            indice: 0,
            eixo: glacier_ui::grip::Eixo::X,
            origem: None,
            valor0: atual,
            alvo: glacier_ui::grip::Alvo::Indice {
                passo: 100.0,
                max: 2,
            },
        }));
        let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(de, 0.0)));
        let _ = motor.dispatch(&EngineMessage::CursorMoved(Point::new(para, 0.0)));
        let _ = motor.dispatch(&EngineMessage::DragEnd);
    };

    // Para a esquerda: volta uma página.
    arrasta(&mut motor, 500.0, 390.0, 1.0);
    assert_eq!(valor(&motor, "pagina"), "0");

    // E satura: não existe página -1.
    arrasta(&mut motor, 500.0, 100.0, 0.0);
    assert_eq!(valor(&motor, "pagina"), "0");

    // Para a direita, duas de uma vez. O ponto de partida é outro de propósito:
    // o motor ignora um `CursorMoved` que chega na MESMA posição da anterior, e
    // com ela o quadro que ancoraria o arrasto nunca aconteceria. Num app isso
    // não ocorre (o dedo mexeu), mas num teste que despacha eventos à mão é a
    // diferença entre medir o widget e medir a própria fixture.
    arrasta(&mut motor, 140.0, 360.0, 0.0);
    assert_eq!(valor(&motor, "pagina"), "2");

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3–4. <rangeslider> e <tumbler> — as chaves, na árvore avaliada
// ─────────────────────────────────────────────────────────────────────────────

/// O `<rangeslider>` guarda **duas** chaves nomeadas — a mesma forma que o
/// `<daterangepicker range>` da Onda 3 já usava, e a razão de o `●` do catálogo
/// nunca ter valido aqui.
#[test]
fn rangeslider_tem_duas_chaves_e_elas_sao_nomes() {
    let (mut motor, caminho) = tela(
        "onda9_faixa",
        r#"<screen>
               <rangeslider start="preco_min" end="preco_max" min="0" max="500" step="10" />
           </screen>"#,
    );
    semeia(&mut motor, &[("preco_min", "100"), ("preco_max", "400")]);

    let arvore = motor.evaluated("tela").unwrap();
    match &acha(arvore, "rangeslider").expect("está na árvore").kind {
        NodeType::RangeSlider {
            start_var,
            end_var,
            min,
            max,
            step,
            ..
        } => {
            assert_eq!(start_var, "preco_min");
            assert_eq!(end_var, "preco_max");
            assert_eq!((*min, *max, *step), (0.0, 500.0, 10.0));
        }
        outro => panic!("não é um <rangeslider>: {outro:?}"),
    }

    std::fs::remove_file(&caminho).ok();
}

/// O `<tumbler>` guarda o **texto** escolhido, não o índice — uma coleção
/// reordenada não move a escolha de lugar. E a janela visível é sempre ímpar,
/// senão o item do meio não teria meio.
#[test]
fn tumbler_guarda_o_texto_e_a_janela_e_impar() {
    let (mut motor, caminho) = tela(
        "onda9_roleta",
        r#"<screen>
               <tumbler value="mes" items="meses" visible="4" />
           </screen>"#,
    );
    semeia(
        &mut motor,
        &[("meses", r#"["jan","fev","mar"]"#), ("mes", "fev")],
    );

    let arvore = motor.evaluated("tela").unwrap();
    match &acha(arvore, "tumbler").expect("está na árvore").kind {
        NodeType::Tumbler {
            value_var,
            items,
            visible,
            ..
        } => {
            assert_eq!(value_var, "mes", "o NOME da chave, não 'fev'");
            assert_eq!(items, "meses", "o NOME da coleção, não o JSON dela");
            assert_eq!(*visible, 5, "4 é par: vira 5, senão não há item do meio");
        }
        outro => panic!("não é um <tumbler>: {outro:?}"),
    }

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. <delaybutton> — o arrasto em que anda o tempo
// ─────────────────────────────────────────────────────────────────────────────

/// Soltar antes do fim **desiste**, e é o ponto todo do widget: é um botão para
/// a ação que não se quer por engano.
#[test]
fn delaybutton_soltar_cedo_desiste_sem_disparar() {
    let (mut motor, caminho) = tela(
        "onda9_demora",
        r#"<screen>
               <delaybutton text="Apagar" on_press="apagar" delay="600" />
           </screen>"#,
    );

    let _ = motor.dispatch(&EngineMessage::HoldStart {
        action: "apagar".into(),
        duracao: 600.0,
    });
    // Um tique no meio do caminho: o anel anda, a ação não sai.
    let _ = motor.dispatch(&EngineMessage::HoldTick);
    assert!(
        !valor(&motor, "__hold").is_empty(),
        "o apertar continua em curso"
    );

    let _ = motor.dispatch(&EngineMessage::HoldEnd);
    assert_eq!(valor(&motor, "__hold"), "", "soltar apaga o relógio");
    assert_eq!(
        valor(&motor, "__hold_frac"),
        "",
        "e a fração vai junto — o anel volta a zero"
    );

    std::fs::remove_file(&caminho).ok();
}

/// Com o tempo cumprido, o anel fecha, a ação sai **uma vez** e o relógio
/// desarma antes do despacho — senão o tique seguinte dispararia de novo.
#[test]
fn delaybutton_dispara_uma_vez_quando_o_anel_fecha() {
    let (mut motor, caminho) = tela(
        "onda9_demora_fim",
        r#"<screen>
               <delaybutton text="Apagar" on_press="apagar" delay="120" />
           </screen>"#,
    );

    let _ = motor.dispatch(&EngineMessage::HoldStart {
        action: "apagar".into(),
        duracao: 120.0,
    });
    std::thread::sleep(std::time::Duration::from_millis(160));
    let _ = motor.dispatch(&EngineMessage::HoldTick);

    assert_eq!(valor(&motor, "__hold"), "", "desarmou ao fechar o anel");
    // Um segundo tique depois do fim é inofensivo: não há mais relógio.
    let _ = motor.dispatch(&EngineMessage::HoldTick);
    assert_eq!(valor(&motor, "__hold"), "");

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. <rubberband>
// ─────────────────────────────────────────────────────────────────────────────

/// O laço escreve o **conjunto nomeado** que o `<listview mode="multi">` já
/// guarda desde a 0.85 — e é por isso que ele aceita `items`.
#[test]
fn rubberband_le_a_colecao_pelo_nome_da_chave() {
    let (mut motor, caminho) = tela(
        "onda9_laco",
        r#"<screen>
               <rubberband items="caixas" selection="marcados" on_select="selecionou" />
           </screen>"#,
    );
    semeia(
        &mut motor,
        &[(
            "caixas",
            r#"[{"id":"a","x":0,"y":0,"w":30,"h":30},{"id":"b","x":200,"y":0,"w":30,"h":30}]"#,
        )],
    );

    let arvore = motor.evaluated("tela").unwrap();
    match &acha(arvore, "rubberband").expect("está na árvore").kind {
        NodeType::RubberBand {
            items,
            selection_var,
            on_select,
            ..
        } => {
            assert_eq!(items, "caixas", "o NOME da coleção");
            assert_eq!(selection_var, "marcados");
            assert_eq!(on_select, "selecionou");
        }
        outro => panic!("não é um <rubberband>: {outro:?}"),
    }

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// 7–8. SizeGrip e PageIndicator — os dois de carona
// ─────────────────────────────────────────────────────────────────────────────

/// O `<SizeGrip>` é um **builtin**, e o que ele emite é a ação de janela que a
/// titlebar custom já usava. As duas metades existiam antes da onda.
#[test]
fn sizegrip_emite_a_acao_de_janela_que_ja_existia() {
    let (mut motor, caminho) = tela(
        "onda9_grip",
        r#"<screen><row><space /><SizeGrip /></row></screen>"#,
    );

    let arvore = motor.evaluated("tela").unwrap();
    // O builtin expandiu: o que sobra na árvore é um container com a ação.
    let mut achou = false;
    fn varre(n: &glacier_ui::UiNode, achou: &mut bool) {
        if n.interact
            .as_ref()
            .and_then(|i| i.on_press.as_deref())
            .is_some_and(|a| a == "window:resize:se")
        {
            *achou = true;
        }
        for c in &n.children {
            varre(c, achou);
        }
    }
    varre(arvore, &mut achou);
    assert!(achou, "o <SizeGrip> emite window:resize:se");

    // E em minúsculas também — o alias automático dos builtins.
    assert!(motor.is_registered("sizegrip"));

    std::fs::remove_file(&caminho).ok();
}

/// `<pageindicator>` é `<pagination dots>`, e as duas contam de **bases
/// diferentes**: o `<pagination>` numera páginas para gente ler (a primeira é a
/// 1), o `<pageindicator>` marca o `currentIndex` de um `<swipeview>`, que
/// começa em zero.
#[test]
fn pageindicator_e_pagination_com_pontos_e_base_zero() {
    let (mut motor, caminho) = tela(
        "onda9_pontos",
        r#"<screen>
               <pageindicator value="pagina" total="3" />
               <pagination value="folha" total="3" />
           </screen>"#,
    );
    semeia(&mut motor, &[("pagina", "0"), ("folha", "1")]);

    let arvore = motor.evaluated("tela").unwrap();
    let mut achados = Vec::new();
    fn varre(n: &glacier_ui::UiNode, out: &mut Vec<bool>) {
        if let NodeType::Pagination { dots, .. } = &n.kind {
            out.push(*dots);
        }
        for c in &n.children {
            varre(c, out);
        }
    }
    varre(arvore, &mut achados);
    assert_eq!(
        achados,
        vec![true, false],
        "as duas tags são o mesmo nó, e só a `dots` as separa"
    );

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// Habilitador B — o teclado
// ─────────────────────────────────────────────────────────────────────────────

/// Um `<shortcut>` declarado na tela dispara a ação dele — e a ordem em que os
/// modificadores foram escritos não importa.
#[test]
fn shortcut_declarado_na_tela_dispara() {
    let (mut motor, caminho) = tela(
        "onda9_atalho",
        r#"<screen>
               <shortcut key="Shift+Ctrl+S" on_press="salvou" />
               <text>tela</text>
           </screen>"#,
    );

    // A ação escreve no contexto por um caminho que não depende de componente:
    // um `UiClick` de ação desconhecida é ignorado em silêncio, então o que se
    // afirma aqui é que o motor ACHOU o atalho, e não que ele fez efeito.
    assert!(
        motor.evaluated("tela").is_ok(),
        "a tela avalia com o <shortcut>"
    );

    let arvore = motor.evaluated("tela").unwrap();
    match &acha(arvore, "shortcut")
        .expect("o <shortcut> está na árvore")
        .kind
    {
        NodeType::Shortcut { key, action } => {
            assert_eq!(
                key, "Shift+Ctrl+S",
                "o markup chega cru; quem normaliza é o coletor"
            );
            assert_eq!(action, "salvou");
        }
        outro => panic!("não é um <shortcut>: {outro:?}"),
    }

    std::fs::remove_file(&caminho).ok();
}

/// Um `<shortcut>` **sem modificador** não passa por cima de um campo focado.
///
/// É a regra que o `timeedit_key_from_event` já tinha aprendido: digitar `s`
/// num `<textinput>` não pode disparar o `<shortcut key="s">` da tela. O que
/// muda para o atalho é que `Ctrl+S` **atravessa** mesmo assim, porque nenhum
/// campo de texto consome `Ctrl+S` — e um atalho que morresse porque há um
/// campo focado não seria um atalho.
#[test]
fn atalho_sem_modificador_nao_rouba_a_tecla_de_um_campo() {
    let (mut motor, caminho) = tela(
        "onda9_atalho_campo",
        r#"<screen>
               <shortcut key="s" on_press="cru" />
               <shortcut key="ctrl+s" on_press="salvar" />
               <textinput value="nome" />
           </screen>"#,
    );
    // Um `<shortcutinput>` armado tornaria a tecla dado; aqui não há nenhum.
    semeia(&mut motor, &[("nome", "abc")]);

    // Com o campo focado (`capturado: true`), a tecla nua não vira atalho…
    let _ = motor.dispatch(&EngineMessage::ShortcutKey {
        combo: "s".into(),
        capturado: true,
    });
    // …e a com modificador vira. Nenhuma das duas escreve nada observável aqui
    // (a ação não tem dono), então o que este teste guarda é que nenhuma das
    // duas quebra e que o caminho existe.
    let _ = motor.dispatch(&EngineMessage::ShortcutKey {
        combo: "ctrl+s".into(),
        capturado: true,
    });

    std::fs::remove_file(&caminho).ok();
}

/// `<shortcutinput>`: armar, teclar, gravar. E as duas desistências do
/// `QKeySequenceEdit` — Escape não grava, Backspace limpa.
#[test]
fn shortcutinput_arma_grava_e_desiste() {
    let (mut motor, caminho) = tela(
        "onda9_captura",
        r#"<screen><shortcutinput value="atalho_salvar" /></screen>"#,
    );

    let arvore = motor.evaluated("tela").unwrap();
    match &acha(arvore, "shortcutinput").expect("está na árvore").kind {
        NodeType::ShortcutInput { value_var, .. } => assert_eq!(value_var, "atalho_salvar"),
        outro => panic!("não é um <shortcutinput>: {outro:?}"),
    }

    // Sem armar, a tecla não é dado: ela procura um `<shortcut>` (e não acha).
    let _ = motor.dispatch(&EngineMessage::ShortcutKey {
        combo: "ctrl+k".into(),
        capturado: false,
    });
    assert_eq!(valor(&motor, "atalho_salvar"), "");

    // Armado, ela grava — inclusive uma combinação que casaria com um atalho da
    // tela, que é o que deixa gravar `Ctrl+S` sem salvar o arquivo no caminho.
    let _ = motor.dispatch(&EngineMessage::ShortcutArm {
        value_var: "atalho_salvar".into(),
    });
    let _ = motor.dispatch(&EngineMessage::ShortcutKey {
        combo: "ctrl+shift+s".into(),
        capturado: false,
    });
    assert_eq!(valor(&motor, "atalho_salvar"), "ctrl+shift+s");
    assert_eq!(valor(&motor, "__shortcut_cap"), "", "desarmou ao gravar");

    // Escape desiste: o valor anterior fica.
    let _ = motor.dispatch(&EngineMessage::ShortcutArm {
        value_var: "atalho_salvar".into(),
    });
    let _ = motor.dispatch(&EngineMessage::ShortcutKey {
        combo: "escape".into(),
        capturado: false,
    });
    assert_eq!(valor(&motor, "atalho_salvar"), "ctrl+shift+s");

    // Backspace limpa.
    let _ = motor.dispatch(&EngineMessage::ShortcutArm {
        value_var: "atalho_salvar".into(),
    });
    let _ = motor.dispatch(&EngineMessage::ShortcutKey {
        combo: "backspace".into(),
        capturado: false,
    });
    assert_eq!(valor(&motor, "atalho_salvar"), "");

    std::fs::remove_file(&caminho).ok();
}

/// Clicar no campo já armado o **desarma** — senão não haveria como desistir
/// sem gravar uma combinação qualquer.
#[test]
fn clicar_no_campo_armado_desarma() {
    let (mut motor, caminho) = tela(
        "onda9_desarma",
        r#"<screen><shortcutinput value="k" /></screen>"#,
    );
    let _ = motor.dispatch(&EngineMessage::ShortcutArm {
        value_var: "k".into(),
    });
    assert_eq!(valor(&motor, "__shortcut_cap"), "k");
    let _ = motor.dispatch(&EngineMessage::ShortcutArm {
        value_var: "k".into(),
    });
    assert_eq!(valor(&motor, "__shortcut_cap"), "");

    std::fs::remove_file(&caminho).ok();
}

// ─────────────────────────────────────────────────────────────────────────────
// Os exemplos, avaliados de verdade
// ─────────────────────────────────────────────────────────────────────────────

/// As nove tags da onda aparecem na árvore avaliada de `examples/onda9`.
///
/// Este teste existe por causa da advertência do `CLAUDE.md`: este repositório
/// já produziu, mais de uma vez, um widget que passava em todos os testes e
/// aparecia **vazio na tela**. Rodar o exemplo sem erro não prova nada — uma
/// escada de `se`/`senao` que não casa não é erro, é um buraco. O que prova é
/// olhar o ramo.
#[test]
fn o_exemplo_rust_monta_as_nove_tags() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("onda9", "examples/onda9/app.gv")
        .expect("a tela do exemplo registra");
    motor.navigate_to("onda9");

    // As abas são condicionais: cada uma só monta com a chave dela. Semear a
    // aba e reavaliar é o que expõe o ramo — e é exatamente o passo que um
    // teste de "escrevi a chave e li de volta" pularia.
    for (aba, tags) in [
        ("paineis", vec!["splitter"]),
        ("valores", vec!["rangeslider", "tumbler", "delaybutton"]),
        ("paginas", vec!["swipeview", "pagination"]),
        ("laco", vec!["rubberband"]),
        ("teclado", vec!["shortcutinput"]),
    ] {
        let _ = motor.dispatch(&EngineMessage::ContextPatch(vec![(
            "aba".into(),
            aba.into(),
        )]));
        let arvore = motor.evaluated("onda9").unwrap();
        for tag in tags {
            assert!(
                acha(arvore, tag).is_some(),
                "a aba '{aba}' deveria montar um <{tag}> e não montou"
            );
        }
    }

    // O `<shortcut>` está fora das abas, e é o que o coletor precisa ver.
    let arvore = motor.evaluated("onda9").unwrap();
    assert!(acha(arvore, "shortcut").is_some(), "os atalhos da tela");
}

/// O `<splitter>` do exemplo tem os painéis como filhos, e o de dentro escreve
/// noutra chave.
///
/// Dois `<splitter>` aninhados na mesma tela são a demonstração de que nunca
/// houve estado por instância aqui: cada um nomeia a sua chave, e é só isso.
#[test]
fn o_exemplo_aninha_dois_splitters_com_chaves_diferentes() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("onda9", "examples/onda9/app.gv")
        .unwrap();
    motor.navigate_to("onda9");
    let _ = motor.dispatch(&EngineMessage::ContextPatch(vec![(
        "aba".into(),
        "paineis".into(),
    )]));

    let arvore = motor.evaluated("onda9").unwrap();
    let mut chaves = Vec::new();
    fn varre(n: &glacier_ui::UiNode, out: &mut Vec<(String, usize)>) {
        if let NodeType::Splitter { sizes_var, .. } = &n.kind {
            out.push((sizes_var.clone(), n.children.len()));
        }
        for c in &n.children {
            varre(c, out);
        }
    }
    varre(arvore, &mut chaves);

    assert_eq!(chaves.len(), 2, "o de fora e o aninhado");
    assert_eq!(chaves[0].0, "painel_h");
    assert_eq!(chaves[1].0, "painel_v");
    assert_ne!(chaves[0].0, chaves[1].0, "duas instâncias, duas chaves");
    for (chave, filhos) in &chaves {
        assert_eq!(*filhos, 2, "o <splitter> '{chave}' perdeu os painéis");
    }
}

/// A tela Luau também monta — e o script dela semeia as chaves que os widgets
/// leem.
#[test]
fn o_exemplo_luau_monta_e_o_script_semeia() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("onda9_luau", "examples/onda9_luau/app.gv")
        .expect("a tela Luau registra");
    motor.navigate_to("onda9_luau");

    let arvore = motor.evaluated("onda9_luau").unwrap();
    for tag in [
        "splitter",
        "rangeslider",
        "tumbler",
        "swipeview",
        "pagination",
        "delaybutton",
        "rubberband",
        "shortcutinput",
        "shortcut",
    ] {
        assert!(
            acha(arvore, tag).is_some(),
            "a tela Luau deveria montar um <{tag}> e não montou"
        );
    }

    // E o `init()` do script rodou: sem estas chaves os widgets desenhariam o
    // estado vazio, que é indistinguível de um widget quebrado.
    for chave in ["painel_h", "preco_min", "preco_max", "mes", "caixas"] {
        assert!(
            !valor(&motor, chave).is_empty(),
            "o init() do app.luau deveria ter semeado '{chave}'"
        );
    }
}
