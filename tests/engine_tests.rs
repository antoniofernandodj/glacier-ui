use glacier_ui::{GlacierUI, NodeType, UiNode};

#[test]
fn test_parser_basic() {
    let xml = r##"
    <Container padding="15" width="200" background="#FFFFFF">
        <Column spacing="10">
            <Text content="Hello World" size="20" bold="true" />
            <Button text="Click Me" on_click="btn_click" />
        </Column>
    </Container>
    "##;

    let ast = UiNode::parse_xml(xml).unwrap();

    assert_eq!(ast.kind, NodeType::Container);
    assert_eq!(ast.padding.as_deref(), Some("15"));
    assert_eq!(ast.width.as_deref(), Some("200"));
    assert_eq!(ast.background.as_deref(), Some("#FFFFFF"));

    assert_eq!(ast.children.len(), 1);
    let column = &ast.children[0];
    assert_eq!(column.kind, NodeType::Column);
    assert_eq!(column.spacing, Some(10.0));

    assert_eq!(column.children.len(), 2);

    let text = &column.children[0];
    if let NodeType::Text {
        content,
        size,
        bold,
        ..
    } = &text.kind
    {
        assert_eq!(content, "Hello World");
        assert_eq!(*size, Some(20.0));
        assert!(bold);
    } else {
        panic!("First child of Column should be Text");
    }

    let button = &column.children[1];
    if let NodeType::Button { text, on_click, .. } = &button.kind {
        assert_eq!(text, "Click Me");
        assert_eq!(on_click.as_deref(), Some("btn_click"));
    } else {
        panic!("Second child of Column should be Button");
    }
}

