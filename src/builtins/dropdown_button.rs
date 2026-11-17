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
/// O que faltava no `menu` para isto o motor ganhou: `variant`
/// (`primary`/`secondary`, o gatilho com o estilo de botão do tema) e
/// `on_click`, que o faz desenhar o **split button** — as duas metades no
/// mesmo render, com o mesmo texto e padding (alturas iguais) e só os cantos
/// de fora arredondados, o que as faz ler como um grupo. A primeira versão
/// compunha um `button` com um `menu` ao lado e ficou com duas alturas e
/// quatro cantos redondos.
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
/// - `caret`    — o glifo da seta no botão-menu (sem `on_click`). Default: `▾`.
///   No split button a seta é sempre `▾`.
use crate::component::{Component, Context, Template};

pub struct DropdownButton;

impl Component for DropdownButton {
    fn name(&self) -> &str {
        "DropdownButton"
    }

    fn template(&self) -> Template {
        // As duas formas são o MESMO `menu` do motor: com `on_click`, ele se
        // desenha como split button (as duas metades no mesmo render, cantos
        // arredondados só por fora); sem, é um botão só, com a seta no rótulo.
        // A `row` em volta existe porque a raiz de um template é um nó só.
        Template::Inline(
            r##"
                row {
                  template(if = @on_click, not_equals = "") {
                    menu(label = @label, on_click = "app:{on_click}", variant = "{variant|primary}", :items = "{items}") {
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
