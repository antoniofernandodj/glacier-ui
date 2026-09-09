/// `QFontDialog`: o **corpo** do diálogo de escolher fonte.
///
/// ```lua
/// local fonte = pick_font({ title = "Fonte do editor", value = ctx.fonte, size = 15 })
/// if fonte then ctx.fonte = fonte end
/// ```
///
/// # O que ele deve à Onda 10
///
/// O catálogo (§2.10) marcava o `FontDialog` como `Diál ●` e o atribuía ao
/// widget errado: o que faltava nunca foi um diálogo, era o **registro de
/// famílias** (`src/fonts.rs`). Com ele, este corpo é o `<FontSelect>` da §2.4
/// mais um campo de tamanho e uma amostra — peças que já existiam. **Zero
/// linhas de diálogo novas:** ele entra pela mesma porta do `pick_color{}` da
/// Onda 8 (`prompt{ kind = "font" }` em `build_dialog`), e o `●` não podia
/// estar aí desde a 0.94 — o diálogo é singleton no motor.
///
/// # O que ele devolve, e o que fica para depois
///
/// `pick_font` devolve **a família** (uma string), como `pick_color` devolve o
/// hex — é o valor que não tem outra fonte fácil. O campo de tamanho e a
/// amostra são **pré-visualização**: `size=` semeia o valor inicial e a amostra
/// o mostra, mas o retorno não carrega tamanho/estilo. Um `DialogOutcome`
/// tipado (`{ family, size, bold }`) é o próximo passo natural — precisa de uma
/// chave de retorno composta, que o mecanismo de resume (uma chave, uma string)
/// ainda não tem.
///
/// # Por que lê chaves globais em vez de props
///
/// A mesma razão do `__InputDialog`/`__ColorDialog`: este corpo é montado por
/// `render(nome)` como template de topo, não inlinado numa tela, então não há
/// uso de tag onde pendurar props. A configuração chega por chaves que o motor
/// semeia (`__dialog.value` = família, `__dialog.size` = tamanho).
use crate::component::{Component, Context, Template};

/// O nome sob o qual o motor monta este corpo. `__` porque é peça interna, não
/// tag de tela.
pub const FONT_DIALOG_BODY: &str = "__FontDialog";

pub struct FontDialog;

impl Component for FontDialog {
    fn name(&self) -> &str {
        FONT_DIALOG_BODY
    }

    fn template(&self) -> Template {
        // `<FontSelect>` escreve `__dialog.value` sozinho (o payload `pick:` já
        // carrega a chave), e o `<SpinBox>` escreve `__dialog.size` sozinho —
        // por isso este corpo não tem `update`: não há campo decorativo a
        // socorrer, ao contrário do `<TextInput>` do `__InputDialog`.
        Template::Inline(
            r#"<Column spacing="12" width="fill">
                    <FontSelect
                        value="__dialog.value"
                        selected="{__dialog.value}"
                        height="200"
                    />

                    <Row spacing="10" width="fill">
                        <Text content="Tamanho" size="13" />
                        <SpinBox
                            value="__dialog.size"
                            min="6"
                            max="96"
                            step="1"
                            decimals="0"
                            width="110"
                        />
                    </Row>

                    <Container
                        width="fill"
                        padding="12"
                        border_radius="6"
                        border_width="1"
                    >
                        <Text
                            content="Sphinx of black quartz, judge my vow — 0123"
                            font="{__dialog.value}"
                            size="{__dialog.size|16}"
                        />
                    </Container>
                </Column>"#
                .to_string(),
        )
    }

    /// Sem estado próprio: o `<FontSelect>` e o `<SpinBox>` de dentro escrevem
    /// as próprias chaves (`__dialog.value` / `__dialog.size`), e o aceite lê
    /// `__dialog.value` direto. Não há campo decorativo a socorrer como o
    /// `<TextInput>` do `__InputDialog`.
    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {}
}
