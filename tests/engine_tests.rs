use xml_ui::{MotorUI, NoUI, TipoNo};

#[test]
fn test_parser_basic() {
    let xml = r##"
    <Container padding="15" width="200" background="#FFFFFF">
        <Column spacing="10">
            <Text content="Hello World" size="20" bold="true" />
            <Button text="Click Me" onClick="btn_click" />
        </Column>
    </Container>
    "##;

    let ast = NoUI::parse_xml(xml).unwrap();
    
    assert_eq!(ast.tipo, TipoNo::Container);
    assert_eq!(ast.padding.as_deref(), Some("15"));
    assert_eq!(ast.largura.as_deref(), Some("200"));
    assert_eq!(ast.background.as_deref(), Some("#FFFFFF"));
    
    assert_eq!(ast.filhos.len(), 1);
    let column = &ast.filhos[0];
    assert_eq!(column.tipo, TipoNo::Column);
    assert_eq!(column.spacing, Some(10.0));
    
    assert_eq!(column.filhos.len(), 2);
    
    let text = &column.filhos[0];
    if let TipoNo::Text { content, size, bold, .. } = &text.tipo {
        assert_eq!(content, "Hello World");
        assert_eq!(*size, Some(20.0));
        assert!(bold);
    } else {
        panic!("First child of Column should be Text");
    }

    let button = &column.filhos[1];
    if let TipoNo::Button { text, on_click, .. } = &button.tipo {
        assert_eq!(text, "Click Me");
        assert_eq!(on_click.as_deref(), Some("btn_click"));
    } else {
        panic!("Second child of Column should be Button");
    }
}

#[test]
fn test_interpolation() {
    let mut motor = MotorUI::new();
    
    let temp_xml_path = "templates/test_temp.xml";
    std::fs::create_dir_all("templates").ok();
    std::fs::write(
        temp_xml_path,
        r##"<Text content="Welcome, {user_name}! Role: {user_role}" />"##
    ).unwrap();

    motor.registrar_componente("test_comp", temp_xml_path).unwrap();
    
    motor.definir_dado("user_name", "Bob");
    motor.definir_dado("user_role", "Admin");

    let evaluated = motor.evaluated_templates.get("test_comp").unwrap();
    if let TipoNo::Text { content, .. } = &evaluated.tipo {
        assert_eq!(content, "Welcome, Bob! Role: Admin");
    } else {
        panic!("Root node should be evaluated Text");
    }

    std::fs::remove_file(temp_xml_path).ok();
}

#[test]
fn test_includes() {
    let mut motor = MotorUI::new();
    
    std::fs::create_dir_all("templates").ok();
    
    let main_path = "templates/test_main.xml";
    let card_path = "templates/test_card.xml";

    std::fs::write(
        card_path,
        r##"<Container background="#222"><Text content="User: {name}" /></Container>"##
    ).unwrap();

    std::fs::write(
        main_path,
        r##"
        <Column>
            <Include src="test_card" name="Alice" />
            <Include src="test_card" name="Charlie" />
        </Column>
        "##
    ).unwrap();

    motor.registrar_componente("test_card", card_path).unwrap();
    motor.registrar_componente("test_main", main_path).unwrap();

    let evaluated = motor.evaluated_templates.get("test_main").unwrap();
    assert_eq!(evaluated.tipo, TipoNo::Column);
    assert_eq!(evaluated.filhos.len(), 2);

    let first_child = &evaluated.filhos[0];
    assert_eq!(first_child.tipo, TipoNo::Container);
    if let TipoNo::Text { content, .. } = &first_child.filhos[0].tipo {
        assert_eq!(content, "User: Alice");
    } else {
        panic!("Included first child should contain text 'User: Alice'");
    }

    let second_child = &evaluated.filhos[1];
    if let TipoNo::Text { content, .. } = &second_child.filhos[0].tipo {
        assert_eq!(content, "User: Charlie");
    } else {
        panic!("Included second child should contain text 'User: Charlie'");
    }

    std::fs::remove_file(main_path).ok();
    std::fs::remove_file(card_path).ok();
}

