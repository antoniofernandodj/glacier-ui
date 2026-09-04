//! **Onda 5** do `PLANO_WIDGETS.md`: o conteúdo que sai da tela e entra no
//! widget.
//!
//! Rode com: `cargo run --example onda5`
//!
//! A Onda 4 entregou sete widgets que **fazem conta**. Esta entrega seis que
//! respondem outra pergunta, e é a mesma para todos: *de quem é este conteúdo?*
//! A página de uma aba e o painel que flutua eram, até aqui, coisa da tela — um
//! `se`/`senão` para a página, um `stack` para o painel. Agora moram dentro do
//! widget.
//!
//! ```text
//! Habilitador A — nome dinâmico de slot     (motor, uma linha no `eval`)
//! Habilitador B — overlay ancorado genérico (motor, `src/anchored.rs`)
//!
//! 1. Tabs           — a barra MAIS a página          (builtin, hab. A)
//! 2. calendarPopup  — a grade ancorada ao campo      (primitiva, hab. B)
//! 3. Popover        — o painel ancorado ao gatilho   (primitiva, hab. B)
//! 4. Popup          — a mesma primitiva sem âncora   (primitiva, hab. B)
//! 5. Autocomplete   — filtra enquanto se digita      (primitiva, hab. B)
//! 6. Drawer         — a gaveta lateral               (builtin, sem habilitador)
//! ```
//!
//! # O habilitador B não é mais um `stack![]`
//!
//! Vale registrar, porque é a decisão da onda. `src/menu.rs` já sobrepunha
//! painéis, pela técnica pragmática dos diálogos: uma camada do tamanho da
//! janela, com o painel posicionado por um `padding` calculado a partir da
//! posição do **cursor**. O próprio `menu.rs` documenta o limite disso — não há
//! medição antes de posicionar, e a âncora é o cursor, não o widget.
//!
//! `src/anchored.rs` é o caminho que aquele arquivo descreve como o certo e
//! adia por falta de precedente: um `iced::advanced::{Widget, Overlay}`. Com
//! ele, a âncora é o **layout do gatilho** (então o painel acompanha a rolagem)
//! e o painel é **medido antes de posicionado** (então virar para cima quando o
//! rodapé corta é uma conta, não um chute).
//!
//! # O que o app escreve, e o que ele não escreve
//!
//! Repare no `update` abaixo. Ele trata **três** ações, e nenhuma delas abre ou
//! fecha um painel: pressionar o gatilho abre, clicar fora fecha, Esc fecha —
//! tudo dentro do widget, como o `<spinbox>` faz a conta dele. O que o app faz
//! é o que nenhum widget podia saber: o que significa clicar em "Sair", e qual
//! é a data de hoje.
//!
//! A data de hoje é a mesma lição da Onda 3, e ela continua valendo: `today` é
//! **prop, não relógio**. O motor não lê o relógio do sistema em lugar nenhum;
//! quem sabe que dia é hoje é o app.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Onda5;

/// As cidades que o `<autocomplete>` filtra. Uma lista literal, para o exemplo
/// não depender de rede — e propositalmente com acento, que é o que o
/// recorte sem acento do widget existe para resolver: "sao paulo" acha
/// "São Paulo".
const CIDADES: &[&str] = &[
    "São Paulo",
    "Rio de Janeiro",
    "Belo Horizonte",
    "Brasília",
    "Salvador",
    "Fortaleza",
    "Curitiba",
    "Recife",
    "Porto Alegre",
    "Manaus",
    "Belém",
    "Goiânia",
    "São Luís",
    "Maceió",
    "Campo Grande",
    "Florianópolis",
    "Vitória",
    "Natal",
    "João Pessoa",
    "Teresina",
];

/// Os serviços, com `id` separado do rótulo — é o par que o `onSelect` do
/// `<autocomplete>` entrega: o widget grava o RÓTULO no campo (é o que a pessoa
/// vê) e manda o `id` para quem pediu (é o que liga a escolha a um registro).
const SERVICOS: &[(&str, &str)] = &[
    ("api", "API pública"),
    ("db", "Banco primário"),
    ("cache", "Redis"),
    ("fila", "Fila de eventos"),
    ("cdn", "CDN"),
    ("auth", "Autenticação"),
    ("mail", "Disparo de e-mail"),
    ("logs", "Coletor de logs"),
];

impl Component for Onda5 {
    fn name(&self) -> &str {
        "onda5"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda5/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // As abas do `<tabs>`: a MESMA chave que antes alimentava a `<tabbar>`.
        // O que mudou é que a lista não aparece mais uma segunda vez numa
        // escada de `se`/`senão` — as páginas são `slot="…"` no markup.
        ctx.set(
            "abas",
            r#"[{"id":"paineis","label":"Popover + Popup"},
                {"id":"busca","label":"Autocomplete + calendarPopup"},
                {"id":"gaveta","label":"Drawer"}]"#
                .to_string(),
        );
        ctx.set("aba", "paineis".to_string());

