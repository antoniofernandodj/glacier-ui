//! `--install-desktop`: integração com o desktop (Linux) para quem instalou o app
//! por `cargo install` — o `.deb` já instala esses arquivos; o cargo não.
//!
//! Grava, só para o usuário atual, o `.desktop` e os ícones hicolor em
//! `$XDG_DATA_HOME` (padrão `~/.local/share`), com `Exec=` apontando para o
//! executável em uso. Os arquivos saem da [`AssetSource`] do app (normalmente
//! embutida com [`embed_dirs!`](crate::embed_dirs)), numa pasta com a disposição:
//!
//! ```text
//! <base>/<id>.desktop
//! <base>/icons/hicolor/<NxN>/apps/<id>.png     (16 32 48 64 128 256, os que existirem)
//! <base>/icons/hicolor/scalable/apps/<id>.svg  (opcional)
//! ```
//!
//! Nunca roda sozinho: só com o argumento `--install-desktop`.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::asset_source::AssetSource;

/// Onde achar os arquivos de integração. Ver [`GlacierDaemon::install_desktop`](crate::GlacierDaemon::install_desktop).
#[derive(Debug, Clone)]
pub struct DesktopEntry {
    /// Pasta lógica na `AssetSource`, com `/` no fim (ex.: `"packaging/"`).
    pub base: String,
    /// Nome dos arquivos: `<id>.desktop`, `<id>.png`, `<id>.svg`.
    pub id: String,
}

const SIZES: &[&str] = &["16x16", "32x32", "48x48", "64x64", "128x128", "256x256"];

/// Trata `--install-desktop`. `Some(código)` = o argumento era esse e o processo
/// deve terminar com ele; `None` = não era, segue o fluxo normal.
pub fn handle_args(entry: &DesktopEntry, assets: &dyn AssetSource) -> Option<i32> {
    if !std::env::args().skip(1).any(|a| a == "--install-desktop") {
        return None;
    }
    Some(match install(entry, assets) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("--install-desktop falhou: {e}");
            1
        }
    })
}

#[cfg(not(target_os = "linux"))]
fn install(_: &DesktopEntry, _: &dyn AssetSource) -> Result<(), String> {
    Err("só há integração de desktop no Linux".into())
}

#[cfg(target_os = "linux")]
fn install(entry: &DesktopEntry, assets: &dyn AssetSource) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("executável atual: {e}"))?;
    let exe = exe.to_str().ok_or("caminho do executável não é UTF-8")?;
    let data = data_home()?;
    let (base, id) = (&entry.base, &entry.id);

    let desktop = assets
        .read_to_string(&format!("{base}{id}.desktop"))
        .map_err(|e| format!("{base}{id}.desktop: {e}"))?;
    let apps = data.join("applications");
    write(&apps.join(format!("{id}.desktop")), exec_com(&desktop, exe).as_bytes())?;

    let hicolor = data.join("icons/hicolor");
    for size in SIZES {
        if let Ok(png) = assets.read_bytes(&format!("{base}icons/hicolor/{size}/apps/{id}.png")) {
            write(&hicolor.join(size).join(format!("apps/{id}.png")), &png)?;
        }
    }
    if let Ok(svg) = assets.read_bytes(&format!("{base}icons/hicolor/scalable/apps/{id}.svg")) {
        write(&hicolor.join(format!("scalable/apps/{id}.svg")), &svg)?;
    }

    // Best-effort: sem essas ferramentas o menu só atualiza no próximo login.
    let _ = Command::new("update-desktop-database").arg("-q").arg(&apps).status();
    let _ = Command::new("gtk-update-icon-cache").arg("-qtf").arg(&hicolor).status();

    println!("Instalado em {} (Exec={exe})", data.display());
    Ok(())
}

#[cfg(target_os = "linux")]
fn data_home() -> Result<PathBuf, String> {
    if let Some(x) = std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(x));
    }
    let home = std::env::var_os("HOME").ok_or("HOME não definido")?;
    Ok(PathBuf::from(home).join(".local/share"))
}

/// Troca a linha `Exec=` pelo caminho absoluto deste executável (entre aspas,
/// como exige a spec do Desktop Entry quando há espaços ou caracteres especiais).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn exec_com(desktop: &str, exe: &str) -> String {
    // Dois níveis de escape: o do valor do arquivo (`\\` → `\`) e o do Exec (`\` + char).
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

#[cfg(target_os = "linux")]
fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("criar {}: {e}", dir.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("gravar {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_aponta_para_o_executavel_e_preserva_o_resto() {
        let desktop = "[Desktop Entry]\nExec=meuapp\nStartupWMClass=meuapp\n";
        let out = exec_com(desktop, "/home/u/.cargo/bin/meuapp");
        assert!(out.contains("Exec=\"/home/u/.cargo/bin/meuapp\"\n"));
        assert!(!out.contains("Exec=meuapp"));
        assert!(out.contains("StartupWMClass=meuapp"));
    }

    #[test]
    fn exec_escapa_caracteres_especiais() {
        let out = exec_com("Exec=x\n", "/a b/$c");
        assert_eq!(out, "Exec=\"/a b/\\\\$c\"\n");
    }
}
