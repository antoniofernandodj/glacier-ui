//! Prova de conceito da feature `webview`: um botão que abre uma janela cujo
//! conteúdo inteiro é uma webview nativa carregando uma página de verdade.
//! Rode com `cargo run --example webview --features webview`.
//!
//! No Linux, a feature `webview` já força X11 sozinha (ver
//! `GlacierDaemon::run`/`forcar_x11_para_webview` em `src/daemon.rs`) — o
//! `wry` não suporta Wayland nativo nessa plataforma. Isso só tem efeito
//! quando `DISPLAY` já existe (X11 nativo ou XWayland); numa sessão Wayland
//! sem nenhum dos dois, a webview continua indisponível, mas o resto do app
//! roda normal.
//!
//! A demonstração navega direto pra `https://example.com` — sem `<iframe>`
//! embutindo outra coisa, sem servidor local: um site de verdade, mas
//! deliberadamente simples, mantido pela IANA exatamente pra ser usado como
//! alvo de teste, sem política de embedding/anti-abuso pra atrapalhar. Uma
//! versão anterior deste exemplo tentava embutir o IFrame Player do YouTube
//! (um wrapper HTML com `<iframe>`), mas esbarrou em duas camadas de recusa
//! do lado do YouTube — primeiro "Error 153" (a página do wrapper nascia sem
//! origem HTTP de verdade, algo que o `wry` 0.57 não tem como evitar via
//! `with_html`), depois "This video is unavailable" mesmo servindo por HTTP
//! local de verdade (o referrer de IP puro `127.0.0.1` também é recusado).
//! Nenhuma das duas é bug do motor — são políticas do YouTube — então o
//! exemplo evita essa categoria de problema inteira em vez de perseguir mais
//! uma volta de workaround.
use glacier_ui::{Component, Context, GlacierDaemon, Template, WindowSpec};

struct Painel;

impl Component for Painel {
    fn name(&self) -> &str {
        "webview_demo"
    }

    fn template(&self) -> Template {
        Template::File("examples/webview/webview.gv".into())
    }

    fn init(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        if action == "abrir" {
            ctx.open_window(
                WindowSpec::webview("https://example.com")
                    .title("Webview")
                    .size(800.0, 600.0),
            );
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Webview")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Painel)) {
                eprintln!("Erro ao registrar componente: {e}");
            }
            motor.set_initial_screen("webview_demo");
        })
        .run()
}
