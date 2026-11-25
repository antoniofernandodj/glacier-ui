//! Fase 1 do `docs/PLANO_LINGUAGEM_GV_GSS.md`: a linguagem de expressão no `.gv`,
//! de ponta a ponta — o `.gv` escreve a expressão, o motor avalia contra o
//! contexto e a árvore avaliada mostra o resultado.

use glacier_ui::{GlacierUI, NodeType, UiNode};

/// Registra `src` (um `.gv`) como componente `nome`, num arquivo temporário.
fn motor_com(nome: &str, src: &str) -> GlacierUI {
    let dir = std::env::temp_dir().join("glacier_linguagem_expressoes");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{nome}.gv"));
    std::fs::write(&path, src).unwrap();
    let mut motor = GlacierUI::new();
    motor.register_component(nome, path.to_str().unwrap()).unwrap();
    motor
}

/// Os textos visíveis da árvore avaliada, na ordem.
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

#[test]
fn if_de_bloco_com_e_ou_nao() {
    let mut m = motor_com(
        "bloco",
        r#"component {
  column {
    if @logado && !@bloqueado {
      text("entra")
    } else if @logado || @convidado {
      text("espera")
    } else {
      text("fora")
    }
  }
}
"#,
    );
    m.define_data("logado", "true");
    m.define_data("bloqueado", "false");
    assert_eq!(textos(&m.evaluated("bloco").unwrap()), ["entra"]);
    m.define_data("bloqueado", "true");
    assert_eq!(textos(&m.evaluated("bloco").unwrap()), ["espera"]);
    m.define_data("logado", "false");
    m.define_data("convidado", "");
    assert_eq!(textos(&m.evaluated("bloco").unwrap()), ["fora"]);
}

#[test]
fn comparacao_numerica_no_if() {
    let mut m = motor_com(
        "cpu",
        r#"component {
  column {
    if @cpu > 80 {
      text("alto")
    }
    if @cpu <= 80 && @cpu >= 0 {
      text("normal")
    }
  }
}
"#,
    );
    m.define_data("cpu", "93.5");
    assert_eq!(textos(&m.evaluated("cpu").unwrap()), ["alto"]);
    // "9" > "80" seria verdade como texto; como número não é.
    m.define_data("cpu", "9");
    assert_eq!(textos(&m.evaluated("cpu").unwrap()), ["normal"]);
}

#[test]
fn if_como_atributo_aceita_expressao() {
    let mut m = motor_com(
        "attr",
        r#"component {
  column {
    text("salvar", if = @sujo && !@ocupado)
    text("lista vazia", if = @itens is empty)
    text("é inteiro", if = @tipo in ["int", "long"])
  }
}
"#,
    );
    m.define_data("sujo", "true");
    m.define_data("ocupado", "false");
    m.define_data("itens", "[]");
    m.define_data("tipo", "long");
    assert_eq!(
        textos(&m.evaluated("attr").unwrap()),
        ["salvar", "lista vazia", "é inteiro"]
    );
    m.define_data("ocupado", "true");
    m.define_data("itens", "[1]");
    m.define_data("tipo", "texto");
    assert!(textos(&m.evaluated("attr").unwrap()).is_empty());
}

#[test]
fn expressao_dentro_de_texto_e_ternario_em_atributo() {
    let mut m = motor_com(
        "texto",
        r#"component {
  column {
    text("{@cpu > 80 ? '⚠ ' : ''}@cpu%")
    text(@max ? "restaurar" : "maximizar")
    text("{len(@lista)} itens")
  }
}
"#,
    );
    m.define_data("cpu", "91");
    m.define_data("max", "true");
    m.define_data("lista", r#"["a","b","c"]"#);
    assert_eq!(
        textos(&m.evaluated("texto").unwrap()),
        ["⚠ 91%", "restaurar", "3 itens"]
    );
    m.define_data("cpu", "12");
    m.define_data("max", "false");
    assert_eq!(
        textos(&m.evaluated("texto").unwrap()),
        ["12%", "maximizar", "3 itens"]
    );
}

/// `if @mensagem` com um texto qualquer: antes era falso (só `true`/`1`/`yes`/`on`
/// eram verdadeiros) e o template tinha de escrever `@mensagem != ""`.
#[test]
fn texto_nao_vazio_e_verdadeiro() {
    let mut m = motor_com(
        "verdade",
        r#"component {
  column {
    if @mensagem {
      text("@mensagem")
    }
  }
}
"#,
    );
    m.define_data("mensagem", "falhou o deploy");
    assert_eq!(textos(&m.evaluated("verdade").unwrap()), ["falhou o deploy"]);
    m.define_data("mensagem", "");
    assert!(textos(&m.evaluated("verdade").unwrap()).is_empty());
}

