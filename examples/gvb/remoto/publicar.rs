//! Assina uma pasta para servir com `HttpAssets`: gera (na primeira vez) o par
//! de chaves e grava o `manifest.json` assinado dentro da pasta.
//!
//!   cargo run --example publicar_remoto -- examples/gvb/remoto/site /tmp/glacier.chave
//!
//! Imprime a chave pública em hexadecimal — é ela que vai no código do cliente.
//! O arquivo da chave é a parte SECRETA: não vai para o repositório.

use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let (Some(pasta), Some(chave)) = (args.next(), args.next()) else {
        eprintln!("uso: publicar_remoto <pasta> <arquivo-da-chave>");
        std::process::exit(2);
    };
    let chave = PathBuf::from(chave);
    let pkcs8 = if chave.exists() {
        std::fs::read(&chave)?
    } else {
        let nova = glacier_ui::remote_assets::nova_chave()?;
        std::fs::write(&chave, &nova)?;
        eprintln!("chave nova gravada em {}", chave.display());
        nova
    };
    glacier_ui::remote_assets::publicar_manifesto(PathBuf::from(&pasta).as_path(), &pkcs8)?;
    let publica = glacier_ui::remote_assets::chave_publica_de(&pkcs8)?;
    let hex: String = publica.iter().map(|b| format!("{b:02x}")).collect();
    println!("manifest.json assinado em {pasta}/manifest.json");
    println!("chave pública: {hex}");
    Ok(())
}