#[test]
fn test_interpolation() {
    let mut motor = GlacierUI::new();

    let temp_xml_path = "templates/test_temp.gv";
    std::fs::create_dir_all("templates").ok();
    std::fs::write(
        temp_xml_path,
        envolve(r##"<Text content="Welcome, {user_name}! Role: {user_role}" />"##),
    )
    .unwrap();

    motor
        .register_component("test_comp", temp_xml_path)
        .unwrap();

    motor.define_data("user_name", "Bob");
    motor.define_data("user_role", "Admin");

    let evaluated = motor.evaluated("test_comp").unwrap();
    if let NodeType::Text { content, .. } = &evaluated.kind {
        assert_eq!(content, "Welcome, Bob! Role: Admin");
    } else {
        panic!("Root node should be evaluated Text");
    }

    std::fs::remove_file(temp_xml_path).ok();
}

#[test]
fn test_includes() {
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();

    let main_path = "templates/test_main.gv";
    let card_path = "templates/test_card.gv";

    std::fs::write(
        card_path,
        envolve(r##"<Container background="#222"><Text content="User: {name}" /></Container>"##),
    )
    .unwrap();

    std::fs::write(
        main_path,
        envolve(
            r##"
        <Column>
            <Include src="test_card" name="Alice" />
            <Include src="test_card" name="Charlie" />
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("test_card", card_path).unwrap();
    motor.register_component("test_main", main_path).unwrap();

    let evaluated = motor.evaluated("test_main").unwrap();
    assert_eq!(evaluated.kind, NodeType::Column);
    assert_eq!(evaluated.children.len(), 2);

    let first_child = &evaluated.children[0];
    assert_eq!(first_child.kind, NodeType::Container);
    if let NodeType::Text { content, .. } = &first_child.children[0].kind {
        assert_eq!(content, "User: Alice");
    } else {
        panic!("Included first child should contain text 'User: Alice'");
    }

    let second_child = &evaluated.children[1];
    if let NodeType::Text { content, .. } = &second_child.children[0].kind {
        assert_eq!(content, "User: Charlie");
    } else {
        panic!("Included second child should contain text 'User: Charlie'");
    }

    std::fs::remove_file(main_path).ok();
    std::fs::remove_file(card_path).ok();
}

#[test]
fn test_if_else() {
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let path = "templates/test_if.gv";
    std::fs::write(
        path,
        envolve(
            r##"
        <Column>
            <if cond="{logado}">
                <Text content="Olá, {usuario}" />
            </if>
            <else>
                <Text content="Entre, por favor" />
            </else>
            <if cond="{papel}" equals="admin">
                <Text content="painel admin" />
            </if>
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("cond", path).unwrap();

    // Estado inicial: deslogado, papel comum.
    motor.define_data("logado", "false");
    motor.define_data("usuario", "Ana");
    motor.define_data("papel", "user");

    let ev = motor.evaluated("cond").unwrap();
    assert_eq!(ev.children.len(), 1, "só o ramo else deve aparecer");
    if let NodeType::Text { content, .. } = &ev.children[0].kind {
        assert_eq!(content, "Entre, por favor");
    } else {
        panic!("esperava o Text do else");
    }

    // Loga como admin: ramo if + comparação equals=admin.
    motor.define_data("logado", "true");
    motor.define_data("papel", "admin");

    let ev = motor.evaluated("cond").unwrap();
    assert_eq!(ev.children.len(), 2, "ramo if verdadeiro + bloco admin");
    if let NodeType::Text { content, .. } = &ev.children[0].kind {
        assert_eq!(content, "Olá, Ana");
    } else {
        panic!("esperava o Text do if");
    }
    if let NodeType::Text { content, .. } = &ev.children[1].kind {
        assert_eq!(content, "painel admin");
    } else {
        panic!("esperava o Text do bloco admin");
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn test_import_recursivo() {
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();

    let main_path = "templates/test_imp_main.gv";
    let card_path = "templates/test_imp_card.gv";
    let badge_path = "templates/test_imp_badge.gv";

    // badge: folha, sem imports.
    std::fs::write(badge_path, envolve(r##"<Text content="[{label}]" />"##)).unwrap();

    // card: importa badge e o usa pelo nome.
    std::fs::write(
        card_path,
        envolve(
            r##"<import name="Badge" from="templates/test_imp_badge.gv" />
        <Container background="#222">
            <Column>
                <Text content="User: {name}" />
                <Badge label="ok" />
            </Column>
        </Container>"##,
        ),
    )
    .unwrap();

    // main: importa card (que por sua vez importa badge — recursivo).
    std::fs::write(
        main_path,
        envolve(
            r##"<import name="Card" from="templates/test_imp_card.gv" />
        <Column>
            <Card name="Alice" />
        </Column>"##,
        ),
    )
    .unwrap();

    // Apenas o componente de entrada é registrado.
    motor.register_component("main", main_path).unwrap();

    // Os imports recursivos devem ter sido carregados automaticamente.
    assert!(
        motor.is_registered("Card"),
        "Card deveria ter sido importado"
    );
    assert!(
        motor.is_registered("Badge"),
        "Badge deveria ter sido importado recursivamente"
    );

    let evaluated = motor.evaluated("main").unwrap();
    assert_eq!(evaluated.kind, NodeType::Column);
    // O Card expande para um Container; o import declarado não deve virar filho visível.
    assert_eq!(evaluated.children.len(), 1);
    let card = &evaluated.children[0];
    assert_eq!(card.kind, NodeType::Container);

    let inner_col = &card.children[0];
    assert_eq!(inner_col.kind, NodeType::Column);
    // Column interna: Text "User: Alice" + Badge expandido para Text "[ok]".
    assert_eq!(inner_col.children.len(), 2);
    if let NodeType::Text { content, .. } = &inner_col.children[0].kind {
        assert_eq!(content, "User: Alice");
    } else {
        panic!("Esperava Text 'User: Alice'");
    }
    if let NodeType::Text { content, .. } = &inner_col.children[1].kind {
        assert_eq!(content, "[ok]");
    } else {
        panic!("Esperava Badge expandido em Text '[ok]'");
    }

    std::fs::remove_file(main_path).ok();
    std::fs::remove_file(card_path).ok();
    std::fs::remove_file(badge_path).ok();
}

#[test]
fn test_componente_por_nome() {
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();

    let main_path = "templates/test_main_comp.gv";
    let card_path = "templates/test_card_comp.gv";

    std::fs::write(
        card_path,
        envolve(r##"<Container background="#222"><Text content="User: {name}" /></Container>"##),
    )
    .unwrap();

    // Reuse via the component's own tag name instead of <Include>
    std::fs::write(
        main_path,
        envolve(
            r##"
        <Column>
            <UserCard name="Alice" />
            <UserCard name="Charlie" />
        </Column>
        "##,
        ),
    )
    .unwrap();

    // The registered name must match the tag used in the XML.
    motor.register_component("UserCard", card_path).unwrap();
    motor
        .register_component("test_main_comp", main_path)
        .unwrap();

    let evaluated = motor.evaluated("test_main_comp").unwrap();
    assert_eq!(evaluated.kind, NodeType::Column);
    assert_eq!(evaluated.children.len(), 2);

    let first_child = &evaluated.children[0];
    assert_eq!(first_child.kind, NodeType::Container);
    if let NodeType::Text { content, .. } = &first_child.children[0].kind {
        assert_eq!(content, "User: Alice");
    } else {
        panic!("Component first child should contain text 'User: Alice'");
    }

    if let NodeType::Text { content, .. } = &evaluated.children[1].children[0].kind {
        assert_eq!(content, "User: Charlie");
    } else {
        panic!("Component second child should contain text 'User: Charlie'");
    }

    std::fs::remove_file(main_path).ok();
    std::fs::remove_file(card_path).ok();
}

#[test]
fn test_builtin_badge_disponivel_sem_registro() {
    // O app NÃO registra `Badge` — a lib já o registrou sozinha em `new()`.
    // Uma tela pode referenciá-lo por tag e ele resolve, com default e com
    // props sobrescrevendo por instância.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_builtin_badge.gv";
    std::fs::write(
        tela_path,
        envolve(
            r##"
        <Column>
            <Badge />
            <Badge badge_text="Novo" badge_bg="#A6E3A1" />
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("tela_badge", tela_path).unwrap();

    let evaluated = motor.evaluated("tela_badge").unwrap();
    assert_eq!(evaluated.kind, NodeType::Column);
    assert_eq!(evaluated.children.len(), 2);

    // 1º Badge: sem props -> defaults inline (`{prop|default}`), sem estado global.
    let padrao = &evaluated.children[0];
    assert_eq!(padrao.kind, NodeType::Container);
    assert_eq!(padrao.background.as_deref(), Some("#89B4FA"));
    match &padrao.children[0].kind {
        NodeType::Text {
            content,
            color,
            size,
            ..
        } => {
            assert_eq!(content, "Badge");
            assert_eq!(color.as_deref(), Some("#11111B"));
            assert_eq!(*size, Some(13.0)); // default numérico templado
        }
        _ => panic!("Badge padrão deveria conter um Text"),
    }

    // 2º Badge: props sobrescrevem por instância; a omitida (`badge_fg`) mantém o default.
    let custom = &evaluated.children[1];
    assert_eq!(custom.background.as_deref(), Some("#A6E3A1"));
    match &custom.children[0].kind {
        NodeType::Text { content, color, .. } => {
            assert_eq!(content, "Novo");
            assert_eq!(color.as_deref(), Some("#11111B"));
        }
        _ => panic!("Badge custom deveria conter um Text"),
    }

    // O contexto global NÃO foi poluído com defaults (chaves `badge_*`).
    assert!(!motor.context().contains_key("badge_text"));
    assert!(!motor.context().contains_key("badge_bg"));

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_builtin_spinbox_soma_satura_e_nao_colide() {
    // `SpinBox` é o primeiro builtin com comportamento próprio: o `update` dele
    // faz a aritmética. Como o `ctx` é global, a chave-alvo vem por prop e viaja
    // DENTRO da ação (`inc:qtd|1|3|1`) — é isso que deixa duas instâncias na
    // mesma tela independentes. O teste percorre o caminho inteiro: template
    // avaliado -> ação namespaceada -> `dispatch` -> contexto.
    use glacier_ui::EngineMessage;

    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_builtin_spinbox.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <Column>
            <SpinBox value="qtd" min="1" max="3" />
            <SpinBox value="preco" min="0" max="1" step="0.25" width="90" />
        </Column>
        "#,
        ),
    )
    .unwrap();

    motor.register_component("tela_spin", tela_path).unwrap();

    // --- o markup gerado -----------------------------------------------------
    let avaliado = motor.evaluated("tela_spin").unwrap();
    let qtd = &avaliado.children[0];
    assert_eq!(qtd.kind, NodeType::Row);
    // Forma `stacked` (o default): o campo e, colado nele, a coluna com os dois
    // degraus. O `<style>` do template não conta como filho (declaração, não
    // layout) e o `<template if>` não deixa embrulho.
    assert_eq!(qtd.children.len(), 2, "campo + coluna dos degraus");

    match &qtd.children[0].kind {
        NodeType::TextInput {
            value_var,
            on_change,
            ..
        } => {
            assert_eq!(value_var, "qtd", "o campo escreve na chave do app");
            assert_eq!(on_change, "SpinBox::edit:qtd|1|3|1");
        }
        outro => panic!("esperava o campo, veio {outro:?}"),
    }
    assert_eq!(qtd.children[0].width.as_deref(), Some("72")); // default inline

    let degraus = &qtd.children[1];
    assert_eq!(degraus.kind, NodeType::Column);
    assert_eq!(degraus.children.len(), 2, "▴ em cima, ▾ embaixo");
    // O glifo é filho do botão (é ele que carrega o `size`), não o `text=`.
    assert_eq!(
        degraus.children[0].children[0].kind,
        NodeType::Text {
            content: "▴".to_string(),
            size: Some(11.0),
            bold: false,
            color: None,
        }
    );
    // A ação sai prefixada com o dono (`namespace_action`), que é o que faz o
    // motor devolvê-la ao `update` do próprio SpinBox e não à tela.
    let acao_inc = match &degraus.children[0].kind {
        NodeType::Button { on_click, .. } => on_click.clone().expect("degrau ▴ sem ação"),
        outro => panic!("esperava o degrau ▴, veio {outro:?}"),
    };
    assert_eq!(acao_inc, "SpinBox::inc:qtd|1|3|1");

    // --- a aritmética --------------------------------------------------------
    // Chave ainda vazia: o primeiro clique inicializa no mínimo (não min+step).
    let _ = motor.dispatch(&EngineMessage::UiClick(acao_inc.clone()));
    assert_eq!(motor.context().get("qtd").map(String::as_str), Some("1"));

    let _ = motor.dispatch(&EngineMessage::UiClick(acao_inc.clone()));
    let _ = motor.dispatch(&EngineMessage::UiClick(acao_inc.clone()));
    assert_eq!(motor.context().get("qtd").map(String::as_str), Some("3"));

    // No teto, clicar de novo não passa do `max`.
    let _ = motor.dispatch(&EngineMessage::UiClick(acao_inc.clone()));
    assert_eq!(motor.context().get("qtd").map(String::as_str), Some("3"));

    // --- a segunda instância: casas decimais do `step`, e sem colisão ---------
    let preco = &motor.evaluated("tela_spin").unwrap().children[1];
    let inc_preco = match &preco.children[1].children[0].kind {
        NodeType::Button { on_click, .. } => on_click.clone().unwrap(),
        outro => panic!("esperava o degrau ▴ do preço, veio {outro:?}"),
    };
    assert_eq!(inc_preco, "SpinBox::inc:preco|0|1|0.25");

    let _ = motor.dispatch(&EngineMessage::UiClick(inc_preco.clone()));
    // `step="0.25"` -> 2 casas, inclusive na inicialização.
    assert_eq!(
        motor.context().get("preco").map(String::as_str),
        Some("0.00")
    );
    let _ = motor.dispatch(&EngineMessage::UiClick(inc_preco.clone()));
    assert_eq!(
        motor.context().get("preco").map(String::as_str),
        Some("0.25")
    );
    // Soma de f64 sem lixo de ponto flutuante: 0.25*3 formata como "0.75".
    let _ = motor.dispatch(&EngineMessage::UiClick(inc_preco.clone()));
    let _ = motor.dispatch(&EngineMessage::UiClick(inc_preco.clone()));
    assert_eq!(
        motor.context().get("preco").map(String::as_str),
        Some("0.75")
    );

    // A outra instância não se mexeu — o ponto da chave vir por prop.
    assert_eq!(motor.context().get("qtd").map(String::as_str), Some("3"));

    // --- digitação -----------------------------------------------------------
    // O `edit` filtra o que não é número e escreve na chave (sem saturar,
    // como o QSpinBox, que só valida ao terminar a edição).
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "SpinBox::edit:qtd|1|3|1".into(),
        value: "12a.5x".into(),
    });
    assert_eq!(motor.context().get("qtd").map(String::as_str), Some("12.5"));
    // E o clique seguinte satura o que a digitação deixou fora da faixa.
    let _ = motor.dispatch(&EngineMessage::UiClick(acao_inc));
    assert_eq!(motor.context().get("qtd").map(String::as_str), Some("3"));

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_primitivas_onda1_slider_space_radio() {
    // As três primitivas da onda 1 (`PLANO_WIDGETS.md` §6). O `Slider` e o
    // `Radio` seguem o contrato do `<TextInput>`/`<Checkbox>`: disparam a ação
    // com o valor novo e NÃO gravam a chave — quem grava é o app.
    let mut motor = GlacierUI::new();
    motor.define_data("volume", "42");
    motor.define_data("plano", "pro");

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_onda1_primitivas.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <Column>
            <Slider value="volume" min="0" max="100" step="5" onChange="ajustar" />
            <Space />
            <Radio label="Free" value="free" group="plano" onChange="escolher" />
            <Radio label="Pro" value="pro" group="plano" onChange="escolher" />
        </Column>
        "#,
        ),
    )
    .unwrap();
    motor.register_component("tela_onda1", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_onda1").unwrap();
    assert_eq!(avaliado.children.len(), 4);

    match &avaliado.children[0].kind {
        NodeType::Slider {
            value_var,
            on_change,
            min,
            max,
            step,
            step_raw,
            vertical,
            ..
        } => {
            assert_eq!(value_var, "volume");
            assert_eq!(on_change, "ajustar", "a ação é da tela, não namespaceada");
            assert_eq!((*min, *max, *step), (0.0, 100.0, 5.0));
            // O texto cru do step sobrevive ao parse: é dele que saem as casas
            // decimais da saída (ver `NodeType::Slider`).
            assert_eq!(step_raw, "5");
            assert!(!*vertical);
        }
        outro => panic!("esperava o Slider, veio {outro:?}"),
    }
    assert_eq!(avaliado.children[1].kind, NodeType::Space);

    // `group` é o NOME da chave, não o valor — a mesma convenção do `checked=`
    // do `<Checkbox>`. Quem compara é o render (`ctx.get(group) == value`).
    // Interpolar aqui (`group="{plano}"`) faria a busca cair numa chave chamada
    // "free", que não existe, e o grupo inteiro apareceria desmarcado.
    match (&avaliado.children[2].kind, &avaliado.children[3].kind) {
        (
            NodeType::Radio {
                value: v_free,
                group_var: g_free,
                ..
            },
            NodeType::Radio {
                value: v_pro,
                group_var: g_pro,
                ..
            },
        ) => {
            assert_eq!((v_free.as_str(), g_free.as_str()), ("free", "plano"));
            assert_eq!((v_pro.as_str(), g_pro.as_str()), ("pro", "plano"));
        }
        outros => panic!("esperava os dois Radio, veio {outros:?}"),
    }

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_builtin_radiogroup_escreve_a_chave_do_app() {
    // Ao contrário da primitiva `<Radio>`, o builtin grava a chave sozinho —
    // padrão do `SpinBox`: a chave vem por prop e viaja dentro da ação.
    use glacier_ui::EngineMessage;

    let mut motor = GlacierUI::new();
    motor.define_data(
        "planos",
        r#"[{"id":"free","label":"Grátis"},{"id":"pro","label":"Pro"}]"#,
    );
    motor.define_data("plano", "free");

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_builtin_radiogroup.gv";
    std::fs::write(
        tela_path,
        envolve(r#"<RadioGroup value="plano" items="planos" />"#),
    )
    .unwrap();
    motor.register_component("tela_rg", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_rg").unwrap();
    // Column (a raiz do builtin) com uma opção por item da coleção.
    assert_eq!(avaliado.kind, NodeType::Column);
    assert_eq!(avaliado.children.len(), 2);

    let acao_pro = match &avaliado.children[1].kind {
        NodeType::Radio {
            on_change,
            value,
            group_var,
            ..
        } => {
            assert_eq!(value, "pro");
            // O builtin repassa o NOME da chave para a primitiva, que resolve a
            // marcação sozinha — é por isso que ele não precisa de um `active`
            // como o `TabBar` (ver a docstring do RadioGroup).
            assert_eq!(group_var, "plano");
            on_change.clone()
        }
        outro => panic!("esperava a opção Pro, veio {outro:?}"),
    };
    assert_eq!(acao_pro, "RadioGroup::pick:plano|pro");

    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: acao_pro,
        value: "pro".into(),
    });
    assert_eq!(
        motor.context().get("plano").map(String::as_str),
        Some("pro")
    );

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_link_import_sobrescreve_builtin_de_mesmo_nome() {
    // A regra é "registro explícito do app vence o builtin", e ela valia para o
    // `<import>` mas NÃO para o `<link rel="import" as="…">`, que só checava se
    // o nome estava livre. Ficou invisível enquanto os builtins se chamavam
    // `Badge`/`SpinBox`/`TimePicker`; com `Card`/`Frame`/`Avatar` na biblioteca,
    // um app que importasse o próprio `Card` via `<link>` era silenciosamente
    // ignorado e via o builtin renderizar no lugar dele.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let meu_card = "templates/test_meu_card.gv";
    std::fs::write(
        meu_card,
        r#"<component><Text content="CARD DO APP" /></component>"#,
    )
    .unwrap();

    let tela_path = "templates/test_link_import_card.gv";
    std::fs::write(
        tela_path,
        format!(
            r#"<screen title="T">
                <resources><link rel="import" href="{meu_card}" as="Card" /></resources>
                <Column><Card /></Column>
            </screen>"#
        ),
    )
    .unwrap();
    motor.register_component("tela_card", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_card").unwrap();
    match &avaliado.children[0].kind {
        NodeType::Text { content, .. } => assert_eq!(
            content, "CARD DO APP",
            "o import do app tem de vencer o builtin `Card`"
        ),
        outro => panic!("veio o builtin no lugar do componente do app: {outro:?}"),
    }

    std::fs::remove_file(tela_path).ok();
    std::fs::remove_file(meu_card).ok();
}

#[test]
fn test_builtin_em_minusculas_resolve_e_roteia() {
    // Toda primitiva sempre aceitou a grafia minúscula (o `match` de tags lista
    // as variantes à mão); um builtin resolve por igualdade exata de nome, e por
    // isso precisa de um segundo registro — ver `builtins::builtin_aliases`.
    // O teste cobre as duas metades: a tag resolve, E a ação que ela produz
    // (namespaceada com o nome minúsculo) chega no `update` do widget.
    use glacier_ui::EngineMessage;

    let mut motor = GlacierUI::new();
    motor.define_data("qtd", "1");

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_builtin_minusculo.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <Column>
            <groupbox title="Grupo">
                <spinbox value="qtd" min="1" max="3" />
            </groupbox>
            <avatar initials="AF" />
        </Column>
        "#,
        ),
    )
    .unwrap();
    motor.register_component("tela_min", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_min").unwrap();
    assert_eq!(
        avaliado.children.len(),
        2,
        "o groupbox e o avatar resolveram"
    );

    // A ação sai prefixada com o nome pelo qual a tag foi resolvida.
    let inc = encontra_acao(avaliado).expect("o degrau ▴ do spinbox minúsculo");
    assert_eq!(inc, "spinbox::inc:qtd|1|3|1");

    // E o roteamento acha o alias no mapa de componentes e delega ao widget.
    let _ = motor.dispatch(&EngineMessage::UiClick(inc));
    assert_eq!(motor.context().get("qtd").map(String::as_str), Some("2"));

    std::fs::remove_file(tela_path).ok();
}

/// Primeira ação de botão encontrada na subárvore, em ordem de documento.
fn encontra_acao(no: &UiNode) -> Option<String> {
    if let NodeType::Button {
        on_click: Some(a), ..
    } = &no.kind
        && !a.is_empty()
    {
        return Some(a.clone());
    }
    no.children.iter().find_map(encontra_acao)
}

#[test]
fn test_slot_conteudo_do_uso_pertence_a_quem_escreveu() {
    // O `<slot/>` (0.65) é o que destrancou a família dos recipientes. A
    // garantia que o faz valer a pena não é "o conteúdo aparece" — é DE QUEM
    // ele é: avaliado no contexto e com o dono de quem escreveu, não do
    // componente que o embrulha. Sem isso, `on_click="salvar"` viraria
    // `GroupBox::salvar` e morreria no `update` do widget.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_slot_dono.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <GroupBox title="Rede">
            <Button text="Salvar" on_click="salvar" />
            <Text content="{host}" />
        </GroupBox>
        "#,
        ),
    )
    .unwrap();
    motor.define_data("host", "127.0.0.1");
    motor.register_component("tela_slot", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_slot").unwrap();
    // `GroupBox` = Column[ título, Container[ Column[ …conteúdo… ] ] ].
    assert_eq!(avaliado.kind, NodeType::Column);
    let moldura = avaliado
        .children
        .iter()
        .find(|c| c.kind == NodeType::Container)
        .expect("a moldura do GroupBox");
    let corpo = &moldura.children[0];
    assert_eq!(corpo.kind, NodeType::Column);
    assert_eq!(corpo.children.len(), 2, "os dois filhos do uso");

    match &corpo.children[0].kind {
        NodeType::Button { on_click, .. } => assert_eq!(
            on_click.as_deref(),
            Some("salvar"),
            "a ação é da TELA — não pode virar `GroupBox::salvar`"
        ),
        outro => panic!("esperava o botão do conteúdo, veio {outro:?}"),
    }
    // E o conteúdo enxerga o contexto de quem o escreveu, não o do componente.
    match &corpo.children[1].kind {
        NodeType::Text { content, .. } => assert_eq!(content, "127.0.0.1"),
        outro => panic!("esperava o texto do conteúdo, veio {outro:?}"),
    }

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_slot_nomeado_reparte_o_conteudo_por_destino() {
    // Um slot anônimo não bastava para um widget com mais de uma região. Com
    // `slot="footer"` no uso e `<slot name="footer"/>` no template, o conteúdo
    // é repartido — e o que não foi etiquetado continua indo para o anônimo.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_slot_nomeado.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <Card title="Servidor">
            <Text content="corpo A" />
            <template slot="footer"><Button text="Reiniciar" on_click="reiniciar" /></template>
            <Text content="corpo B" />
        </Card>
        "#,
        ),
    )
    .unwrap();
    motor
        .register_component("tela_slot_nom", tela_path)
        .unwrap();

    let avaliado = motor.evaluated("tela_slot_nom").unwrap();
    // O corpo ficou com os DOIS textos anônimos, em ordem de documento, mesmo
    // com o bloco `footer` escrito entre eles.
    assert!(contem_texto(avaliado, "corpo A"));
    assert!(contem_texto(avaliado, "corpo B"));

    // A ação do rodapé é da tela — a regra de posse do slot vale igual para o
    // conteúdo etiquetado.
    let acao = encontra_acao(avaliado).expect("o botão do rodapé");
    assert_eq!(acao, "reiniciar");

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_slot_nomeado_marcador_permite_decorar_o_opcional() {
    // O template não tem como perguntar "veio rodapé?" — o nome do slot não é
    // uma prop. O motor semeia `{slot_<nome>}` na fronteira do componente para
    // cada slot nomeado preenchido, e é isso que deixa o `<Card>` pagar a linha
    // divisória do rodapé só quando existe rodapé.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_slot_marcador.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <Column>
            <Card title="Com"><Text content="c" /><template slot="footer"><Text content="pe" /></template></Card>
            <Card title="Sem"><Text content="c" /></Card>
        </Column>
        "#,
        ),
    )
    .unwrap();
    motor
        .register_component("tela_marcador", tela_path)
        .unwrap();

    let avaliado = motor.evaluated("tela_marcador").unwrap();
    let com = &avaliado.children[0];
    let sem = &avaliado.children[1];

    assert!(contem_texto(com, "pe"));
    assert!(
        conta_regras(com) > conta_regras(sem),
        "o cartão com rodapé paga uma <Rule> a mais que o sem"
    );
    // E o marcador não vaza para o contexto global do app.
    assert!(!motor.context().contains_key("slot_footer"));

    std::fs::remove_file(tela_path).ok();
}

/// Quantas `<Rule>` a subárvore tem.
fn conta_regras(no: &UiNode) -> usize {
    let eu = usize::from(matches!(no.kind, NodeType::Rule { .. }));
    eu + no.children.iter().map(conta_regras).sum::<usize>()
}

#[test]
fn test_slot_reserva_quando_o_uso_nao_passa_nada() {
    // Os filhos do próprio `<slot>` são o conteúdo de reserva: entram só quando
    // quem usou não escreveu nada dentro da tag. Ao contrário do conteúdo do
    // uso, esses são do COMPONENTE — avaliam no contexto dele e enxergam as
    // props da instância.
    let mut motor = GlacierUI::new();
    motor.register(Box::new(CaixaComReserva)).unwrap();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_slot_reserva.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <Column>
            <CaixaComReserva titulo="Rede" />
            <CaixaComReserva titulo="Disco"><Text content="conteudo do uso" /></CaixaComReserva>
        </Column>
        "#,
        ),
    )
    .unwrap();
    motor.register_component("tela_reserva", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_reserva").unwrap();
    match &avaliado.children[0].children[0].kind {
        NodeType::Text { content, .. } => assert_eq!(content, "vazio: Rede"),
        outro => panic!("esperava a reserva, veio {outro:?}"),
    }
    match &avaliado.children[1].children[0].kind {
        NodeType::Text { content, .. } => assert_eq!(content, "conteudo do uso"),
        outro => panic!("esperava o conteúdo do uso, veio {outro:?}"),
    }

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_builtins_onda2_disponiveis_sem_registro() {
    // Os seis recipientes da "onda 2" (`PLANO_WIDGETS.md` §6) resolvem por tag
    // sem o app registrar nada, e cada um embrulha o conteúdo do uso.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_builtins_onda2.gv";
    std::fs::write(
        tela_path,
        envolve(
            r#"
        <Column>
            <Frame shape="none"><Text content="F" /></Frame>
            <Card title="T" subtitle="S"><Text content="C" /></Card>
            <ToolBar><Text content="TB" /></ToolBar>
            <StatusBar message="Pronto"><Text content="SB" /></StatusBar>
        </Column>
        "#,
        ),
    )
    .unwrap();
    motor.register_component("tela_onda2", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_onda2").unwrap();
    assert_eq!(avaliado.children.len(), 4);

    // Cada um dos quatro tem de conter, em algum lugar, o texto do uso.
    for (i, esperado) in ["F", "C", "TB", "SB"].iter().enumerate() {
        assert!(
            contem_texto(&avaliado.children[i], esperado),
            "o widget {i} deveria ter embrulhado o conteúdo {esperado:?}"
        );
    }
    // O cabeçalho do Card sai do par título/subtítulo, que são props.
    assert!(contem_texto(&avaliado.children[1], "T"));
    assert!(contem_texto(&avaliado.children[1], "S"));
    // A mensagem da StatusBar também é prop, não conteúdo.
    assert!(contem_texto(&avaliado.children[3], "Pronto"));

    // Nenhum default de prop vazou para o contexto global.
    assert!(!motor.context().contains_key("shape"));
    assert!(!motor.context().contains_key("message"));

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_builtin_tabbar_escreve_a_chave_do_app() {
    // `TabBar` usa o padrão do `SpinBox`: a chave vem por prop e viaja dentro
    // da ação (`pick:aba|rede`), então o `update` do widget sabe onde escrever
    // e duas barras na mesma tela não colidem.
    use glacier_ui::EngineMessage;

    let mut motor = GlacierUI::new();
    motor.define_data(
        "abas",
        r#"[{"id":"geral","label":"Geral"},{"id":"rede","label":"Rede"}]"#,
    );
    motor.define_data("aba", "geral");

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_builtin_tabbar.gv";
    std::fs::write(
        tela_path,
        envolve(r#"<TabBar value="aba" active="{aba}" items="abas" />"#),
    )
    .unwrap();
    motor.register_component("tela_tabs", tela_path).unwrap();

    let avaliado = motor.evaluated("tela_tabs").unwrap();
    assert_eq!(avaliado.kind, NodeType::Row);
    assert_eq!(avaliado.children.len(), 2, "uma aba por item da coleção");

    // A ativa é a que veio em `active` — o destaque sai do `bold` do rótulo.
    match &avaliado.children[0].children[0].kind {
        NodeType::Text { bold, .. } => assert!(*bold, "a aba ativa vem em negrito"),
        outro => panic!("esperava o rótulo da aba, veio {outro:?}"),
    }
    match &avaliado.children[1].children[0].kind {
        NodeType::Text { bold, .. } => assert!(!*bold, "a inativa não"),
        outro => panic!("esperava o rótulo da aba, veio {outro:?}"),
    }

    let clique_rede = match &avaliado.children[1].kind {
        NodeType::Button { on_click, .. } => on_click.clone().expect("aba sem ação"),
        outro => panic!("esperava a aba, veio {outro:?}"),
    };
    assert_eq!(clique_rede, "TabBar::pick:aba|rede");

    let _ = motor.dispatch(&EngineMessage::UiClick(clique_rede));
    assert_eq!(motor.context().get("aba").map(String::as_str), Some("rede"));

    std::fs::remove_file(tela_path).ok();
}

/// Componente de teste cujo `<slot>` traz conteúdo de reserva.
struct CaixaComReserva;
impl glacier_ui::Component for CaixaComReserva {
    fn name(&self) -> &str {
        "CaixaComReserva"
    }
    fn template(&self) -> glacier_ui::Template {
        glacier_ui::Template::Inline(
            r#"<Column><slot><Text content="vazio: {titulo}" /></slot></Column>"#.into(),
        )
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut glacier_ui::Context) {}
}

/// Procura um `Text` com este conteúdo em qualquer profundidade da subárvore.
fn contem_texto(no: &UiNode, alvo: &str) -> bool {
    if let NodeType::Text { content, .. } = &no.kind
        && content == alvo
    {
        return true;
    }
    no.children.iter().any(|f| contem_texto(f, alvo))
}

#[test]
fn test_builtin_spinbox_layout_inline() {
    // A prop `layout="inline"` troca a forma: os dois degraus vão para as
    // pontas (`−  campo  +`, o SpinBox do Qt Quick) em vez de empilharem à
    // direita do campo. É um `<template if equals>` dentro do builtin, então o
    // teste também cobre que o ramo não escolhido não deixa nó nenhum para trás.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let tela_path = "templates/test_builtin_spinbox_inline.gv";
    std::fs::write(
        tela_path,
        envolve(r#"<SpinBox value="zoom" min="25" max="400" step="25" layout="inline" />"#),
    )
    .unwrap();
    motor
        .register_component("tela_spin_inline", tela_path)
        .unwrap();

    let spin = motor.evaluated("tela_spin_inline").unwrap();
    assert_eq!(spin.kind, NodeType::Row);
    assert_eq!(spin.children.len(), 3, "− + campo + +");

    let acao = |n: &UiNode| match &n.kind {
        NodeType::Button { on_click, .. } => on_click.clone().expect("degrau sem ação"),
        outro => panic!("esperava um degrau, veio {outro:?}"),
    };
    assert_eq!(acao(&spin.children[0]), "SpinBox::dec:zoom|25|400|25");
    assert_eq!(acao(&spin.children[2]), "SpinBox::inc:zoom|25|400|25");
    // Os glifos desta forma são `−`/`+` (os `▾`/`▴` são os do `stacked`).
    let glifo = |n: &UiNode| match &n.children[0].kind {
        NodeType::Text { content, .. } => content.clone(),
        outro => panic!("esperava o glifo, veio {outro:?}"),
    };
    assert_eq!(glifo(&spin.children[0]), "−");
    assert_eq!(glifo(&spin.children[2]), "+");
    assert!(matches!(spin.children[1].kind, NodeType::TextInput { .. }));

    std::fs::remove_file(tela_path).ok();
}

#[test]
fn test_prefixo_app_escapa_do_namespace_do_dono() {
    // O escape em si, isolado do TimePicker: dentro de um componente, `app:`
    // impede o prefixo de dono; sem ele, a ação continua namespaceada.
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let comp_path = "templates/test_app_prefix_comp.gv";
    let tela_path = "templates/test_app_prefix_tela.gv";
    std::fs::write(
        comp_path,
        envolve(
            r#"<Column>
                <Button text="dele" on_click="minha_acao" />
                <Button text="do app" on_click="app:acao_do_app" />
            </Column>"#,
        ),
    )
    .unwrap();
    std::fs::write(
        tela_path,
        envolve(
            r#"<Column>
                <import name="Delegante" from="templates/test_app_prefix_comp.gv" />
                <Delegante />
            </Column>"#,
        ),
    )
    .unwrap();

    motor
        .register_component("tela_app_prefix", tela_path)
        .unwrap();

    let avaliado = motor.evaluated("tela_app_prefix").unwrap();
    let col = &avaliado.children[0];
    let acao = |i: usize| match &col.children[i].kind {
        NodeType::Button { on_click, .. } => on_click.clone().unwrap_or_default(),
        outro => panic!("esperava botão, veio {outro:?}"),
    };
    assert_eq!(acao(0), "Delegante::minha_acao");
    assert_eq!(acao(1), "acao_do_app");

    std::fs::remove_file(comp_path).ok();
    std::fs::remove_file(tela_path).ok();
}

/// Guarda o exemplo `examples/timepicker/`: as três tags do Qt, ligadas às
/// chaves certas, e nenhum script.
///
/// Até a 0.67 este exemplo tinha ~40 linhas de Luau montando um seletor à mão,
/// porque o `TimePicker` era um builtin que só delegava. Virou primitiva.
#[test]
fn test_exemplo_timepicker_ponta_a_ponta() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("tela_hora", "examples/timepicker/app.gv")
        .expect("registrar a tela do exemplo");
    motor.define_data("data", "2026-09-01");
    motor.define_data("hora", "13:45:02");
    motor.set_initial_screen("tela_hora");

    let tela = motor.evaluated("tela_hora").unwrap();
    let mut achados: Vec<(String, bool, bool)> = Vec::new();
    fn anda(n: &UiNode, out: &mut Vec<(String, bool, bool)>) {
        if let NodeType::DateTimeEdit {
            value_var,
            date,
            time,
            ..
        } = &n.kind
        {
            out.push((value_var.clone(), *date, *time));
        }
        for f in &n.children {
            anda(f, out);
        }
    }
    anda(tela, &mut achados);

    // Um `<dateedit>`, um `<timeedit>` e um `<datetimeedit>`, no mínimo.
    assert!(
        achados.iter().any(|(k, d, t)| k == "data" && *d && !*t),
        "faltou o QDateEdit ligado a 'data': {achados:?}"
    );
    assert!(
        achados.iter().any(|(k, d, t)| k == "hora" && !*d && *t),
        "faltou o QTimeEdit ligado a 'hora': {achados:?}"
    );
    assert!(
        achados.iter().any(|(k, d, t)| k == "quando" && *d && *t),
        "faltou o QDateTimeEdit ligado a 'quando': {achados:?}"
    );
}

/// O exemplo `data_hora_luau`: os campos com `onChange` **delegam**, e é o
/// script que decide se o valor entra. Guarda as duas metades — a regra que
/// aceita e a que recusa.
#[test]
fn test_exemplo_data_hora_luau_valida_no_script() {
    use glacier_ui::EngineMessage;

    let mut motor = GlacierUI::new();
    motor
        .register_component("reserva", "examples/data_hora_luau/app.gv")
        .expect("registrar a tela do exemplo");
    motor.set_initial_screen("reserva");

    // O `init()` do Luau semeou tudo — o `main.rs` não chama `define_data`.
    assert_eq!(
        motor.context().get("checkin").map(String::as_str),
        Some("2026-09-10"),
        "o init() do app.luau precisa ter rodado"
    );
    assert!(
        motor
            .context()
            .get("resumo")
            .is_some_and(|r| r.contains("2 noites")),
        "o resumo é calculado no script: {:?}",
        motor.context().get("resumo")
    );

    // Uma saída DEPOIS da entrada é aceita e o resumo acompanha.
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "set_checkout".into(),
        value: "2026-09-15".into(),
    });
    assert_eq!(
        motor.context().get("checkout").map(String::as_str),
        Some("2026-09-15")
    );
    assert!(
        motor
            .context()
            .get("resumo")
            .is_some_and(|r| r.contains("5 noites"))
    );
    assert_eq!(motor.context().get("aviso").map(String::as_str), Some(""));

    // Uma saída ANTES da entrada é recusada: a chave não muda e o aviso aparece.
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "set_checkout".into(),
        value: "2026-09-01".into(),
    });
    assert_eq!(
        motor.context().get("checkout").map(String::as_str),
        Some("2026-09-15"),
        "o valor recusado não pode ter sido gravado"
    );
    assert!(
        motor
            .context()
            .get("aviso")
            .is_some_and(|a| a.contains("depois da entrada")),
        "o script tem de explicar a recusa"
    );

    // E um preset escreve a chave direto, sem passar pelo widget.
    let _ = motor.dispatch(&EngineMessage::UiClick("turno_manha".into()));
    assert_eq!(
        motor.context().get("chegada").map(String::as_str),
        Some("08:00")
    );

    // O `<datetimeedit>` do lembrete tem a regra mais sutil: ele carrega hora e
    // o check-in não, então a comparação é entre os DIAS. Um lembrete no próprio
    // dia da entrada entra, mesmo com hora — que é o que a comparação de string
    // inteira recusaria (`"2026-09-10 08:00" > "2026-09-10"`).
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "set_lembrete".into(),
        value: "2026-09-10 08:00".into(),
    });
    assert_eq!(
        motor.context().get("lembrete").map(String::as_str),
        Some("2026-09-10 08:00"),
        "um lembrete no dia da entrada é válido"
    );

    // Um dia depois da entrada é recusado.
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "set_lembrete".into(),
        value: "2026-09-11 08:00".into(),
    });
    assert_eq!(
        motor.context().get("lembrete").map(String::as_str),
        Some("2026-09-10 08:00"),
        "o valor recusado não pode ter sido gravado"
    );
}

/// A cadeia `<script src="…luau">` -> `init()` -> `ctx` -> binding, ponta a
/// ponta, num exemplo de verdade.
///
/// Morava no teste do `timepicker` até a 0.68, quando aquele exemplo perdeu o
/// script (o widget passou a fazer o trabalho). Mudou de exemplo para a
/// cobertura não sumir junto.
#[test]
fn test_exemplo_luau_externo_ponta_a_ponta() {
    use glacier_ui::EngineMessage;

    let mut motor = GlacierUI::new();
    motor
        .register_component("contador", "examples/contador_externo/contador_externo.gv")
        .expect("registrar a tela do exemplo");
    motor.set_initial_screen("contador");

    // O `init()` do Lua só roda se o `<script src>` foi ligado e resolvido
    // relativo ao template.
    assert_eq!(
        motor.context().get("contador").map(String::as_str),
        Some("0"),
        "o init() do .luau precisa ter rodado"
    );

    // E um handler do script responde a uma ação vinda da UI.
    let _ = motor.dispatch(&EngineMessage::UiClick("incrementar".into()));
    assert_eq!(
        motor.context().get("contador").map(String::as_str),
        Some("1")
    );
}

#[test]
fn test_template_default_inline() {
    use glacier_ui::process_template;
    use std::collections::HashMap;

    let mut ctx = HashMap::new();
    ctx.insert("nome".to_string(), "Ana".to_string());

    // Chave presente: usa o valor (o default é ignorado).
    assert_eq!(process_template("Oi {nome|visitante}", &ctx), "Oi Ana");
    // Chave ausente: cai no default.
    assert_eq!(
        process_template("Oi {cargo|visitante}", &ctx),
        "Oi visitante"
    );
    // Sem default e ausente: vazio (comportamento antigo, inalterado).
    assert_eq!(process_template("Oi {cargo}", &ctx), "Oi ");
    // Espaços em torno da chave e do default são aparados.
    assert_eq!(process_template("{ cargo | dev }", &ctx), "dev");
}

#[test]
fn test_atributo_numerico_templado() {
    // `size` (numérico) recebe `{prop}` de uma instância de componente e é
    // resolvido no eval — antes só atributos string aceitavam template.
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let card_path = "templates/test_num_card.gv";
    let main_path = "templates/test_num_main.gv";

    std::fs::write(card_path, envolve(r##"<Text content="oi" size="{s}" />"##)).unwrap();
    std::fs::write(
        main_path,
        envolve(
            r##"<Column>
            <NumCard s="28" />
            <NumCard />
        </Column>"##,
        ),
    )
    .unwrap();

    motor.register_component("NumCard", card_path).unwrap();
    motor
        .register_component("test_num_main", main_path)
        .unwrap();

    let evaluated = motor.evaluated("test_num_main").unwrap();
    // Com prop: size templado resolve para 28.
    match &evaluated.children[0].kind {
        NodeType::Text { size, .. } => assert_eq!(*size, Some(28.0)),
        _ => panic!("esperava Text"),
    }
    // Sem prop: `{s}` resolve vazio -> não parseia -> size fica None.
    match &evaluated.children[1].kind {
        NodeType::Text { size, .. } => assert_eq!(*size, None),
        _ => panic!("esperava Text"),
    }

    std::fs::remove_file(card_path).ok();
    std::fs::remove_file(main_path).ok();
}

#[test]
fn test_foreach_com_componente() {
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();

    let main_path = "templates/test_lista.gv";
    let card_path = "templates/test_cartao.gv";

    // Componente reutilizável que recebe props.
    std::fs::write(
        card_path,
        envolve(
            r##"<Container background="#222"><Text content="{nome} - {cargo}" /></Container>"##,
        ),
    )
    .unwrap();

    // Usa o componente pelo nome dentro de um ForEach, passando campos como props.
    std::fs::write(
        main_path,
        envolve(
            r##"
        <Column>
            <ForEach items="membros" var="m">
                <Cartao nome="{m.nome}" cargo="{m.cargo}" />
            </ForEach>
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("Cartao", card_path).unwrap();
    motor.register_component("test_lista", main_path).unwrap();

    let data = r#"[
        {"nome": "Ana", "cargo": "Dev"},
        {"nome": "Bruno", "cargo": "Design"}
    ]"#;
    motor.define_data("membros", data);

    let evaluated = motor.evaluated("test_lista").unwrap();
    assert_eq!(evaluated.kind, NodeType::Column);
    assert_eq!(evaluated.children.len(), 2);

    // Cada iteração do loop deve produzir o Container do componente,
    // com as props já substituídas pelos valores do item.
    let primeiro = &evaluated.children[0];
    assert_eq!(primeiro.kind, NodeType::Container);
    if let NodeType::Text { content, .. } = &primeiro.children[0].kind {
        assert_eq!(content, "Ana - Dev");
    } else {
        panic!("Esperava Text dentro do primeiro cartão");
    }

    if let NodeType::Text { content, .. } = &evaluated.children[1].children[0].kind {
        assert_eq!(content, "Bruno - Design");
    } else {
        panic!("Esperava Text dentro do segundo cartão");
    }

    std::fs::remove_file(main_path).ok();
    std::fs::remove_file(card_path).ok();
}

#[test]
fn test_navegacao_historico() {
    let mut motor = GlacierUI::new();

    motor.set_initial_screen("home");
    assert_eq!(motor.current_screen(), Some("home"));

    motor.navigate_to("config");
    motor.navigate_to("perfil");
    assert_eq!(motor.current_screen(), Some("perfil"));

    // NavigateBack desempilha o histórico na ordem inversa.
    motor.navigate_back();
    assert_eq!(motor.current_screen(), Some("config"));
    motor.navigate_back();
    assert_eq!(motor.current_screen(), Some("home"));

    // Histórico vazio: navigate_back não muda a tela.
    motor.navigate_back();
    assert_eq!(motor.current_screen(), Some("home"));

    // Navigate para a tela já ativa não empilha duplicado.
    motor.navigate_to("home");
    motor.navigate_back();
    assert_eq!(motor.current_screen(), Some("home"));
}

#[test]
fn test_foreach() {
    let mut motor = GlacierUI::new();

    let path = "templates/test_foreach.gv";
    std::fs::create_dir_all("templates").ok();
    std::fs::write(
        path,
        envolve(
            r##"
        <Column>
            <ForEach items="items" var="it">
                <Text content="Item: {it.name} ({it.val})" />
            </ForEach>
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("test_for", path).unwrap();

    let data = r#"[
        {"name": "X", "val": "1"},
        {"name": "Y", "val": "2"}
    ]"#;
    motor.define_data("items", data);

    let evaluated = motor.evaluated("test_for").unwrap();
    assert_eq!(evaluated.kind, NodeType::Column);
    assert_eq!(evaluated.children.len(), 2);

    if let NodeType::Text { content, .. } = &evaluated.children[0].kind {
        assert_eq!(content, "Item: X (1)");
    } else {
        panic!("First child should be Text Item: X (1)");
    }

    if let NodeType::Text { content, .. } = &evaluated.children[1].kind {
        assert_eq!(content, "Item: Y (2)");
    } else {
        panic!("Second child should be Text Item: Y (2)");
    }

    std::fs::remove_file(path).ok();
}

// --- Nested components: behavior composition -------------------------------

use glacier_ui::{Component, Context, EngineMessage, Template};

/// Embrulha o layout de um template de teste no cabeçalho que todo `.gv` passou
/// a exigir na 0.61. Os testes escrevem arquivos, e é a arquivo que a regra se
/// aplica — markup inline (`Template::Inline`) segue sendo fragmento.
///
/// **Sem quebra de linha nas emendas** de propósito: alguns testes asseguram a
/// LINHA de um diagnóstico, e um `\n` depois do `<component>` deslocaria todas
/// elas em um.
fn envolve(layout: impl AsRef<str>) -> String {
    format!("<component>{}</component>", layout.as_ref())
}

/// Child component with its own behavior. Its button action is `ping`.
struct ChildComp;
impl Component for ChildComp {
    fn name(&self) -> &str {
        "ChildComp"
    }
    fn template(&self) -> Template {
        Template::Inline(r#"<Container><Button text="C" on_click="ping" /></Container>"#.into())
    }
    fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
        if action == "ping" {
            ctx.set("child_pinged", "true");
        }
    }
}

/// Parent owns ChildComp and references it in its own template.
struct ParentComp;
impl Component for ParentComp {
    fn name(&self) -> &str {
        "parent"
    }
    fn template(&self) -> Template {
        Template::Inline(
            r#"<Container><Button text="P" on_click="parent_act" /><ChildComp /></Container>"#
                .into(),
        )
    }
    fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
        if action == "parent_act" {
            ctx.set("parent_acted", "true");
        }
    }
    fn children(&self) -> Vec<Box<dyn Component>> {
        vec![Box::new(ChildComp)]
    }
}

/// Collects every `Button.on_click` in an evaluated tree.
fn collect_clicks(node: &UiNode, out: &mut Vec<String>) {
    if let NodeType::Button {
        on_click: Some(a), ..
    } = &node.kind
    {
        out.push(a.clone());
    }
    for c in &node.children {
        collect_clicks(c, out);
    }
}

#[test]
fn test_nested_component_action_namespacing() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(ParentComp)).unwrap();
    motor.set_initial_screen("parent");

    // Both the child template (registered in cascade) and the parent exist.
    assert!(motor.is_registered("parent"));
    assert!(motor.is_registered("ChildComp"));

    // The child's action got namespaced; the parent's stayed plain.
    let evaluated = motor.evaluated("parent").unwrap();
    let mut clicks = Vec::new();
    collect_clicks(evaluated, &mut clicks);
    assert!(
        clicks.contains(&"parent_act".to_string()),
        "got {:?}",
        clicks
    );
    assert!(
        clicks.contains(&"ChildComp::ping".to_string()),
        "got {:?}",
        clicks
    );
}

#[test]
fn test_nested_component_action_routing() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(ParentComp)).unwrap();
    motor.set_initial_screen("parent");

    // A namespaced action reaches the child's update, not the parent's.
    let _ = motor.dispatch(&EngineMessage::UiClick("ChildComp::ping".into()));
    assert_eq!(
        motor.get_data("child_pinged").map(String::as_str),
        Some("true")
    );
    assert_eq!(motor.get_data("parent_acted"), None);

    // A plain action falls back to the active screen (the parent).
    let _ = motor.dispatch(&EngineMessage::UiClick("parent_act".into()));
    assert_eq!(
        motor.get_data("parent_acted").map(String::as_str),
        Some("true")
    );
}

// --- Drag-and-drop list reordering ------------------------------------------

/// A reorderable list (`ForEach ... onReorder="reordered" reorderKey="key"`)
/// with a `dragHandle` on each item's `Text`. Records the final order it's
/// asked to persist.
struct EnvComp;
impl Component for EnvComp {
    fn name(&self) -> &str {
        "envcomp"
    }
    fn template(&self) -> Template {
        Template::Inline(
            r#"
            <Column>
                <ForEach items="rows" var="e" onReorder="reordered" reorderKey="key">
                    <Row>
                        <Text content="{e.key}" dragHandle="true" />
                    </Row>
                </ForEach>
            </Column>
        "#
            .into(),
        )
    }
    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        if action == "reordered" {
            ctx.set("last_order", value.unwrap_or_default());
        }
    }
}

#[test]
fn test_drag_reorder_end_to_end() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(EnvComp)).unwrap();
    motor.set_initial_screen("envcomp");
    motor.define_data("rows", r#"[{"key":"a"},{"key":"b"},{"key":"c"}]"#);

    // Grab "a", drag it over "c" — order live-reflows to [b, c, a].
    let _ = motor.dispatch(&EngineMessage::DragStart {
        list: "rows".into(),
        reorder_key: "key".into(),
        on_reorder: "reordered".into(),
        order: vec!["a".into(), "b".into(), "c".into()],
        key: "a".into(),
    });
    let _ = motor.dispatch(&EngineMessage::DragHover {
        list: "rows".into(),
        key: "c".into(),
    });
    assert_eq!(
        motor.get_data("rows").map(String::as_str),
        Some(r#"[{"key":"b"},{"key":"c"},{"key":"a"}]"#),
        "context should reflect the live reflow while still dragging",
    );
    assert_eq!(
        motor.get_data("last_order"),
        None,
        "onReorder only fires on drop"
    );

    // Drop: the component's `update` receives the final order.
    let _ = motor.dispatch(&EngineMessage::DragEnd);
    assert_eq!(
        motor.get_data("last_order").map(String::as_str),
        Some(r#"["b","c","a"]"#)
    );

    // A stray release with nothing in progress is a harmless no-op.
    let _ = motor.dispatch(&EngineMessage::DragEnd);
}

#[test]
fn test_drag_hover_ignores_other_lists_and_self() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(EnvComp)).unwrap();
    motor.set_initial_screen("envcomp");
    motor.define_data("rows", r#"[{"key":"a"},{"key":"b"}]"#);

    let _ = motor.dispatch(&EngineMessage::DragStart {
        list: "rows".into(),
        reorder_key: "key".into(),
        on_reorder: "reordered".into(),
        order: vec!["a".into(), "b".into()],
        key: "a".into(),
    });
    // Hovering a different list, or the dragged item itself, changes nothing.
    let _ = motor.dispatch(&EngineMessage::DragHover {
        list: "other".into(),
        key: "b".into(),
    });
    let _ = motor.dispatch(&EngineMessage::DragHover {
        list: "rows".into(),
        key: "a".into(),
    });
    assert_eq!(
        motor.get_data("rows").map(String::as_str),
        Some(r#"[{"key":"a"},{"key":"b"}]"#)
    );

    let _ = motor.dispatch(&EngineMessage::DragEnd);
    assert_eq!(
        motor.get_data("last_order").map(String::as_str),
        Some(r#"["a","b"]"#)
    );
}

#[test]
fn test_gss_fill_and_max_width_resolve_from_class() {
    // `.panel { width: fill; max-width: N }` — the responsive readability-cap
    // pattern (fill up to N, shrink below). Both must land on the node.
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let gss = "templates/test_maxw.gss";
    std::fs::write(gss, ".panel { width: fill; max-width: 640; }").unwrap();

    let path = "templates/test_maxw.gv";
    std::fs::write(path, envolve(r##"<Container class="panel" />"##)).unwrap();

    motor.load_stylesheet(gss).unwrap();
    motor.register_component("maxw", path).unwrap();

    let n = motor.evaluated("maxw").unwrap();
    assert_eq!(
        n.width.as_deref(),
        Some("fill"),
        "width: fill applies from the class"
    );
    assert_eq!(n.max_width, Some(640.0), "max-width applies from the class");

    std::fs::remove_file(gss).ok();
    std::fs::remove_file(path).ok();
}

/// Helper: extract the `color` of an evaluated Text node.
fn text_color(node: &NodeType) -> Option<String> {
    if let NodeType::Text { color, .. } = node {
        color.clone()
    } else {
        None
    }
}

#[test]
fn test_link_stylesheet_is_global() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let global_gss = "templates/test_global.gss";
    std::fs::write(global_gss, ".box { padding: 5; color: #111111; }").unwrap();

    // Only component A declares this <link>, but it must still reach B: a
    // `<link rel="stylesheet">` is always global, regardless of which
    // template declares it. It overrides `.box`'s padding and adds `.linked`.
    let linked_gss = "templates/test_linked.gss";
    std::fs::write(
        linked_gss,
        ".box { padding: 9; } .linked { color: #abcabc; }",
    )
    .unwrap();

    // A links the sheet (as a top-level sibling, before its root, to
    // exercise the <link> hoisting in parse_xml).
    let a_path = "templates/test_scoped_a.gv";
    std::fs::write(
        a_path,
        envolve(
            r##"
        <link rel="stylesheet" href="templates/test_linked.gss" />
        <Text class="box linked" content="A" />
        "##,
        ),
    )
    .unwrap();

    // B doesn't declare the <link> itself, but should see its effect anyway.
    let b_path = "templates/test_scoped_b.gv";
    std::fs::write(
        b_path,
        envolve(r##"<Text class="box linked" content="B" />"##),
    )
    .unwrap();

    motor.load_stylesheet(global_gss).unwrap();
    motor.register_component("a", a_path).unwrap();
    motor.register_component("b", b_path).unwrap();

    let a = motor.evaluated("a").unwrap().clone();
    let b = motor.evaluated("b").unwrap().clone();

    assert_eq!(
        a.padding.as_deref(),
        Some("9"),
        "linked class overrides global padding in A"
    );
    assert_eq!(
        text_color(&a.kind).as_deref(),
        Some("#abcabc"),
        "linked class color applies in A"
    );

    // B: the sheet A linked applies here too, since <link rel="stylesheet"> is global.
    assert_eq!(
        b.padding.as_deref(),
        Some("9"),
        "linked sheet reaches B even though only A declared the <link>"
    );
    assert_eq!(
        text_color(&b.kind).as_deref(),
        Some("#abcabc"),
        "linked class color reaches B too"
    );

    std::fs::remove_file(global_gss).ok();
    std::fs::remove_file(linked_gss).ok();
    std::fs::remove_file(a_path).ok();
    std::fs::remove_file(b_path).ok();
}

#[test]
fn test_inline_style_block_default_is_global() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let global_gss = "templates/test_istyle_global.gss";
    std::fs::write(global_gss, ".box { padding: 5; color: #111111; }").unwrap();

    // A declares a plain (unscoped) inline <style>, which is global by
    // default — it overrides `.box` and adds `.inlined` for every component.
    let a_path = "templates/test_istyle_a.gv";
    std::fs::write(
        a_path,
        envolve(
            r##"
        <style>
            .box { padding: 9; }
            .inlined { color: #abcabc; }
        </style>
        <Text class="box inlined" content="A" />
        "##,
        ),
    )
    .unwrap();

    // B declares nothing, but should see A's plain <style> anyway.
    let b_path = "templates/test_istyle_b.gv";
    std::fs::write(
        b_path,
        envolve(r##"<Text class="box inlined" content="B" />"##),
    )
    .unwrap();

    motor.load_stylesheet(global_gss).unwrap();
    motor.register_component("a", a_path).unwrap();
    motor.register_component("b", b_path).unwrap();

    let a = motor.evaluated("a").unwrap().clone();
    let b = motor.evaluated("b").unwrap().clone();

    assert_eq!(
        a.padding.as_deref(),
        Some("9"),
        "inline <style> overrides global padding"
    );
    assert_eq!(
        text_color(&a.kind).as_deref(),
        Some("#abcabc"),
        "inline class color applies in A"
    );

    // B: A's plain inline <style> reaches it too, since it's global by default.
    assert_eq!(
        b.padding.as_deref(),
        Some("9"),
        "B sees A's unscoped inline <style> too"
    );
    assert_eq!(
        text_color(&b.kind).as_deref(),
        Some("#abcabc"),
        "B sees A's unscoped inline class color too"
    );

    std::fs::remove_file(global_gss).ok();
    std::fs::remove_file(a_path).ok();
    std::fs::remove_file(b_path).ok();
}

#[test]
fn test_inline_style_block_scoped_true_is_scoped() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    // Global sheet seen by everyone.
    let global_gss = "templates/test_istyle_scoped_global.gss";
    std::fs::write(global_gss, ".box { padding: 5; color: #111111; }").unwrap();

    // A declares an inline <style scoped="true">, which overrides `.box` and
    // adds `.scoped` only within A's own subtree.
    let a_path = "templates/test_istyle_scoped_a.gv";
    std::fs::write(
        a_path,
        envolve(
            r##"
        <style scoped="true">
            .box { padding: 9; }
            .scoped { color: #abcabc; }
        </style>
        <Text class="box scoped" content="A" />
        "##,
        ),
    )
    .unwrap();

    // B declares nothing: it only sees the global sheet.
    let b_path = "templates/test_istyle_scoped_b.gv";
    std::fs::write(
        b_path,
        envolve(r##"<Text class="box scoped" content="B" />"##),
    )
    .unwrap();

    motor.load_stylesheet(global_gss).unwrap();
    motor.register_component("a", a_path).unwrap();
    motor.register_component("b", b_path).unwrap();

    let a = motor.evaluated("a").unwrap().clone();
    let b = motor.evaluated("b").unwrap().clone();

    // A: scoped `.box` overrides padding (9 vs global 5); `.scoped` provides color.
    assert_eq!(
        a.padding.as_deref(),
        Some("9"),
        "scoped class should override global padding"
    );
    assert_eq!(
        text_color(&a.kind).as_deref(),
        Some("#abcabc"),
        "scoped class color applies in A"
    );

    // B: only the global `.box` applies; `.scoped` is invisible outside A's scope.
    assert_eq!(b.padding.as_deref(), Some("5"), "B uses global padding");
    assert_eq!(
        text_color(&b.kind).as_deref(),
        Some("#111111"),
        "B uses global color; scoped class has no effect"
    );

    std::fs::remove_file(global_gss).ok();
    std::fs::remove_file(a_path).ok();
    std::fs::remove_file(b_path).ok();
}

#[test]
fn test_inline_style_overrides_linked_by_document_order() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    // A linked sheet sets the color; a later inline <style> overrides it —
    // both are global, but a component's own <link>s are installed before its
    // own inline blocks, so document order still determines who wins.
    let linked = "templates/test_istyle_order.gss";
    std::fs::write(linked, ".tag { color: #aaaaaa; padding: 3; }").unwrap();

    let path = "templates/test_istyle_order.gv";
    std::fs::write(
        path,
        envolve(
            r##"
        <link rel="stylesheet" href="templates/test_istyle_order.gss" />
        <style>.tag { color: #bbbbbb; }</style>
        <Text class="tag" content="x" />
        "##,
        ),
    )
    .unwrap();

    motor.register_component("ord", path).unwrap();

    let n = motor.evaluated("ord").unwrap();
    assert_eq!(
        text_color(&n.kind).as_deref(),
        Some("#bbbbbb"),
        "later inline <style> wins over the linked sheet"
    );
    assert_eq!(
        n.padding.as_deref(),
        Some("3"),
        "padding still comes from the linked sheet"
    );

    std::fs::remove_file(linked).ok();
    std::fs::remove_file(path).ok();
}

#[test]
fn test_inline_style_reloads_with_template() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let path = "templates/test_istyle_reload.gv";
    std::fs::write(
        path,
        envolve(r##"<style>.t { color: #010101; }</style><Text class="t" content="x" />"##),
    )
    .unwrap();
    motor.register_component("rel", path).unwrap();
    let n = motor.evaluated("rel").unwrap();
    assert_eq!(text_color(&n.kind).as_deref(), Some("#010101"));

    // Edit the inline style; bump mtime so the reload check picks it up.
    std::thread::sleep(std::time::Duration::from_millis(10));
    std::fs::write(
        path,
        envolve(r##"<style>.t { color: #020202; }</style><Text class="t" content="x" />"##),
    )
    .unwrap();
    let _ = filetime_touch(path);
    motor.check_reload();

    let n = motor.evaluated("rel").unwrap();
    assert_eq!(
        text_color(&n.kind).as_deref(),
        Some("#020202"),
        "inline style rebuilds when the template reloads"
    );

    std::fs::remove_file(path).ok();
}

/// Sets a file's mtime to now, so `check_reload` reliably sees it as changed
/// even on filesystems with coarse timestamps.
fn filetime_touch(path: &str) -> std::io::Result<()> {
    use std::time::SystemTime;
    let f = std::fs::OpenOptions::new().write(true).open(path)?;
    f.set_modified(SystemTime::now())
}

#[test]
fn test_inline_attribute_wins_over_class() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let gss = "templates/test_inline.gss";
    std::fs::write(gss, ".tag { color: #aaaaaa; padding: 3; }").unwrap();

    let path = "templates/test_inline.gv";
    // Inline color overrides the class; padding falls back to the class.
    std::fs::write(
        path,
        envolve(r##"<Text class="tag" content="x" color="#ff0000" />"##),
    )
    .unwrap();

    motor.load_stylesheet(gss).unwrap();
    motor.register_component("inline", path).unwrap();

    let n = motor.evaluated("inline").unwrap();
    assert_eq!(
        text_color(&n.kind).as_deref(),
        Some("#ff0000"),
        "inline color wins"
    );
    assert_eq!(
        n.padding.as_deref(),
        Some("3"),
        "padding comes from the class"
    );

    std::fs::remove_file(gss).ok();
    std::fs::remove_file(path).ok();
}

#[test]
fn test_link_rel_import() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let child = "templates/test_li_child.gv";
    std::fs::write(child, envolve(r##"<Text content="child:{x}" />"##)).unwrap();

    let parent = "templates/test_li_parent.gv";
    // Declarative import via <link>; the component is then referenced by name.
    std::fs::write(
        parent,
        envolve(
            r##"
        <link rel="import" href="templates/test_li_child.gv" as="ChildLink" />
        <Column>
            <ChildLink x="42" />
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("parent", parent).unwrap();

    // The imported component must be registered and inlined with its prop.
    assert!(
        motor.is_registered("ChildLink"),
        "import should register the component"
    );
    let ev = motor.evaluated("parent").unwrap();
    assert_eq!(ev.children.len(), 1);
    if let NodeType::Text { content, .. } = &ev.children[0].kind {
        assert_eq!(content, "child:42");
    } else {
        panic!("expected the imported Text");
    }

    std::fs::remove_file(child).ok();
    std::fs::remove_file(parent).ok();
}

#[test]
fn test_textarea_parses_and_syncs() {
    // A `<TextArea>` parses to its own node and the engine seeds a stateful
    // editor buffer from the bound context value.
    let xml = r##"<TextArea value="dotenv" placeholder="KEY=VALUE" onChange="env_changed" />"##;
    let ast = UiNode::parse_xml(xml).unwrap();
    match &ast.kind {
        NodeType::TextArea {
            value_var,
            placeholder,
            on_change,
            readonly,
        } => {
            assert_eq!(value_var, "dotenv");
            assert_eq!(placeholder, "KEY=VALUE");
            assert_eq!(on_change, "env_changed");
            assert!(!readonly, "readonly deve ser falso por padrão");
        }
        other => panic!("expected TextArea, got {other:?}"),
    }

    // `readonly` liga via atributo.
    let ro = UiNode::parse_xml(r##"<TextArea value="x" readonly="true" />"##).unwrap();
    match &ro.kind {
        NodeType::TextArea { readonly, .. } => assert!(readonly, "readonly=true deve parsear"),
        other => panic!("expected TextArea, got {other:?}"),
    }

    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_textarea.gv";
    std::fs::write(tpl, envolve(xml)).unwrap();
    motor.register_component("tacomp", tpl).unwrap();
    // Só a tela ativa (ou um `keep_evaluated`) fica avaliada — ver reevaluate_all.
    motor.set_initial_screen("tacomp");
    motor.define_data("dotenv", "FOO=1\nBAR=2");
    // A reevaluation seeds the editor buffer from the context without panicking.
    motor.reevaluate_all().unwrap();
    assert!(motor.render("tacomp").is_ok());

    std::fs::remove_file(tpl).ok();
}

#[test]
fn test_select_parses_and_renders() {
    // A `<Select>` parses to its own node and renders from a context JSON array,
    // marking the bound value as selected.
    let xml = r##"<Select options="repos" value="chosen" onChange="pick" placeholder="escolha" labelField="full_name" valueField="clone_url" />"##;
    let ast = UiNode::parse_xml(xml).unwrap();
    match &ast.kind {
        NodeType::Select {
            options,
            value_var,
            on_change,
            placeholder,
            label_field,
            value_field,
            ..
        } => {
            assert_eq!(options, "repos");
            assert_eq!(value_var, "chosen");
            assert_eq!(on_change, "pick");
            assert_eq!(placeholder, "escolha");
            assert_eq!(label_field, "full_name");
            assert_eq!(value_field, "clone_url");
        }
        other => panic!("expected Select, got {other:?}"),
    }

    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_select.gv";
    std::fs::write(tpl, envolve(xml)).unwrap();
    motor.register_component("selcomp", tpl).unwrap();
    motor.set_initial_screen("selcomp");
    motor.define_data(
        "repos",
        r##"[{"full_name":"org/a","clone_url":"https://x/a.git"},{"full_name":"org/b","clone_url":"https://x/b.git"}]"##,
    );
    motor.define_data("chosen", "https://x/b.git");
    motor.reevaluate_all().unwrap();
    assert!(motor.render("selcomp").is_ok());

    std::fs::remove_file(tpl).ok();
}

#[test]
fn test_if_else_inside_foreach() {
    // Regression: `<if>`/`<else>` nested directly under a `<ForEach>` must be
    // resolved per item (only the matching branch renders), not emitted as
    // plain nodes with both branches expanded.
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let data = "templates/test_ifforeach.json";
    std::fs::write(
        data,
        r##"{ "rows": [ {"filler":"0","name":"api"}, {"filler":"1","name":"x"}, {"filler":"0","name":"web"} ] }"##,
    )
    .unwrap();

    let tpl = "templates/test_ifforeach.gv";
    std::fs::write(
        tpl,
        envolve(
            r##"
        <link rel="data" href="templates/test_ifforeach.json" as="d" />
        <Column>
            <ForEach items="d.rows" var="r">
                <if cond="{r.filler}" equals="1">
                    <Text content="GAP" />
                </if>
                <else>
                    <Text content="{r.name}" />
                </else>
            </ForEach>
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("ifforeach", tpl).unwrap();

    let ev = motor.evaluated("ifforeach").unwrap();
    let texts: Vec<String> = ev
        .children
        .iter()
        .filter_map(|c| {
            if let NodeType::Text { content, .. } = &c.kind {
                Some(content.clone())
            } else {
                None
            }
        })
        .collect();
    // Exactly one node per item, picking the right branch.
    assert_eq!(texts, vec!["api", "GAP", "web"]);

    std::fs::remove_file(data).ok();
    std::fs::remove_file(tpl).ok();
}

#[test]
fn test_link_rel_data() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let data = "templates/test_data.json";
    std::fs::write(
        data,
        r##"{ "title": "Olá", "users": [ {"name": "Ana"}, {"name": "Bob"} ] }"##,
    )
    .unwrap();

    let tpl = "templates/test_data.gv";
    std::fs::write(
        tpl,
        envolve(
            r##"
        <link rel="data" href="templates/test_data.json" as="app" />
        <Column>
            <Text content="{app.title}" />
            <ForEach items="app.users" var="u">
                <Text content="{u.name}" />
            </ForEach>
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("datacomp", tpl).unwrap();

    // Object field flattened to `app.title`.
    assert_eq!(motor.get_data("app.title").map(String::as_str), Some("Olá"));

    let ev = motor.evaluated("datacomp").unwrap();
    // 1 title + 2 ForEach-expanded users.
    assert_eq!(ev.children.len(), 3, "title + two users");
    let names: Vec<String> = ev
        .children
        .iter()
        .filter_map(|c| {
            if let NodeType::Text { content, .. } = &c.kind {
                Some(content.clone())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(names, vec!["Olá", "Ana", "Bob"]);

    std::fs::remove_file(data).ok();
    std::fs::remove_file(tpl).ok();
}

#[test]
fn test_link_rel_theme() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let theme = "templates/test_theme.json";
    std::fs::write(
        theme,
        r##"{ "name": "test", "background": "#102030", "text": "#FFFFFF", "primary": "#A0B0C0", "success": "#00FF00", "danger": "#FF0000" }"##,
    ).unwrap();

    let tpl = "templates/test_theme.gv";
    std::fs::write(
        tpl,
        envolve(
            r##"
        <link rel="theme" href="templates/test_theme.json" />
        <Text content="x" />
        "##,
        ),
    )
    .unwrap();

    // Default theme before loading anything is Dark.
    assert!(motor.custom_theme().is_none());

    motor.register_component("themecomp", tpl).unwrap();

    assert!(
        motor.custom_theme().is_some(),
        "theme link should set a custom theme"
    );
    let bg = motor.theme().palette().background;
    assert!((bg.r - 16.0 / 255.0).abs() < 1e-6, "background red channel");
    assert!(
        (bg.g - 32.0 / 255.0).abs() < 1e-6,
        "background green channel"
    );
    assert!(
        (bg.b - 48.0 / 255.0).abs() < 1e-6,
        "background blue channel"
    );

    std::fs::remove_file(theme).ok();
    std::fs::remove_file(tpl).ok();
}

// ── New widgets & async bridge (v0.2) ───────────────────────────────────────

#[test]
fn parses_new_widget_tags() {
    let xml = r##"
    <Column>
        <Scrollable direction="vertical"><Text content="a" /></Scrollable>
        <Checkbox label="Remember" checked="remember" onToggle="toggle_remember" />
        <Toggle label="Enabled" checked="enabled" onToggle="toggle_enabled" />
        <Rule direction="horizontal" />
        <Svg source="icons/rocket.svg" color="#89B4FA" />
    </Column>
    "##;
    let ast = UiNode::parse_xml(xml).unwrap();
    let kinds: Vec<&NodeType> = ast.children.iter().map(|c| &c.kind).collect();
    assert!(matches!(kinds[0], NodeType::Scrollable { .. }));
    assert!(matches!(kinds[1], NodeType::Checkbox { .. }));
    assert!(matches!(kinds[2], NodeType::Toggle { .. }));
    assert!(matches!(kinds[3], NodeType::Rule { horizontal: true }));
    assert!(matches!(kinds[4], NodeType::Svg { .. }));

    if let NodeType::Checkbox {
        label,
        checked_var,
        on_toggle,
        tristate,
    } = &ast.children[1].kind
    {
        assert_eq!(label, "Remember");
        assert_eq!(checked_var, "remember");
        assert_eq!(on_toggle, "toggle_remember");
        assert!(!tristate, "sem o atributo, um checkbox é binário");
    } else {
        panic!("expected checkbox");
    }
}

#[test]
fn parses_font_gradient_text_align() {
    let xml =
        r##"<Text content="Hi" font="mono" gradient="180 #000000 #FFFFFF" textAlign="center" />"##;
    let ast = UiNode::parse_xml(xml).unwrap();
    assert_eq!(ast.font.as_deref(), Some("mono"));
    assert_eq!(ast.gradient.as_deref(), Some("180 #000000 #FFFFFF"));
    assert_eq!(ast.text_align.as_deref(), Some("center"));
}

#[test]
fn context_patch_merges_into_context() {
    use glacier_ui::EngineMessage;
    let mut motor = GlacierUI::new();
    let _task = motor.dispatch(&EngineMessage::ContextPatch(vec![
        ("status".into(), "ok".into()),
        ("count".into(), "3".into()),
    ]));
    assert_eq!(motor.get_data("status").map(String::as_str), Some("ok"));
    assert_eq!(motor.get_data("count").map(String::as_str), Some("3"));
}

#[test]
fn gss_supports_font_and_text_align() {
    use glacier_ui::StyleSheet;
    let sheet = StyleSheet::parse(".mono { font: mono; text-align: center; }").unwrap();
    let rule = sheet.rules.get("mono").unwrap();
    assert_eq!(rule.font.as_deref(), Some("mono"));
    assert_eq!(rule.text_align.as_deref(), Some("center"));
}

#[test]
fn test_directives_as_attributes() {
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let path = "templates/test_directives_attr.gv";
    std::fs::write(
        path,
        envolve(
            r##"
        <Column>
            <Text content="Olá, {usuario}" if="{logado}" />
            <Text content="Entre, por favor" senao />
            <Text content="painel admin" if="{papel}" equals="admin" />
            <Text content="painel comum" if="{papel}" notEquals="admin" />
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("cond_attr", path).unwrap();

    // Estado inicial: deslogado, papel comum
    motor.define_data("logado", "false");
    motor.define_data("usuario", "Ana");
    motor.define_data("papel", "user");

    let ev = motor.evaluated("cond_attr").unwrap();
    // O primeiro Text (if) é ocultado. O segundo Text (senao) é exibido.
    // O terceiro (if papel equals admin) é ocultado. O quarto (if papel notEquals admin) é exibido.
    assert_eq!(ev.children.len(), 2);
    if let NodeType::Text { content, .. } = &ev.children[0].kind {
        assert_eq!(content, "Entre, por favor");
    } else {
        panic!("esperava o Text do senao");
    }
    if let NodeType::Text { content, .. } = &ev.children[1].kind {
        assert_eq!(content, "painel comum");
    } else {
        panic!("esperava o Text de papel comum");
    }

    // Logado como admin
    motor.define_data("logado", "true");
    motor.define_data("papel", "admin");

    let ev = motor.evaluated("cond_attr").unwrap();
    // O primeiro Text (if) é exibido. O segundo (senao) é ocultado.
    // O terceiro (if papel equals admin) é exibido. O quarto (if papel notEquals admin) é ocultado.
    assert_eq!(ev.children.len(), 2);
    if let NodeType::Text { content, .. } = &ev.children[0].kind {
        assert_eq!(content, "Olá, Ana");
    } else {
        panic!("esperava o Text do if");
    }
    if let NodeType::Text { content, .. } = &ev.children[1].kind {
        assert_eq!(content, "painel admin");
    } else {
        panic!("esperava o Text do admin");
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn test_precedence_foreach_if_attributes() {
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let path = "templates/test_precedence.gv";
    std::fs::write(
        path,
        envolve(
            r##"
        <Column>
            <Text content="Item: {u.nome}" for-each="usuarios" var="u" if="{u.ativo}" />
        </Column>
        "##,
        ),
    )
    .unwrap();

    motor.register_component("precedence", path).unwrap();

    let json = serde_json::json!([
        { "nome": "Clara", "ativo": "true" },
        { "nome": "Sophia", "ativo": "false" },
        { "nome": "Mateus", "ativo": "true" }
    ])
    .to_string();
    motor.define_data("usuarios", &json);

    let ev = motor.evaluated("precedence").unwrap();
    // Deve renderizar apenas "Clara" e "Mateus", pois "Sophia" tem ativo="false".
    assert_eq!(ev.children.len(), 2);
    if let NodeType::Text { content, .. } = &ev.children[0].kind {
        assert_eq!(content, "Item: Clara");
    } else {
        panic!("esperava o primeiro item");
    }
    if let NodeType::Text { content, .. } = &ev.children[1].kind {
        assert_eq!(content, "Item: Mateus");
    } else {
        panic!("esperava o segundo item");
    }

    std::fs::remove_file(path).ok();
}

#[test]
fn test_unknown_extension_falls_back_to_xml() {
    // Extensão desconhecida (.tmpl) deve usar o parser XML.
    let mut motor = GlacierUI::new();

    std::fs::create_dir_all("templates").ok();
    let path = "templates/test_fallback.tmpl";
    std::fs::write(
        path,
        envolve(r##"<Text content="via XML fallback" size="18" />"##),
    )
    .unwrap();

    motor.register_component("fallback", path).unwrap();

    let ev = motor.evaluated("fallback").unwrap();
    if let NodeType::Text { content, .. } = &ev.kind {
        assert_eq!(content, "via XML fallback");
    } else {
        panic!("esperava um Text parseado pelo fallback XML");
    }

    std::fs::remove_file(path).ok();
}

// --- Formulários (`<Form>` / `formControl`) ---------------------------------

/// Coleta, em ordem de documento, cada nó com `formControl` definido: o nome
/// do controle e o próprio nó (já avaliado/hidratado), clonado para escapar do
/// empréstimo da árvore.
fn collect_form_inputs(node: &UiNode, out: &mut Vec<(String, UiNode)>) {
    if let Some(name) = &node.form_control {
        out.push((name.clone(), node.clone()));
    }
    for child in &node.children {
        collect_form_inputs(child, out);
    }
}

/// Componente com um `<Form>` de dois campos, usado pelos testes de
/// hidratação e de dispatch abaixo.
struct FormTestComp;
impl Component for FormTestComp {
    fn name(&self) -> &str {
        "formtest"
    }
    fn template(&self) -> Template {
        Template::Inline(
            r#"
            <Form onSubmit="enviar">
                <TextInput formControl="usuario" />
                <TextInput formControl="senha" secure="true" />
            </Form>
        "#
            .into(),
        )
    }
    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            "usuario" => {
                ctx.set("usuario", value.unwrap_or_default());
            }
            "senha" => {
                ctx.set("senha", value.unwrap_or_default());
            }
            _ => {}
        }
    }
    // `onSubmit` ("enviar") is routed to `on_form_submit`, not `update` — see
    // `test_ui_submit_always_dispatches_regardless_of_next_focus` below.
    fn on_form_submit(&mut self, _action: &str, ctx: &mut Context) {
        ctx.set("enviado", "true");
    }
}

#[test]
fn test_form_hydrates_scope_submit_and_next_focus() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(FormTestComp)).unwrap();
    motor.set_initial_screen("formtest");

    let evaluated = motor.evaluated("formtest").unwrap();
    let mut inputs = Vec::new();
    collect_form_inputs(evaluated, &mut inputs);
    assert_eq!(
        inputs.len(),
        2,
        "esperava 2 inputs ligados a formControl, veio {:?}",
        inputs.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );

    let (usuario_name, usuario) = &inputs[0];
    let (senha_name, senha) = &inputs[1];
    assert_eq!(usuario_name, "usuario");
    assert_eq!(senha_name, "senha");

    // O `onSubmit` do `<Form>` chega em todo controle, para o Enter sempre
    // disparar a submissão — independente de qual campo está com foco.
    assert_eq!(usuario.form_submit_action.as_deref(), Some("enviar"));
    assert_eq!(senha.form_submit_action.as_deref(), Some("enviar"));

    // Mesmo `scope` (prefixo do id de foco) em ambos, por pertencerem ao
    // mesmo `<Form>`.
    assert!(usuario.form_scope.is_some());
    assert_eq!(usuario.form_scope, senha.form_scope);

    // Enter em "usuario" também avança o foco para "senha"; em "senha" (o
    // último campo) não há próximo.
    assert_eq!(usuario.form_next_focus.as_deref(), Some("senha"));
    assert_eq!(senha.form_next_focus, None);
}

#[test]
fn test_form_control_input_dispatches_like_on_change() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(FormTestComp)).unwrap();
    motor.set_initial_screen("formtest");

    // `TextInput formControl="usuario"` sem `onChange` explícito usa o nome
    // do controle como ação — o mesmo canal que um `onChange` manual usaria.
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "usuario".into(),
        value: "ana".into(),
    });
    assert_eq!(motor.get_data("usuario").map(String::as_str), Some("ana"));
}

#[test]
fn test_ui_submit_always_dispatches_regardless_of_next_focus() {
    // Enter num campo com próximo: dispara `onSubmit` e pede foco adiante.
    let mut motor = GlacierUI::new();
    motor.register(Box::new(FormTestComp)).unwrap();
    motor.set_initial_screen("formtest");
    let _ = motor.dispatch(&EngineMessage::UiSubmit {
        action: "enviar".into(),
        next_focus: Some("glacier_form::formtest::senha".into()),
    });
    assert_eq!(motor.get_data("enviado").map(String::as_str), Some("true"));

    // Enter no último campo (sem próximo): ainda assim dispara `onSubmit` — a
    // decisão de aceitar ou não fica com o `on_form_submit` do componente (via
    // `Form::is_valid()`), não com o motor.
    let mut motor2 = GlacierUI::new();
    motor2.register(Box::new(FormTestComp)).unwrap();
    motor2.set_initial_screen("formtest");
    let _ = motor2.dispatch(&EngineMessage::UiSubmit {
        action: "enviar".into(),
        next_focus: None,
    });
    assert_eq!(motor2.get_data("enviado").map(String::as_str), Some("true"));
}

#[test]
fn test_form_control_defaults_value_and_on_change() {
    let xml = r#"
        <Form onSubmit="entrar">
            <TextInput formControl="usuario" />
        </Form>
    "#;
    let ast = UiNode::parse_xml(xml).unwrap();
    match &ast.kind {
        NodeType::Form { on_submit, .. } => assert_eq!(on_submit.as_deref(), Some("entrar")),
        other => panic!("esperava NodeType::Form, veio {:?}", other),
    }

    let input = &ast.children[0];
    assert_eq!(input.form_control.as_deref(), Some("usuario"));
    match &input.kind {
        NodeType::TextInput {
            value_var,
            on_change,
            ..
        } => {
            assert_eq!(value_var, "usuario");
            assert_eq!(on_change, "usuario");
        }
        other => panic!("esperava NodeType::TextInput, veio {:?}", other),
    }
}

#[test]
fn test_form_control_respects_explicit_value_and_on_change() {
    let xml = r#"
        <Form>
            <TextInput formControl="usuario" value="outro_valor" onChange="outraAcao" />
        </Form>
    "#;
    let ast = UiNode::parse_xml(xml).unwrap();
    let input = &ast.children[0];
    match &input.kind {
        NodeType::TextInput {
            value_var,
            on_change,
            ..
        } => {
            assert_eq!(value_var, "outro_valor");
            assert_eq!(on_change, "outraAcao");
        }
        other => panic!("esperava NodeType::TextInput, veio {:?}", other),
    }
}

/// Sanity check on the actual shipped template (`examples/formulario_login.rs`
/// uses this same path): parses and evaluates end-to-end and has the two
/// expected `formControl`-bound inputs in order. Loading the real file keeps a
/// broken example template from slipping through `cargo test`.
#[test]
fn test_formulario_login_example_template_parses_and_evaluates() {
    let mut motor = GlacierUI::new();
    motor
        .register_component(
            "formulario_login_smoke",
            "examples/formulario_login/formulario_login.gv",
        )
        .expect("o template do exemplo formulario_login deve parsear e avaliar sem erro");

    let evaluated = motor.evaluated("formulario_login_smoke").unwrap();
    let mut inputs = Vec::new();
    collect_form_inputs(evaluated, &mut inputs);
    assert_eq!(
        inputs.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
        vec!["username".to_string(), "password".to_string()],
    );
    assert_eq!(inputs[0].1.form_next_focus.as_deref(), Some("password"));
    assert_eq!(inputs[1].1.form_next_focus, None);
}

// ── Fragment (multi-root component templates) ───────────────────────────────

/// A component whose template is a fragment (an `if`/`else` pair) splices the
/// matching branch into the parent — no wrapper node, and the branch is chosen
/// per-instance from the passed prop.
#[test]
fn test_fragment_component_splices_if_else_branch() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let card = "templates/test_frag_card.gv";
    let main = "templates/test_frag_main.gv";
    std::fs::write(
        card,
        envolve(
            r#"
        <Column if="{filler}" equals="1" class="filler" />
        <Column else class="card">
          <Text content="{name}" />
        </Column>
        "#,
        ),
    )
    .unwrap();
    std::fs::write(
        main,
        envolve(
            r#"
        <import name="FragCard" from="templates/test_frag_card.gv" />
        <Column class="grid">
          <FragCard filler="0" name="Alice" />
          <FragCard filler="1" name="Zzz" />
        </Column>
        "#,
        ),
    )
    .unwrap();

    motor.register_component("test_frag_main", main).unwrap();
    let evaluated = motor.evaluated("test_frag_main").unwrap();

    assert_eq!(evaluated.kind, NodeType::Column);
    // Two spliced siblings — neither is a Fragment wrapper.
    assert_eq!(
        evaluated.children.len(),
        2,
        "fragment children should be spliced, not wrapped"
    );
    assert!(
        evaluated
            .children
            .iter()
            .all(|c| c.kind != NodeType::Fragment)
    );

    // `class` is resolved into style fields (and cleared) during evaluation,
    // so branches are identified by their structure instead: the `else` card
    // branch has the name `Text`; the `if` filler branch is empty.
    //
    // First instance (filler="0") → the `else` card branch, carrying the name.
    let first = &evaluated.children[0];
    assert_eq!(
        first.children.len(),
        1,
        "card branch has one child (the name Text)"
    );
    if let NodeType::Text { content, .. } = &first.children[0].kind {
        assert_eq!(content, "Alice");
    } else {
        panic!("card branch should contain the name Text");
    }
    // Second instance (filler="1") → the empty `if` filler branch.
    assert!(
        evaluated.children[1].children.is_empty(),
        "filler branch is empty"
    );

    std::fs::remove_file(card).ok();
    std::fs::remove_file(main).ok();
}

// ── Registro unificado: register_component liga Luau se houver <script> ──────

/// `register_component` presume que "sempre pode haver Luau": um template com um
/// bloco `<script>` tem seu comportamento Luau ligado automaticamente (sem um
/// `register_luau` à parte). O `init()` semeia o estado e a ação roteia para a
/// função Luau de mesmo nome.
#[test]
fn test_register_component_wires_luau_when_script_present() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let path = "templates/test_scripted_unified.gv";
    std::fs::write(
        path,
        envolve(
            r#"
<Container>
  <Text content="{n}" />
  <Button text="+" onClick="inc" />
</Container>
<script>
function init() ctx.n = ctx.n or 0 end
function inc() ctx.n = ctx.n + 1 end
</script>
"#,
        ),
    )
    .unwrap();

    motor.register_component("scripted", path).unwrap();
    motor.set_initial_screen("scripted");

    // init() do <script> semeou o estado.
    assert_eq!(motor.context().get("n").map(String::as_str), Some("0"));

    // A ação "inc" roteia para a função Luau homônima do componente scriptado.
    let _ = motor.dispatch(&glacier_ui::EngineMessage::UiClick("inc".into()));
    assert_eq!(motor.context().get("n").map(String::as_str), Some("1"));

    std::fs::remove_file(path).ok();
}

/// Sem `<script>`, `register_component` continua só-UI: nenhuma behavior é
/// registrada, então uma ação sem dono simplesmente não faz nada (não entra em
/// pânico) — o mesmo que antes da unificação.
#[test]
fn test_register_component_ui_only_when_no_script() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let path = "templates/test_uionly_unified.gv";
    std::fs::write(
        path,
        envolve(r#"<Container><Button text="x" onClick="nada" /></Container>"#),
    )
    .unwrap();

    motor.register_component("uionly", path).unwrap();
    motor.set_initial_screen("uionly");
    // Ação sem behavior é no-op (não deve entrar em pânico).
    let _ = motor.dispatch(&glacier_ui::EngineMessage::UiClick("nada".into()));

    std::fs::remove_file(path).ok();
}

/// `GlacierUI::register` (o caminho `Box<dyn Component>`) também liga o
/// `<script>` do template, quando houver: por ação, a função Lua de mesmo
/// nome vence se existir; senão cai no hook Rust correspondente.
struct HybridComp;
impl Component for HybridComp {
    fn name(&self) -> &str {
        "hybrid"
    }
    fn template(&self) -> Template {
        Template::File("templates/test_hybrid_register.gv".into())
    }
    fn init(&mut self, ctx: &mut Context) {
        // O <script> não define init(): este init() Rust deve rodar no lugar.
        ctx.set("seeded", "rust-init");
    }
    fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
        // O <script> não define "rust_only": esta ação deve cair aqui.
        if action == "rust_only" {
            ctx.set("from", "rust");
        }
    }
}

#[test]
fn test_register_wires_luau_as_layer_over_rust_component() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let path = "templates/test_hybrid_register.gv";
    std::fs::write(
        path,
        envolve(
            r#"
<Container>
  <Button text="lua" onClick="lua_only" />
  <Button text="rust" onClick="rust_only" />
</Container>
<script>
function lua_only() ctx.from = "lua" end
</script>
"#,
        ),
    )
    .unwrap();

    motor.register(Box::new(HybridComp)).unwrap();
    motor.set_initial_screen("hybrid");

    // init() Rust rodou (o <script> não define init()).
    assert_eq!(
        motor.get_data("seeded").map(String::as_str),
        Some("rust-init")
    );

    // Ação com função Lua correspondente: o Lua vence.
    let _ = motor.dispatch(&EngineMessage::UiClick("lua_only".into()));
    assert_eq!(motor.get_data("from").map(String::as_str), Some("lua"));

    // Ação que o <script> não trata: cai no update() Rust.
    let _ = motor.dispatch(&EngineMessage::UiClick("rust_only".into()));
    assert_eq!(motor.get_data("from").map(String::as_str), Some("rust"));

    std::fs::remove_file(path).ok();
}

#[test]
fn test_text_child_content() {
    // Child text is accepted and normalized (trim + collapse whitespace).
    let xml = "<Text>  lorem   ipsum \n  dolor  </Text>";
    let ast = UiNode::parse_xml(xml).unwrap();
    if let NodeType::Text { content, .. } = &ast.kind {
        assert_eq!(content, "lorem ipsum dolor");
    } else {
        panic!("Root should be Text");
    }
}

#[test]
fn test_text_child_wins_over_attribute() {
    // When both are given, the child takes precedence.
    let xml = r#"<Text content="from attr">from child</Text>"#;
    let ast = UiNode::parse_xml(xml).unwrap();
    if let NodeType::Text { content, .. } = &ast.kind {
        assert_eq!(content, "from child");
    } else {
        panic!("Root should be Text");
    }
}

#[test]
fn test_text_attribute_fallback_when_no_child() {
    let xml = r#"<Text content="only attr" />"#;
    let ast = UiNode::parse_xml(xml).unwrap();
    if let NodeType::Text { content, .. } = &ast.kind {
        assert_eq!(content, "only attr");
    } else {
        panic!("Root should be Text");
    }
}

#[test]
fn tooltip_parses_interpolates_and_renders() {
    // Atributo cru: `tooltip` e o alias `title` (HTML-like) parseiam pro mesmo
    // campo; `tooltipPosition` é opcional (default resolvido em widget.rs, não
    // no parser — aqui só confere que o valor cru sobrevive).
    let ast = UiNode::parse_xml(r#"<Button text="x" tooltip="Ajuda" />"#).unwrap();
    assert_eq!(ast.tooltip.as_deref(), Some("Ajuda"));

    let ast_alias = UiNode::parse_xml(r#"<Button text="x" title="Ajuda 2" />"#).unwrap();
    assert_eq!(ast_alias.tooltip.as_deref(), Some("Ajuda 2"));

    let ast_pos =
        UiNode::parse_xml(r#"<Button text="x" tooltip="Ajuda" tooltipPosition="left" />"#).unwrap();
    assert_eq!(ast_pos.tooltip_position.as_deref(), Some("left"));

    // Sem tooltip, o campo fica None (não vira string vazia nem afeta o render).
    let ast_none = UiNode::parse_xml(r#"<Button text="x" />"#).unwrap();
    assert_eq!(ast_none.tooltip, None);

    // Interpolação (`tooltip="{var}"`) + render de ponta a ponta, com um botão
    // (mouse_area) E um nó puro (row, sem on_press) — o wrap de tooltip fica
    // depois do mouse_area em widget.rs; isso confere que ambos os caminhos
    // (com e sem mouse_area) compilam/renderizam sem panicar.
    let xml = r#"
        <Column>
            <Button text="Doc" tooltip="{help_text}" onClick="noop" />
            <Row tooltip="linha sem clique" tooltipPosition="bottom">
                <Text content="ícone" />
            </Row>
        </Column>
    "#;
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_tooltip.gv";
    std::fs::write(tpl, envolve(xml)).unwrap();
    motor.register_component("tipcomp", tpl).unwrap();
    motor.define_data("help_text", "Ajuda interpolada");
    motor.reevaluate_all().unwrap();

    let evaluated = motor.evaluated("tipcomp").unwrap();
    let button_node = &evaluated.children[0];
    assert_eq!(button_node.tooltip.as_deref(), Some("Ajuda interpolada"));
    let row_node = &evaluated.children[1];
    assert_eq!(row_node.tooltip.as_deref(), Some("linha sem clique"));
    assert_eq!(row_node.tooltip_position.as_deref(), Some("bottom"));

    assert!(
        motor.render("tipcomp").is_ok(),
        "render com tooltip não deve panicar"
    );

    std::fs::remove_file(tpl).ok();
}

// --- Estilos builtin (glacier_ui::style) ------------------------------------

/// Componente mínimo com um Button "pelado" (sem class/color inline) — o alvo
/// típico de uma regra de tag `Button { }` de um estilo builtin.
struct GaleriaMinima;
impl Component for GaleriaMinima {
    fn name(&self) -> &str {
        "galeria_minima"
    }
    fn template(&self) -> Template {
        Template::Inline(r#"<Container><Button text="Ok" on_click="ok" /></Container>"#.into())
    }
    fn update(&mut self, _action: &str, _v: Option<&str>, _ctx: &mut Context) {}
}

/// Cor de fundo do primeiro Button da tela avaliada (o `color` do nó).
fn cor_do_botao(motor: &mut GlacierUI, tela: &str) -> Option<String> {
    fn acha(node: &UiNode) -> Option<String> {
        if let NodeType::Button { color, .. } = &node.kind {
            return color.clone();
        }
        node.children.iter().find_map(acha)
    }
    acha(motor.evaluated(tela).unwrap())
}

#[test]
fn set_style_aplica_tag_rule_tema_e_contexto() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(GaleriaMinima)).unwrap();
    motor.set_initial_screen("galeria_minima");

    // Sem estilo: botão pelado não tem cor (fica no default do iced).
    assert_eq!(cor_do_botao(&mut motor, "galeria_minima"), None);

    motor.set_style(&glacier_ui::style::FUSION_DARK).unwrap();

    // A regra de tag `Button { color }` do estilo pintou o botão…
    assert_eq!(
        cor_do_botao(&mut motor, "galeria_minima").as_deref(),
        Some("#3c3f41")
    );
    // …o tema custom foi instalado com a paleta do estilo…
    assert!(motor.custom_theme().is_some());
    // …e o nome do ativo foi publicado no contexto.
    assert_eq!(
        motor
            .get_data(glacier_ui::style::CONTEXT_KEY)
            .map(String::as_str),
        Some("fusion-dark")
    );
}

#[test]
fn estilo_e_underlay_gss_do_app_vence() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(GaleriaMinima)).unwrap();
    motor.set_initial_screen("galeria_minima");

    // App carrega seu próprio `.gss` com uma regra de tag para Button…
    let gss = "templates/test_estilo_app.gss";
    std::fs::create_dir_all("templates").ok();
    std::fs::write(gss, "Button { color: #ff0000; }").unwrap();
    motor.load_stylesheet(gss).unwrap();

    // …e só então o estilo builtin entra (a ordem não importa: underlay fica
    // sempre abaixo). A regra do app vence.
    motor.set_style(&glacier_ui::style::FUSION).unwrap();
    assert_eq!(
        cor_do_botao(&mut motor, "galeria_minima").as_deref(),
        Some("#ff0000")
    );

    std::fs::remove_file(gss).ok();
}

#[test]
fn trocar_estilo_substitui_a_folha_em_vez_de_empilhar() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(GaleriaMinima)).unwrap();
    motor.set_initial_screen("galeria_minima");

    motor.set_style(&glacier_ui::style::FROST).unwrap();
    let quantas = motor.stylesheets().len();
    motor.set_style(&glacier_ui::style::PHANTOM).unwrap();
    assert_eq!(motor.stylesheets().len(), quantas);
    assert_eq!(
        cor_do_botao(&mut motor, "galeria_minima").as_deref(),
        Some("#46494c")
    );
}

#[test]
fn acoes_builtin_style_trocam_o_estilo() {
    let mut motor = GlacierUI::new();
    motor.register(Box::new(GaleriaMinima)).unwrap();
    motor.set_initial_screen("galeria_minima");

    // Botão: `on_click="style:<nome>"`.
    let _ = motor.dispatch(&EngineMessage::UiClick("style:fusion".into()));
    assert_eq!(
        motor
            .get_data(glacier_ui::style::CONTEXT_KEY)
            .map(String::as_str),
        Some("fusion")
    );

    // Select: `onChange="style:set"` com o valor escolhido.
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "style:set".into(),
        value: "phantom".into(),
    });
    assert_eq!(
        motor
            .get_data(glacier_ui::style::CONTEXT_KEY)
            .map(String::as_str),
        Some("phantom")
    );

    // Nome desconhecido: ignorado (loga e mantém o estilo atual).
    let _ = motor.dispatch(&EngineMessage::UiClick("style:nao_existe".into()));
    assert_eq!(
        motor
            .get_data(glacier_ui::style::CONTEXT_KEY)
            .map(String::as_str),
        Some("phantom")
    );
}

#[test]
fn checkbox_tristate_parseia() {
    let ast = UiNode::parse_xml(
        r#"<Checkbox label="Tri" checked="estado" onToggle="estado" tristate="true" />"#,
    )
    .unwrap();
    assert!(matches!(
        ast.kind,
        NodeType::Checkbox { tristate: true, .. }
    ));
}

// --- `<Button>` com filhos (Fase 1, item 1 do plano de convergência de
// templates GUI↔webui: elimina o hack de `<row on_click>` no lugar de um
// `<Button>` de verdade — ver `docs/plano-convergencia-templates-gui-webui.md`
// no rustploy) --------------------------------------------------------------

/// `text="…"` continua funcionando (nenhum filho) — o atalho não quebrou.
#[test]
fn button_sem_filhos_usa_o_atalho_text() {
    let ast = UiNode::parse_xml(r#"<Button text="Salvar" on_click="save" />"#).unwrap();
    assert_eq!(ast.children.len(), 0);
    match &ast.kind {
        NodeType::Button { text, .. } => assert_eq!(text, "Salvar"),
        other => panic!("esperava Button, veio {other:?}"),
    }
}

/// Um filho único (ex.: só um ícone) é capturado como filho do Button, não
/// jogado fora — antes deste item o parser já guardava `children` (o campo é
/// genérico pra todo `NodeType`), só o renderer ignorava.
#[test]
fn button_com_um_filho_captura_o_filho() {
    let ast =
        UiNode::parse_xml(r#"<Button on_click="menu"><Text content="☰" /></Button>"#).unwrap();
    assert_eq!(ast.children.len(), 1);
    match &ast.children[0].kind {
        NodeType::Text { content, .. } => assert_eq!(content, "☰"),
        other => panic!("esperava Text, veio {other:?}"),
    }
}

/// Vários filhos (o caso que `nav_item.gv` cobria com `<row on_press>` em vez
/// de `<button>`: ícone + rótulo lado a lado) — todos capturados, na ordem.
#[test]
fn button_com_varios_filhos_captura_todos_em_ordem() {
    let ast = UiNode::parse_xml(
        r#"<Button on_click="nav_projects"><Text content="▤" /><Text content="Projects" /></Button>"#,
    )
    .unwrap();
    assert_eq!(ast.children.len(), 2);
    let contents: Vec<&str> = ast
        .children
        .iter()
        .map(|c| match &c.kind {
            NodeType::Text { content, .. } => content.as_str(),
            other => panic!("esperava Text, veio {other:?}"),
        })
        .collect();
    assert_eq!(contents, ["▤", "Projects"]);
}

/// Ponta a ponta: um `<Button>` com filhos (ícone+rótulo, o caso que
/// `nav_item.gv` cobria com `<row on_press>`) renderiza sem panicar — a
/// mudança em `widget.rs` (conteúdo do botão vira `children` quando há
/// algum, em vez de só `text=`) constrói o `Row` implícito sem quebrar.
#[test]
fn button_com_filhos_renderiza() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_button_children.gv";
    std::fs::write(
        tpl,
        envolve(r#"<Button on_click="nav_projects"><Text content="▤" /><Text content="Projects" /></Button>"#),
    )
    .unwrap();
    motor.register_component("btncomp", tpl).unwrap();
    motor.set_initial_screen("btncomp");
    motor.reevaluate_all().unwrap();
    assert!(motor.render("btncomp").is_ok());

    std::fs::remove_file(tpl).ok();
}

// --- `else-if` (Fase 1, item 2 do plano de convergência de templates:
// aplaina cadeias de tela que hoje precisam de um `<if>` aninhado dentro de
// cada `<else>` — ver `docs/plano-convergencia-templates-gui-webui.md` no
// rustploy) -----------------------------------------------------------------

/// Todo `Text` avaliado, em ordem de documento — o jeito mais simples de
/// checar "exatamente esse branch renderizou, e mais nenhum" sem depender de
/// como o widget final é montado.
fn all_texts(node: &UiNode) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(node: &UiNode, out: &mut Vec<String>) {
        if let NodeType::Text { content, .. } = &node.kind {
            out.push(content.clone());
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(node, &mut out);
    out
}

/// Forma **atributo** (`else-if="{x}" equals="…"`, em qualquer elemento) —
/// varre os 4 estados (cada branch + nenhum) e confere que só o certo
/// renderiza, nunca dois ao mesmo tempo.
#[test]
fn else_if_atributo_encadeia_com_if_anterior() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_else_if_attr.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <Text if="{x}" equals="a">A</Text>
            <Text else-if="{x}" equals="b">B</Text>
            <Text else-if="{x}" equals="c">C</Text>
            <Text else>D</Text>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("eiattr", tpl).unwrap();
    motor.set_initial_screen("eiattr");

    for (x, expected) in [("a", "A"), ("b", "B"), ("c", "C"), ("z", "D")] {
        motor.define_data("x", x);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("eiattr").unwrap()),
            vec![expected.to_string()],
            "x={x} deveria renderizar só {expected:?}"
        );
    }

    std::fs::remove_file(tpl).ok();
}

/// Forma **tag** (`<ElseIf cond="…" equals="…">`, a mesma sintaxe de
/// `<If cond="…">`/`<Else>` já usadas em `shell.gv`/`home.gv`) — mesma
/// varredura da versão em atributo.
#[test]
fn else_if_tag_encadeia_com_if_anterior() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_else_if_tag.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <If cond="{x}" equals="a"><Text>A</Text></If>
            <ElseIf cond="{x}" equals="b"><Text>B</Text></ElseIf>
            <ElseIf cond="{x}" equals="c"><Text>C</Text></ElseIf>
            <Else><Text>D</Text></Else>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("eitag", tpl).unwrap();
    motor.set_initial_screen("eitag");

    for (x, expected) in [("a", "A"), ("b", "B"), ("c", "C"), ("z", "D")] {
        motor.define_data("x", x);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("eitag").unwrap()),
            vec![expected.to_string()],
            "x={x} deveria renderizar só {expected:?}"
        );
    }

    std::fs::remove_file(tpl).ok();
}

/// Short-circuit: uma vez que um branch já casou, os `else-if` seguintes nem
/// avaliam a própria condição — mesmo que ela também desse `true` de forma
/// isolada. Aqui os dois `else-if` têm a MESMA condição (`equals="b"`); se o
/// short-circuit não existisse os dois renderizariam.
#[test]
fn else_if_nao_avalia_apos_um_branch_anterior_ja_ter_casado() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_else_if_short_circuit.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <Text if="{x}" equals="a">A</Text>
            <Text else-if="{x}" equals="b">B1</Text>
            <Text else-if="{x}" equals="b">B2</Text>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("eishort", tpl).unwrap();
    motor.set_initial_screen("eishort");
    motor.define_data("x", "b");
    motor.reevaluate_all().unwrap();
    assert_eq!(
        all_texts(motor.evaluated("eishort").unwrap()),
        vec!["B1".to_string()],
        "só o primeiro else-if que casar deve renderizar, nunca os dois"
    );

    std::fs::remove_file(tpl).ok();
}

// --- `one_of`/`equals_any` (Fase 1, item 3 do plano de convergência de
// templates: mantém um item de nav "aceso" em várias sub-telas sem inventar
// gramática de expressão — ver `docs/plano-convergencia-templates-gui-
// webui.md` no rustploy) -----------------------------------------------------

/// Forma **atributo** — `one_of="a b c"` casa qualquer token da lista;
/// qualquer outro valor não casa. Mesmo cenário do plano: manter "Projects"
/// aceso em `projects`/`project`/`new_service`/`service`.
#[test]
fn one_of_atributo_casa_qualquer_token_da_lista() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_one_of_attr.gv";
    std::fs::write(
        tpl,
        // O nó com `if=`/`one_of=` precisa ser FILHO de algo — a filtragem
        // roda em `expand_children` (quando um pai avalia seus filhos), não
        // na raiz avaliada diretamente, então a raiz sozinha ignoraria a
        // condição.
        envolve(
            r#"<Column><Text if="{view}" one_of="projects project new_service service">aceso</Text></Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("oneofattr", tpl).unwrap();
    motor.set_initial_screen("oneofattr");

    for view in ["projects", "project", "new_service", "service"] {
        motor.define_data("view", view);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("oneofattr").unwrap()),
            vec!["aceso".to_string()],
            "view={view} deveria casar one_of"
        );
    }

    for view in ["deployments", "settings", ""] {
        motor.define_data("view", view);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("oneofattr").unwrap()),
            Vec::<String>::new(),
            "view={view:?} não deveria casar one_of"
        );
    }

    std::fs::remove_file(tpl).ok();
}

/// Forma **tag** — `<If cond="…" one_of="a b c">`/`<ElseIf … one_of="…">` —
/// mesma varredura, e confirma que `one_of` funciona encadeado num
/// `else-if` também (não só no `if` inicial da cadeia).
#[test]
fn one_of_tag_funciona_em_if_e_em_else_if() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_one_of_tag.gv";
    std::fs::write(
        tpl,
        envolve(r#"<Column>
            <If cond="{view}" one_of="deployments deploy_engine"><Text>ops</Text></If>
            <ElseIf cond="{view}" one_of="projects project new_service service"><Text>projects</Text></ElseIf>
            <Else><Text>outro</Text></Else>
        </Column>"#),
    )
    .unwrap();
    motor.register_component("oneoftag", tpl).unwrap();
    motor.set_initial_screen("oneoftag");

    for (view, expected) in [
        ("deployments", "ops"),
        ("deploy_engine", "ops"),
        ("project", "projects"),
        ("service", "projects"),
        ("settings", "outro"),
    ] {
        motor.define_data("view", view);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("oneoftag").unwrap()),
            vec![expected.to_string()],
            "view={view} deveria renderizar {expected:?}"
        );
    }

    std::fs::remove_file(tpl).ok();
}

