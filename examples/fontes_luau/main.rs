//! **Onda 10, em Luau**: a fonte que o motor não sabia nomear, do lado do
//! script.
//!
//! Rode com: `cargo run --example fontes_luau` (nesta máquina, com
//! `WGPU_BACKEND=gl`).
//!
//! # O que este exemplo mostra e a versão Rust não mostra
//!
//! O **`pick_font{}`** — o `QFontDialog`. Ele entra pela mesma porta do
//! `pick_color{}` da Onda 8 (`prompt{ kind = "font" }`), com corpo em markup
//! (`<fontselect>` + tamanho + amostra) e **zero linhas de diálogo novas**.
//!
//! O resto é a mesma observação da Onda 9/11: o registro de famílias é do
//! motor (`font_named` no builder, abaixo), o `<fontselect>` é markup puro que
//! lê a chave `__fonts`, e o que sobra para o Luau são as quatro coisas de
//! sempre — semear e reagir a um clique. `pick_font` suspende a corrotina como
//! `confirm`/`fetch` e devolve a família (uma string) ou `nil`.
use glacier_ui::GlacierDaemon;

/// Igual ao exemplo `fontes`: lê `.ttf` do sistema e o leaka para `&'static`,
/// para não versionar um binário de fonte no repositório.
fn carrega(caminhos: &[&str]) -> Option<&'static [u8]> {
    for caminho in caminhos {
        if let Ok(bytes) = std::fs::read(caminho) {
            return Some(Box::leak(bytes.into_boxed_slice()));
        }
    }
    None
}

fn main() -> iced::Result {
    let mut daemon = GlacierDaemon::new().title("Glacier — Onda 10 (Luau)");

    let familias: [(&str, &[&str]); 4] = [
        (
            "DejaVu Serif",
            &[
                "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",
                "/usr/share/fonts/dejavu/DejaVuSerif.ttf",
                "/usr/share/fonts/TTF/DejaVuSerif.ttf",
            ],
        ),
        (
            "DejaVu Sans Mono",
            &[
                "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
                "/usr/share/fonts/dejavu/DejaVuSansMono.ttf",
                "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
            ],
        ),
        (
            "Liberation Serif",
            &[
                "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
                "/usr/share/fonts/liberation/LiberationSerif-Regular.ttf",
            ],
        ),
        (
            "Liberation Mono",
            &[
                "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
                "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
            ],
        ),
    ];

    for (nome, caminhos) in familias {
        match carrega(caminhos) {
            Some(bytes) => daemon = daemon.font_named(nome, bytes),
            None => eprintln!("fontes_luau: '{nome}' não encontrada — pulando"),
        }
    }

    daemon
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) =
                motor.register_component("fontes_luau", "examples/fontes_luau/app.gv")
            {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("fontes_luau");
        })
        .run()
}
