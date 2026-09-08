//! **Onda 9** do `PLANO_WIDGETS.md`: o ponteiro preso — e o teclado que ninguém
//! escutava.
//!
//! Rode com: `cargo run --example onda9`
//!
//! ```text
//! Habilitador A — o ARRASTO como capacidade        (motor, `src/grip.rs`)
//!
//! 1. Splitter     — painéis redimensionáveis        (primitiva)
//! 2. RangeSlider  — dois cursores numa faixa        (primitiva)
//! 3. Tumbler      — a roleta de valores             (primitiva)
//! 4. SwipeView    — páginas deslizáveis             (primitiva)
//! 5. DelayButton  — o botão que se segura           (primitiva)
//! 6. RubberBand   — retângulo de seleção            (primitiva)
//! 7. SizeGrip     — o canto que redimensiona        (builtin)
//!    PageIndicator — os pontinhos, de carona no 4   (a mesma tag do Pagination)
//!
//! Habilitador B — a SUBSCRIPTION de teclado        (motor, `src/keys.rs`)
//!
//! 8. Shortcut/Action — atalhos globais no markup    (motor + tag)
//! 9. ShortcutInput   — captura uma combinação       (primitiva)
//! ```
//!
//! # A descoberta da onda: o motor já arrastava três vezes
//!
//! O `PLANO_WIDGETS.md` listava `Splitter`, `MdiArea` e `SwipeView` como
//! "widgets isolados" — três assuntos. São **um**: o ponteiro apertado ao longo
//! do tempo, escrevendo num valor enquanto anda. O que os escondia era a
//! **tabela**, que agrupa por onde o widget aparece na tela (botões, entradas
//! numéricas, containers, navegação, janela, overlays) — o eixo errado para
//! enxergar mecanismo compartilhado.
//!
//! E o mecanismo já existia, escrito três vezes em lugares que não se
//! conheciam: o `__drag_key` da lista reordenável, o `__colgrip` da alça de
//! coluna do `<tableheader>` (Onda 6) e o `Program::State` do `<dial>` (Onda 7).
//! Nenhum dos três estava catalogado como *capacidade* no §3 do plano — a
//! terceira vez que o gargalo real não estava naquela lista, depois da medição
//! de colunas (Onda 6) e do corpo do diálogo (Onda 8).
//!
//! # A 14ª reclassificação de `●`, e a última em que a pergunta cabia
//!
//! Seis dos sete estavam marcados `Comp ●` — "exige estado por instância".
//! Nenhum exige, e desta vez o motivo é o **mesmo para os seis**, o que é a
//! própria evidência: o que o arrasto move é sempre um valor que o app nomeia
//! (`sizes="painel"`, `value="pagina"`, `start`/`end`), e o que sobra dele
//! (*estou arrastando? desde onde?*) é global por natureza, porque só se arrasta
//! uma coisa por vez numa tela.
//!
//! Repare no `update` abaixo: como na Onda 7, ele trata **uma** ação genérica, e
//! nenhum dos nove widgets tem handler próprio.
//!
//! # O item que encolheu no caminho
//!
//! O `SizeGrip` entrou na fila como o sétimo consumidor do arrasto — o único
//! cujo alvo não é uma chave, mas a janela. Ao escrever, não sobrou consumidor:
//! `window:resize:se` já era uma ação da titlebar custom e `cursor="se"` já era
//! um atributo universal. Ele virou um **builtin** de vinte linhas
//! (`src/builtins/size_grip.rs`), e a linha do catálogo que o chamava de "Motor"
//! estava errada — a 15ª correção de nível deste documento, e a primeira para o
//! lado da infraestrutura em vez do estado.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Onda9;

/// As caixas que o `<rubberband>` seleciona: `id` mais geometria.
///
/// A geometria mora no **dado**, e não no widget, e é o que torna o laço
/// genérico: ele não sabe o que são essas caixas, só quais delas o retângulo
/// tocou. Quem desenha as caixas na tela é o `.gv` ao lado, com um `for-each`
/// sobre esta mesma lista.
const CAIXAS: &[(&str, f32, f32)] = &[
    ("api", 20.0, 20.0),
    ("banco", 130.0, 20.0),
    ("fila", 240.0, 20.0),
    ("cdn", 20.0, 96.0),
    ("cache", 130.0, 96.0),
    ("proxy", 240.0, 96.0),
];

const LADO: f32 = 84.0;
const ALTURA: f32 = 60.0;

impl Component for Onda9 {
    fn name(&self) -> &str {
        "onda9"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda9/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set(
            "abas",
            r#"[{"id":"paineis","label":"Splitter + SizeGrip"},
                {"id":"valores","label":"RangeSlider + Tumbler + DelayButton"},
                {"id":"paginas","label":"SwipeView + PageIndicator"},
                {"id":"laco","label":"RubberBand"},
                {"id":"teclado","label":"Shortcut + ShortcutInput"}]"#
                .to_string(),
        );
        ctx.set("aba", "paineis".to_string());