/// As formas antigas continuam valendo, sem mudar de sentido.
#[test]
fn formas_antigas_continuam() {
    let mut m = motor_com(
        "antigas",
        r#"component {
  column {
    if @aba == "logs" {
      text("logs")
    }
    text("admin", if = @papel, equals = admin)
    text("não admin", if = @papel, not_equals = admin)
    text("largura {largura|fill}")
  }
}
"#,
    );
    m.define_data("aba", "logs");
    m.define_data("papel", "admin");
    assert_eq!(
        textos(&m.evaluated("antigas").unwrap()),
        ["logs", "admin", "largura fill"]
    );
}

/// Mudar uma chave lida só pelo lado direito de um `&&` reavalia a tela (a
/// dependência foi registrada pela expressão).
#[test]
fn dependencia_do_lado_direito_e_registrada() {
    let mut m = motor_com(
        "deps",
        r#"component {
  column {
    if @a && @b {
      text("os dois")
    }
  }
}
"#,
    );
    m.define_data("a", "true");
    m.define_data("b", "false");
    assert!(textos(&m.evaluated("deps").unwrap()).is_empty());
    m.define_data("b", "true");
    assert_eq!(textos(&m.evaluated("deps").unwrap()), ["os dois"]);
}

#[test]
fn erro_de_sintaxe_na_condicao_e_erro_de_parse_com_linha() {
    let src = "component {\n  column {\n    if @a && {\n      text(\"x\")\n    }\n  }\n}\n";
    let err = UiNode::parse_gv_in(src, Some("ruim.gv")).unwrap_err().to_string();
    assert!(err.contains(":3"), "devia citar a linha 3: {err}");
    assert!(err.contains("condição inválida"), "{err}");

    let src = "component {\n  text(\"x\", if = @a ==)\n}\n";
    let err = UiNode::parse_gv_in(src, Some("ruim.gv")).unwrap_err().to_string();
    assert!(err.contains("expressão inválida em `if`"), "{err}");
}

/// O tamanho do primeiro `text` com `conteudo` na árvore avaliada.
fn tamanho_de(no: &UiNode, conteudo: &str) -> Option<f32> {
    if let NodeType::Text { content, size, .. } = &no.kind
        && content == conteudo
    {
        return *size;
    }
    no.children.iter().find_map(|c| tamanho_de(c, conteudo))
}

/// Fase 2: `class:nome = expr` liga a classe pela condição — um nó só, em vez
/// de repetir o nó inteiro para trocar uma classe (o caso do `nav_item`).
#[test]
fn classe_condicional_liga_e_desliga_a_regra() {
    let mut m = motor_com(
        "classe",
        r#"component {
  resources {
    style """
    .item { size: 12; }
    .item--ativo { size: 20; }
    .grande { size: 30; }
    """
  }
  column {
    text("A", class = item, class:item--ativo = @view == "a")
    text("B", class:grande = @view in ["b", "c"])
  }
}
"#,
    );
    m.define_data("view", "a");
    let ev = m.evaluated("classe").unwrap();
    assert_eq!(tamanho_de(&ev, "A"), Some(20.0));
    assert_eq!(tamanho_de(&ev, "B"), None);
    m.define_data("view", "c");
    let ev = m.evaluated("classe").unwrap();
    assert_eq!(tamanho_de(&ev, "A"), Some(12.0));
    assert_eq!(tamanho_de(&ev, "B"), Some(30.0));
}

/// Fase 2: qualquer atributo aceita expressão — o ícone da barra de título
/// troca sem duplicar o botão.
#[test]
fn atributo_qualquer_com_ternario() {
    let m_src = r#"component {
  column {
    svg(src = @maximizada ? "restaurar.svg" : "maximizar.svg")
  }
}
"#;
    let mut m = motor_com("svg_ternario", m_src);
    m.define_data("maximizada", "true");
    let ev = m.evaluated("svg_ternario").unwrap();
    let dbg = format!("{ev:?}");
    assert!(dbg.contains("restaurar.svg"), "{dbg}");
    m.define_data("maximizada", "false");
    let dbg = format!("{:?}", m.evaluated("svg_ternario").unwrap());
    assert!(dbg.contains("maximizar.svg"), "{dbg}");
}

