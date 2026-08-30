//! {{titulo}} — casca da janela.
//!
//! Este arquivo NÃO descreve a interface. Ele sobe o runner multi-janela
//! ([`GlacierDaemon`]), registra a tela raiz e configura o que um template não
//! tem como declarar: janela sem decoração do SO, geometria lembrada entre
//! execuções e a raiz gravável do `storage` do Luau.
//!
//! Todo o resto — layout, estilo, navegação, rede — vive em `views/`, e muda a
//! quente: salvar um `.gv` ou um `.gss` com o app aberto já aplica.

use glacier_ui::{GlacierDaemon, window};

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        // Sem `.title()` nem `.main_size()`: quem declara título e tamanho é o
        // `<screen>` de `views/app.gv`, junto da tela que eles descrevem — e o
        // título recarrega a quente, sem recompilar. O builder só opinaria
        // sobre o que o template deixasse em branco.
        .main_window(janela_principal())
        // Janelas-filhas abertas por `open_window(...)` no Luau também nascem
        // sem decoração, porque o template delas traz a própria titlebar; sem
        // isto o SO desenharia a nativa por baixo e elas destoariam.
        .child_window(|_spec, settings| {
            settings.decorations = false;
        })
        // Grava tamanho/posição ao fechar e restaura ao abrir (sob o
        // `storage_dir`). No Wayland só o tamanho volta.
        .remember_window_geometry(true)
        // Raiz gravável do global `storage` do Luau. Sem isto ele gravaria
        // relativo aos assets, que num app instalado costuma ser read-only.
        .storage_dir(diretorio_de_dados())
        .main(|motor| {
            // Um registro só: `views/app.gv` traz os outros por
            // `<link rel="import">`, e o motor os carrega em cascata.
            if let Err(erro) = motor.register_component("app", "views/app.gv") {
                // O Display do erro já traz arquivo:linha:coluna, o trecho e a
                // dica — reembrulhar só esconderia isso.
                eprintln!("{erro}");
            }
            motor.set_initial_screen("app");
        })
        .run()
}

/// Cromo estático da janela principal. Tamanho e mínimo NÃO saem daqui: são do
/// `<screen>` de `views/app.gv`, e a geometria lembrada ganha dos dois no boot.
///
/// `decorations: false` troca a titlebar do SO pela que o template desenha (as
/// ações `window:*` do `app.gv`); `exit_on_close_request: false` faz o pedido
/// de fechar passar pelo daemon, que assim salva a geometria antes de sair.
fn janela_principal() -> window::Settings {
    window::Settings {
        decorations: false,
        exit_on_close_request: false,
        ..Default::default()
    }
}

/// Onde o `storage` do Luau grava. Segue a convenção do SO quando as variáveis
/// de ambiente estão lá (`$XDG_DATA_HOME`/`$APPDATA`), com o diretório atual
/// como último recurso — um app recém-criado não deveria falhar por causa disso.
fn diretorio_de_dados() -> std::path::PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("{{nome_projeto}}")
}
