//! Materializa um preset no disco: cria o diretório do projeto e escreve nele
//! os arquivos embutidos, com os marcadores substituídos.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::embedded;

/// Um preset do questionário: o conjunto de arquivos sob `templates/<id>/`.
pub struct Preset {
    pub id: &'static str,
    pub titulo: &'static str,
    pub descricao: &'static str,
    /// O que o preset demonstra, mostrado no resumo antes de executar.
    pub destaques: &'static [&'static str],
}

pub static PRESETS: &[Preset] = &[
    Preset {
        id: "completo",
        titulo: "App completo",
        descricao: "Janela sem decoração com titlebar própria, tema + .gss, componentes, navegação, fetch e toasts.",
        destaques: &[
            "views/app.gv com <screen>, <resources> e titlebar custom",
            "views/components/ — componentes com <props>",
            "views/scripts/handlers/ — comportamento em Luau, um módulo por domínio",
            "views/styles/ — theme.json (var()) + app.gss com pseudo-estados e @media",
        ],
    },
    Preset {
        id: "minimo",
        titulo: "Mínimo",
        descricao: "Uma tela, um .gss e um <script> Luau. O menor projeto que ainda mostra a ideia.",
        destaques: &[
            "views/contador.gv — uma <screen> com <script> Luau embutido",
            "views/styles/app.gss — classes em vez de atributos inline",
        ],
    },
    Preset {
        id: "janelas",
        titulo: "Multi-janela + bandeja",
        descricao: "GlacierDaemon com open_window, broadcast entre janelas, ícone de bandeja e instância única.",
        destaques: &[
            "open_window/broadcast/close_window entre janelas isoladas",
            "feature `tray`: o app sobrevive ao fechar a última janela",
            "single_instance + remember_window_geometry + storage_dir",
        ],
    },
    Preset {
        id: "rust",
        titulo: "Componente em Rust",
        descricao: "O trait Component com estado tipado em Rust, em vez de comportamento em Luau.",
        destaques: &[
            "src/contador.rs — impl Component (name/template/init/update)",
            "estado tipado em Rust, template só com markup",
        ],
    },
];

pub fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

/// Extensões cujo conteúdo passa pela substituição de marcadores. O resto
/// (ícones, fontes) é copiado byte a byte — um `replace` num PNG o corromperia.
const TEXTUAIS: &[&str] = &[
    "gv", "gss", "luau", "rs", "toml", "json", "md", "luaurc", "txt",
];

/// Escreve o preset em `destino`. O diretório não pode existir: sobrescrever um
/// projeto já começado seria a única operação irreversível desta CLI.
pub fn criar(
    destino: &Path,
    preset: &str,
    nome: &str,
    versao_motor: &str,
) -> io::Result<Vec<PathBuf>> {
    if destino.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} já existe", destino.display()),
        ));
    }

    let arquivos = embedded::arquivos_do_preset(preset);
    if arquivos.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("preset '{preset}' não tem arquivos embutidos"),
        ));
    }

    let mut escritos = Vec::new();
    for (rel, bytes) in arquivos {
        let rel = renomear(rel);
        let alvo = destino.join(&rel);
        if let Some(pai) = alvo.parent() {
            fs::create_dir_all(pai)?;
        }

        let ehtexto = alvo
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| TEXTUAIS.contains(&e))
            || rel.ends_with(".gitignore")
            || rel.ends_with(".luaurc");

        if ehtexto && let Ok(texto) = std::str::from_utf8(bytes) {
            fs::write(&alvo, substituir(texto, nome, versao_motor))?;
        } else {
            fs::write(&alvo, bytes)?;
        }
        escritos.push(PathBuf::from(rel));
    }
    escritos.sort();
    Ok(escritos)
}

/// Desfaz os dois disfarces que um arquivo veste dentro de `templates/`, ambos
/// para não ser confundido com um arquivo de verdade do repositório do glacier:
///
/// - `gitignore` → `.gitignore`: com o ponto, ele passaria a valer para o
///   próprio repositório, escondendo os arquivos do preset de quem editasse os
///   templates.
/// - `Cargo.toml.template` → `Cargo.toml`: `cargo package` PULA todo
///   subdiretório que tenha um `Cargo.toml` (ele o lê como outro pacote), então
///   com o nome real os presets ficariam de fora do `.crate` publicado — e a
///   CLI instalada do crates.io não teria projeto nenhum para criar. O sufixo
///   também impede que um `cd` para dentro de `templates/` esbarre num pacote
///   que não é membro deste workspace.
fn renomear(rel: &str) -> String {
    let rel = rel.strip_suffix(".template").unwrap_or(rel);
    match rel.rsplit_once('/') {
        Some((dir, "gitignore")) => format!("{dir}/.gitignore"),
        None if rel == "gitignore" => ".gitignore".to_string(),
        _ => rel.to_string(),
    }
}