/// Fase 8: `:not(…)` exclui pelo composto, e `:is(…)` multiplica o seletor.
#[test]
fn seletores_not_e_is() {
    let mut m = motor_com(
        "not_is",
        r#"component {
  resources {
    style """
    .item:not(.item--off) { size: 20; }
    text:is(.a, .b) { size: 30; }
    """
  }
  column {
    text("ligado", class = item)
    text("desligado", class = "item item--off")
    text("A", class = a)
    text("B", class = b)
    text("C", class = c)
  }
}
"#,
    );
    let ev = m.evaluated("not_is").unwrap();
    assert_eq!(tamanho_de(&ev, "ligado"), Some(20.0));
    assert_eq!(tamanho_de(&ev, "desligado"), None);
    assert_eq!(tamanho_de(&ev, "A"), Some(30.0));
    assert_eq!(tamanho_de(&ev, "B"), Some(30.0));
    assert_eq!(tamanho_de(&ev, "C"), None);
}

/// O valor de ação que um `.gv` com `nome(args)` produz depois de avaliado.
fn acao_de(no: &UiNode) -> Option<String> {
    if let NodeType::Button { on_click, .. } = &no.kind
        && let Some(a) = on_click
    {
        return Some(a.clone());
    }
    no.children.iter().find_map(acao_de)
}

/// Fase 5: `on_click = nome(@x, true)` vira `nome(["…",true])` no nó avaliado;
/// `window.resize("nw")` vira a forma antiga dos embutidos.
#[test]
fn acao_com_argumentos_no_gv() {
    let mut m = motor_com(
        "acoes",
        r#"component {
  column {
    button("Exportar", on_click = exportar(@item.key, true, 2))
  }
}
"#,
    );
    m.define_data("item.key", "API_URL");
    let a = acao_de(&m.evaluated("acoes").unwrap()).unwrap();
    assert!(a.ends_with(r#"exportar(["API_URL",true,2])"#), "{a}");

    let mut m = motor_com(
        "acoes_embutidas",
        r#"component {
  column {
    button("↘", on_click = window.resize("se"))
  }
}
"#,
    );
    let a = acao_de(&m.evaluated("acoes_embutidas").unwrap()).unwrap();
    assert!(a.ends_with("window:resize:se"), "{a}");

    // Embutido despachado em Rust pelo prefixo: a forma de chamada volta a
    // `textarea_end:log`, e não vira a chamada JSON de um handler Luau.
    let mut m = motor_com(
        "acoes_textarea",
        r#"component {
  column {
    button("↓", on_click = textarea_end("log"))
  }
}
"#,
    );
    let a = acao_de(&m.evaluated("acoes_textarea").unwrap()).unwrap();
    assert!(a.ends_with("textarea_end:log"), "{a}");
}

/// Fase 5: `t(…)` sem aspas nem chaves — posicional e em atributo — dá o mesmo
/// que a forma longa `"{t('…')}"`, e o argumento nomeado aceita expressão.
#[test]
fn i18n_curto_igual_ao_longo() {
    let mut m = motor_com(
        "i18n_curto",
        r#"component {
  column {
    text(t("comum.salvar"))
    text("{t('comum.salvar')}")
    button(t("comum.n_itens", n = @total), on_click = nada)
  }
}
"#,
    );
    m.define_data("total", "3");
    let t = textos(&m.evaluated("i18n_curto").unwrap());
    assert_eq!(t[0], t[1], "curto e longo devem dar o mesmo: {t:?}");
}

fn desabilitado(no: &UiNode, rotulo: &str) -> Option<bool> {
    if let NodeType::Button { text, .. } = &no.kind
        && text == rotulo
    {
        return Some(no.disabled == Some(true));
    }
    no.children.iter().find_map(|c| desabilitado(c, rotulo))
}

/// Fase 6: com `busy_buttons`, o botão cujo handler está ocupado
/// (`__busy.<handler>`) sai desabilitado sem `disabled = @busy_x`; `busy = false`
/// desliga isso num nó.
#[test]
fn botao_ocupado_automatico() {
    glacier_ui::eval::set_busy_buttons(true);
    let mut m = motor_com(
        "ocupado",
        r#"component {
  column {
    button("Salvar", on_click = salvar)
    button("Exportar", on_click = exportar(@id))
    button("Livre", on_click = salvar, busy = false)
  }
}
"#,
    );
    m.define_data("__busy.salvar", "true");
    m.define_data("__busy.exportar", "true");
    let ev = m.evaluated("ocupado").unwrap();
    assert_eq!(desabilitado(&ev, "Salvar"), Some(true));
    assert_eq!(desabilitado(&ev, "Exportar"), Some(true));
    assert_eq!(desabilitado(&ev, "Livre"), Some(false));
    m.define_data("__busy.salvar", "false");
    let ev = m.evaluated("ocupado").unwrap();
    assert_eq!(desabilitado(&ev, "Salvar"), Some(false));
    glacier_ui::eval::set_busy_buttons(false);
}
