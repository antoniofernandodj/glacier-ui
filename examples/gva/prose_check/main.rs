//! Verificador, SEM janela, do `<prose syntax content>`: roda o parser de blocos contra
//! entradas difíceis e imprime o que saiu. Ver `docs/TEXTO.md`.
//!
//!   cargo run --example prose_check

use glacier_ui::prose::{Bloco, Sintaxe, de_conteudo};

fn mostrar(titulo: &str, sintaxe: Sintaxe, conteudo: &str) -> Vec<Bloco> {
    println!("\n=== {titulo}");
    let blocos = de_conteudo(sintaxe, conteudo);
    for b in &blocos {
        match b {
            Bloco::Paragrafo(s) => println!("  P   {}", resumo(s)),
            Bloco::Titulo { nivel, spans } => println!("  H{nivel}  {}", resumo(spans)),
            Bloco::Citacao { paragrafos, autoria } => {
                println!("  Q   {} par., autoria: {}", paragrafos.len(), resumo(autoria))
            }
            Bloco::Lista { ordenada, itens } => {
                println!("  L   {} ({} itens)", if *ordenada { "ol" } else { "ul" }, itens.len())
            }
            Bloco::Figura { src, alt, legenda } => {
                println!("  FIG src={src:?} alt={alt:?} legenda={}", resumo(legenda))
            }
            Bloco::Codigo(c) => println!("  PRE {c:?}"),
            Bloco::Regua => println!("  HR"),
            Bloco::Markdown(m) => println!("  MD  {} bytes", m.len()),
            Bloco::Erro(e) => println!("  ERRO {e}"),
        }
    }
    blocos
}

