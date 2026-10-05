//! Regras de leitura do `.gvb` que não podem regredir.

use glacier_ui::gvb::desugar;

fn xml(s: &str) -> String {
    desugar(s).unwrap().split_whitespace().collect::<String>()
}

/// Atributos no cabeçalho, entre parênteses; as chaves são só dos filhos.
#[test]
fn atributos_no_cabecalho_e_filhos_nas_chaves() {
    assert_eq!(
        xml("row(class = linha) {\n  Card(n = A)\n  Card(n = B)\n}\n"),
        r#"<rowclass="linha"><Cardn="A"/><Cardn="B"/></row>"#
    );
}

/// A quebra de linha não significa nada dentro dos parênteses, e a última
/// vírgula é opcional.
#[test]
fn atributos_quebram_onde_quiser() {
    let a = xml("rangeslider(start = lo, end = hi, min = 0)");
    let b = xml("rangeslider(\n  start = lo,\n  end = hi,\n  min = 0,\n)");
    assert_eq!(a, b);
    assert_eq!(a, r#"<rangesliderstart="lo"end="hi"min="0"/>"#);
}

/// O texto é um atributo, em uma linha e em `"""` (que colapsa o espaço como a
/// prosa do `.gva`); o texto solto depois do cabeçalho é erro.
#[test]
fn texto_e_atributo() {
    assert_eq!(
        xml(r#"text(class = tag_off, content = "parado")"#),
        r#"<textclass="tag_off"content="parado"/>"#
    );
    let prosa = desugar("button(text = \"\"\"\n  Ir para\n  o monitor\n\"\"\", on_click = ir)").unwrap();
    assert!(prosa.contains(r#"text="Ir para o monitor""#), "{prosa}");
    let e = desugar(r#"button(on_click = ir) "Ir""#).unwrap_err();
    assert!(e.message.contains("texto solto"), "{}", e.message);
}

/// O corpo cru de um `script`/`style` é a exceção: fica depois do cabeçalho.
#[test]
fn corpo_cru_de_script() {
    assert_eq!(
        xml("script(lang = python) \"\"\"x = 1\"\"\""),
        r#"<scriptlang="python">x=1</script>"#
    );
}

/// `@chave` é dado, nome nu é nome de chave; valor com vírgula pede aspas.
#[test]
fn valores() {
    assert_eq!(
        xml(r#"progressbar(value = x, label = @x, items = "a,b")"#),
        r#"<progressbarvalue="x"label="{x}"items="a,b"/>"#
    );
}

/// `each` leva atributos entre parênteses depois da variável.
#[test]
fn each_com_atributos() {
    assert_eq!(
        xml("each @itens as i(fallback = Vazio) { Item(n = @i) }"),
        r#"<foreachitems="itens"var="i"fallback="Vazio"><Itemn="{i}"/></foreach>"#
    );
}

/// O que a sintaxe antiga escrevia deixa de ser aceito, com a dica do novo.
#[test]
fn atributo_solto_dentro_das_chaves_e_erro() {
    let e = desugar("button { text: x }").unwrap_err();
    assert!(e.message.contains("fora dos parênteses"), "{}", e.message);
}

/// A ligação leva `:` no nome do atributo e dessugara para `:nome="chave"`; o
/// valor (`@x`) e o texto continuam como eram.
#[test]
fn ligacao_leva_dois_pontos() {
    assert_eq!(
        xml("progressbar(:value = progresso, max = 10)"),
        r#"<progressbar:value="progresso"max="10"/>"#
    );
    // o nome montado a partir de um dado também é uma ligação
    assert_eq!(
        xml(r#"input(:value = "campo_@id")"#),
        r#"<input:value="campo_{id}"/>"#
    );
    // o que não é ligação segue sendo texto, e `@x` o valor
    assert_eq!(
        xml("text(content = @progresso, class = nota)"),
        r#"<textcontent="{progresso}"class="nota"/>"#
    );
}

/// Uma ligação sem o `:` é erro no `.gvb` — a família de bug que falhava calada
/// (`value = @pct` num `progressbar` procurava uma chave chamada "42").
#[test]
fn ligacao_sem_dois_pontos_e_erro() {
    for src in [
        "progressbar(value = progresso)",
        "progressbar(value = @progresso)",
        "checkbox(label = Ok, checked = ativo)",
        "tabbar(value = aba, :items = abas)",
    ] {
        let e = desugar(src).unwrap_err();
        assert!(e.message.contains("precisa do `:`"), "{src}: {}", e.message);
    }
    // um `radio` guarda o valor dele em `value`, e a ligação é o `group`
    assert!(desugar("radio(label = Mensal, value = mensal, :group = plano)").is_ok());
    // o `<check>` de uma `<tray>` é um item de bandeja, não um checkbox
    assert!(desugar("tray { check(label = Avisos, checked = @avisos) }").is_ok());
}

/// Um atributo sem `= valor` é um marcador: `prop(component, name = linha)`.
#[test]
fn marcador_sem_valor() {
    assert_eq!(
        xml("prop(component, name = linha)"),
        r#"<propcomponent=""name="linha"/>"#
    );
    assert_eq!(xml("template(else) { text(content = x) }"), r#"<templateelse=""><textcontent="x"/></template>"#);
    assert_eq!(xml("template(else, class = a)"), r#"<templateelse=""class="a"/>"#);
}
