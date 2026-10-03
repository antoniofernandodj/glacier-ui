use glacier_ui::GlacierDaemon;

/// O app, igual no desktop e no Android.
pub fn run() -> glacier_ui::iced::Result {
    let daemon = GlacierDaemon::new();

    // No Android não há diretório de trabalho com arquivos: `views/` entra no
    // `.so`. No desktop ela segue no disco, que é o que dá o hot-reload.
    //
    // Arquivo novo em `views/` (outra tela, outro .gss, uma imagem) entra nesta
    // lista também. Um arquivo esquecido aqui funciona no desktop e, no
    // celular, falha com "não está entre os assets embutidos" no logcat
    // (`make logcat`).
    #[cfg(target_os = "android")]
    let daemon = daemon.assets(std::sync::Arc::new(glacier_ui::embed_assets![
        "views/app.gv",
        "views/styles/app.gss",
        "views/styles/theme.json",
    ]));

    daemon.main_template("views/app.gv").run()
}

/// O ponto de entrada do `NativeActivity`. O `AndroidApp` precisa ser entregue
/// ao `iced_winit` ANTES de qualquer janela: é com ele que o winit cria o
/// `EventLoop`.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: iced_winit::android::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("{{nome_crate}}"),
    );
    log::info!("android_main: iniciando");

    iced_winit::android::set_android_app(app);

    if let Err(erro) = run() {
        log::error!("o app terminou com erro: {erro}");
    }
}