#[test]
fn test_componente_por_nome() {
    let mut motor = MotorUI::new();

    std::fs::create_dir_all("templates").ok();

    let main_path = "templates/test_main_comp.xml";
    let card_path = "templates/test_card_comp.xml";

    std::fs::write(
        card_path,
        r##"<Container background="#222"><Text content="User: {name}" /></Container>"##
    ).unwrap();

    // Reuse via the component's own tag name instead of <Include>
    std::fs::write(
        main_path,
        r##"
        <Column>
            <UserCard name="Alice" />
            <UserCard name="Charlie" />
        </Column>
        "##
    ).unwrap();

    // The registered name must match the tag used in the XML.
    motor.registrar_componente("UserCard", card_path).unwrap();
    motor.registrar_componente("test_main_comp", main_path).unwrap();

    let evaluated = motor.evaluated_templates.get("test_main_comp").unwrap();
    assert_eq!(evaluated.tipo, TipoNo::Column);
    assert_eq!(evaluated.filhos.len(), 2);

    let first_child = &evaluated.filhos[0];
    assert_eq!(first_child.tipo, TipoNo::Container);
    if let TipoNo::Text { content, .. } = &first_child.filhos[0].tipo {
        assert_eq!(content, "User: Alice");
    } else {
        panic!("Component first child should contain text 'User: Alice'");
    }

    if let TipoNo::Text { content, .. } = &evaluated.filhos[1].filhos[0].tipo {
        assert_eq!(content, "User: Charlie");
    } else {
        panic!("Component second child should contain text 'User: Charlie'");
    }

    std::fs::remove_file(main_path).ok();
    std::fs::remove_file(card_path).ok();
}

#[test]
fn test_foreach_com_componente() {
    let mut motor = MotorUI::new();

    std::fs::create_dir_all("templates").ok();

    let main_path = "templates/test_lista.xml";
    let card_path = "templates/test_cartao.xml";

    // Componente reutilizável que recebe props.
    std::fs::write(
        card_path,
        r##"<Container background="#222"><Text content="{nome} - {cargo}" /></Container>"##
    ).unwrap();

    // Usa o componente pelo nome dentro de um ForEach, passando campos como props.
    std::fs::write(
        main_path,
        r##"
        <Column>
            <ForEach items="membros" var="m">
                <Cartao nome="{m.nome}" cargo="{m.cargo}" />
            </ForEach>
        </Column>
        "##
    ).unwrap();

    motor.registrar_componente("Cartao", card_path).unwrap();
    motor.registrar_componente("test_lista", main_path).unwrap();

    let data = r#"[
        {"nome": "Ana", "cargo": "Dev"},
        {"nome": "Bruno", "cargo": "Design"}
    ]"#;
    motor.definir_dado("membros", data);

    let evaluated = motor.evaluated_templates.get("test_lista").unwrap();
    assert_eq!(evaluated.tipo, TipoNo::Column);
    assert_eq!(evaluated.filhos.len(), 2);

    // Cada iteração do loop deve produzir o Container do componente,
    // com as props já substituídas pelos valores do item.
    let primeiro = &evaluated.filhos[0];
    assert_eq!(primeiro.tipo, TipoNo::Container);
    if let TipoNo::Text { content, .. } = &primeiro.filhos[0].tipo {
        assert_eq!(content, "Ana - Dev");
    } else {
        panic!("Esperava Text dentro do primeiro cartão");
    }

    if let TipoNo::Text { content, .. } = &evaluated.filhos[1].filhos[0].tipo {
        assert_eq!(content, "Bruno - Design");
    } else {
        panic!("Esperava Text dentro do segundo cartão");
    }

    std::fs::remove_file(main_path).ok();
    std::fs::remove_file(card_path).ok();
}

#[test]
fn test_foreach() {
    let mut motor = MotorUI::new();
    
    let path = "templates/test_foreach.xml";
    std::fs::create_dir_all("templates").ok();
    std::fs::write(
        path,
        r##"
        <Column>
            <ForEach items="items" var="it">
                <Text content="Item: {it.name} ({it.val})" />
            </ForEach>
        </Column>
        "##
    ).unwrap();

    motor.registrar_componente("test_for", path).unwrap();
    
    let data = r#"[
        {"name": "X", "val": "1"},
        {"name": "Y", "val": "2"}
    ]"#;
    motor.definir_dado("items", data);

    let evaluated = motor.evaluated_templates.get("test_for").unwrap();
    assert_eq!(evaluated.tipo, TipoNo::Column);
    assert_eq!(evaluated.filhos.len(), 2);

    if let TipoNo::Text { content, .. } = &evaluated.filhos[0].tipo {
        assert_eq!(content, "Item: X (1)");
    } else {
        panic!("First child should be Text Item: X (1)");
    }

    if let TipoNo::Text { content, .. } = &evaluated.filhos[1].tipo {
        assert_eq!(content, "Item: Y (2)");
    } else {
        panic!("Second child should be Text Item: Y (2)");
    }

    std::fs::remove_file(path).ok();
}

