//! Todo `.gv` do repositório parseia, e todo `.gv` está na forma com cabeçalho.
//!
//! Desde a 0.61 a segunda parte é redundante com a primeira — um arquivo sem
//! cabeçalho não parseia mais. O teste mantém as duas assertivas mesmo assim,
//! porque elas falham por motivos diferentes e a mensagem importa: a primeira
//! pega um exemplo quebrado (o erro do parser diz o quê e onde), a segunda diz
//! qual arquivo ficou para trás numa migração.

use glacier_ui::UiNode;
use std::path::{Path, PathBuf};

fn gvs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            gvs(&p, out);
        } else if p.extension().is_some_and(|e| e == "gv") {
            out.push(p);
        }
    }
}

/// Tira os comentários para "a primeira tag" ser a primeira tag de verdade.
fn sem_comentarios(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut resto = src;
    while let Some(i) = resto.find("<!--") {
        out.push_str(&resto[..i]);
        resto = match resto[i..].find("-->") {
            Some(j) => &resto[i + j + 3..],
            None => "",
        };
    }
    out.push_str(resto);
    out
}

#[test]
fn todo_exemplo_parseia_e_tem_cabecalho() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut arquivos = Vec::new();
    gvs(&raiz.join("examples"), &mut arquivos);
    gvs(&raiz.join("templates"), &mut arquivos);
    arquivos.sort();
    assert!(
        arquivos.len() >= 35,
        "só {} .gv encontrados — o caminho está errado?",
        arquivos.len()
    );

    for arquivo in arquivos {
        let src = std::fs::read_to_string(&arquivo).expect("ler .gv");
        let relativo = arquivo.strip_prefix(raiz).unwrap_or(&arquivo).display();

        let primeira = sem_comentarios(&src)
            .split('<')
            .nth(1)
            .unwrap_or("")
            .trim_start()
            .to_string();
        assert!(
            primeira.starts_with("screen") || primeira.starts_with("component"),
            "{relativo}: começa com <{}…>, e todo .gv começa com <screen> ou <component>",
            primeira.split_whitespace().next().unwrap_or("?")
        );

        // O `<script>` é recortado por texto antes do parse (`strip_script`), e
        // é o motor que faz isso — aqui ele é só mais um filho do <resources>,
        // que a avaliação descarta como qualquer declaração.
        UiNode::parse_xml_in(&src, Some(&relativo.to_string()))
            .unwrap_or_else(|e| panic!("{relativo} não parseia: {e}"));
    }
}
