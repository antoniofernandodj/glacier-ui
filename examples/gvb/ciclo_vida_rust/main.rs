//! Ciclo de vida de tela em **Rust**: `Component::on_enter`, `on_leave` e
//! `on_destroy` implementados direto no componente.
//!
//! O markup mora em `inicio.gvb`/`painel.gvb` (estilo em `estilo.gss`); o Rust só
//! tem a lógica.
//!
//! Os ganchos aparecem na tela (contadores) e no terminal. Ao lado roda um
//! servidor SSE + WebSocket real (mesma thread do exemplo `ciclo_vida_luau`,
//! incluída por `#[path]`) — a tela `painel` não abre stream nenhum aqui, porque
//! streams de vida longa são pedidos pela camada Luau (`sse`/`websocket`); veja
//! `ciclo_vida_luau` para `scope = "screen"` em ação.
//!
//! Rode com: `cargo run --example gvb_ciclo_vida_rust`
//! Feche a janela e veja `on_destroy` no terminal.

#[path = "../ciclo_vida_luau/servidor.rs"]
mod servidor;

use glacier_ui::{Component, Context, GlacierDaemon, Template};

/// Tela `inicio`: só navega e mostra o rastro deixado pelos ganchos de `painel`.
struct Inicio;

impl Component for Inicio {
    fn name(&self) -> &str {
        "inicio"
    }

    fn template(&self) -> Template {
        Template::File("examples/gvb/ciclo_vida_rust/inicio.gvb".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("entradas", "0");
        ctx.set("saidas", "0");
        ctx.set("ultimo", "(nenhum)");
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        if action == "ir" {
            ctx.navigate_to("painel");
        }
    }

    /// Fechar a janela: o motor está sendo descartado.
    fn on_destroy(&mut self, _ctx: &mut Context) {
        println!("[inicio] on_destroy — a janela está fechando");
    }
}

/// Tela `painel`: conta as entradas e saídas pelos ganchos.
struct Painel {
    entradas: u32,
    saidas: u32,
}

impl Component for Painel {
    fn name(&self) -> &str {
        "painel"
    }

    fn template(&self) -> Template {
        Template::File("examples/gvb/ciclo_vida_rust/painel.gvb".into())
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        if action == "voltar" {
            ctx.navigate_back();
        }
    }

    /// Chegou nesta tela (depois da troca). Não roda para a tela inicial.
    fn on_enter(&mut self, ctx: &mut Context) {
        self.entradas += 1;
        ctx.set("entradas", self.entradas.to_string());
        ctx.set("ultimo", format!("painel.on_enter #{}", self.entradas));
        println!("[painel] on_enter #{}", self.entradas);
    }

    /// Está saindo desta tela (ainda é a atual). Se houvesse streams com
    /// `scope = "screen"`, o motor os fecharia logo depois.
    fn on_leave(&mut self, ctx: &mut Context) {
        self.saidas += 1;
        ctx.set("saidas", self.saidas.to_string());
        ctx.set("ultimo", format!("painel.on_leave #{}", self.saidas));
        println!("[painel] on_leave #{}", self.saidas);
    }

    fn on_destroy(&mut self, _ctx: &mut Context) {
        // Só a tela ATUAL recebe `on_destroy` ao fechar a janela.
        println!("[painel] on_destroy");
    }
}

fn main() -> iced::Result {
    servidor::iniciar();

    GlacierDaemon::new()
        .main(|motor| {
            motor.register(Box::new(Inicio)).expect("inicio");
            motor
                .register(Box::new(Painel {
                    entradas: 0,
                    saidas: 0,
                }))
                .expect("painel");
            motor.set_initial_screen("inicio");
        })
        .run()
}
