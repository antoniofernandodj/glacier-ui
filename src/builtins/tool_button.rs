/// `QToolButton`: o botão de barra de ferramentas — um ícone (e opcionalmente
/// um rótulo) que só ganha fundo quando o ponteiro passa por cima.
///
/// É o irmão discreto do `<Button>`: mesma ação, peso visual muito menor. Numa
/// barra com dez ações, dez botões primários competiriam entre si e com o
/// conteúdo da tela; o `autoRaise` do Qt existe exatamente para isso, e é o
/// default aqui.
///
/// ```xml
/// <ToolButton icon="💾" tooltip="Salvar" on_click="salvar" />
/// <ToolButton icon="🗑" text="Excluir" layout="beside" on_click="excluir" />
/// ```
///
/// # Delegação
///
/// Como o [`super::time_picker::TimePicker`], este widget **delega**: quem
/// decide o que o clique faz é o app. Por isso o template escreve
/// `on_click="app:{on_click}"` — sem o escape, `namespace_action` entregaria
/// `ToolButton::salvar` ao `update` daqui, que não conhece ação nenhuma do app,
/// e o botão simplesmente não faria nada (ver [`crate::eval::APP_ACTION_PREFIX`]).
///
/// # Ícone: glifo ou SVG
///
/// `icon` é um glifo (emoji, símbolo Unicode) e `icon_src` é o caminho de um
/// `.svg`. Passe **um** dos dois; com `icon_src` presente o glifo é ignorado.
/// O SVG herda `icon_color` — um `.svg` monocromático é o que a barra costuma
/// querer, e é o que o `<Svg color=…>` do motor sabe recolorir.
///
/// # As três formas
///
/// A prop `layout` espelha o `Qt::ToolButtonStyle`:
///
/// - `icon` (**default**) — só o ícone. `QToolButton` clássico de barra.
/// - `beside` — ícone e rótulo lado a lado (`ToolButtonTextBesideIcon`).
/// - `under` — rótulo sob o ícone (`ToolButtonTextUnderIcon`), a forma das
///   barras grandes/de toque.
///
/// # Props
///
/// - `on_click`   — **ação do app** disparada no clique.
/// - `icon`       — glifo do ícone. Default: `●`.
/// - `icon_src`   — caminho de um `.svg`; vence o `icon` quando presente.
/// - `icon_color` — cor do SVG. Default: o cinza neutro da folha.
/// - `text`       — rótulo, usado por `layout="beside"`/`"under"`.
/// - `layout`     — `icon` (default), `beside` ou `under`.
/// - `icon_size`  — corpo do glifo / lado do SVG. Default: `16`.
/// - `text_size`  — corpo do rótulo. Default: `12`.
/// - `padding`    — área de clique em volta. Default: `6 8`.
/// - `tooltip`    — dica; num botão só-ícone é ela que diz o que ele faz, então
///   vale sempre preencher em `layout="icon"`.
///
/// # Aparência
///
/// A folha global `.toolbutton` é o `autoRaise`: fundo transparente parado,
/// cinza translúcido no `:hover` e mais escuro no `:active`. O mesmo cinza com
/// alfa dos outros builtins, que clareia no tema escuro e escurece no claro.
/// Redefinir `.toolbutton` numa folha do app repinta — inclusive para dar a ele
/// um fundo permanente, se a barra pedir.
use crate::component::{Component, Context, Template};

pub struct ToolButton;

impl Component for ToolButton {
    fn name(&self) -> &str {
        "ToolButton"
    }

    fn template(&self) -> Template {
        // O ícone aparece nos três braços do `layout`, e em cada um repete o
        // par glifo/SVG — seis ocorrências para três formas. A alternativa
        // seria um `<slot/>` para o ícone, mas o slot é do CONTEÚDO de quem
        // usa o widget, não um mecanismo de reuso interno do template; um
        // componente interno (`ToolButtonIcon`) resolveria a repetição ao custo
        // de um nome público a mais na biblioteca. Repetir é mais barato.
        Template::Inline(
            r#"<Button
                    class="toolbutton"
                    on_click="app:{on_click}"
                    padding="{padding|6 8}"
                    tooltip="{tooltip}"
                >
                    <style>
                        .toolbutton {
                            color: #00000000;
                            border-width: 0;
                            border-radius: 5;
                        }
                        .toolbutton:hover  { background: #8080803d; }
                        .toolbutton:active { background: #80808066; }
                        .toolbutton-icon   { color: #80868d; }
                        .toolbutton-label  { color: #80868d; }
                    </style>

                    <template if="{layout|icon}" equals="beside">
                        <Row spacing="6" align_y="center">
                            <template if="{icon_src}" notEquals="">
                                <Svg
                                    source="{icon_src}"
                                    color="{icon_color|#80868d}"
                                    width="{icon_size|16}"
                                    height="{icon_size|16}"
                                />
                            </template>
                            <template else>
                                <Text
                                    class="toolbutton-icon"
                                    content="{icon|●}"
                                    size="{icon_size|16}"
                                />
                            </template>
                            <Text
                                class="toolbutton-label"
                                content="{text}"
                                size="{text_size|12}"
                            />
                        </Row>
                    </template>

                    <template else-if="{layout|icon}" equals="under">
                        <Column spacing="3" align_x="center">
                            <template if="{icon_src}" notEquals="">
                                <Svg
                                    source="{icon_src}"
                                    color="{icon_color|#80868d}"
                                    width="{icon_size|16}"
                                    height="{icon_size|16}"
                                />
                            </template>
                            <template else>
                                <Text
                                    class="toolbutton-icon"
                                    content="{icon|●}"
                                    size="{icon_size|16}"
                                />
                            </template>
                            <Text
                                class="toolbutton-label"
                                content="{text}"
                                size="{text_size|12}"
                            />
                        </Column>
                    </template>

                    <template else>
                        <template if="{icon_src}" notEquals="">
                            <Svg
                                source="{icon_src}"
                                color="{icon_color|#80868d}"
                                width="{icon_size|16}"
                                height="{icon_size|16}"
                            />
                        </template>
                        <template else>
                            <Text
                                class="toolbutton-icon"
                                content="{icon|●}"
                                size="{icon_size|16}"
                            />
                        </template>
                    </template>
                </Button>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Delegante: o clique é do app, entregue pelo prefixo `app:`.
    }
}
