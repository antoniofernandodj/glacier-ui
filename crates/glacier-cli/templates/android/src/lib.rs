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

// O ponto de entrada do `NativeActivity` (`android_main`). A macro registra o
// `AndroidApp` ANTES de qualquer janela, liga o logcat (tag = nome do crate) e
// chama `run()` — o mesmo que o desktop chama. Fora do Android ela não gera nada.
glacier_ui::android_main!(run);
