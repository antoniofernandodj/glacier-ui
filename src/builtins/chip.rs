/// O `Chip`: um [`super::badge::Badge`] com um "×" — a etiqueta removível de um
/// campo de tags/filtros.
///
/// ```xml
/// <Chip label="produção" on_remove="remover_tag:producao" />
/// ```
///
/// # Por que ele esperou a Onda 11
///
/// A §6.3 do `PLANO_WIDGETS.md` sempre disse que ele "sai junto de um campo de
/// tags — que por sua vez quer o `contains` da Onda 4" (feito na 0.85). O que
/// faltava não era o `contains`: era assunto de verdade por perto para não
/// abrir uma rodada sozinho — e a Onda 11 é esse assunto.
///
/// # Sem estado
///
/// Como todo builtin desta lib, o `Chip` não guarda nada: `on_remove` é a ação
/// que a TELA trata (tipicamente removendo o item de uma coleção e
/// reescrevendo a chave), não algo que o widget faz sozinho. Sem `on_remove`,
/// o "×" nem aparece — um chip sem como remover é só um `Badge` com outro
/// nome, e não vale desenhar um "×" morto.
///
/// # Props
///
/// - `label`     — o texto. Obrigatório (default vazio).
/// - `on_remove` — ação disparada ao clicar no "×". Ausente = "×" some.
/// - `bg`/`fg`   — cores, como no `Badge`. Defaults: `.chip-pill`/`.chip-label`.
/// - `size`      — tamanho do texto. Default `13`.
///
/// # Classes
///
/// `class` no uso pinta a pílula (a raiz); `label_class` e `close_class`
/// alcançam o texto e o "×", como `text_class` no `Badge`.
use crate::component::{Component, Context, Template};

pub struct Chip;

impl Component for Chip {
    fn name(&self) -> &str {
        "Chip"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Row
                    class="chip-pill"
                    background="{bg}"
                    padding="4 6 4 10"
                    border_radius="12"
                    spacing="6"
                    align_y="center"
                >
                    <style>
                        .chip-pill { background: #89B4FA33; }
                        .chip-label { color: #cdd6f4; }
                        .chip-close { color: #7f849c; }
                    </style>

                    <Text
                        class="chip-label {label_class}"
                        content="{label}"
                        color="{fg}"
                        size="{size|13}"
                    />

                    <template if="{on_remove}" notEquals="">
                        <Button
                            class="chip-close {close_class}"
                            text="×"
                            on_click="{on_remove}"
                        />
                    </template>
                </Row>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: sem estado, sem comportamento.
    }
}
