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
///
/// # Classes nos nós de dentro
///
/// `class` no uso aplica na raiz — que aqui é um `<Container>` sem desenho
/// nenhum, só largura. Quem tem borda e fundo é a caixa de dentro, e é por
/// isso que este widget precisa das props mais do que os outros:
///
/// - `box_class`     — a caixa que desenha (`filled` e `box`; no `none` não há
///   caixa a estilizar).
/// - `content_class` — a coluna dos filhos, nas três formas.
///
/// ```xml
/// <frame shape="filled" box_class="painel">
///     <text content="dentro" />
/// </frame>
/// ```
use crate::component::{Component, Context, Template};

pub struct Frame;

impl Component for Frame {
    fn name(&self) -> &str {
        "Frame"
    }

    fn template(&self) -> Template {
        // O braço `filled` já foi DOIS, e a causa era do motor: um campo
        // resolvia por `inline.or_else(classe)`, então `background="{background}"`
        // vencia `.frame-filled` **mesmo resolvendo para vazio** — o `filled`
        // sem prop saía sem fundo nenhum, idêntico ao `none`. A saída da época
        // foi emitir o atributo só quando a prop existisse, o que custou um
        // `if`/`else` e uma cópia do braço inteiro.
        //
        // Desde a 0.89 o `resolve` do eval descarta o vazio antes de consultar
        // a classe, e o mesmo atributo cobre os dois casos: prop por instância,
        // classe quando ela não vem. Os dois braços viraram um.
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
                        <Container
                            class="frame-filled {box_class}"
                            background="{background}"
                            padding="{padding|12}"
                            width="{width|fill}"
                        >
                            <Column class="{content_class}" spacing="{spacing|8}" width="{width|fill}">
                                <slot/>
                            </Column>
                        </Container>
                    </template>

                    <template else-if="{shape|box}" equals="none">
                        <Column
                            class="{content_class}"
                            spacing="{spacing|8}"
                            padding="{padding|12}"
                            width="{width|fill}"
                        >
                            <slot/>
                        </Column>
                    </template>

                    <template else>
                        <Container
                            class="frame-box {box_class}"
                            padding="{padding|12}"
                            width="{width|fill}"
                        >
                            <Column class="{content_class}" spacing="{spacing|8}" width="{width|fill}">
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
