//! Cada preset do `glacier new` abre num motor de verdade.
//!
//! O `cargo check` do projeto gerado cobre o Rust, e `tests/exemplos_gv.rs`
//! cobre o parse de cada `.gv` isolado. Falta o que só aparece quando o motor
//! monta o conjunto: um `<link rel="import">` apontando para o diretório errado,
//! uma stylesheet com caminho relativo à pasta errada, um `<script src>` que não
//! existe, um erro de sintaxe no Luau. Nenhum deles falha na compilação — todos
//! falham na primeira vez que alguém roda o projeto recém-criado.
//!
//! O teste materializa o preset num diretório temporário (como a CLI faria),
//! entra nele e registra os templates de entrada.

use std::path::{Path, PathBuf};

use glacier_ui::GlacierUI;

/// Preset e as entradas que ele sobe: o arquivo que o `GlacierDaemon` acha
/// sozinho (`views/app.gvb`, ou o que o `src/main.rs` passa em `main_template`)
/// e, quando há, a tela que uma janela abre só em runtime por `open_window` — o
/// `detalhe` do preset multi-janela, que é uma `screen` do `app` dele e não é
/// alcançada pelo carregamento da janela principal.
const PRESETS: &[(&str, &[(&str, Option<&str>)])] = &[
    ("minimo", &[("views/app.gvb", None)]),
    ("completo", &[("views/app.gvb", None)]),
    (
        "janelas",
        &[
            ("views/painel.gvb", None),
            ("views/painel.gvb", Some("detalhe")),
        ],
    ),
    ("rust", &[("views/contador.gvb", None)]),
    ("wasm32", &[("views/contador.gvb", None)]),
    ("android", &[("views/app.gvb", None)]),
    ("catalogo", &[("views/app.gvb", None)]),
    ("dashboard", &[("views/app.gvb", None)]),
    ("formulario", &[("views/app.gvb", None)]),
    ("crud", &[("views/app.gvb", None)]),
];

/// Sobe `arquivo` como o daemon subiria: raiz `app(...)` é um manifesto
/// (`register_app`, ou `register_app_screen` para a janela de uma tela só);
/// qualquer outra raiz é uma tela (`register_component`). Ver o
/// `template_setup` do `daemon.rs`.
fn carregar(arquivo: &str, tela: Option<&str>) -> glacier_ui::Result<()> {
    let mut motor = GlacierUI::new();
    match tela {
        Some(tela) => motor.register_app_screen(arquivo, tela).map(|_| ()),
        None if motor.is_app_file(arquivo) => motor.register_app(arquivo),
        None => motor.register_component("tela", arquivo),
    }
}

#[test]
fn todo_preset_da_cli_carrega_num_motor() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("crates/glacier-cli/templates");
    let cwd_original = std::env::current_dir().expect("cwd");

    for (preset, entradas) in PRESETS {
        let destino = std::env::temp_dir().join(format!("glacier-preset-{preset}"));
        let _ = std::fs::remove_dir_all(&destino);

        // O `_comum` é mesclado em todo preset pela CLI (ver
        // `glacier_cli::embedded`); sem ele faltaria o `glacier.d.luau`, que não
        // é carregado pelo motor mas é parte do que o preset entrega.
        copiar(&raiz.join("_comum"), &destino);
        copiar(&raiz.join(preset), &destino);

        // Os caminhos dentro dos templates (`views/styles/app.gss`) são
        // relativos ao diretório de onde o app roda — é o que o `chdir` simula.
        std::env::set_current_dir(&destino).expect("entrar no preset");

        for (arquivo, tela) in entradas.iter() {
            if let Err(erro) = carregar(arquivo, *tela) {
                // Volta antes do panic: um cwd deixado no temporário derrubaria
                // os testes seguintes por um motivo sem relação com eles.
                std::env::set_current_dir(&cwd_original).expect("voltar ao cwd");
                panic!("preset '{preset}', {arquivo} {tela:?}: {erro}");
            }
        }

        std::env::set_current_dir(&cwd_original).expect("voltar ao cwd");
        let _ = std::fs::remove_dir_all(&destino);
    }
}

/// Cópia recursiva com substituição dos marcadores — o mesmo que
/// `glacier_cli::scaffold::criar` faz ao materializar o preset.
fn copiar(origem: &Path, destino: &Path) {
    let entradas =
        std::fs::read_dir(origem).unwrap_or_else(|e| panic!("ler {}: {e}", origem.display()));

    std::fs::create_dir_all(destino).expect("criar destino");
    for entrada in entradas.flatten() {
        let de = entrada.path();
        let para: PathBuf = destino.join(entrada.file_name());

        if de.is_dir() {
            copiar(&de, &para);
            continue;
        }
        match std::fs::read_to_string(&de) {
            Ok(texto) => std::fs::write(
                &para,
                texto
                    .replace("{{nome_projeto}}", "preset-teste")
                    .replace("{{nome_crate}}", "preset_teste")
                    .replace("{{titulo}}", "Preset Teste")
                    .replace("{{versao_motor}}", env!("CARGO_PKG_VERSION")),
            )
            .expect("gravar arquivo do preset"),
            // Binário (o ícone da bandeja): copiado como está.
            Err(_) => {
                std::fs::copy(&de, &para).expect("copiar binário do preset");
            }
        }
    }
}
