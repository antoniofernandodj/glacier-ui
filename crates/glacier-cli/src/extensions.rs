//! Instalação das extensões de VS Code embutidas no binário.
//!
//! O caminho é: `package.json` → `.vsix` (ver [`crate::vsix`]) → `<editor>
//! --install-extension`. Nada de `npx`/`vsce`: quem roda `cargo install
//! glacier-cli` não necessariamente tem Node, e a extensão é só markup +
//! JavaScript já pronto.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::embedded::{EXTENSOES, Extensao};
use crate::prompt::Estilo;
use crate::vsix::{Manifesto, empacotar};

/// Um editor da família VS Code detectado no `PATH`.
pub struct Editor {
    pub comando: String,
    pub nome: &'static str,
}

/// Comandos procurados, na ordem de preferência.
const CANDIDATOS: &[(&str, &str)] = &[
    ("code", "VS Code"),
    ("code-insiders", "VS Code Insiders"),
    ("cursor", "Cursor"),
    ("codium", "VSCodium"),
    ("windsurf", "Windsurf"),
];

/// Procura os editores no `PATH`. Varredura direta em vez de `which`/`where`:
/// não depende de um utilitário externo e não paga um processo por candidato.
pub fn detectar() -> Vec<Editor> {
    let Some(path) = std::env::var_os("PATH") else {
        return Vec::new();
    };
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();

    // No Windows o executável no PATH é `code.cmd`; no resto, sem sufixo.
    let sufixos: &[&str] = if cfg!(windows) {
        &[".cmd", ".exe", ".bat", ""]
    } else {
        &[""]
    };

    CANDIDATOS
        .iter()
        .filter(|(cmd, _)| {
            dirs.iter().any(|d| {
                sufixos
                    .iter()
                    .any(|s| d.join(format!("{cmd}{s}")).is_file())
            })
        })
        .map(|(cmd, nome)| Editor {
            comando: (*cmd).to_string(),
            nome,
        })
        .collect()
}

/// `true` quando este binário foi compilado com as extensões embutidas (ver
/// `build.rs`: um `cargo publish` sem `make sync-extensions` produz uma CLI sem
/// elas, e é melhor dizer isso do que instalar nada em silêncio).
pub fn disponiveis() -> bool {
    !EXTENSOES.is_empty()
}

/// Nomes legíveis das extensões embutidas, para o resumo do questionário.
pub fn nomes() -> Vec<String> {
    EXTENSOES
        .iter()
        .filter_map(|ext| {
            let m = manifesto(ext).ok()?;
            Some(format!("{} {}", m.id(), m.versao))
        })
        .collect()
}

/// Empacota e instala todas as extensões embutidas no editor dado. Devolve
/// quantas entraram; um erro numa extensão é reportado e não aborta o resto —
/// falhar a segunda não é motivo para desfazer a primeira.
pub fn instalar(editor: &Editor, e: &Estilo) -> io::Result<usize> {
    if EXTENSOES.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "esta build não tem extensões embutidas",
        ));
    }

    let saida_dir = std::env::temp_dir().join("glacier-extensions");
    fs::create_dir_all(&saida_dir)?;

    let mut instaladas = 0;
    for ext in EXTENSOES {
        let manifesto = match manifesto(ext) {
            Ok(m) => m,
            Err(erro) => {
                println!("  {} {}: {erro}", e.vermelho("✘"), ext.origem);
                continue;
            }
        };

        let vsix = saida_dir.join(format!("{}-{}.vsix", manifesto.nome, manifesto.versao));
        fs::write(&vsix, empacotar(&manifesto, ext.arquivos))?;

        let resultado = Command::new(&editor.comando)
            .arg("--install-extension")
            .arg(&vsix)
            .arg("--force")
            .output();

        match resultado {
            Ok(saida) if saida.status.success() => {
                println!(
                    "  {} {} {}",
                    e.verde("✔"),
                    manifesto.id(),
                    e.fraco(&manifesto.versao)
                );
                let _ = fs::remove_file(&vsix);
                instaladas += 1;
            }
            Ok(saida) => {
                // O editor já explica o que houve; repetir a mensagem dele é
                // mais útil do que traduzir um código de saída.
                let motivo = String::from_utf8_lossy(&saida.stderr);
                println!(
                    "  {} {}: {}",
                    e.vermelho("✘"),
                    manifesto.id(),
                    primeira_linha(motivo.trim())
                );
                println!(
                    "      {}",
                    e.fraco(&format!("o .vsix ficou em {}", vsix.display()))
                );
            }
            Err(erro) => {
                println!("  {} {}: {erro}", e.vermelho("✘"), manifesto.id());
                println!(
                    "      {}",
                    e.fraco(&format!("o .vsix ficou em {}", vsix.display()))
                );
            }
        }
    }
    Ok(instaladas)
}

fn manifesto(ext: &Extensao) -> io::Result<Manifesto> {
    let package_json = ext
        .arquivos
        .iter()
        .find(|(rel, _)| *rel == "package.json")
        .map(|(_, bytes)| *bytes)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("extensão '{}' sem package.json", ext.origem),
            )
        })?;
    Manifesto::ler(package_json)
}

fn primeira_linha(s: &str) -> &str {
    s.lines().find(|l| !l.trim().is_empty()).unwrap_or("falhou")
}
