//! **Onda 7** do `PLANO_WIDGETS.md`: o `canvas` — e os sete widgets que saem
//! dele.
//!
//! Rode com: `cargo run --example onda7`
//!
//! ```text
//! Habilitador — o canvas como CAPACIDADE, não como tag  (motor, `src/canvas.rs`)
//!
//! 1. Dial       — o knob rotativo                       (primitiva)
//! 2. Gauge      — o medidor de arco, com faixas          (primitiva)
//! 3. LcdNumber  — sete segmentos                         (primitiva)
//! 4. Sparkline  — a linha sem moldura                    (primitiva)
//! 5. LineChart  — a linha com eixos                      (primitiva)
//! 6. BarChart   — barras                                 (primitiva)
//! 7. PieChart   — setores, e a rosquinha                 (primitiva)
//! ```
//!
//! # A decisão da onda: o `canvas` não vira tag
//!
//! O `iced` tem `canvas`, e a tradução literal do `QGraphicsView` seria expor um
//! `<canvas>` ao markup com um callback de desenho em Rust. Este projeto não
//! faz isso, e a razão é a promessa da primeira linha do README: estrutura,
//! estilo e comportamento **declarados**. Uma superfície de desenho livre
//! devolveria ao app um bloco imperativo que o `.gv` não lê, o `.gss` não
//! estiliza e o lado Luau não alcança — três regressões para ganhar uma.
//!
//! O `canvas` ficou como capacidade (`src/canvas.rs`: arcos, séries, escalas,
//! sete segmentos) e o que o app vê são sete tags que se comportam como todas
//! as outras — valor numa chave que ele nomeia, cor da paleta do tema,
//! `on_change` para delegar. Repare no `update` abaixo: ele trata **uma** ação
//! genérica, e nenhum dos sete widgets precisa de handler próprio.
//!
//! # A segunda decisão: `canvas` na mão, e não `plotters`
//!
//! A §4 do plano deixava isso em aberto desde a primeira revisão. Fechou na
//! mão, por três motivos em ordem de peso: a **cor** (o `plotters` traz o
//! sistema de estilo dele, e um gráfico que ignora o tema é um retângulo
//! estrangeiro no meio do app), a **manutenção** (o `plotters-iced` oficial
//! parou no `iced 0.13`; para o 0.14 só existe um fork de comunidade) e o
//! **tamanho** (este crate já compila `wgpu`, `naga`, Luau e os codecs
//! estaticamente).
//!
//! # A terceira reclassificação de `●` deste catálogo
//!
//! `Dial`, `Gauge` e `LcdNumber` estavam catalogados como **componente**, e o
//! `Dial` marcado com `●` — "exige estado por instância". Não exige. O que
//! parecia estado por instância era o **valor**, e o valor sempre coube numa
//! chave que o app nomeia (`value="volume"`, como `value="nota"`). O único
//! estado interno de verdade — se a alça está presa agora — vive no
//! `canvas::Program::State`, que o `iced` dá por widget.
//!
//! É a terceira vez: o `Spinner` desceu de `●` na 0.66 e o `Rating` na 0.85. A
//! lição já é regra, e vale para a próxima onda.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Onda7;

/// A latência de cada serviço nas últimas 24 horas — a série que alimenta as
/// três `<sparkline>`.
///
/// Números crus num array: é a forma curta que a convenção `items` aceita, e a
/// que um `<sparkline>` quer (ele não desenha rótulo nenhum). A forma longa —
/// `[{"label": "Jan", "value": 12}]` — aparece logo abaixo, nas vendas.
const LAT_API: &[f64] = &[
    38.0, 41.0, 39.0, 44.0, 52.0, 47.0, 43.0, 40.0, 38.0, 37.0, 39.0, 45.0, 61.0, 58.0, 49.0, 44.0,
    41.0, 40.0, 42.0, 46.0, 51.0, 48.0, 43.0, 39.0,
];
const LAT_DB: &[f64] = &[
    12.0, 11.0, 13.0, 12.0, 14.0, 19.0, 24.0, 21.0, 17.0, 15.0, 14.0, 13.0, 12.0, 12.0, 13.0, 16.0,
    22.0, 28.0, 31.0, 26.0, 19.0, 15.0, 13.0, 12.0,
];
const LAT_FILA: &[f64] = &[
    88.0, 92.0, 85.0, 79.0, 81.0, 96.0, 140.0, 165.0, 132.0, 104.0, 91.0, 86.0, 83.0, 80.0, 84.0,
    93.0, 118.0, 151.0, 129.0, 102.0, 90.0, 86.0, 82.0, 79.0,
];

/// As vendas do semestre, na forma longa: rótulo e valor.
///
/// É o que o eixo X de um `<linechart>` e de um `<barchart>` desenha — e a
/// razão de a convenção aceitar as duas formas em vez de exigir a longa sempre.
const VENDAS: &[(&str, f64)] = &[
    ("jan", 128.0),
    ("fev", 141.0),
    ("mar", 119.0),
    ("abr", 173.0),
    ("mai", 186.0),
    ("jun", 164.0),
];

