/// `QFrame`: a moldura sozinha — borda, relevo e recheio configuráveis, sem
/// título e sem semântica de agrupamento.
///
/// É o [`super::group_box::GroupBox`] descascado, e a peça mais genérica da
/// família que o `<slot/>` destrancou: onde o `GroupBox` diz *"estes controles
/// são um grupo chamado Rede"*, o `Frame` só diz *"desenhe uma caixa em volta
/// disto"*.
///
/// ```xml
/// <Frame shape="filled" padding="16">
///     <Text content="dentro de uma caixa preenchida" />
/// </Frame>
/// ```
///
/// # As três formas
///
/// A prop `shape` cobre o que o `QFrame::Shape` do Qt separa em variantes:
///
/// - `box` (**default**) — borda de 1px em volta. O `QFrame::Box`.
/// - `filled` — sem borda, com fundo próprio. O `QFrame::Panel`: a caixa se
///   destaca por contraste, não por contorno. É a forma para painel/superfície.
/// - `none` — nem borda nem fundo. O `QFrame::NoFrame`: sobra um recipiente que
///   só aplica `padding`/`spacing`, útil para padronizar espaçamento sem
///   desenhar nada.
///
/// Sem sombra: o motor não tem campo de sombra em `UiNode`, então o `Raised`/
/// `Sunken` do Qt não tem como ser reproduzido — a distinção que sobra é
/// contorno vs. contraste, que é o que as três formas acima entregam.
///
/// # Props
///
/// - `shape`      — `box` (default), `filled` ou `none`.
/// - `padding`    — espaço interno. Default: `12`.
/// - `spacing`    — espaço entre os filhos. Default: `8`.
/// - `width`      — largura. Default: `fill`.
/// - `background` — cor de fundo do `filled`, por instância. Omitida, o fundo
///   vem da folha (`.frame-filled`) — que é o caminho para repintar todos de
///   uma vez numa `.gss` do app.
///
/// # Aparência
///
/// Mesma regra dos outros builtins: as cores vêm de um `<style>` global do
/// próprio template, instalado antes de qualquer `.gss` do app — redefinir
/// `.frame-box` / `.frame-filled` numa folha do app repinta os dois.
use crate::component::{Component, Context, Template};

pub struct Frame;

impl Component for Frame {
    fn name(&self) -> &str {
        "Frame"
    }

    fn template(&self) -> Template {
        // O braço `filled` é duplicado por causa do `background`, e a razão é
        // sutil: o eval resolve um campo com
        // `inline.map(process_tpl).or_else(classe)`, então um atributo escrito
        // no markup vence a classe **mesmo quando resolve para vazio**. Um
        // `background="{background|}"` único não cairia para `.frame-filled` —
        // gravaria `""` e o `filled` sairia sem fundo nenhum, idêntico ao
        // `none` (foi o que aconteceu na primeira versão). Só emitindo o
        // atributo quando a prop existe é que os dois caminhos convivem: prop
        // por instância, classe quando ela não vem.
        Template::Inline(
            r#"<Container width="{width|fill}">
                    <style>
                        .frame-box {
                            border-width: 1;
                            border-color: #80808059;
                            border-radius: 6;
                        }
                        .frame-filled {
                            background: #8080801f;
                            border-radius: 6;
                        }
                    </style>

                    <template if="{shape|box}" equals="filled">
                        <template if="{background}" notEquals="">
                            <Container
                                class="frame-filled"
                                background="{background}"
                                padding="{padding|12}"
                                width="{width|fill}"
                            >
                                <Column spacing="{spacing|8}" width="{width|fill}">
                                    <slot/>
                                </Column>
                            </Container>
                        </template>
                        <template else>
                            <Container
                                class="frame-filled"
                                padding="{padding|12}"
                                width="{width|fill}"
                            >
                                <Column spacing="{spacing|8}" width="{width|fill}">
                                    <slot/>
                                </Column>
                            </Container>
                        </template>
                    </template>

                    <template else-if="{shape|box}" equals="none">
                        <Column
                            spacing="{spacing|8}"
                            padding="{padding|12}"
                            width="{width|fill}"
                        >
                            <slot/>
                        </Column>
                    </template>

                    <template else>
                        <Container
                            class="frame-box"
                            padding="{padding|12}"
                            width="{width|fill}"
                        >
                            <Column spacing="{spacing|8}" width="{width|fill}">
                                <slot/>
                            </Column>
                        </Container>
                    </template>
                </Container>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Recipiente puro, como o `GroupBox`.
    }
}