// --- `empty`/`not_empty` (Fase 1, item 4 do plano de convergência de
// templates: aposenta os `*_count` que só existiam pra comparar com
// `equals="0"` — `cond` já é o JSON cru de uma lista no contexto — ver
// `docs/plano-convergencia-templates-gui-webui.md` no rustploy) ------------

/// Forma **atributo** — `empty`/`not_empty` (bare, como `else`) direto
/// sobre uma chave de lista, sem precisar de um `*_count` companheiro.
/// Cobre lista vazia, lista com item, chave ausente e JSON malformado (os
/// dois últimos contam como vazio — "sem lista ainda").
#[test]
fn empty_e_not_empty_atributo_leem_a_lista_direto() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_empty_attr.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <Text if="{items}" empty>nada</Text>
            <Text if="{items}" not_empty>tem coisa</Text>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("emptyattr", tpl).unwrap();
    motor.set_initial_screen("emptyattr");

    for (items, expected) in [
        ("[]", "nada"),
        (r#"[{"name":"x"}]"#, "tem coisa"),
        ("", "nada"),         // chave ausente
        ("not json", "nada"), // JSON malformado
    ] {
        motor.define_data("items", items);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("emptyattr").unwrap()),
            vec![expected.to_string()],
            "items={items:?} deveria renderizar {expected:?}"
        );
    }

    std::fs::remove_file(tpl).ok();
}

