//! A inicialização do Android, escondida do app.
//!
//! O `NativeActivity` carrega a `cdylib` do app e chama `android_main(app)`. O
//! winit só cria o `EventLoop` no Android com esse `AndroidApp` na mão, e o
//! `iced_winit` publicado não tem por onde recebê-lo — o fork
//! `antoniofernandodj/iced_winit` tem (`iced_winit::android::set_android_app`).
//!
//! O app não escreve nada disto: a macro [`android_main!`](crate::android_main)
//! gera o `android_main` e chama [`init`] e [`reportar_erro`]. O mesmo `run()`
//! serve ao desktop e ao celular.
//!
//! Só existe com a feature `android`, no alvo `android`.

pub use iced_winit::android::AndroidApp;

/// Registra o `AndroidApp` no `iced_winit` e liga o logger do `logcat`.
/// Chamada pela macro, antes de qualquer janela; `tag` é o filtro do
/// `adb logcat -s <tag>`.
pub fn init(app: AndroidApp, tag: &str) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag(tag),
    );
    log::info!("android_main: iniciando");
    iced_winit::android::set_android_app(app);
}

/// O `run()` do app terminou com erro: no Android não há terminal, então o
/// único lugar onde ele aparece é o `logcat`.
pub fn reportar_erro(erro: &dyn std::fmt::Display) {
    log::error!("o app terminou com erro: {erro}");
}
