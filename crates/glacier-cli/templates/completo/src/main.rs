use glacier_ui::{GlacierDaemon, window};

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main_window(window::Settings {
            decorations: false,
            exit_on_close_request: false,
            ..Default::default()
        })
        .child_window(|_spec, settings| {
            settings.decorations = false;
        })
        .remember_window_geometry(true)
        .storage_dir(diretorio_de_dados())
        .main(|motor| {
            if let Err(erro) = motor.register_component("app", "views/app.gv") {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("app");
        })
        .run()
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
