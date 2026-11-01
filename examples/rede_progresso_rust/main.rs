//! `on_progress` em Rust: a mesma barra do `rede_progresso`, num `Component`.
//!
//!   (o exemplo liga e desliga o servidor de teste sozinho)
//!   WGPU_BACKEND=gl cargo run --example rede_progresso_rust
//!
//! `Request::on_progress` roda na tarefa de rede, não na thread da UI: o closure só anota o
//! número num estado compartilhado, e a `subscription` do componente o entrega ao contexto
//! a cada 100 ms (como um `ContextPatch`).

use std::sync::Mutex;
use std::time::Duration;

use glacier_ui::net::{AbortHandle, Progress, ProgressPhase, Request};
use glacier_ui::{Component, Context, EffectOutcome, EngineMessage, GlacierDaemon, Template};

const BASE: &str = "http://127.0.0.1:8096";
const MB: u64 = 1024 * 1024;

/// O que a rede anotou desde a última entrega à UI.
static ANOTADO: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

fn anota(p: Progress) {
    let fase = match p.phase {
        ProgressPhase::Upload => "upload",
        ProgressPhase::Download => "download",
    };
    let total = p.total.map_or("?".to_string(), |t| (t / MB).to_string());
    let pct = p.total.filter(|t| *t > 0).map_or(0.0, |t| p.done as f64 / t as f64 * 100.0);
    *ANOTADO.lock().unwrap() = vec![
        ("pct".into(), pct.to_string()),
        ("estado".into(), format!("{fase}: {} / {total} MB", p.done / MB)),
    ];
}

#[derive(Default)]
struct Progresso {
    ctl: Option<AbortHandle>,
}

impl Component for Progresso {
    fn name(&self) -> &str {
        "rede_progresso_rust"
    }

    fn template(&self) -> Template {
        Template::File("examples/rede_progresso_rust/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("pct", "0");
        ctx.set("estado", "Escolha uma operação.");
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "enviar" | "baixar" => {
                ctx.set("pct", "0");
                let ctl = AbortHandle::new();
                self.ctl = Some(ctl.clone());
                let enviar = action == "enviar";
                ctx.perform(async move {
                    let req = if enviar {
                        Request::post(format!("{BASE}/enviar")).body("x".repeat((50 * MB) as usize))
                    } else {
                        Request::get(format!("{BASE}/grande?mb=50"))
                    };
                    let r = req.abortable(&ctl).on_progress(anota).send().await;
                    let estado = match (r.ok, enviar) {
                        (true, true) => format!("pronto: {}", r.body),
                        (true, false) => format!("pronto: {} MB recebidos", r.body.len() as u64 / MB),
                        (false, _) => format!("falhou: {}", r.error),
                    };
                    ANOTADO.lock().unwrap().clear(); // o resultado final vale mais que um progresso atrasado
                    EffectOutcome::data(vec![("estado".into(), estado)])
                });
            }
            "cancelar" => {
                if let Some(c) = &self.ctl {
                    c.abort();
                }
            }
            _ => {}
        }
    }

    fn subscription(&self) -> iced::Subscription<EngineMessage> {
        iced::time::every(Duration::from_millis(100)).map(|_| {
            EngineMessage::ContextPatch(std::mem::take(&mut *ANOTADO.lock().unwrap()))
        })
    }
}

#[path = "../rede_completa/servidor_teste.rs"]
mod servidor_teste;

fn main() -> iced::Result {
    let _servidor = servidor_teste::ligar();
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Progresso::default())) {
                eprintln!("{e}");
            }
            motor.set_initial_screen("rede_progresso_rust");
        })
        .run()
}
