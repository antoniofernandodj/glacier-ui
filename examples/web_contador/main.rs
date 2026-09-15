//! O contador que roda no desktop E no navegador, do mesmo código.
//!
//! Nativo:  `cargo run --example web_contador`
//! Web:     `make web-contador` (compila para wasm32 e serve em localhost:8080)
//!
//! A única diferença entre os dois é de onde vêm os arquivos: no navegador não
//! há disco, então o `.gv`/`.gss`/tema entram no binário por `embed_assets!`. No
//! desktop continua o disco, que é o que dá hot-reload. Ver docs/WEB.md.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Contador {
    valor: i32,
}

impl Component for Contador {
    fn name(&self) -> &str {
        "contador"
    }

    fn template(&self) -> Template {
        Template::File("examples/web_contador/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("contador", self.valor.to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            "zerar" => self.valor = 0,
            _ => return,
        }
        ctx.set("contador", self.valor.to_string());
    }
}

fn main() -> glacier_ui::iced::Result {
    let daemon = GlacierDaemon::new().title("Glacier na web");

    #[cfg(target_arch = "wasm32")]
    let daemon = daemon.assets(std::sync::Arc::new(glacier_ui::embed_assets![
        "examples/web_contador/app.gv",
        "examples/web_contador/app.gss",
        "examples/web_contador/theme.json",
    ]));

    daemon
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Contador { valor: 0 })) {
                eprintln!("Erro ao registrar o componente: {e}");
            }
            motor.set_initial_screen("contador");
        })
        .run()
}
