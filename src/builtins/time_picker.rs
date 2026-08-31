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
/// - `on_change`   — ação **do app** disparada quando o texto é editado.
/// - `on_pick`     — ação **do app** disparada quando o botão de "relógio" é clicado.
/// - `placeholder` — texto de dica. Default: `HH:MM`.
/// - `width`       — largura do campo de texto. Default: `120`.
/// - `pick_icon`   — ícone/texto do botão. Default: `⏰`.
///
/// ```xml
/// <TimePicker value="hora_almoço" on_change="validar_hora" on_pick="abrir_modal_hora" />
/// ```
///
/// # Delegação: por que `app:` na frente das ações
///
/// Este é um widget que **delega**: quem decide o que fazer é o app, não ele.
/// Toda ação escrita no template de um componente é prefixada com o dono
/// (`namespace_action`), e sem escape as ações recebidas por prop virariam
/// `TimePicker::validar_hora` — entregues ao `update` deste builtin, que não as
/// conhece. O `app:` (ver [`crate::eval::APP_ACTION_PREFIX`]) devolve a ação a
/// quem a definiu. Handler do app mora na tela.
///
/// O `value` do campo é o **nome da chave** de contexto; quem escreve nela é o
/// handler de `on_change` do app (o motor não escreve sozinho num `<TextInput>`).
/// O texto digitado chega ao handler como argumento — em Luau, o parâmetro da
/// função ou o global `value`:
///
/// ```lua
/// function validar_hora(digitado)
///     ctx.hora_almoço = digitado
/// end
/// ```
use crate::component::{Component, Context, Template};

pub struct TimePicker;

impl Component for TimePicker {
    fn name(&self) -> &str {
        "TimePicker"
    }

    fn template(&self) -> Template {
        // `value` (não `value_var`, que é só o nome do campo interno do
        // `NodeType`) é o atributo que o parser lê — ver `parser.rs`, braço
        // "TextInput". `{value}` interpola para o *nome* da chave de contexto.
        Template::Inline(
            r#"<Row spacing="4" align_y="center">
                    <TextInput
                        value="{value}"
                        onChange="app:{on_change}"
                        placeholder="{placeholder|HH:MM}"
                        width="{width|120}"
                    />
                    <Button
                        text="{pick_icon|⏰}"
                        on_click="app:{on_pick}"
                        padding="6 10"
                    />
                </Row>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Delegante: todo comportamento é do app, via `on_change`/`on_pick`.
    }
}