/// `texto[estilo]` por trecho: b=negrito i=itálico c=código, @=link(destino).
fn resumo(spans: &[glacier_ui::parser::TextSpan]) -> String {
    spans
        .iter()
        .map(|s| {
            let mut m = String::new();
            if s.bold { m.push('b') }
            if s.italic { m.push('i') }
            if s.code { m.push('c') }
            if let Some(l) = &s.link {
                m.push_str(&format!("@{:?}", l.href.as_deref().or(l.navigate.as_deref()).or(l.on_click.as_deref())));
            }
            if m.is_empty() { format!("{:?}", s.content) } else { format!("{:?}[{m}]", s.content) }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn contagem(blocos: &[Bloco], f: impl Fn(&Bloco) -> bool) -> usize {
    blocos.iter().filter(|b| f(b)).count()
}

fn main() {
    let mut falhas = 0;
    let mut confere = |nome: &str, ok: bool| {
        println!("{} {nome}", if ok { "ok  " } else { "FALHA" });
        if !ok {
            falhas += 1;
        }
    };

    // 1. HTML de CMS: <p> sem fechar, <li> sem fechar, entidades, script, atributos de evento
    let html = r#"<!DOCTYPE html><html><head><title>x</title><style>p{}</style></head><body>
      <div class="artigo"><h1>Título &amp; subtítulo</h1>
      <p>Primeiro <strong>parágrafo</strong> com <em>ênfase</em> e &ldquo;aspas&rdquo;&nbsp;aqui.
      <p>Segundo, sem fechar o anterior.<br>Quebra de linha.</p>
      <script>alert(1)</script>
      <blockquote><p>Foi uma decisão difícil.</p><footer>Ana Souza</footer></blockquote>
      <ul><li>um<li>dois</ul>
      <figure><img src="fotos/praia.png" alt="Praia" onerror="evil()"><figcaption>A praia ao <b>amanhecer</b>.</figcaption></figure>
      <p><a href="javascript:alert(1)" onclick="x()">link ruim</a> e <a href="https://exemplo.com/a?b=1&amp;c=2">link bom</a>.</p>
      <img src="https://tracker.com/p.gif">
      </div></body></html>"#;
    let b = mostrar("HTML sujo de CMS", Sintaxe::Html, html);
    confere("html: 1 título", contagem(&b, |x| matches!(x, Bloco::Titulo { nivel: 1, .. })) == 1);
    confere("html: <p> sem fechar viram 3 parágrafos", contagem(&b, |x| matches!(x, Bloco::Paragrafo(_))) == 3);
    confere("html: script e style sumiram", !format!("{b:?}").contains("alert"));
    confere("html: citação com autoria", b.iter().any(|x| matches!(x, Bloco::Citacao { autoria, .. } if !autoria.is_empty())));
    confere("html: lista de 2 itens", b.iter().any(|x| matches!(x, Bloco::Lista { itens, .. } if itens.len() == 2)));
    confere("html: figura com legenda rica", b.iter().any(|x| matches!(x, Bloco::Figura { legenda, .. } if legenda.iter().any(|s| s.bold))));
    confere("html: entidades decodificadas", format!("{b:?}").contains("Título & subtítulo") && format!("{b:?}").contains("“aspas”"));
    confere("html: javascript: não vira link", !format!("{b:?}").contains("javascript"));
    confere("html: link https vira link", format!("{b:?}").contains("https://exemplo.com/a?b=1&c=2"));
    confere("html: onclick nunca vira ação", !format!("{b:?}").contains("on_click: Some"));

    // 2. gva dinâmico válido, com figura
    let gva = r#"<h2>Manchete</h2>
      <p>Texto com <b>negrito</b> e <a href="https://exemplo.com">link</a>.</p>
      <figure><img src="a.png" alt="x"/><figcaption>Legenda <i>rica</i></figcaption></figure>"#;
    let b = mostrar("gva dinâmico", Sintaxe::Gva, gva);
    confere("gva: título, parágrafo e figura", b.len() == 3 && matches!(b[2], Bloco::Figura { .. }));

    // 3. gva dinâmico com ação do app: RECUSADO
    let b = mostrar("gva com on_click (deve recusar)", Sintaxe::Gva, r#"<p><a on_click="app::apagar">x</a></p>"#);
    confere("gva: on_click vindo de dado é erro", matches!(b.as_slice(), [Bloco::Erro(e)] if e.contains("on_click")));
    let b = mostrar("gva com navigate_to (deve recusar)", Sintaxe::Gva, r#"<p><a navigate_to="admin">x</a></p>"#);
    confere("gva: navigate_to vindo de dado é erro", matches!(b.as_slice(), [Bloco::Erro(e)] if e.contains("navigate_to")));

    // 4. gva malformado e fora da gramática: erro visível, não pânico
    let b = mostrar("gva malformado", Sintaxe::Gva, "<p>aberto sem fechar");
    confere("gva malformado vira erro", matches!(b.as_slice(), [Bloco::Erro(_)]));
    let b = mostrar("gva com <div> (fora da gramática)", Sintaxe::Gva, "<div>x</div>");
    confere("gva: <div> direto é erro", matches!(b.as_slice(), [Bloco::Erro(_)]));

    // 5. gvb dinâmico: `@` de e-mail NÃO é interpolação
    let gvb = "p {\n  span(\"Escreva para \") a(\"nós\", href = \"mailto:redacao@jornal.com\")\n  span(\" ou ana@jornal.com.\")\n}\nh3(\"Seção\")\nfigure {\n  img(src = \"b.png\", alt = \"y\")\n  figcaption { span(\"Legenda gvb\") }\n}";
    let b = mostrar("gvb dinâmico (com @)", Sintaxe::Gvb, gvb);
    confere("gvb: e-mail com @ fica literal", format!("{b:?}").contains("ana@jornal.com") && !format!("{b:?}").contains("{jornal"));
    confere("gvb: figura com legenda", b.iter().any(|x| matches!(x, Bloco::Figura { legenda, .. } if !legenda.is_empty())));
    let b = mostrar("gvb com on_click (deve recusar)", Sintaxe::Gvb, "p { a(\"x\", on_click = \"app::apagar\") }");
    confere("gvb: on_click vindo de dado é erro", matches!(b.as_slice(), [Bloco::Erro(e)] if e.contains("on_click")));

    // 6. Markdown passa adiante
    let b = mostrar("markdown", Sintaxe::Markdown, "# Oi\n\n![x](a.png \"Legenda\")");
    confere("markdown: um bloco Markdown", matches!(b.as_slice(), [Bloco::Markdown(_)]));

    // 7. tamanho e profundidade
    let enorme = "a".repeat(glacier_ui::prose::LIMITE_BYTES + 1);
    let b = mostrar("conteúdo acima do limite", Sintaxe::Html, &enorme);
    confere("acima do limite vira erro", matches!(b.as_slice(), [Bloco::Erro(_)]));
    let fundo = format!("{}x{}", "<div>".repeat(5000), "</div>".repeat(5000));
    let t = std::time::Instant::now();
    let b = mostrar("5000 níveis de <div>", Sintaxe::Html, &fundo);
    confere("aninhamento absurdo não trava nem entra em pânico", t.elapsed().as_secs() < 5 && !b.is_empty());

    // 8. texto sem nenhuma tag, e vazio
    let b = mostrar("html só texto", Sintaxe::Html, "Só um texto\n\ncom linhas");
    confere("html só texto vira parágrafo", matches!(b.as_slice(), [Bloco::Paragrafo(_)]));
    let b = mostrar("vazio", Sintaxe::Html, "   ");
    confere("vazio não gera blocos", b.is_empty());

    println!("\n{}", if falhas == 0 { "tudo certo".to_string() } else { format!("{falhas} falha(s)") });
    if falhas > 0 {
        std::process::exit(1);
    }
}
