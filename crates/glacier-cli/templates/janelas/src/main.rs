use glacier_ui::{
    GlacierDaemon, TrayActions, TrayConfig, TrayItem, notifications_enabled,
    set_notifications_enabled, window,
};

const ICONE: &[u8] = include_bytes!("../assets/icone.png");

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main_window(window::Settings {
            icon: window::icon::from_file_data(ICONE, None).ok(),
            ..Default::default()
        })
        .remember_window_geometry(true)
        .storage_dir(diretorio_de_dados())
        .tray(TrayConfig {
            icon: ICONE.to_vec(),
            tooltip: "{{titulo}}".to_string(),
            items: vec![
                TrayItem::button("abrir", "Abrir {{titulo}}"),
                TrayItem::button("notificacoes", "Desligar notificações"),
                TrayItem::separator(),
                TrayItem::button("sair", "Sair"),
            ],
        })
        .on_tray(ao_clicar_na_bandeja)
        .single_instance("{{nome_crate}}")
        .main(|motor| {
            if let Err(erro) = motor.register_component("painel", "views/painel.gv") {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("painel");
        })
        .run()
}

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

fn diretorio_de_dados() -> std::path::PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("{{nome_projeto}}")
}
