/// O aviso que **fica na tela**: uma faixa com ícone, título, descrição e,
/// opcionalmente, um "×" — o `Alert` do gpui-kit (e do shadcn).
///
/// ```gv
/// alert("Backup atrasado", variant = warning, description = "O último rodou há 3 dias.")
///
/// alert(title = "Deploy concluído", variant = success, on_close = fechar_aviso) {
///   button("Ver logs", on_click = ver_logs)
/// }
/// ```
///
/// # Alert, toast e dialog
///
/// Os três avisam; o que muda é quanto o aviso exige:
///
/// - `toast()` — passa sozinho em alguns segundos. Não é tag: é uma função.
/// - `dialog`/`confirm()` — interrompe; a tela espera a resposta.
/// - `alert` — **fica** no lugar onde foi escrito, sem interromper nada, até o
///   estado que o fez aparecer mudar (ou até alguém clicar o "×"). É o aviso
///   que descreve uma *condição* da tela, não um *evento*.
///
/// # Sem estado
///
/// Como todo builtin desta lib, o `alert` não se esconde sozinho: o "×"
/// dispara `on_close` na **tela** (pelo prefixo `app:`), e é a tela que
/// apaga a chave que mantinha o aviso visível (`alert(…, if = @aviso)`). Sem
/// `on_close`, o "×" nem aparece — um × que não fecha nada seria mentira.
///
/// # Props
///
/// - `title`       — o título; é o primeiro argumento sem nome. Vazio = sem
///   título.
/// - `description` — o texto de baixo. Vazio = ausente.
/// - `variant`     — `info` (default), `success`, `warning` ou `error`: as
///   mesmas quatro do `toast()`.
/// - `icon`        — troca o glifo da variante (`ℹ`/`✓`/`⚠`/`✕`). `icon = none`
///   tira o ícone.
/// - `on_close`    — ação da tela disparada pelo "×". Ausente = sem "×".
/// - `width`       — Default: `fill`.
/// - `title_class` / `description_class` — classes dos dois textos (`class`
///   no uso pinta a faixa, a raiz).
///
/// O que se escreve **dentro** da tag (botões, um link) entra sob a
/// descrição — o `slot`. Como todo conteúdo de slot, as ações ali são da tela.
///
/// # Aparência
///
/// `.alert` (moldura comum) mais `.alert-<variante>` (cor), e o ícone em
/// `.alert-icon-<variante>`. Fundo translúcido da cor da variante: a faixa
/// costuma ser larga, e um fundo opaco seria uma área grande pintada a mais.
use crate::component::{Component, Context, Template};

pub struct Alert;

impl Component for Alert {
    fn name(&self) -> &str {
        "Alert"
    }

    fn template(&self) -> Template {
        // A variante entra na CLASSE (`alert-{variant|info}`), não numa escada
        // de `if`: a cor é estilo, e uma classe deixa o app repintar uma
        // variante inteira num lugar só. O glifo é a exceção — texto não é
        // estilo —, por isso ele sai de uma escada `if`/`else_if`.
        Template::Inline(
            r##"
                row(class = "alert alert-{variant|info}", padding = "10 12", spacing = 10, width = "{width|fill}") {
                  style """
                                        .alert { border_width: 1; border_radius: 8; }
                                        .alert-info    { background: #89B4FA1a; border_color: #89B4FA73; }
                                        .alert-success { background: #A6E3A11a; border_color: #A6E3A173; }
                                        .alert-warning { background: #F9E2AF1a; border_color: #F9E2AF80; }
                                        .alert-error   { background: #F38BA81a; border_color: #F38BA880; }
                                        .alert-icon-info    { color: #89B4FA; }
                                        .alert-icon-success { color: #A6E3A1; }
                                        .alert-icon-warning { color: #F9E2AF; }
                                        .alert-icon-error   { color: #F38BA8; }
                                        .alert-description  { color: #80868d; }
                                        .alert-close { color: #00000000; text_color: #80868d; border_width: 0; }
                                        .alert-close:hover { text_color: #cdd6f4; }
                                    """
                  template(if = "{icon|}", not_equals = none) {
                    template(if = "{icon|}", not_equals = "") {
                      text(class = "alert-icon alert-icon-{variant|info}", content = @icon, size = 16, bold = true)
                    }
                    template(else_if = "{variant|info}", equals = success) {
                      text(class = "alert-icon alert-icon-success", content = "✓", size = 16, bold = true)
                    }
                    template(else_if = "{variant|info}", equals = warning) {
                      text(class = "alert-icon alert-icon-warning", content = "⚠", size = 16, bold = true)
                    }
                    template(else_if = "{variant|info}", equals = error) {
                      text(class = "alert-icon alert-icon-error", content = "✕", size = 16, bold = true)
                    }
                    template(else) {
                      text(class = "alert-icon alert-icon-info", content = "ℹ", size = 16, bold = true)
                    }
                  }
                  column(spacing = 4, width = fill) {
                    template(if = @title, not_equals = "") {
                      text(class = "alert-title {title_class}", content = @title, size = 14, bold = true)
                    }
                    template(if = @description, not_equals = "") {
                      text(class = "alert-description {description_class}", content = @description, size = 13)
                    }
                    slot
                  }
                  template(if = @on_close, not_equals = "") {
                    button(class = alert-close, text = "×", on_click = "app:{on_close}", padding = "0 4")
                  }
                }
            "##
            .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: o "×" é da tela (`app:`).
    }
}
