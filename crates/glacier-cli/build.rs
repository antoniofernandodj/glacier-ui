//! Embute, no binário, as duas árvores de arquivos que a CLI distribui:
//!
//! 1. `templates/` — os presets de projeto que o `glacier new` materializa.
//! 2. As extensões de VS Code (`glacier-view` e `glacier-gss`), que o
//!    `glacier install-extensions` empacota em `.vsix` e manda para o editor.
//!
//! Ambas viram um `&[(caminho, bytes)]` num arquivo gerado em `OUT_DIR`,
//! incluído por `src/embedded.rs`. `include_bytes!` com caminho absoluto é o
//! que faz o conteúdo entrar no binário; o `rerun-if-changed` de cada arquivo é
//! o que faz uma edição em `templates/` recompilar a CLI.
//!
//! ## De onde saem as extensões
//!
//! Em um checkout do repositório elas vivem em `../../editors/`, fora deste
//! crate — e `cargo publish` não empacota nada fora do diretório do crate. Por
//! isso a busca é em duas etapas: primeiro `extensions/` aqui dentro (a cópia
//! que `make sync-extensions` gera antes de publicar, listada no `include` do
//! Cargo.toml), depois `../../editors/`. Publicar sem rodar o `sync` gera uma
//! CLI sem extensões embutidas — o `install-extensions` diz isso em vez de
//! fingir que instalou.

use std::fs;
use std::path::{Path, PathBuf};

/// Padrões que nunca entram no `.vsix` (os mesmos dos `.vscodeignore` das
/// extensões, que não são lidos aqui para não ter um mini-glob no build).
const EXT_EXCLUDED: &[&str] = &[".vscodeignore", ".gitignore", "Makefile", ".vscode"];

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));

    let mut gerado = String::from("// Gerado por build.rs — não edite.\n\n");

    gerado.push_str(&emitir_templates(&manifest.join("templates")));
    gerado.push_str(&emitir_extensoes(&manifest));
    gerado.push_str(&emitir_versao_do_motor(&manifest));

    fs::write(out_dir.join("embedded.rs"), gerado).expect("gravar embedded.rs");
}

/// `TEMPLATES`: todos os arquivos sob `templates/`, com o caminho relativo a ela
/// (`completo/views/app.gv`). O primeiro segmento é o nome do preset.
fn emitir_templates(raiz: &Path) -> String {
    let mut arquivos = Vec::new();
    coletar(raiz, raiz, &mut arquivos, &[]);
    arquivos.sort();

    let mut s = String::from("pub static TEMPLATES: &[(&str, &[u8])] = &[\n");
    for (rel, abs) in &arquivos {
        println!("cargo:rerun-if-changed={}", abs.display());
        s.push_str(&format!("    ({:?}, include_bytes!({:?})),\n", rel, abs));
    }
    s.push_str("];\n\n");
    s
}

/// `EXTENSOES`: uma entrada por diretório de extensão encontrado (o que tem um
/// `package.json` na raiz), com os arquivos que vão para dentro do `.vsix`.
fn emitir_extensoes(manifest: &Path) -> String {
    let raiz = localizar_extensoes(manifest);

    let dirs: Vec<PathBuf> = match &raiz {
        Some(r) => {
            let mut v: Vec<PathBuf> = fs::read_dir(r)
                .map(|it| {
                    it.flatten()
                        .map(|e| e.path())
                        .filter(|p| p.join("package.json").is_file())
                        .collect()
                })
                .unwrap_or_default();
            v.sort();
            v
        }
        None => Vec::new(),
    };

    let mut listas = String::new();
    let mut tabela = String::from("pub static EXTENSOES: &[Extensao] = &[\n");

    for (i, dir) in dirs.iter().enumerate() {
        let mut arquivos = Vec::new();
        coletar(dir, dir, &mut arquivos, EXT_EXCLUDED);
        arquivos.sort();

        let ident = format!("EXT_{i}");
        listas.push_str(&format!("static {ident}: &[(&str, &[u8])] = &[\n"));
        for (rel, abs) in &arquivos {
            println!("cargo:rerun-if-changed={}", abs.display());
            listas.push_str(&format!("    ({:?}, include_bytes!({:?})),\n", rel, abs));
        }
        listas.push_str("];\n\n");

        let nome = dir.file_name().unwrap_or_default().to_string_lossy();
        tabela.push_str(&format!(
            "    Extensao {{ origem: {nome:?}, arquivos: {ident} }},\n"
        ));
    }
    tabela.push_str("];\n\n");

    format!("{listas}{tabela}")
}

