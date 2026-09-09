//! **Onda 11, habilitador C** do `PLANO_WIDGETS.md`: séries múltiplas nos
//! gráficos — o corte que a Onda 11 tinha deixado para depois.
//!
//! Rode com: `cargo run --example series_multiplas` (nesta máquina, com
//! `WGPU_BACKEND=gl`).
//!
//! ```text
//! O habilitador — uma convenção de dados, uma legenda, um ciclo de cores
//!
//!   series="chave"  → [ { name, points, color? }, … ]
//!     · a MESMA Moldura e a MESMA escala 1·2·5 da Onda 7
//!     · limites_multi() só concatena os pontos para a faixa do eixo
//!     · cor_ciclica() — o ciclo do tema que o <piechart> já usava
//!     · uma legenda no canto, translúcida
//!
//! Vale para <linechart> e portanto para area= / points=
//!   → o AreaChart/Scatter da §2.13 sai de 🟡 para ✅
//! ```
//!
//! # O que NÃO mudou
//!
//! `<barchart>` e `<piechart>` seguem série-única de propósito (barras
//! agrupadas e setores concêntricos são outra decisão de layout), e a leitura
//! continua sendo em tempo de render: o `update` abaixo troca a chave inteira
//! e o gráfico se redesenha sozinho, sem handler próprio.
use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct SeriesMultiplas;

const BASE: &str = r##"[
    { "name": "API",   "points": [{"label":"seg","value":120},{"label":"ter","value":132},{"label":"qua","value":101},{"label":"qui","value":134},{"label":"sex","value":90},{"label":"sáb","value":230},{"label":"dom","value":210}] },
    { "name": "Web",   "points": [{"label":"seg","value":220},{"label":"ter","value":182},{"label":"qua","value":191},{"label":"qui","value":234},{"label":"sex","value":290},{"label":"sáb","value":330},{"label":"dom","value":310}] },
    { "name": "Fila",  "points": [{"label":"seg","value":150},{"label":"ter","value":232},{"label":"qua","value":201},{"label":"qui","value":154},{"label":"sex","value":190},{"label":"sáb","value":330},{"label":"dom","value":410}], "color": "#f38ba8" }
]"##;

const ESPARSO: &str = r#"[
    { "name": "p50", "points": [12, 19, 7, 24, 15, 9, 22, 30, 18] },
    { "name": "p95", "points": [40, 55, 38, 62, 51, 44, 70, 88, 61] }
]"#;

impl Component for SeriesMultiplas {
    fn name(&self) -> &str {
        "series_multiplas"
    }

    fn template(&self) -> Template {
        Template::File("examples/series_multiplas/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("requisicoes", BASE.to_string());
        ctx.set("latencia", ESPARSO.to_string());
        ctx.set("status", "3 séries — arraste os dados com o botão".to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            // Rotaciona os pontos de cada série uma casa à esquerda — o
            // suficiente para ver que o desenho é lido do contexto a cada
            // quadro, sem reavaliar a árvore.
            "girar" => {
                let girado = rotaciona_series(ctx.get("requisicoes").map(String::as_str).unwrap_or("[]"));
                ctx.set("requisicoes", girado);
                ctx.set("status", "girado — o gráfico seguiu sem handler próprio".to_string());
            }
            "resetar" => {
                ctx.set("requisicoes", BASE.to_string());
                ctx.set("status", "de volta ao início".to_string());
            }
            _ => {}
        }
    }
}

/// Move o primeiro ponto de cada série para o fim. Puro `serde_json`, sem
/// depender do modelo de `charts.rs` — é dado do app.
fn rotaciona_series(bruto: &str) -> String {
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(bruto) else {
        return bruto.to_string();
    };
    if let Some(series) = v.as_array_mut() {
        for s in series.iter_mut() {
            if let Some(pts) = s.get_mut("points").and_then(|p| p.as_array_mut())
                && !pts.is_empty()
            {
                let primeiro = pts.remove(0);
                pts.push(primeiro);
            }
        }
    }
    serde_json::to_string(&v).unwrap_or_else(|_| bruto.to_string())
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — séries múltiplas")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(SeriesMultiplas)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("series_multiplas");
        })
        .run()
}
