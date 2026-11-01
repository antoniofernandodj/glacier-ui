mod contador;

use glacier_ui::GlacierDaemon;

use crate::contador::Contador;

fn main() -> glacier_ui::iced::Result {
    let daemon = GlacierDaemon::new();

    // No navegador não há disco: os arquivos de `views/` entram no binário.
    // No desktop eles continuam no disco, que é o que dá o hot-reload.
    //
    // Arquivo novo em `views/` (outra tela, outro .gss, uma imagem) entra
    // nesta lista também. Um arquivo esquecido aqui funciona no desktop e, na
    // web, falha com "não está entre os assets embutidos" no console (F12).
    #[cfg(target_arch = "wasm32")]
    let daemon = daemon.assets(std::sync::Arc::new(glacier_ui::embed_assets![
        "views/contador.gv",
        "views/styles/app.gss",
        "views/styles/theme.json",
    ]));

    daemon
        .main(|motor| {
            if let Err(erro) = motor.register(Box::new(Contador::new())) {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("contador");
        })
        .run()
}