/// As fatias do `<piechart>`. Mesma forma longa.
const FATIAS: &[(&str, f64)] = &[
    ("API", 42.0),
    ("Banco", 27.0),
    ("Fila", 18.0),
    ("CDN", 13.0),
];

impl Component for Onda7 {
    fn name(&self) -> &str {
        "onda7"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda7/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set(
            "abas",
            r#"[{"id":"medidores","label":"Dial + Gauge + LCD"},
                {"id":"graficos","label":"Sparkline + Line + Bar + Pie"},
                {"id":"fecho","label":"O que fechou"}]"#
                .to_string(),
        );
        ctx.set("aba", "medidores".to_string());

        // ── Os medidores ────────────────────────────────────────────────
        //
        // Uma chave por widget, e nada mais. Nenhuma delas é escrita por este
        // arquivo depois daqui: quem escreve `volume` e `brilho` é o próprio
        // `<dial>`, porque o markup não lhe deu `onChange`.
        ctx.set("volume", "7".to_string());
        ctx.set("brilho", "60".to_string());
        ctx.set("cpu", "72".to_string());
        ctx.set("memoria", "38.4".to_string());
        ctx.set("disco", "91".to_string());
        ctx.set("rede", "420".to_string());

        // As faixas de um medidor podem vir de uma CHAVE, e não só do atributo:
        // é o caso em que os limites são do backend, não do desenho. O
        // `<gauge>` do disco usa esta; os outros escrevem o JSON inline.
        ctx.set(
            "limites_disco",
            r##"[{"to":70,"color":"#A6E3A1"},{"to":90,"color":"#F9E2AF"},{"to":100,"color":"#F38BA8"}]"##
                .to_string(),
        );

        // O `<lcdnumber>` lê o valor como TEXTO — é o que deixa um relógio
        // passar inteiro pelos dois-pontos.
        ctx.set("relogio", hora_local());

        // ── As séries ───────────────────────────────────────────────────
        ctx.set("lat_api", serie_curta(LAT_API));
        ctx.set("lat_db", serie_curta(LAT_DB));
        ctx.set("lat_fila", serie_curta(LAT_FILA));
        ctx.set("p95_api", format!("{:.0}", p95(LAT_API)));
        ctx.set("p95_db", format!("{:.0}", p95(LAT_DB)));
        ctx.set("p95_fila", format!("{:.0}", p95(LAT_FILA)));

        ctx.set("vendas", serie_longa(VENDAS));
        ctx.set("fatias", serie_longa(FATIAS));

        // A série vazia é de propósito: os quatro gráficos escrevem "sem dados"
        // em vez de desenhar nada. Um gráfico que desenha nada é
        // indistinguível de um gráfico quebrado.
        ctx.set("serie_vazia", "[]".to_string());

        ctx.set("status", "Pronto".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            // O braço genérico de sempre: a ação É o nome da chave. É por aqui
            // que o `<slider>` da CPU passa — e repare que nenhum dos SETE
            // widgets desta onda precisa de um braço próprio.
            chave if value.is_some() => {
                ctx.set(chave, value.unwrap_or_default().to_string());
                ctx.set("status", format!("{chave} = {}", value.unwrap_or_default()));
            }
            _ => {}
        }
    }
}

/// `[38, 41, 39]` — a forma curta da convenção `items`.
fn serie_curta(vs: &[f64]) -> String {
    let arr: Vec<serde_json::Value> = vs.iter().map(|v| serde_json::json!(v)).collect();
    serde_json::Value::Array(arr).to_string()
}

/// `[{"label": "jan", "value": 128}]` — a forma longa, com rótulo de eixo.
fn serie_longa(vs: &[(&str, f64)]) -> String {
    let arr: Vec<serde_json::Value> = vs
        .iter()
        .map(|(l, v)| serde_json::json!({ "label": l, "value": v }))
        .collect();
    serde_json::Value::Array(arr).to_string()
}

/// O p95 da série, pelo método do índice — o número que a coluna da direita
/// mostra ao lado de cada `<sparkline>`.
///
/// Mora no app, e é o ponto: o widget desenha a série, o app decide o que ela
/// significa. Um gráfico que calculasse percentil sozinho estaria adivinhando.
fn p95(vs: &[f64]) -> f64 {
    if vs.is_empty() {
        return 0.0;
    }
    let mut ord = vs.to_vec();
    ord.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let i = ((ord.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
    ord[i.min(ord.len() - 1)]
}

/// A hora do sistema como `HH:MM`, para o `<lcdnumber>`.
///
/// A mesma lição da Onda 3 e da Onda 5, agora num display: **o relógio é do
/// app**, não do motor. O widget desenha o texto que a chave tem; quem sabe que
/// horas são é quem escreve a chave. Do lado Luau isto é `date.now()`.
fn hora_local() -> String {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let do_dia = s.rem_euclid(86_400);
    format!("{:02}:{:02}", do_dia / 3600, (do_dia % 3600) / 60)
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 7")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda7)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda7");
        })
        .run()
}
