//! Parse de todo `.gv` de um diretório — a régua de "as regras novas da
//! linguagem não quebraram um app real". Só roda com `GLACIER_CORPUS` apontado
//! para o diretório (ex.: as telas do rustploy-gui):
//!
//! ```sh
//! GLACIER_CORPUS=../rustploy/rustploy-gui/views cargo test --test corpus_gv -- --nocapture
//! ```

fn gvs(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            gvs(&p, out);
        } else if p.extension().is_some_and(|x| x == "gv") {
            out.push(p);
        }
    }
}

#[test]
fn todo_gv_do_corpus_parseia() {
    let Ok(raiz) = std::env::var("GLACIER_CORPUS") else {
        eprintln!("GLACIER_CORPUS não definido — nada a fazer");
        return;
    };
    let mut arquivos = Vec::new();
    for d in raiz.split(':') {
        gvs(std::path::Path::new(d), &mut arquivos);
    }
    let mut falhas = Vec::new();
    for a in &arquivos {
        let src = std::fs::read_to_string(a).unwrap();
        // As regras da linguagem (expressões, `else`, `class:`) moram no
        // `desugar`; o parse do XML resultante depende do carregador (scripts
        // embutidos, imports), que o `register_component` cobre nos outros testes.
        let r = glacier_ui::gv::desugar(&src).map(|_| ()).map_err(|d| {
            format!("{}:{}: {}", a.display(), d.line, d.message)
        });
        if let Err(e) = r {
            falhas.push(format!("{}: {e}", a.display()));
        }
    }
    println!("{} arquivos .gv, {} falhas", arquivos.len(), falhas.len());
    assert!(falhas.is_empty(), "{}", falhas.join("\n\n"));
}

/// `GLACIER_CORPUS_MIGRAR=dir`: migra o diretório (ATENÇÃO: grava nele — aponte
/// para uma cópia) e confere que todo `.gv` continua parseando e quantas formas
/// depreciadas sobram.
#[test]
fn migrar_o_corpus() {
    let Ok(raiz) = std::env::var("GLACIER_CORPUS_MIGRAR") else {
        return;
    };
    let mudados = glacier_ui::ferramentas::fmt(std::path::Path::new(&raiz), true, false, false).unwrap();
    let trocas: usize = mudados.iter().map(|m| m.1).sum();
    let mut arquivos = Vec::new();
    gvs(std::path::Path::new(&raiz), &mut arquivos);
    let mut falhas = Vec::new();
    let mut restantes = Vec::new();
    for a in &arquivos {
        let src = std::fs::read_to_string(a).unwrap();
        match glacier_ui::gv::formas_depreciadas(&src) {
            Ok(f) => restantes.extend(f.into_iter().map(|(l, m)| format!("{}:{l}: {m}", a.display()))),
            Err(d) => falhas.push(format!("{}:{}: {}", a.display(), d.line, d.message)),
        }
    }
    println!("{} arquivos mudados, {trocas} trocas; {} formas restantes", mudados.len(), restantes.len());
    for r in &restantes {
        println!("  {r}");
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// `GLACIER_MIGRAR_MD=arquivo.md`: migra os blocos ```gv de um Markdown (grava).
#[test]
fn migrar_markdown() {
    let Ok(p) = std::env::var("GLACIER_MIGRAR_MD") else {
        return;
    };
    let src = std::fs::read_to_string(&p).unwrap();
    let (novo, n) = glacier_ui::ferramentas::migrar_markdown(&src);
    std::fs::write(&p, novo).unwrap();
    println!("{p}: {n} troca(s)");
}