/// Marcadores em chave dupla — `{{nome}}` — porque a chave simples já é a
/// sintaxe de data binding dos `.gv` (`{contador}`), e um marcador em chave
/// simples seria indistinguível de um binding de verdade.
fn substituir(texto: &str, nome: &str, versao_motor: &str) -> String {
    texto
        .replace("{{nome_projeto}}", nome)
        .replace("{{nome_crate}}", &nome_crate(nome))
        .replace("{{titulo}}", &titulo(nome))
        .replace("{{versao_motor}}", versao_motor)
}

/// Nome do projeto → identificador Rust (`meu-app` → `meu_app`), usado onde o
/// hífen não é aceito: `application_id`, nome de módulo, chave de storage.
pub fn nome_crate(nome: &str) -> String {
    nome.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Nome do projeto → título de janela (`meu-app` → `Meu App`).
fn titulo(nome: &str) -> String {
    nome.split(['-', '_', ' '])
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(primeira) => primeira.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Valida o nome antes de criar qualquer coisa: o `name` de um `Cargo.toml`
/// aceita letras, dígitos, `-` e `_`, e não pode começar com dígito.
pub fn validar_nome(nome: &str) -> Result<(), String> {
    if nome.is_empty() {
        return Err("o nome não pode ser vazio".into());
    }
    if nome.starts_with(|c: char| c.is_ascii_digit()) {
        return Err("o nome não pode começar com dígito (é o nome do crate)".into());
    }
    if let Some(c) = nome
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '-' && *c != '_')
    {
        return Err(format!("caractere inválido para um nome de crate: '{c}'"));
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn titulo_capitaliza_e_troca_separadores() {
        assert_eq!(titulo("meu-app"), "Meu App");
        assert_eq!(titulo("painel_de_controle"), "Painel De Controle");
    }

    #[test]
    fn nome_crate_troca_hifen_por_underscore() {
        assert_eq!(nome_crate("meu-app"), "meu_app");
    }

    #[test]
    fn nome_invalido_e_recusado() {
        assert!(validar_nome("2fast").is_err());
        assert!(validar_nome("meu app").is_err());
        assert!(validar_nome("meu-app_2").is_ok());
    }

    #[test]
    fn marcador_em_chave_dupla_nao_toca_binding_do_gv() {
        let gv = r#"<text content="{contador}" /> <!-- {{nome_projeto}} -->"#;
        let saida = substituir(gv, "meu-app", "0.61");
        assert!(saida.contains("{contador}"));
        assert!(saida.contains("meu-app"));
    }

    #[test]
    fn os_disfarces_do_template_sao_desfeitos_ao_criar() {
        assert_eq!(renomear("gitignore"), ".gitignore");
        assert_eq!(renomear("sub/gitignore"), "sub/.gitignore");
        assert_eq!(renomear("Cargo.toml.template"), "Cargo.toml");
        assert_eq!(renomear("views/app.gv"), "views/app.gv");
    }

    /// Todo preset anunciado no questionário precisa existir de fato — o build
    /// embute `templates/<id>/`, e um `id` sem diretório só falharia na hora de
    /// criar o projeto, depois do usuário responder tudo.
    #[test]
    fn todo_preset_tem_arquivos_embutidos() {
        for p in PRESETS {
            assert!(
                !embedded::arquivos_do_preset(p.id).is_empty(),
                "preset '{}' não tem templates/{}/",
                p.id,
                p.id
            );
        }
    }

    /// E todo preset precisa ser um projeto Cargo de verdade.
    #[test]
    fn todo_preset_tem_cargo_toml_e_main() {
        for p in PRESETS {
            let arquivos = embedded::arquivos_do_preset(p.id);
            // Pelo nome de DEPOIS do `renomear`: é o que o projeto criado tem.
            let tem = |alvo: &str| arquivos.iter().any(|(c, _)| renomear(c) == alvo);
            assert!(tem("Cargo.toml"), "preset '{}' sem Cargo.toml", p.id);
            assert!(tem("src/main.rs"), "preset '{}' sem src/main.rs", p.id);
        }
    }
}
