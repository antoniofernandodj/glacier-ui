/// Um seletor de tempo composto por um campo de texto e um botão de ação.
///
/// Como o Glacier não possui um widget nativo de "relógio" no `iced`, este
/// builtin atua como um orquestrador semântico: ele agrupa um `TextInput`
/// (para digitação direta ou validação de formato) e um `Button` (para
/// disparar a ação de abrir um modal de seleção customizado ou um diálogo
/// nativo do SO).
///
/// Props (todas opcionais; o default vem do próprio template via `{prop|def}`):
/// - `value`       — nome da variável de contexto que guarda o tempo (ex: "14:30").
/// - `on_change`   — ação disparada quando o texto é editado diretamente.
/// - `on_pick`     — ação disparada quando o botão de "relógio" é clicado.
/// - `placeholder` — texto de dica. Default: `HH:MM`.
/// - `width`       — largura do campo de texto. Default: `120`.
/// - `pick_icon`   — ícone/texto do botão. Default: `⏰`.
///
/// ```xml
/// <TimePicker value="hora_almoço" on_change="validar_hora" on_pick="abrir_modal_hora" />
/// ```
use crate::component::{Component, Context, Template};

pub struct TimePicker;

impl Component for TimePicker {
    fn name(&self) -> &str {
        "TimePicker"
    }

    fn template(&self) -> Template {
        // O `value_var` do TextInput recebe `{value}`, que o eval engine
        // interpolará para o *nome* da variável de contexto (ex: "hora_almoço").
        // O mesmo vale para as ações.
        Template::Inline(
            r#"<Row spacing="4" align_y="center">
                    <TextInput
                        value_var="{value}"
                        on_change="{on_change}"
                        placeholder="{placeholder|HH:MM}"
                        width="{width|120}"
                    />
                    <Button
                        text="{pick_icon|⏰}"
                        on_click="{on_pick}"
                        padding="6 10"
                    />
                </Row>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: sem estado, sem comportamento, sem `init`.
    }
}