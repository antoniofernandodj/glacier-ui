//! Confere o dessugarador do `.gvb`: converte um `.gvb`, parseia, e compara a
//! árvore com a do `.gv` equivalente. `cargo run --example gvb_check -- A.gvb B.gv`
//! (com um terceiro argumento, `--xml`, imprime o XML gerado).
use glacier_ui::parser::UiNode;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
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
    if args.get(1).map(String::as_str) == Some("--xml") {
        match glacier_ui::gvb::desugar(&gvb) {
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
