/// `QStatusBar`: o rodapé de status — uma mensagem à esquerda e os indicadores
/// permanentes à direita.
///
/// ```xml
/// <StatusBar message="Pronto">
///     <Badge badge_text="3 erros" badge_bg="#F38BA8" />
///     <Text content="UTF-8" />
/// </StatusBar>
/// ```
///
/// A divisão espelha o `QStatusBar` de verdade, que tem duas zonas com regras
/// diferentes: o `showMessage()` (texto transitório, à esquerda) e os
/// *permanent widgets* (`addPermanentWidget`, à direita, que a mensagem nunca
/// cobre). Aqui a mensagem é a prop `message` e os permanentes são o `<slot/>`
/// — o empurrão para a direita sai do `width="fill"` na mensagem, que come todo
/// o espaço sobrando antes do conteúdo.
///
/// # Props
///
/// - `message`  — o texto da esquerda. Vazio = zona vazia (o conteúdo continua
///   à direita, porque quem empurra é a largura, não o texto).
/// - `padding`  — espaço interno. Default: `4 10`.
/// - `spacing`  — espaço entre os itens da direita. Default: `10`.
/// - `size`     — corpo do texto da mensagem. Default: `12`.
/// - `divider`  — linha de separação sobre a barra. Default: `true`.
/// - `width`    — largura. Default: `fill`.
///
/// # Aparência
///
/// `.statusbar` (a faixa) e `.statusbar-message` (o texto apagado da esquerda),
/// na folha global do template — redefiníveis por uma `.gss` do app.
use crate::component::{Component, Context, Template};

pub struct StatusBar;

impl Component for StatusBar {
    fn name(&self) -> &str {
        "StatusBar"
    }

    fn template(&self) -> Template {
        // A `<Row>` de fora tem `width="fill"` e a mensagem TAMBÉM: é a
        // mensagem que ocupa a sobra e joga o `<slot/>` para a borda direita.
        // Sem o `fill` no texto, os dois ficariam colados à esquerda.
        Template::Inline(
            r#"<Column spacing="0" width="{width|fill}">
                    <style>
                        .statusbar { background: #8080801f; }
                        .statusbar-message { color: #80868d; }
                    </style>

                    <template if="{divider|true}" equals="true">
                        <Rule />
                    </template>

                    <Container class="statusbar" padding="{padding|4 10}" width="{width|fill}">
                        <Row spacing="{spacing|10}" align_y="center" width="{width|fill}">
                            <Text
                                class="statusbar-message"
                                content="{message}"
                                size="{size|12}"
                                width="fill"
                            />
                            <slot/>
                        </Row>
                    </Container>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Recipiente puro.
    }
}