/// Forma **tag** — `<If cond="…" empty>`/`<ElseIf … not_empty>` — mesma
/// varredura, e confirma que `not_empty` funciona encadeado num `else-if`.
#[test]
fn empty_e_not_empty_tag_funcionam_em_if_e_else_if() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_empty_tag.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <If cond="{items}" empty><Text>nada</Text></If>
            <ElseIf cond="{items}" not_empty><Text>tem coisa</Text></ElseIf>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("emptytag", tpl).unwrap();
    motor.set_initial_screen("emptytag");

    for (items, expected) in [("[]", "nada"), (r#"[1,2]"#, "tem coisa")] {
        motor.define_data("items", items);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("emptytag").unwrap()),
            vec![expected.to_string()],
            "items={items:?} deveria renderizar {expected:?}"
        );
    }

    std::fs::remove_file(tpl).ok();
}

// --- `<hr>` como alias de `<Rule>` (Fase 1, item 5 do plano de convergência
// de templates: aliases de tag, o mecanismo já existe (if/se, on_change/
// on-change…) — só faltava esse — ver docs/plano-convergencia-templates-
// gui-webui.md no rustploy) --------------------------------------------------

#[test]
fn hr_e_alias_de_rule() {
    for tag in ["<hr />", "<Hr />", "<HR />", "<rule />", "<Rule />"] {
        let ast = UiNode::parse_xml(tag).unwrap_or_else(|e| panic!("{tag}: {e}"));
        assert!(
            matches!(ast.kind, NodeType::Rule { horizontal: true }),
            "{tag} deveria parsear como NodeType::Rule"
        );
    }
}

