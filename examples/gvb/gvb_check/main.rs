//! Confere o dessugarador do `.gvb`: converte um `.gvb`, parseia, e compara a
//! árvore com a do `.gv` equivalente. `cargo run --example gvb_check -- A.gvb B.gv`
//! (com um terceiro argumento, `--xml`, imprime o XML gerado). `--same` aceita
//! um `.xml` no lugar de um dos lados.
use glacier_ui::parser::UiNode;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `--same A.gvb B.gvb`: as duas escritas dão a mesma árvore? (sem `node_id` nem
    // `line`, e com os campos em ordem). Para conferir uma reformatação.
    if args.first().map(String::as_str) == Some("--same") {
        let tree = |f: &str| -> Result<String, String> {
            let src = std::fs::read_to_string(f).map_err(|e| e.to_string())?;
            // o que o scaffold do CLI substitui
            let src = src
                .replace("{{nome_projeto}}", "meu-app")
                .replace("{{nome_crate}}", "meu_app")
                .replace("{{titulo}}", "Meu App")
                .replace("{{versao_motor}}", "0.0.0");
            // um `.xml` entra como está (o XML que um parser antigo gerou)
            let xml = if f.ends_with(".xml") { src.clone() } else { glacier_ui::gvb::desugar(&src).map_err(|d| d.message)? };
            let (markup, script) = glacier_ui::eval::strip_script(&xml);
            let markup = glacier_ui::eval::normalize_bare_directives(&markup);
            let t = UiNode::parse_xml_with_source(&markup, &src, None).map_err(|e| e.to_string())?;
            let mut ls: Vec<String> = format!("{t:#?}")
                .lines()
                .filter(|l| !l.contains("node_id:") && !l.trim_start().starts_with("line:"))
                .map(|l| l.trim().to_string())
                .collect();
            ls.sort();
            Ok(format!("{}\n--script--\n{}", ls.join("\n"), script.unwrap_or_default()))
        };
        match (tree(&args[1]), tree(&args[2])) {
            (Ok(a), Ok(b)) if a == b => println!("IGUAIS"),
            (Ok(a), Ok(b)) => {
                let (la, lb): (Vec<_>, Vec<_>) = (a.lines().collect(), b.lines().collect());
                let oa: Vec<_> = la.iter().filter(|l| !lb.contains(l)).take(3).collect();
                let ob: Vec<_> = lb.iter().filter(|l| !la.contains(l)).take(3).collect();
                println!("DIFERENTES — só no 1º: {oa:?} · só no 2º: {ob:?}");
            }
            (a, b) => println!("ERRO {:?} / {:?}", a.err(), b.err()),
        }
        return;
    }
    // `--styled a.gvb b.gva …`: avalia cada tela com o `.gss` aplicado e imprime a
    // árvore RESOLVIDA (largura, cor, tamanho… já vindos das classes), sem
    // `node_id`, `line` e os campos de classe. Duas escritas da mesma tela — uma com
    // o estilo inline, outra com ele no `.gss` — têm de imprimir o mesmo.
    if args.first().map(String::as_str) == Some("--styled") {
        for f in &args[1..] {
            let mut ui = glacier_ui::GlacierUI::new();
            println!("=== {f}");
            if let Err(e) = ui.register_component("t", f) {
                println!("ERRO {e}");
                continue;
            }
            match ui.evaluated("t") {
                Ok(t) => {
                    for l in format!("{t:#?}").lines() {
                        let tr = l.trim_start();
                        if tr.starts_with("node_id:") || tr.starts_with("line:") || tr.starts_with("class") {
                            continue;
                        }
                        println!("{l}");
                    }
                }
                Err(e) => println!("ERRO {e}"),
            }
        }
        return;
    }
    // `--load a.gvb b.gva …`: registra cada arquivo pelo CARREGADOR de verdade
    // (leitura, `.gvb` dessugarado, `<script>`, `<link>`), sem abrir janela.
    if args.first().map(String::as_str) == Some("--load") {
        let (mut ok, mut ruim) = (0, 0);
        for f in &args[1..] {
            let mut ui = glacier_ui::GlacierUI::new();
            match ui.register_component("t", f) {
                Ok(()) => ok += 1,
                Err(e) => {
                    ruim += 1;
                    println!("ERRO {f}: {e}");
                }
            }
        }
        println!("{ok} carregados, {ruim} com erro");
        return;
    }
    let gvb = std::fs::read_to_string(&args[0]).expect("lendo o .gvb");
    if matches!(args.get(1).map(String::as_str), Some("--xml" | "--xml-lenient")) {
        // `--xml-lenient` aceita uma ligação sem o `:` (os `.gvb` de antes da marca)
        match glacier_ui::gvb::desugar_with(&gvb, args[1] == "--xml") {
            Ok(x) => print!("{x}"),
            Err(d) => eprintln!("{}", glacier_ui::error::GlacierError::Xml(Box::new(d))),
        }
        return;
    }
    let gv = std::fs::read_to_string(&args[1]).expect("lendo o .gv");
    let a = UiNode::parse_gvb_in(&gvb, Some(&args[0]));
    let b = UiNode::parse_xml_in(&gv, Some(&args[1]));
    match (a, b) {
        (Ok(a), Ok(b)) => {
            let (da, db) = (format!("{a:#?}"), format!("{b:#?}"));
            if da == db {
                println!("IGUAIS ({} linhas de árvore)", da.lines().count());
            } else {
                std::fs::write("/tmp/gvb_a.txt", &da).ok();
                std::fs::write("/tmp/gvb_b.txt", &db).ok();
                println!("DIFERENTES — diff /tmp/gvb_a.txt /tmp/gvb_b.txt");
            }
        }
        (a, b) => {
            println!("gvb: {:?}\ngv: {:?}", a.err().map(|e| e.to_string()), b.err().map(|e| e.to_string()));
        }
    }
}
