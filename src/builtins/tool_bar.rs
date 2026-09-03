/// `QToolBar`: a faixa de ações no topo da tela — normalmente uma fileira de
/// [`super::tool_button::ToolButton`], com separadores entre os grupos.
///
/// ```xml
/// <ToolBar>
///     <ToolButton icon="📄" tooltip="Novo"  on_click="novo" />
///     <ToolButton icon="💾" tooltip="Salvar" on_click="salvar" />
///     <Rule direction="vertical" />
///     <ToolButton icon="🗑" tooltip="Excluir" on_click="excluir" />
/// </ToolBar>
/// ```
///
/// A barra não sabe o que carrega: o conteúdo é `<slot/>`, então o que entra
/// nela é decisão de quem a usa — botões, um `<Select>`, um `<TextInput>` de
/// busca. É o mesmo contrato do `QToolBar`, que aceita qualquer `QWidget` via
/// `addWidget`, não só `QAction`.
///
/// Com o `<MenuBar>` (já nativo) e o [`super::status_bar::StatusBar`], fecha o
/// esqueleto de uma `QMainWindow`: menu, barra de ferramentas, conteúdo, rodapé.
///
/// # Props
///
/// - `padding`  — espaço interno da faixa. Default: `6 8`.
/// - `spacing`  — espaço entre os itens. Default: `4`.
/// - `divider`  — linha de separação sob a barra. Default: `true`; `false` para
///   encostar a barra direto no conteúdo (uma tela que já separa por cor).
/// - `width`    — largura. Default: `fill` — uma barra de ferramentas que não
///   atravessa a janela não lê como barra.
/// - `align_y`  — alinhamento vertical dos itens. Default: `center`.
/// - `bar_class`     — classe da faixa pintada (o `<Container>`).
/// - `content_class` — classe da `<Row>` que segura os itens.
///
/// Como na `<statusbar>`, `class` no uso aplica na coluna externa (faixa +
/// divisória); quem tem fundo é a faixa, e é `bar_class` que a alcança.
///
/// # Aparência
///
/// `.toolbar` é a faixa e `.toolbar-divider` a linha; ambas na folha global do
/// template, redefiníveis por uma `.gss` do app. O fundo default é o mesmo
/// cinza translúcido dos outros builtins — de propósito quase invisível: a
/// barra deve se distinguir do conteúdo sem virar o elemento mais forte da tela.
use crate::component::{Component, Context, Template};

pub struct ToolBar;

impl Component for ToolBar {
    fn name(&self) -> &str {
        "ToolBar"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Column spacing="0" width="{width|fill}">
                    <style>
                        .toolbar { background: #8080801f; }
                    </style>

                    <Container class="toolbar {bar_class}" padding="{padding|6 8}" width="{width|fill}">
                        <Row
                            class="{content_class}"
                            spacing="{spacing|4}"
                            align_y="{align_y|center}"
                            width="{width|fill}"
                        >
                            <slot/>
                        </Row>
                    </Container>

                    <template if="{divider|true}" equals="true">
                        <Rule />
                    </template>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Recipiente puro: as ações são dos botões que o app pôs dentro, e
        // pertencem a ele (o conteúdo do slot nem passa por aqui).
    }
}
