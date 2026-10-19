//! Helpers de `build.rs` para apps glacier-ui. Ver o README.
//!
//! Tudo aqui lê o ambiente do cargo (`CARGO_MANIFEST_DIR`, `OUT_DIR`,
//! `CARGO_CFG_TARGET_OS`…) e fala com o cargo por `cargo:` na saída padrão.

use std::path::{Path, PathBuf};

/// Extensões de imagem que [`stage_images`] leva para o staging.
const IMAGE_EXTS: &[&str] = &[
    "png", "svg", "webp", "jpg", "jpeg", "gif", "ico", "avif", "bmp",
];

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} não definido (rode dentro de um build.rs)"))
}

fn ext_lower(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

// ── Windows ──────────────────────────────────────────────────────────────────

/// Embute o `.rc` (ícone, manifest, `VERSIONINFO`) no `.exe`; no-op fora do Windows.
///
/// Passa ao `.rc` duas macros com a versão do crate, prefixadas por `prefix`:
/// `<PREFIX>_VER_COMMA` (`1,2,3,0`) e `<PREFIX>_VER_STR` (`"1.2.3"`). `watch` são
/// os arquivos que o `.rc` inclui (para o cargo refazer o build ao mudarem).
pub fn windows_resource(rc: &str, prefix: &str, watch: &[&str]) {
    println!("cargo:rerun-if-changed={rc}");
    for f in watch {
        println!("cargo:rerun-if-changed={f}");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut parts: Vec<String> = version
        .split(['.', '-', '+'])
        .filter(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_owned)
        .collect();
    while parts.len() < 4 {
        parts.push("0".into());
    }
    let macros = [
        format!("{prefix}_VER_COMMA={}", parts[..4].join(",")),
        format!("{prefix}_VER_STR=\"{version}\""),
    ];
    embed_resource::compile(rc, &macros);
}

// ── Imagens ──────────────────────────────────────────────────────────────────

/// Leva as imagens de `src` (relativo a `CARGO_MANIFEST_DIR`) para
/// `$OUT_DIR/<out_subdir>`, **preservando a estrutura de pastas** e reduzindo os
/// raster cuja maior dimensão passe de `max_dim` (Lanczos3, preservando a
/// proporção, re-encodados como PNG). Vetores e formatos raros são copiados como
/// estão; o que não é imagem fica de fora.
///
/// Serve para embutir logos pequenos: um logo de 512 px mostrado a 30 px custa
/// binário e serrilha na GPU. Embuta o resultado com
/// `glacier_ui::embed_dirs!` (`"$OUT_DIR/<out_subdir>"`).
pub fn stage_images(src: &str, out_subdir: &str, max_dim: u32) {
    let src = Path::new(&env("CARGO_MANIFEST_DIR")).join(src);
    let dst = Path::new(&env("OUT_DIR")).join(out_subdir);
    println!("cargo:rerun-if-changed={}", src.display());
    let _ = std::fs::remove_dir_all(&dst);
    std::fs::create_dir_all(&dst).expect("criar staging de imagens");
    copy_images(&src, &dst, max_dim);
}

fn copy_images(src: &Path, dst: &Path, max_dim: u32) {
    let entries = std::fs::read_dir(src).unwrap_or_else(|e| panic!("lendo {}: {e}", src.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            copy_images(&path, &dst.join(entry.file_name()), max_dim);
            continue;
        }
        let ext = ext_lower(&path);
        let Some(ext) = ext.as_deref().filter(|e| IMAGE_EXTS.contains(e)) else {
            continue;
        };
        std::fs::create_dir_all(dst).expect("criar diretório de staging");
        let out = dst.join(entry.file_name());
        if matches!(ext, "png" | "webp" | "jpg" | "jpeg" | "gif" | "bmp") {
            let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("lendo {}: {e}", path.display()));
            let staged = downscale_png(&bytes, max_dim).unwrap_or(bytes);
            std::fs::write(&out, staged).unwrap_or_else(|e| panic!("gravando {}: {e}", out.display()));
        } else {
            std::fs::copy(&path, &out).unwrap_or_else(|e| panic!("copiando {}: {e}", path.display()));
        }
    }
}

/// Reduz a imagem se a maior dimensão passar de `max_dim`; `None` = não precisa
/// (ou não decodifica), e o chamador usa os bytes originais.
fn downscale_png(bytes: &[u8], max_dim: u32) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    if img.width().max(img.height()) <= max_dim {
        return None;
    }
    let resized = img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3);
    let mut out = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .ok()?;
    Some(out)
}

// ── .desktop no `cargo install` (Linux) ──────────────────────────────────────

