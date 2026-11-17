/// O **botão com menu**: o `DropdownButton` do gpui-kit. Duas formas.
///
/// Com `on_click`, é o *split button* — a ação principal à esquerda e um "▾"
/// colado que abre as alternativas:
///
/// ```gv
/// dropdownbutton(label = Deploy, on_click = deploy, :items = acoes_deploy)
/// ```
///
/// Sem `on_click`, o botão inteiro abre o menu:
///
/// ```gv
/// dropdownbutton(label = Exportar) {
///   menuitem(label = CSV, on_click = exportar_csv)
///   menuitem(label = JSON, on_click = exportar_json)
/// }
/// ```
///
/// # Sobre o `menu` do motor, não sobre o `popover`
///
/// O `menu` avulso (fora de uma `menubar`) já era um botão que abre um menu:
/// painel pintado com a paleta do tema, que fecha sozinho ao escolher, com
/// separador, submenu, `checked` e `disabled`. Um `popover` daria a âncora
/// na borda do botão, mas o painel dele não tem fundo próprio — e o `.gss` não
/// alcança as cores do tema, então um fundo opaco escolhido aqui ficaria
/// errado em metade dos temas; e fechar o painel **e** avisar a tela num
/// clique só pediria motor novo. O preço do `menu`: o painel abre no ponto do
/// clique, não alinhado à borda do botão.
///
/// O que faltava no `menu` para isto era o gatilho com cara de botão: o
/// atributo `variant` (`primary`/`secondary`) do motor, que a metade do "▾"
/// usa para casar com a metade da ação principal (um `button` sem `color`, que
/// já é o primário do tema).
///
/// # As opções
///
/// Por markup (o `slot`: `menuitem`, `menuseparator`, `menu` para submenu) ou
/// por uma chave com o array JSON do `menu` (`{label, action, separator,
/// items, icon, disabled, checked}`) — ou os dois, que se somam. As ações das
/// duas formas são da **tela**.
///
/// # Props
///
/// - `label`    — o texto do botão.
/// - `on_click` — a ação principal. Presente = *split button*; ausente = o
///   botão inteiro abre o menu.
/// - `items`    — **nome da chave** com o array JSON das opções.
/// - `variant`  — `primary` (default, o botão do tema) ou `flat` (fundo
///   transparente, como numa barra de ferramentas).
/// - `caret`    — o glifo da seta. Default: `▾`.
/// - `button_class` — classe do botão da ação principal.
use crate::component::{Component, Context, Template};

pub struct DropdownButton;

impl Component for DropdownButton {
    fn name(&self) -> &str {
        "DropdownButton"
    }

    fn template(&self) -> Template {
        // `spacing = 1`: a fresta entre as duas metades é o que faz o split
        // button ler como dois alvos de clique, sem precisar de borda por
        // canto (que o motor não tem).
        Template::Inline(
            r##"
                row(spacing = 1, align_y = center) {
                  style """
                                        .dropdownbutton-flat {
                                            color: #00000000;
                                            text_color: #80868d;
                                            border_width: 0;
                                            border_radius: 4;
                                        }
                                        .dropdownbutton-flat:hover { background: #8080803d; }
                                    """
                  template(if = @on_click, not_equals = "") {
                    template(if = "{variant|primary}", equals = flat) {
                      button(class = "dropdownbutton-flat {button_class}", text = @label, on_click = "app:{on_click}", padding = "6 12")
                    }
                    template(else) {
                      button(class = @button_class, text = @label, on_click = "app:{on_click}", padding = "6 12")
                    }
                    menu(label = "{caret|▾}", variant = "{variant|primary}", :items = "{items}") {
                      slot
                    }
                  }
                  template(else) {
                    menu(label = "{label} {caret|▾}", variant = "{variant|primary}", :items = "{items}") {
                      slot
                    }
                  }
                }
            "##
            .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Delegante: a ação principal vai para a tela por `app:`, e as do
        // menu já nascem da tela (markup do slot) ou sem dono (JSON).
    }
}
