/// `QSizeGrip`: o canto que redimensiona a janela.
///
/// ```xml
/// <row class="rodape">
///     <text class="status">Pronto</text>
///     <space />
///     <SizeGrip />
/// </row>
/// ```
///
/// # O item mais barato da Onda 9, e vale registrar por quê
///
/// O catálogo o listava como **Motor** (`§2.9`, ⬜ P3, "canto de
/// redimensionamento"), e a Onda 9 o pôs na fila como o sétimo consumidor do
/// arrasto — o único cujo alvo não é uma chave, mas a janela. Ao escrever, não
/// sobrou consumidor nenhum: **as duas metades já existiam**, e nenhuma delas é
/// do `crate::grip`.
///
/// - `window:resize:se` já era uma ação da titlebar custom, traduzida em
///   `iced::window::drag_resize` (`daemon.rs`, `window_control`). É o SO que
///   arrasta a janela, não o motor;
/// - `cursor="se"` já era um atributo universal de qualquer nó, mapeado em
///   `cursor_interaction` (`widget.rs`) para `ResizingDiagonallyDown`.
///
/// Sobrou desenhar o cantinho — e é por isso que ele é **builtin**, e não
/// Motor: um builtin é exatamente "o que dá para compor de primitivas com só
/// props". A reclassificação é a 15ª deste catálogo, e a primeira em que o
/// erro foi para o lado do *nível de infraestrutura* em vez do de estado.
///
/// # Props
///
/// - `corner` — de qual canto ele puxa. Default `se` (o de baixo à direita, o
///   do `QSizeGrip`); `sw` para janelas com a barra do outro lado.
/// - `size`   — o lado do quadrado, em pixels. Default 14.
///
/// # Uma janela sem decoração é o caso de uso
///
/// Com a decoração do SO ligada, a borda da janela já redimensiona e este
/// widget é decorativo. Ele existe para o app que desenha a própria titlebar
/// (`decorations="false"`), onde essa borda não existe — o mesmo motivo pelo
/// qual o `window:drag` existe.
use crate::component::{Component, Context, Template};

pub struct SizeGrip;

impl Component for SizeGrip {
    fn name(&self) -> &str {
        "SizeGrip"
    }

    fn template(&self) -> Template {
        // As três barrinhas na diagonal são desenhadas com `<rule>`, e não com
        // um `.svg`: um builtin que dependesse de arquivo deixaria de ser
        // `Template::Inline` (compilado no binário) e quebraria a garantia de
        // que `GlacierUI::new()` é infalível — ver o cabeçalho de `builtins`.
        //
        // O `on_press` e não `on_click`: quem redimensiona é o SO a partir do
        // botão APERTADO, e um clique só chega no soltar, quando não há mais o
        // que arrastar.
        Template::Inline(
            r#"<Container
                    class="sizegrip {grip_class}"
                    width="{size|14}"
                    height="{size|14}"
                    on_press="window:resize:{corner|se}"
                    cursor="{corner|se}"
                    tooltip="{tooltip|}"
                >
                    <style>
                        .sizegrip { padding: 2; }
                        .sizegrip-marca { color: var(--sizegrip-cor, #6C7086); }
                    </style>
                    <Column spacing="2" align_x="end">
                        <Rule class="sizegrip-marca" width="4" />
                        <Rule class="sizegrip-marca" width="8" />
                        <Rule class="sizegrip-marca" width="12" />
                    </Column>
                </Container>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Nada, e literalmente: a ação que ele emite é do motor
        // (`window:resize:…`), tratada no daemon antes de chegar a componente
        // nenhum.
    }
}
