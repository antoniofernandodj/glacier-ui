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
    let a = xml("rangeslider(:start = lo, :end = hi, min = 0)");
    let b = xml("rangeslider(\n  :start = lo,\n  :end = hi,\n  min = 0,\n)");
    assert_eq!(a, b);
    assert_eq!(a, r#"<rangeslider:start="lo":end="hi"min="0"/>"#);
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
        xml(r#"progressbar(:value = x, label = @x, items = "a,b")"#),
        r#"<progressbar:value="x"label="{x}"items="a,b"/>"#
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

/// `l"""` guarda as quebras de linha e tira a indentação comum; o `@nome` segue
/// interpolando e as aspas `"` entram sem escape. É o contrário do `"""`, que
/// dobra as linhas num espaço só.
#[test]
fn bloco_de_linhas() {
    let src = "textarea(placeholder = l\"\"\"\n    services:\n      app:\n        image: @img\n  \"\"\")";
    let x = desugar(src).unwrap();
    assert!(
        x.contains(r#"placeholder="services:&#10;  app:&#10;    image: {img}""#),
        "{x}"
    );
    // aspas dentro, sem escape; linha em branco no meio é preservada (vazia)
    let x = desugar("t(p = l\"\"\"\n  a: [\"sh\", \"-c\"]\n\n  b\n\"\"\")").unwrap();
    assert!(x.contains(r#"p="a: [&quot;sh&quot;, &quot;-c&quot;]&#10;&#10;b""#), "{x}");
    // numa linha só, e o `l` que não abre `"""` segue sendo um valor nu
    assert!(desugar("t(p = l\"\"\"x\"\"\")").unwrap().contains(r#"p="x""#));
    assert!(desugar("t(p = lixo)").unwrap().contains(r#"p="lixo""#));
    // texto na primeira linha entra como está e o recuo comum passa a ser o dele
    let x = desugar("t(p = l\"\"\"a\n   b\n\"\"\")").unwrap();
    assert!(x.contains(r#"p="a&#10;   b""#), "{x}");
}

/// O `l"""` come linhas do `.gvb`, mas o resto do arquivo continua apontando a
/// linha certa: o XML alcança a do próximo nó.
#[test]
fn bloco_de_linhas_nao_desloca_as_linhas() {
    let src = "column {\n  textarea(p = l\"\"\"\n    a\n    b\n  \"\"\")\n  text(content = x)\n}\n";
    let x = desugar(src).unwrap();
    assert_eq!(x.lines().position(|l| l.contains("<text ")), Some(5), "{x}");
    // uma string nunca fechada aponta o `l"""`
    let e = desugar("t(p = l\"\"\"abc)").unwrap_err();
    assert!(e.message.contains("nunca fechada"), "{}", e.message);
}