// --- `href` de `<link rel="import">` relativo ao arquivo importador (Fase
// 1, item 6 do plano de convergência de templates: mesma resolução que o
// `require` do Luau já tem desde a 0.22 — ver docs/plano-convergencia-
// templates-gui-webui.md no rustploy) ----------------------------------------

/// `href="child.gv"` (nome nu, sem `./` nem caminho completo) resolve
/// relativo ao DIRETÓRIO do `.gv` que declara o `<link>` — não ao CWD do
/// processo — mesmo quando o CWD (raiz do workspace, no teste real) é outro
/// diretório qualquer. Reproduz exatamente o caso que motivou o item: dois
/// `.gv` vizinhos numa subpasta (`components/`), um importando o outro.
#[test]
fn import_href_relativo_ao_arquivo_importador() {
    let mut motor = GlacierUI::new();
    let dir = "templates/import_rel_sub";
    std::fs::create_dir_all(dir).ok();
    let parent_path = format!("{dir}/parent.gv");
    let child_path = format!("{dir}/child.gv");

    std::fs::write(&child_path, envolve(r#"<Text content="do filho" />"#)).unwrap();
    std::fs::write(
        &parent_path,
        envolve(r#"<link rel="import" href="child.gv" as="Child" /><Column><Child /></Column>"#),
    )
    .unwrap();

    motor
        .register_component("import_rel_parent", &parent_path)
        .unwrap();
    let evaluated = motor.evaluated("import_rel_parent").unwrap();
    assert_eq!(
        evaluated.children.len(),
        1,
        "esperava o <Child/> ter resolvido e renderizado"
    );
    match &evaluated.children[0].kind {
        NodeType::Text { content, .. } => assert_eq!(content, "do filho"),
        other => panic!("esperava Text, veio {other:?}"),
    }

    std::fs::remove_file(&parent_path).ok();
    std::fs::remove_file(&child_path).ok();
}

/// Retrocompatibilidade: um `href` no estilo antigo (caminho completo a
/// partir da raiz do workspace, como todo `.gv` real do rustploy escreve
/// hoje) continua funcionando quando o candidato relativo ao importador não
/// existe — a resolução cai pro `href` literal, igual ao comportamento de
/// antes deste item.
#[test]
fn import_href_absoluto_continua_funcionando_como_fallback() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates/import_abs_sub").ok();
    // O filho mora numa pasta que NÃO é a do pai — só alcançável pelo
    // caminho "absoluto" (relativo ao CWD do processo), nunca por um
    // candidato relativo ao diretório do pai.
    let child_path = "templates/import_abs_child_only_here.gv";
    let parent_path = "templates/import_abs_sub/parent.gv";

    std::fs::write(child_path, envolve(r#"<Text content="raiz" />"#)).unwrap();
    std::fs::write(
        parent_path,
        envolve(format!(
            r#"<link rel="import" href="{child_path}" as="Child" /><Column><Child /></Column>"#
        )),
    )
    .unwrap();

    motor
        .register_component("import_abs_parent", parent_path)
        .unwrap();
    let evaluated = motor.evaluated("import_abs_parent").unwrap();
    assert_eq!(
        evaluated.children.len(),
        1,
        "esperava o <Child/> ter resolvido pelo caminho absoluto"
    );
    match &evaluated.children[0].kind {
        NodeType::Text { content, .. } => assert_eq!(content, "raiz"),
        other => panic!("esperava Text, veio {other:?}"),
    }

    std::fs::remove_file(child_path).ok();
    std::fs::remove_file(parent_path).ok();
}

// --- `platform="desktop"`/`"web"` (Fase 1, item 7 — opcional — do plano de
// convergência de templates: deixa cromo só-desktop e só-web no MESMO
// arquivo em vez de forçar dois — ver docs/plano-convergencia-templates-
// gui-webui.md no rustploy) --------------------------------------------------

/// Nenhum alvo hoje compila a wasm32, então `current_platform()` é
/// `"desktop"` sempre, em qualquer suíte de teste — trava esse fato (se
/// algum dia isso rodar num alvo wasm32, o teste avisa em vez de mentir).
#[test]
fn current_platform_e_desktop_neste_alvo() {
    assert_eq!(glacier_ui::eval::current_platform(), "desktop");
}

/// `platform=` funciona em QUALQUER elemento, sozinho — não precisa de
/// `cond`/`if=` nenhum pra existir (é um filtro independente da cadeia
/// if/else-if/else). `web` nunca casa neste alvo; `desktop` sempre casa.
#[test]
fn platform_filtra_sozinho_sem_precisar_de_cond() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_platform_bare.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <Text platform="desktop">só desktop</Text>
            <Text platform="web">só web</Text>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("platformbare", tpl).unwrap();
    motor.set_initial_screen("platformbare");
    motor.reevaluate_all().unwrap();
    assert_eq!(
        all_texts(motor.evaluated("platformbare").unwrap()),
        vec!["só desktop".to_string()],
        "só o nó platform=\"desktop\" deveria sobreviver neste alvo"
    );

    std::fs::remove_file(tpl).ok();
}

/// Combinado com `if`/`else-if` no MESMO nó (`platform` filtra primeiro,
/// sem participar da cadeia — não mexe em `last_if`): um `<else-if
/// platform="web">` nunca casando não atrapalha o branch seguinte.
#[test]
fn platform_combinado_com_if_nao_atrapalha_a_cadeia() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_platform_chain.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <Text if="{x}" equals="a">A</Text>
            <Text else-if="{x}" equals="b" platform="web">B (só web, nunca aqui)</Text>
            <Text else-if="{x}" equals="b">B</Text>
            <Text else>C</Text>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("platformchain", tpl).unwrap();
    motor.set_initial_screen("platformchain");

    for (x, expected) in [("a", "A"), ("b", "B"), ("z", "C")] {
        motor.define_data("x", x);
        motor.reevaluate_all().unwrap();
        assert_eq!(
            all_texts(motor.evaluated("platformchain").unwrap()),
            vec![expected.to_string()],
            "x={x} deveria renderizar só {expected:?}"
        );
    }

    std::fs::remove_file(tpl).ok();
}

// --- `<template>` (estudo pós-Fase-1: unifica `<ForEach>`/`<If>`/
// `<ElseIf>`/`<Else>` numa tag só, mapeando para os mesmos NodeType — ver
// docs/plano-convergencia-templates-gui-webui.md no rustploy e o nome
// `<template x-if>`/`<template x-for>` que a Fase 4 (transpilação p/ Alpine)
// já teria como alvo) --------------------------------------------------

/// `parse_xml` isolado: cada flavor de `<template>` resolve pro `NodeType`
/// certo, sem precisar montar um `GlacierUI` inteiro — mesmo estilo de
/// `hr_e_alias_de_rule`.
#[test]
fn template_resolve_para_o_nodetype_certo_conforme_o_atributo_presente() {
    let for_each = UiNode::parse_xml(r#"<template for-each="items" var="i"><Text/></template>"#)
        .expect("for-each");
    assert!(
        matches!(for_each.kind, NodeType::ForEach { .. }),
        "template com for-each deveria virar NodeType::ForEach, foi {:?}",
        for_each.kind
    );

    // `else` bare (sem valor) só vira `else=""` via `normalize_bare_directives`,
    // que roda no carregamento de arquivo — não em `parse_xml` isolado (ver
    // `normalize_bare_directives_nao_mexe_em_else_com_valor_ja_presente`).
    let else_node = UiNode::parse_xml(r#"<template else=""><Text/></template>"#).expect("else");
    assert!(
        matches!(else_node.kind, NodeType::Else),
        "template com else bare deveria virar NodeType::Else, foi {:?}",
        else_node.kind
    );

    let else_if = UiNode::parse_xml(r#"<template else-if="{x}" equals="a"><Text/></template>"#)
        .expect("else-if");
    assert!(
        matches!(else_if.kind, NodeType::ElseIf { .. }),
        "template com else-if deveria virar NodeType::ElseIf, foi {:?}",
        else_if.kind
    );

    let if_node =
        UiNode::parse_xml(r#"<template if="{x}" equals="a"><Text/></template>"#).expect("if");
    assert!(
        matches!(if_node.kind, NodeType::If { .. }),
        "template com if deveria virar NodeType::If, foi {:?}",
        if_node.kind
    );

    let bare = UiNode::parse_xml(r#"<template><Text/></template>"#).expect("bare");
    assert!(
        matches!(bare.kind, NodeType::If { ref cond, .. } if cond == "true"),
        "template sem atributo nenhum deveria virar um If sempre-verdadeiro, foi {:?}",
        bare.kind
    );
}

/// `<template if=…>`/`<template else-if=…>`/`<template else>` encadeiam
/// exatamente como `<If>`/`<ElseIf>`/`<Else>` — e, como essas tags, cada
/// branch **agrupa múltiplos filhos como irmãos do pai**, sem nenhum
/// `<Column>`/`<Row>` extra por baixo (é isso que a forma-atributo `if=`
/// num elemento comum NÃO consegue: ela sempre produz UM nó — o próprio
/// elemento —, nunca uma lista de irmãos).
#[test]
fn template_if_else_if_else_agrupam_varios_filhos_sem_wrapper() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_template_if_chain.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <template if="{view}" equals="a">
                <Text>A1</Text>
                <Text>A2</Text>
            </template>
            <template else-if="{view}" equals="b">
                <Text>B1</Text>
            </template>
            <template else>
                <Text>C1</Text>
                <Text>C2</Text>
            </template>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("templateifchain", tpl).unwrap();
    motor.set_initial_screen("templateifchain");

    for (view, expected) in [
        ("a", vec!["A1", "A2"]),
        ("b", vec!["B1"]),
        ("z", vec!["C1", "C2"]),
    ] {
        motor.define_data("view", view);
        motor.reevaluate_all().unwrap();
        let evaluated = motor.evaluated("templateifchain").unwrap();
        assert_eq!(evaluated.kind, NodeType::Column);
        // Os filhos do branch que casou estão DIRETO sob o <Column> — sem
        // nenhum nó intermediário representando o <template>.
        assert_eq!(
            evaluated.children.len(),
            expected.len(),
            "view={view}: esperava {} filho(s) direto(s) do Column, não um wrapper",
            expected.len()
        );
        assert_eq!(
            all_texts(evaluated),
            expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "view={view}"
        );
    }

    std::fs::remove_file(tpl).ok();
}

/// `<template for-each=… var=…>` itera como `<ForEach>` — cada iteração
/// pode emitir MAIS de um nó, todos irmãos diretos do pai, sem um wrapper
/// por item.
#[test]
fn template_for_each_itera_varios_filhos_por_item_sem_wrapper() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_template_foreach.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <template for-each="items" var="it">
                <Text>{it.name}</Text>
                <Text>#{it.val}</Text>
            </template>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("templateforeach", tpl).unwrap();

    let data = r#"[{"name": "X", "val": "1"}, {"name": "Y", "val": "2"}]"#;
    motor.define_data("items", data);

    let evaluated = motor.evaluated("templateforeach").unwrap();
    assert_eq!(evaluated.kind, NodeType::Column);
    // 2 itens × 2 nós cada = 4 filhos diretos, sem wrapper por item.
    assert_eq!(evaluated.children.len(), 4);
    assert_eq!(
        all_texts(evaluated),
        vec![
            "X".to_string(),
            "#1".to_string(),
            "Y".to_string(),
            "#2".to_string()
        ]
    );

    std::fs::remove_file(tpl).ok();
}

/// `<template>` sem atributo nenhum é um agrupador incondicional — sempre
/// hoista os filhos como irmãos. Serve pra um componente devolver mais de
/// uma raiz sem precisar de um `<Row>`/`<Column>` artificial.
#[test]
fn template_bare_agrupa_filhos_incondicionalmente() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tpl = "templates/test_template_bare.gv";
    std::fs::write(
        tpl,
        envolve(
            r#"<Column>
            <template>
                <Text>um</Text>
                <Text>dois</Text>
            </template>
            <Text>tres</Text>
        </Column>"#,
        ),
    )
    .unwrap();
    motor.register_component("templatebare", tpl).unwrap();
    motor.set_initial_screen("templatebare");
    motor.reevaluate_all().unwrap();

    let evaluated = motor.evaluated("templatebare").unwrap();
    assert_eq!(
        evaluated.children.len(),
        3,
        "sem wrapper: 3 filhos diretos do Column"
    );
    assert_eq!(
        all_texts(evaluated),
        vec!["um".to_string(), "dois".to_string(), "tres".to_string()]
    );

    std::fs::remove_file(tpl).ok();
}

/// O `title=` do `<screen>` tem de acompanhar o hot-reload: editar o arquivo e
/// salvar troca o título sem recompilar. (Regressão: o `check_reload` tem um
/// caminho próprio de aplicação, separado do registro, e no primeiro corte ele
/// atualizava `<import>`/`<link>` mas esquecia o `<screen>` — o título ficava
/// congelado no valor com que o app subiu.)
#[test]
fn test_screen_meta_reloads_with_template() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    let path = "templates/test_screen_reload.gv";
    std::fs::write(
        path,
        r#"<screen title="Antes" size="800 600"><Text content="x" /></screen>"#,
    )
    .unwrap();
    motor.register_component("tela", path).unwrap();
    motor.set_initial_screen("tela");
    assert_eq!(
        motor.current_screen_meta().and_then(|m| m.title.as_deref()),
        Some("Antes")
    );

    std::thread::sleep(std::time::Duration::from_millis(10));
    std::fs::write(
        path,
        r#"<screen title="Depois" size="900 640"><Text content="x" /></screen>"#,
    )
    .unwrap();
    let _ = filetime_touch(path);
    motor.check_reload();

    let meta = motor
        .current_screen_meta()
        .expect("metadados após o reload");
    assert_eq!(meta.title.as_deref(), Some("Depois"));
    assert_eq!(meta.size, Some((900.0, 640.0)));

    // Trocar o <screen> por um <component> (o cabeçalho sem metadados de
    // janela) também tem de apagar os metadados no motor.
    std::thread::sleep(std::time::Duration::from_millis(10));
    std::fs::write(path, envolve(r#"<Text content="x" />"#)).unwrap();
    let _ = filetime_touch(path);
    motor.check_reload();
    assert!(motor.current_screen_meta().is_none());

    std::fs::remove_file(path).ok();
}

/// `class` escrita no USO de um componente aplica na raiz expandida — e a
/// escada de especificidade que a 0.69 fixou: ela VENCE a classe do template e
/// PERDE para os atributos inline do template. Antes da 0.69 isto era um no-op
/// silencioso: a classe era lida, viajava no mapa de props e não pintava nada.
#[test]
fn class_no_uso_de_componente_pinta_a_raiz_expandida() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();

    // O template do componente declara `background` por CLASSE e `padding`
    // inline — os dois lados da regra, num nó só.
    let comp = "templates/class_uso_comp.gv";
    std::fs::write(
        comp,
        r#"<component>
            <resources><style>
            .interno { background: #111111; border-radius: 3; }
            </style></resources>
            <Column class="interno" padding="7">
                <Text content="oi" />
            </Column>
        </component>"#,
    )
    .unwrap();

    let tela = "templates/class_uso_tela.gv";
    std::fs::write(
        tela,
        envolve(
            r#"
        <style>
        .de_fora { background: #abcdef; padding: 99; }
        </style>
        <Column>
            <Caixa class="de_fora" />
            <Caixa />
        </Column>
        "#,
        ),
    )
    .unwrap();

    motor.register_component("Caixa", comp).unwrap();
    motor.register_component("tela", tela).unwrap();

    let raiz = motor.evaluated("tela").unwrap();
    fn achar_col(n: &glacier_ui::UiNode) -> Option<&glacier_ui::UiNode> {
        if matches!(n.kind, NodeType::Column) && n.children.len() == 2 {
            return Some(n);
        }
        n.children.iter().find_map(achar_col)
    }
    let lista = achar_col(&raiz).expect("a Column com os dois usos");
    let com = &lista.children[0];
    let sem = &lista.children[1];

    // 1. A classe do uso VENCE a classe do template.
    assert_eq!(
        com.background.as_deref(),
        Some("#abcdef"),
        "a classe do uso deve vencer a classe do template"
    );
    // 2. E PERDE para o atributo inline do template.
    assert_eq!(
        com.padding.as_deref(),
        Some("7"),
        "o inline do template deve vencer a classe do uso"
    );
    // 3. O que só o template declara sobrevive (não houve clobber).
    assert_eq!(com.border_radius, Some(3.0), "o resto do template sobrevive");
    // 4. A instância SEM classe não é contaminada pela irmã — é o teste do
    //    cache: as duas têm o mesmo template e node_ids diferentes.
    assert_eq!(
        sem.background.as_deref(),
        Some("#111111"),
        "a instância sem classe fica com o estilo do template"
    );

    std::fs::remove_file(comp).ok();
    std::fs::remove_file(tela).ok();
}

/// O mesmo, mas num BUILTIN da lib (`spinbox`), que é o caso que originou a
/// mudança — e junto as duas props novas: `field_class` e `form_control`
/// chegam ao `<TextInput>` de dentro, e a `<Form>` que envolve o widget
/// hidrata esse campo (id de foco + ação de submit + qual é o controle
/// seguinte, para onde o Enter avança), porque a hidratação roda depois da
/// expansão de componente. Nada disto tem a ver com Tab: a travessia por Tab é
/// um listener global do motor e independe de `formControl`.
#[test]
fn spinbox_repassa_field_class_e_form_control() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let p = "templates/spin_repasse.gv";
    std::fs::write(
        p,
        envolve(
            r#"
        <style>
        .campo_num { background: #123456; }
        .moldura   { border-radius: 9; }
        </style>
        <Form name="f" on_submit="salvar">
            <TextInput formControl="antes" value="antes" />
            <SpinBox value="qtd" min="1" max="9"
                     class="moldura" field_class="campo_num" form_control="qtd" />
            <TextInput formControl="depois" value="depois" />
        </Form>
        "#,
        ),
    )
    .unwrap();
    motor.register_component("proto", p).unwrap();

    let raiz = motor.evaluated("proto").unwrap();
    fn achar_form(n: &glacier_ui::UiNode) -> Option<&glacier_ui::UiNode> {
        if matches!(n.kind, NodeType::Form { .. }) {
            return Some(n);
        }
        n.children.iter().find_map(achar_form)
    }
    let form = achar_form(&raiz).expect("Form");
    let linha = &form.children[1];
    let campo = &linha.children[0];

    // `class` no uso → a Row inteira (o widget), que antes da 0.69 não recebia
    // nada.
    assert_eq!(
        linha.border_radius,
        Some(9.0),
        "class no uso estiliza a raiz do builtin (a Row)"
    );
    // `field_class` → só o campo de dentro. (A classe não sobrevive como
    // string: o eval a resolve em campos de estilo, daí olharmos a cor.)
    assert_eq!(
        campo.background.as_deref(),
        Some("#123456"),
        "field_class estiliza o campo de dentro"
    );
    // `form_control` → o campo entra na Form.
    assert_eq!(campo.form_control.as_deref(), Some("qtd"));
    assert_eq!(
        campo.form_submit_action.as_deref(),
        Some("salvar"),
        "a Form hidrata um controle que só existe depois da expansão"
    );
    assert_eq!(
        campo.form_next_focus.as_deref(),
        Some("depois"),
        "e o Enter avança para o controle seguinte, na ordem do documento"
    );

    std::fs::remove_file(p).ok();
}

/// O teclado do `<datetimeedit>` (0.70): digitar algarismos numa seção, com o
/// avanço automático do `QDateTimeEdit`. O caminho todo — tecla -> descritor
/// `__timeedit` -> chave de contexto — sem depender de um display.
#[test]
fn timeedit_teclado_digita_e_avanca() {
    use glacier_ui::{EngineMessage, TimeEditKey};

    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let p = "templates/te_teclado.gv";
    std::fs::write(
        p,
        envolve(r#"<datetimeedit value="quando" />"#),
    )
    .unwrap();
    motor.register_component("te", p).unwrap();
    motor.define_data("quando", "2026-03-10 08:30");
    // Seleciona a seção da HORA: é o que um clique nela grava.
    motor.define_data("__timeedit", "quando|h|1100||");
    motor.reevaluate_all().unwrap();

    let tecla = |m: &mut GlacierUI, k: TimeEditKey| {
        let _ = m.dispatch(&EngineMessage::TimeEditKey(k));
    };

    // "0" depois "9" -> 09h, e a seção enche: avança sozinha para o minuto.
    tecla(&mut motor, TimeEditKey::Algarismo(0));
    tecla(&mut motor, TimeEditKey::Algarismo(9));
    assert_eq!(motor.context().get("quando").map(String::as_str), Some("2026-03-10 09:30"));
    assert!(
        motor.context().get("__timeedit").unwrap().starts_with("quando|m|"),
        "encheu a hora -> pula para o minuto, como no Qt: {:?}",
        motor.context().get("__timeedit")
    );

    // "4" no minuto: 4 cabe, mas "4X" não passa de 59 em todo X? passa (45),
    // então fica na seção. "5" compõe 45.
    tecla(&mut motor, TimeEditKey::Algarismo(4));
    tecla(&mut motor, TimeEditKey::Algarismo(5));
    assert_eq!(motor.context().get("quando").map(String::as_str), Some("2026-03-10 09:45"));

    // ▲ na seção do minuto: 45 -> 46, e a digitação recomeça (o "4" anterior
    // não pode compor "464").
    tecla(&mut motor, TimeEditKey::Passo(1));
    assert_eq!(motor.context().get("quando").map(String::as_str), Some("2026-03-10 09:46"));
    tecla(&mut motor, TimeEditKey::Algarismo(7));
    assert_eq!(
        motor.context().get("quando").map(String::as_str),
        Some("2026-03-10 09:07"),
        "depois de uma seta, o algarismo recomeça a seção"
    );

    std::fs::remove_file(p).ok();
}

/// ← → andam entre as seções sem alterar valor, e cada seção vira dentro de si.
#[test]
fn timeedit_teclado_move_secao_e_seta_satura() {
    use glacier_ui::{EngineMessage, TimeEditKey};

    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let p = "templates/te_move.gv";
    std::fs::write(p, envolve(r#"<timeedit value="hora" />"#)).unwrap();
    motor.register_component("te2", p).unwrap();
    motor.define_data("hora", "23:59");
    motor.define_data("__timeedit", "hora|h|0100||");
    motor.reevaluate_all().unwrap();

    let tecla = |m: &mut GlacierUI, k: TimeEditKey| {
        let _ = m.dispatch(&EngineMessage::TimeEditKey(k));
    };

    // ▲ na hora: 23 vira 00 (cada seção vira DENTRO de si — o minuto não muda).
    tecla(&mut motor, TimeEditKey::Passo(1));
    assert_eq!(motor.context().get("hora").map(String::as_str), Some("00:59"));

    // → move para o minuto sem tocar no valor.
    tecla(&mut motor, TimeEditKey::Move(1));
    assert_eq!(motor.context().get("hora").map(String::as_str), Some("00:59"));
    assert!(motor.context().get("__timeedit").unwrap().starts_with("hora|m|"));

    // ▲ no minuto: 59 vira 00, e a hora continua onde estava.
    tecla(&mut motor, TimeEditKey::Passo(1));
    assert_eq!(
        motor.context().get("hora").map(String::as_str),
        Some("00:00"),
        "o minuto vira dentro de si, sem empurrar a hora"
    );

    std::fs::remove_file(p).ok();
}

/// Sem seção selecionada, a tecla não é do widget: o motor a ignora em vez de
/// mexer em alguma chave por conta própria.
#[test]
fn timeedit_teclado_sem_selecao_nao_faz_nada() {
    use glacier_ui::{EngineMessage, TimeEditKey};

    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let p = "templates/te_sem_sel.gv";
    std::fs::write(p, envolve(r#"<timeedit value="hora" />"#)).unwrap();
    motor.register_component("te3", p).unwrap();
    motor.define_data("hora", "08:30");
    motor.reevaluate_all().unwrap();

    let _ = motor.dispatch(&EngineMessage::TimeEditKey(TimeEditKey::Passo(1)));
    assert_eq!(motor.context().get("hora").map(String::as_str), Some("08:30"));

    std::fs::remove_file(p).ok();
}

/// Clicar em outra coisa abandona a seção selecionada — senão as setas ▲▼
/// continuariam mexendo num `<datetimeedit>` de longe, depois de o usuário já
/// ter saído dele.
#[test]
fn timeedit_clique_em_outro_widget_larga_a_secao() {
    use glacier_ui::{EngineMessage, TimeEditKey};

    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let p = "templates/te_larga.gv";
    std::fs::write(
        p,
        envolve(
            r#"<column>
                 <timeedit value="hora" />
                 <button text="Outro" on_click="nada" />
               </column>"#,
        ),
    )
    .unwrap();
    motor.register_component("te4", p).unwrap();
    motor.define_data("hora", "08:30");
    motor.define_data("__timeedit", "hora|h|0100||");
    motor.reevaluate_all().unwrap();

    // Com a seção selecionada, a seta funciona.
    let _ = motor.dispatch(&EngineMessage::TimeEditKey(TimeEditKey::Passo(1)));
    assert_eq!(motor.context().get("hora").map(String::as_str), Some("09:30"));

    // Um clique em outro widget larga a seleção...
    let _ = motor.dispatch(&EngineMessage::UiClick("nada".to_string()));
    // ...e a partir daí a seta não é mais dela.
    let _ = motor.dispatch(&EngineMessage::TimeEditKey(TimeEditKey::Passo(1)));
    assert_eq!(
        motor.context().get("hora").map(String::as_str),
        Some("09:30"),
        "depois de clicar fora, ▲ não mexe mais no widget"
    );

    std::fs::remove_file(p).ok();
}
