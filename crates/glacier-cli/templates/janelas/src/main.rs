//! {{titulo}} — multi-janela com ícone de bandeja.
//!
//! Cada janela é um motor Glacier INDEPENDENTE: contexto e estado isolados,
//! fechar uma não afeta as outras. A coordenação entre elas é explícita, por
//! `open_window` / `broadcast` / `close_window` (ver `views/painel.gv`).
//!
//! Com a bandeja ligada, fechar a última janela não encerra o app — ele se
//! recolhe, e o menu da bandeja controla o ciclo de vida.

use glacier_ui::{
    GlacierDaemon, TrayActions, TrayConfig, TrayItem, notifications_enabled,
    set_notifications_enabled, window,
};

/// Ícone da bandeja, embutido no binário para não depender do diretório de onde
/// o app roda.
const ICONE: &[u8] = include_bytes!("../assets/icone.png");

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        // Título e tamanho saem do `<screen>` de views/painel.gv.
        .main_window(window::Settings {
            icon: window::icon::from_file_data(ICONE, None).ok(),
            ..Default::default()
        })
        .remember_window_geometry(true)
        .storage_dir(diretorio_de_dados())
        .tray(bandeja())
        .on_tray(ao_clicar_na_bandeja)
        // Instância única: com bandeja o app sobrevive ao fechar a janela, e sem
        // isto clicar no lançador de novo abriria um segundo processo enquanto o
        // primeiro segue vivo e invisível. A segunda tentativa pinga esta e sai;
        // a instância viva reabre e foca a janela principal.
        .single_instance("{{nome_crate}}")
        .main(|motor| {
            if let Err(erro) = motor.register_component("painel", "views/painel.gv") {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("painel");
        })
        .run()
}

/// Menu da bandeja. Os `id` daqui são o que chega em [`ao_clicar_na_bandeja`].
fn bandeja() -> TrayConfig {
    TrayConfig {
        icon: ICONE.to_vec(),
        tooltip: "{{titulo}}".to_string(),
        items: vec![
            TrayItem::button("abrir", "Abrir {{titulo}}"),
            // O rótulo começa em "Desligar" porque as notificações começam
            // ligadas (default do motor).
            TrayItem::button("notificacoes", "Desligar notificações"),
            TrayItem::separator(),
            TrayItem::button("sair", "Sair"),
        ],
    }
}

/// `abrir`/`sair` são ações do runner. `notificacoes` alterna o interruptor
/// global e reflete o novo estado no rótulo do próprio item — o menu é a única
/// superfície onde esse estado aparece.
fn ao_clicar_na_bandeja(id: &str, bandeja: &mut TrayActions) {
    match id {
        "abrir" => bandeja.open_main(),
        "sair" => bandeja.quit(),
        "notificacoes" => {
            let ligadas = !notifications_enabled();
            set_notifications_enabled(ligadas);
            bandeja.set_label(
                "notificacoes",
                if ligadas {
                    "Desligar notificações"
                } else {
                    "Ligar notificações"
                },
            );
        }
        _ => {}
    }
}

/// Onde o `storage` do Luau e a geometria lembrada são gravados.
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
