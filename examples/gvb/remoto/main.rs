//! Abre um app servido por HTTP — o Glacier como "micro navegador". Ver
//! `docs/PLANO_SUBSTITUIBILIDADE.md`, Fase 1.
//!
//! As duas formas de usar a rede, com o MESMO `HttpAssets`:
//!
//!   app inteiro remoto   cargo run --example remoto -- app    <url> <chave-hex>
//!   só telas remotas     cargo run --example remoto -- telas  <url> <chave-hex>
//!   sem assinatura (dev) ... -- app <url> sem-verificacao
//!   só carregar e sair   ... -- --verificar app <url> <chave-hex>
//!   acompanhar mudanças  ... -- --observar  app <url> <chave-hex>   (10s, sem janela)
//!
//! Para testar localmente:
//!
//!   cargo run --example publicar_remoto -- examples/gvb/remoto/site /tmp/glacier.chave
//!   (cd examples/gvb/remoto/site && python3 -m http.server 8099)
//!   WGPU_BACKEND=gl cargo run --example remoto -- app http://127.0.0.1:8099 <chave-hex>

use std::sync::Arc;

use glacier_ui::{AssetSource, DiskAssets, GlacierDaemon, HttpAssets, RoteadorAssets};

fn uso() -> ! {
    eprintln!("uso: remoto [--verificar] <app|telas> <url> <chave-hex|sem-verificacao>");
    std::process::exit(2);
}

fn main() -> iced::Result {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let so_verificar = args.first().is_some_and(|a| a == "--verificar");
    let observar = args.first().is_some_and(|a| a == "--observar");
    if so_verificar || observar {
        args.remove(0);
    }
    let [modo, url, chave] = args.as_slice() else { uso() };

    let http = if chave == "sem-verificacao" {
        HttpAssets::sem_verificacao(url)
    } else {
        match glacier_ui::remote_assets::chave_publica_de_hex(chave) {
            Ok(k) => HttpAssets::assinado(url, k),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(2);
            }
        }
    };

    if observar {
        // O que o hot-reload enxerga: o `modified` de cada arquivo avança quando o
        // servidor passa a servir uma versão nova (e assinada).
        let _ = http.read_to_string("app.gss");
        let mut visto = http.modified("app.gss");
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let agora = http.modified("app.gss");
            if agora != visto {
                let bytes = http.read_to_string("app.gss").map(|t| t.len()).unwrap_or(0);
                println!("app.gss mudou -> agora {bytes} bytes");
                visto = agora;
            }
            if let Some(e) = http.ultimo_erro() {
                println!("erro visível: {e}");
                break;
            }
        }
        return Ok(());
    }
    if so_verificar {
        // Carrega o app como o motor carregaria e imprime o que chegou.
        for caminho in ["app.gvb", "app.gss", "novidades.gvb"] {
            match http.read_to_string(caminho) {
                Ok(t) => println!("ok    {caminho} ({} bytes)", t.len()),
                Err(e) => {
                    println!("ERRO  {caminho}: {e}");
                    std::process::exit(1);
                }
            }
        }
        return Ok(());
    }

    match modo.as_str() {
        "app" => GlacierDaemon::new()
            .assets(Arc::new(http))
            .main_template("app.gvb")
            .run(),
        "telas" => GlacierDaemon::new()
            .assets(Arc::new(RoteadorAssets::new(DiskAssets).montar("remoto/", http)))
            .main_template("examples/gvb/remoto/local.gvb")
            .run(),
        _ => uso(),
    }
}