/// O cargo não tem hook de pós-instalação, e o `.desktop`/ícones só chegariam via
/// `.deb`. O `build.rs` roda na máquina do usuário durante o `cargo install`,
/// então é onde dá para gravá-los em `~/.local/share`. Só age quando o build É um
/// `cargo install` (o cargo compila num diretório `cargo-install*`; se isso mudar,
/// só deixa de instalar sozinho e `--install-desktop` do app continua valendo).
/// Nunca falha o build: erro vira `cargo:warning`.
///
/// `bin` é o nome do executável (e o `<bin>.desktop` em `packaging/`); `packaging`
/// é a pasta com `<bin>.desktop` e `icons/hicolor/**`; `env_var` (ex.:
/// `MEUAPP_INSTALL_DESKTOP`) desliga com `=0` e força com `=1`.
pub fn desktop_on_cargo_install(bin: &str, packaging: &str, env_var: &str) {
    println!("cargo:rerun-if-env-changed={env_var}");
    println!("cargo:rerun-if-changed={packaging}/{bin}.desktop");
    println!("cargo:rerun-if-changed={packaging}/icons");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") || !cfg!(target_os = "linux") {
        return;
    }
    match std::env::var(env_var).ok().as_deref() {
        Some("0") => return,
        Some("1") => {}
        _ if is_cargo_install() => {}
        _ => return,
    }
    if let Err(e) = write_desktop_entry(bin, packaging) {
        println!("cargo:warning={bin}: não instalei o .desktop ({e}); use `{bin} --install-desktop`");
    }
}

fn is_cargo_install() -> bool {
    std::env::var("OUT_DIR").is_ok_and(|out| {
        Path::new(&out)
            .components()
            .any(|c| c.as_os_str().to_string_lossy().starts_with("cargo-install"))
    })
}

fn write_desktop_entry(bin: &str, packaging: &str) -> Result<(), String> {
    let pkg = Path::new(&env("CARGO_MANIFEST_DIR")).join(packaging);
    let home = std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .ok_or("HOME não definido")?;
    let home = PathBuf::from(home);
    let data = match std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        Some(x) => PathBuf::from(x),
        None => home.join(".local/share"),
    };
    // Onde o `cargo install` vai pôr o binário: --root não chega aqui, então vale
    // CARGO_INSTALL_ROOT, CARGO_HOME ou ~/.cargo (o padrão do cargo).
    let bin_dir = if let Some(r) = std::env::var_os("CARGO_INSTALL_ROOT").filter(|v| !v.is_empty()) {
        PathBuf::from(r).join("bin")
    } else if let Some(c) = std::env::var_os("CARGO_HOME").filter(|v| !v.is_empty()) {
        PathBuf::from(c).join("bin")
    } else {
        home.join(".cargo/bin")
    };
    let exe = bin_dir.join(bin);
    let exe = exe.to_str().ok_or("caminho do executável não é UTF-8")?;

    let desktop = std::fs::read_to_string(pkg.join(format!("{bin}.desktop"))).map_err(|e| e.to_string())?;
    let apps = data.join("applications");
    write_file(&apps.join(format!("{bin}.desktop")), exec_com(&desktop, exe).as_bytes())?;

    let dst = data.join("icons/hicolor");
    copy_tree(&pkg.join("icons/hicolor"), &dst)?;

    let _ = std::process::Command::new("update-desktop-database").arg("-q").arg(&apps).status();
    let _ = std::process::Command::new("gtk-update-icon-cache").arg("-qtf").arg(&dst).status();
    Ok(())
}

/// Troca a linha `Exec=` pelo caminho absoluto do executável, entre aspas e com o
/// escape do Desktop Entry. Duplica `glacier_ui::desktop_entry` de propósito: este
/// crate não pode depender do motor (é build-dependency).
fn exec_com(desktop: &str, exe: &str) -> String {
    let mut quoted = String::from("\"");
    for c in exe.chars() {
        if matches!(c, '\\' | '"' | '`' | '$') {
            quoted.push_str("\\\\");
        }
        quoted.push(c);
    }
    quoted.push('"');
    desktop
        .lines()
        .map(|l| {
            if l.starts_with("Exec=") {
                format!("Exec={quoted}")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("criar {}: {e}", dir.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("gravar {}: {e}", path.display()))
}

fn copy_tree(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("ler {}: {e}", src.display()))?
        .flatten()
    {
        let (from, to) = (entry.path(), dst.join(entry.file_name()));
        if from.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            write_file(&to, &std::fs::read(&from).map_err(|e| e.to_string())?)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_aponta_para_o_executavel_e_escapa() {
        let out = exec_com("Exec=x\nName=y\n", "/a b/$c");
        assert_eq!(out, "Exec=\"/a b/\\\\$c\"\nName=y\n");
    }

    #[test]
    fn imagem_pequena_nao_e_reduzida_e_grande_sim() {
        let mut png = Vec::new();
        image::RgbaImage::new(200, 100)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        assert!(downscale_png(&png, 256).is_none());
        let reduzido = downscale_png(&png, 50).unwrap();
        let img = image::load_from_memory(&reduzido).unwrap();
        assert_eq!((img.width(), img.height()), (50, 25));
    }
}