/// `extensions/` dentro do crate (cópia de publicação) tem precedência sobre
/// `../../editors/` (o checkout), para que o .crate publicado use o que foi de
/// fato empacotado nele.
fn localizar_extensoes(manifest: &Path) -> Option<PathBuf> {
    let vendored = manifest.join("extensions");
    let checkout = manifest.join("../../editors");

    // Os dois diretórios são observados mesmo quando não existem: `cargo publish`
    // é precedido de um `make sync-extensions`, que CRIA `extensions/`, e sem
    // isto a build seguinte continuaria com os arquivos do checkout embutidos —
    // uma troca de fonte que não recompila é a pior forma de errar.
    println!("cargo:rerun-if-changed={}", vendored.display());
    println!("cargo:rerun-if-changed={}", checkout.display());

    if vendored.is_dir() {
        return Some(vendored);
    }
    if checkout.is_dir() {
        return Some(checkout);
    }
    println!(
        "cargo:warning=nenhuma árvore de extensões encontrada (nem `extensions/`, nem `../../editors/`): \
         a CLI será compilada sem extensões embutidas"
    );
    None
}

/// Versão do motor que os `Cargo.toml` gerados vão pedir, lida do `Cargo.toml`
/// da raiz do workspace para não haver duas fontes de verdade. Só o `MAJOR.MINOR`
/// entra no arquivo gerado: um projeto novo não deve travar no patch.
fn emitir_versao_do_motor(manifest: &Path) -> String {
    let cargo_raiz = manifest.join("../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", cargo_raiz.display());

    let versao = fs::read_to_string(&cargo_raiz)
        .ok()
        .and_then(|txt| {
            txt.lines()
                .take_while(|l| !l.starts_with("[dependencies]"))
                .find_map(|l| {
                    let v = l.strip_prefix("version")?.trim_start().strip_prefix('=')?;
                    Some(v.trim().trim_matches('"').to_string())
                })
        })
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    let mut partes = versao.split('.');
    let curta = match (partes.next(), partes.next()) {
        (Some(maior), Some(menor)) => format!("{maior}.{menor}"),
        _ => versao,
    };

    format!("pub const VERSAO_MOTOR: &str = {curta:?};\n")
}

/// Percorre `dir` recursivamente acumulando `(caminho relativo a `base`, caminho
/// absoluto)`. Diretórios e nomes em `excluir` são pulados.
fn coletar(base: &Path, dir: &Path, saida: &mut Vec<(String, PathBuf)>, excluir: &[&str]) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };
    for entrada in entradas.flatten() {
        let caminho = entrada.path();
        let nome = entrada.file_name().to_string_lossy().to_string();

        if excluir.contains(&nome.as_str()) || nome.ends_with(".vsix") {
            continue;
        }
        if caminho.is_dir() {
            coletar(base, &caminho, saida, excluir);
        } else if let Ok(rel) = caminho.strip_prefix(base) {
            // Sempre `/`, mesmo no Windows: o caminho relativo é a chave usada
            // dentro do zip do .vsix, onde a barra é obrigatória.
            let rel = rel.to_string_lossy().replace('\\', "/");
            saida.push((rel, caminho.clone()));
        }
    }
}