        // Uma chave por painel. Nenhuma delas é escrita por este arquivo depois
        // daqui: quem as abre e fecha é o widget.
        ctx.set("menu_usuario", String::new());
        ctx.set("filtros", String::new());
        ctx.set("lateral", String::new());
        ctx.set("atalhos", String::new());
        ctx.set("gaveta", String::new());

        // O conteúdo do painel de filtros. Semeá-las é o que faz a tela abrir
        // coerente com o que o app considera o estado inicial — o `<checkbox>`
        // e o `<slider>` só LEEM estas chaves; quem as escreve é o braço
        // genérico do `update` abaixo, porque os dois são primitivas da Onda 1
        // (disparam a ação, não gravam).
        ctx.set("so_no_ar", "true".to_string());
        ctx.set("arquivados", "false".to_string());
        ctx.set("uso_max", "80".to_string());

        // O `<autocomplete>`: a lista inteira numa chave, o texto noutra. Quem
        // recorta é o widget — é a razão de ser dele.
        let cidades: Vec<serde_json::Value> =
            CIDADES.iter().map(|c| serde_json::json!(c)).collect();
        ctx.set("cidades", serde_json::Value::Array(cidades).to_string());
        ctx.set("cidade", String::new());

        let servicos: Vec<serde_json::Value> = SERVICOS
            .iter()
            .map(|(id, label)| serde_json::json!({ "id": id, "label": label }))
            .collect();
        ctx.set("servicos", serde_json::Value::Array(servicos).to_string());
        ctx.set("servico_busca", String::new());
        ctx.set("servico_id", "—".to_string());

        // As duas datas do `calendarPopup`. `hoje` é PROP, não relógio: o motor
        // não lê a hora do sistema em lugar nenhum, e sem ela nenhum dia fica
        // destacado na grade.
        ctx.set("hoje", hoje_iso());
        ctx.set("entrada", hoje_iso());
        ctx.set("saida", String::new());

        ctx.set("escolha", "—".to_string());
        ctx.set("status", "Pronto".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            // Os botões da gaveta trocam de aba. Repare que a gaveta continua
            // aberta: fechá-la ao navegar seria uma decisão de app, e este
            // exemplo prefere deixar visível que ela empurra em vez de cobrir.
            a if a.starts_with("ir:") => {
                let aba = a.trim_start_matches("ir:");
                ctx.set("aba", aba.to_string());
                ctx.set("status", format!("Aba: {aba}"));
            }
            // Os itens do popover de usuário. O painel já se fechou sozinho
            // quando este handler roda — o clique dentro dele é do botão, e o
            // botão de dentro não fecha nada; quem fecha é o clique SEGUINTE,
            // fora. Para fechar ao escolher, zere a chave aqui: é uma linha, e
            // é o app decidindo.
            a if a.starts_with("escolher:") => {
                let item = a.trim_start_matches("escolher:");
                ctx.set("escolha", item.to_string());
                ctx.set("menu_usuario", String::new());
                ctx.set("status", format!("Menu: {item}"));
            }
            // O `<autocomplete>` com `onSelect` DELEGA: ele entrega o `id`, e
            // quem decide o que fazer com ele é o app. Sem `onSelect`, o widget
            // grava o rótulo na chave e ninguém passa por aqui — que é o caso
            // do campo "Cidade" logo acima na tela.
            "escolher_servico" => {
                let id = value.unwrap_or_default();
                ctx.set("servico_id", id.to_string());
                ctx.set("status", format!("Serviço: {id}"));
            }
            // O caminho comum de todo widget que grava sozinho: a ação É o nome
            // da chave.
            chave if value.is_some() => {
                ctx.set(chave, value.unwrap_or_default().to_string());
            }
            _ => {}
        }
    }
}

/// A data de hoje em ISO, calculada do relógio do sistema.
///
/// Mora no app, não no motor, e é a lição da Onda 3 pela negativa: `today` é
/// prop. Do lado Luau isto é `date.today()`, uma linha — ver
/// `examples/onda5_luau`.
fn hoje_iso() -> String {
    let segundos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let dias = segundos.div_euclid(86_400);

    // O algoritmo `civil_from_days` de Howard Hinnant — o inverso exato do
    // `days_from_civil` que o motor usa para saber em que dia da semana um mês
    // começa (`src/widget.rs`).
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 5")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda5)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda5");
        })
        .run()
}