        // ── 1. Splitter ─────────────────────────────────────────────────
        //
        // A chave guarda TRILHAS, no mesmo formato do `columns` do `<grid>` —
        // e é a mesma chave que a alça de coluna de um `<tableheader>` escreve,
        // porque agora é o mesmo código. `fill` é o painel que fica com o
        // resto; arrastar a alça converte o painel tocado em pixels fixos.
        ctx.set("painel_h", "260 fill".to_string());
        ctx.set("painel_v", "140 fill".to_string());

        // ── 2. RangeSlider ──────────────────────────────────────────────
        //
        // DUAS chaves, que é a mesma forma do `<daterangepicker range>` da Onda
        // 3 — e a razão de o `●` do catálogo nunca ter valido aqui.
        ctx.set("preco_min", "120".to_string());
        ctx.set("preco_max", "780".to_string());

        // ── 3. Tumbler ──────────────────────────────────────────────────
        //
        // A chave guarda o TEXTO escolhido, não o índice: reordenar a coleção
        // não move a escolha de lugar.
        ctx.set(
            "meses",
            r#"["janeiro","fevereiro","março","abril","maio","junho",
                "julho","agosto","setembro","outubro","novembro","dezembro"]"#
                .to_string(),
        );
        ctx.set("mes", "março".to_string());
        ctx.set("regiao", "sudeste".to_string());

        // ── 4. SwipeView + PageIndicator ────────────────────────────────
        //
        // O índice, base ZERO — é o `currentIndex` do QML. As duas tags leem a
        // MESMA chave, que é o que o catálogo previa ao chamar o
        // `PageIndicator` de "irmão visual do Pagination, mesma chave".
        ctx.set("pagina", "0".to_string());

        // ── 6. RubberBand ───────────────────────────────────────────────
        ctx.set("caixas", caixas_json());
        // O conjunto nomeado que o laço escreve — a mesma grafia separada por
        // vírgula que o `<listview mode="multi">` guarda desde a 0.85, e que o
        // `contains` do condicional lê.
        ctx.set("marcados", String::new());

        // ── 9. ShortcutInput ────────────────────────────────────────────
        ctx.set("atalho_salvar", "ctrl+s".to_string());
        ctx.set("atalho_buscar", "ctrl+shift+p".to_string());

        ctx.set("status", "Pronto — experimente Ctrl+S".to_string());
        ctx.set("salvamentos", "0".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            // Os dois `<shortcut>` da tela. São ações comuns: um atalho não é
            // um tipo novo de evento, é o teclado despachando o que um botão
            // despacharia.
            "salvar" => {
                let n = ctx
                    .get("salvamentos")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0)
                    + 1;
                ctx.set("salvamentos", n.to_string());
                ctx.set(
                    "status",
                    format!("Salvo ({n}x) — pelo atalho ou pelo botão"),
                );
            }
            "buscar" => ctx.set("status", "Busca aberta pelo atalho".to_string()),

            // O `<delaybutton>`: a ação só chega aqui quando o anel FECHA.
            // Soltar antes não despacha nada — não há o que tratar.
            "apagar_tudo" => {
                ctx.set("marcados", String::new());
                ctx.set("status", "Tudo apagado — o anel fechou".to_string());
            }

            // O `<rubberband>`: a geometria do laço ao soltar. O que ele
            // SELECIONOU já está em `marcados`, escrito pelo próprio widget; o
            // que chega aqui é o retângulo, para quem quiser fazer outra coisa
            // com ele.
            "lacou" => {
                let quantos = ctx
                    .get("marcados")
                    .map(|s| s.split(',').filter(|t| !t.trim().is_empty()).count())
                    .unwrap_or(0);
                ctx.set(
                    "status",
                    format!("Laço {} → {quantos} serviço(s)", value.unwrap_or_default()),
                );
            }

            // O braço genérico de sempre: a ação É o nome da chave. É por aqui
            // que passa o `onChange` do `<swipeview>` — e é o único dos nove
            // widgets desta onda que o markup mandou delegar.
            chave if value.is_some() => {
                let v = value.unwrap_or_default();
                ctx.set(chave, v.to_string());
                ctx.set("status", format!("{chave} = {v}"));
            }
            _ => {}
        }
    }
}

/// As caixas na forma que o `<rubberband>` lê (`id`/`x`/`y`/`w`/`h`) — e que o
/// `for-each` do `.gv` também usa para desenhá-las.
fn caixas_json() -> String {
    let arr: Vec<serde_json::Value> = CAIXAS
        .iter()
        .map(|(id, x, y)| {
            serde_json::json!({
                "id": id, "x": x, "y": y, "w": LADO, "h": ALTURA,
                // O `.gv` posiciona por `padding`, então ele quer os números
                // já prontos: um markup não faz aritmética, e essa é a lição
                // que a Onda 8 aprendeu com o `<wizard>`.
                "rotulo": id,
            })
        })
        .collect();
    serde_json::Value::Array(arr).to_string()
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 9")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda9)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda9");
        })
        .run()
}
