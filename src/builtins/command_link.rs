/// `QCommandLinkButton`: um botão de duas linhas — título forte, descrição
/// apagada — com uma seta indicando que ele **leva a algum lugar**. O widget
/// de "próximo passo" de um assistente ou de uma tela de escolha (o clássico
/// instalador do Windows: "Instalação típica → " / "recomendado para a
/// maioria dos usuários").
///
/// ```xml
/// <CommandLink
///     title="Instalação típica"
///     description="Recomendado para a maioria dos usuários"
///     on_click="instalar:tipica"
/// />
/// ```
///
/// # Por que ele esperou a Onda 11
///
/// Sem estado desde sempre (a §6.3 do `PLANO_WIDGETS.md` já dizia isso) — o
/// que faltava era assunto de verdade por perto para justificar a rodada, não
/// nenhum habilitador. A Onda 11 é esse assunto.
///
/// # Por que `<Button>` com filhos, não `<Row on_press>`
///
/// O `<Button>` do motor aceita filhos desde a convergência de templates: sem
/// eles, `text=` é o atalho de sempre; com mais de um, viram um `<Row>`
/// implícito. Isso dá ao `CommandLink` o `on_click` de **soltar** (não de
/// pressionar, como um `on_press` genérico daria) e o estado `disabled`/hover
/// do botão de verdade, de graça.
///
/// # Props
///
/// - `title`       — a linha forte. Obrigatória (default vazio).
/// - `description` — a linha apagada. Ausente = só o título, sem a segunda
///   linha (não uma linha vazia ocupando espaço).
/// - `on_click`    — ação ao soltar.
/// - `width`       — largura. Default `fill`.
///
/// # Classes
///
/// `class` no uso pinta o botão inteiro (a raiz); `title_class`/`desc_class`
/// alcançam as duas linhas de texto.
use crate::component::{Component, Context, Template};

pub struct CommandLink;

impl Component for CommandLink {
    fn name(&self) -> &str {
        "CommandLink"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Button on_click="{on_click}" width="{width|fill}">
                    <style>
                        .commandlink-title { color: #cdd6f4; }
                        .commandlink-desc { color: #7f849c; }
                        .commandlink-arrow { color: #7f849c; }
                    </style>

                    <Row spacing="12" align_y="center" width="fill">
                        <Column spacing="2" width="fill">
                            <Text
                                class="commandlink-title {title_class}"
                                content="{title}"
                                bold="true"
                            />
                            <template if="{description}" notEquals="">
                                <Text
                                    class="commandlink-desc {desc_class}"
                                    content="{description}"
                                    size="12"
                                />
                            </template>
                        </Column>
                        <Text class="commandlink-arrow" content="→" size="18" />
                    </Row>
                </Button>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Sem estado, como todo o resto da família de botões desta lib.
    }
}
