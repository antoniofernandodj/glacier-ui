/// Uma "pílula" de rótulo — texto curto sobre um fundo arredondado, o clássico
/// selo de status/contagem ("Novo", "3", "Beta"). Puramente apresentacional.
///
/// Props (todas opcionais; o default vem do próprio template via `{prop|def}`):
/// - `badge_text`  — o rótulo. Default: `Badge`.
/// - `badge_bg`    — cor de fundo. Default: `#89B4FA`.
/// - `badge_fg`    — cor do texto. Default: `#11111B`.
/// - `badge_size`  — tamanho do texto (numérico, templado). Default: `13`.
/// - `text_class`  — classe aplicada **ao texto de dentro**, não à pílula.
///
/// ```xml
/// <Badge badge_text="Novo" badge_bg="#A6E3A1" badge_size="15" />
/// ```
///
/// # Estilizar por classe
///
/// `class` no uso pinta a **pílula** (a raiz expandida); `text_class` alcança o
/// `<Text>` de dentro, que a classe do uso não atinge — a classe do uso aplica
/// só na raiz, por decisão da 0.69.
///
/// ```xml
/// <Badge class="selo" text_class="selo_txt" badge_text="Novo" />
/// ```
use crate::component::{Component, Context, Template};

pub struct Badge;

impl Component for Badge {
    fn name(&self) -> &str {
        "Badge"
    }

    fn template(&self) -> Template {
        // Defaults inline via `{prop|default}` — sem semear o contexto global.
        // `size` é numérico e ainda assim aceita `{prop}` (resolvido no eval).
        //
        // As duas CORES são a exceção: o default delas mora numa classe, não no
        // `{prop|def}`. É o que deixa `text_class` (e um `class` no uso) pintar
        // de fato — um default inline resolve sempre, e um valor inline vence
        // qualquer classe. Com a prop vazia caindo na classe (ver `resolve` em
        // `eval.rs`), a escada fica: prop > classe injetada > default da lib.
        Template::Inline(
            r#"<Container
                    class="badge-pill"
                    background="{badge_bg}"
                    padding="4 10"
                    border_radius="12"
                >
                    <style>
                        .badge-pill { background: #89B4FA; }
                        .badge-text { color: #11111B; }
                    </style>

                    <Text
                        class="badge-text {text_class}"
                        content="{badge_text|Badge}"
                        color="{badge_fg}"
                        size="{badge_size|13}"
                        bold="true"
                    />
                </Container>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: sem estado, sem comportamento, sem `init`.
    }
}
