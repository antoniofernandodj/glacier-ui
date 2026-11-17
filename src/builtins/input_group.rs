/// O **campo com acessórios**: um texto, ícone ou botão colado antes e/ou
/// depois do campo, dentro de **uma** moldura só — o `Input Group` do
/// gpui-kit (e do Bootstrap).
///
/// ```gv
/// inputgroup(prefix = "R$", suffix = ",00") {
///   input(form_control = preco)
/// }
///
/// inputgroup(prefix = "https://", action = Copiar, on_action = "clipboard:dominio") {
///   input(:value = dominio, on_change = dominio)
/// }
/// ```
///
/// # O botão colado: `action`
///
/// Um botão no fim do grupo é o caso comum (Copiar, Buscar, Enviar), e ele só
/// lê como parte do grupo quando **encosta**: a altura inteira do campo, sem
/// folga, cantos retos do lado do campo e redondos do lado de fora. É o que
/// `action`/`on_action` desenham (com o `border_radius = "0 5 5 0"` por canto,
/// 0.164). Um `button` posto no slot `suffix` fica solto lá dentro — o slot é
/// para ícone ou texto, não para botão.
///
/// A altura casa por **aritmética**, não por `height = fill`: o `textinput` e
/// o `button` do iced têm o mesmo corpo de texto e o mesmo padding vertical
/// padrão (5 px), então `padding = "5 14"` dá ao botão a altura natural do
/// campo. A primeira versão usava `height = fill` dentro da `row` que encolhe
/// e o botão saiu mais baixo que o próprio rótulo. Consequência: um campo com
/// padding vertical próprio (`.minha_classe { padding: 13 14 }`) desencontra
/// as alturas — repinte o botão por `action_class` com o mesmo padding.
///
/// # Uma moldura, não duas
///
/// O que distingue isto de um `row { text input text }` é a moldura: o grupo
/// desenha a borda em volta de tudo e **tira a do campo de dentro**, por uma
/// regra de descendente na folha da lib (`inputgroup textinput { … }`). O
/// conteúdo de slot tem o **uso** como pai — não a `row` de dentro do template
/// —, então a regra nomeia o tipo do uso (as duas grafias), não a classe da
/// moldura.
///
/// Consequência: uma **classe** do app no campo (`.dialog__input { border…
/// }`) vence essa regra (classe pesa mais que tag), e o campo volta a ter
/// borda própria. Dentro de um `inputgroup`, deixe borda e fundo para o grupo.
///
/// # Limites
///
/// - Sem anel de foco: a borda do campo é a do grupo, e o grupo não sabe
///   quando o campo de dentro está focado. O cursor piscando é o sinal.
/// - O `:invalid` de um campo de `form` não aparece (ele pinta a borda do
///   campo, que aqui é zero). Mostre a mensagem — um `field(error = …)` em
///   volta do grupo é o par natural.
///
/// # Props
///
/// - `prefix` / `suffix` — texto colado antes / depois do campo. Para um
///   ícone, o slot nomeado de mesmo nome (`template(slot = prefix) { … }`),
///   que vence a prop.
/// - `action` / `on_action` — o botão colado no fim (o rótulo e a ação da
///   tela). Ocupa o lugar do `suffix`. `action_class` repinta o botão.
/// - `width`  — Default: `fill`.
/// - `addon_class` — classe dos dois textos (`class` no uso pinta a moldura,
///   a raiz).
///
/// # Aparência
///
/// `.inputgroup` (a moldura) e `.inputgroup-addon` (os textos), em cinza
/// translúcido, que serve a tema claro e escuro.
use crate::component::{Component, Context, Template};

pub struct InputGroup;

impl Component for InputGroup {
    fn name(&self) -> &str {
        "InputGroup"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r##"
                row(class = inputgroup, spacing = 0, align_y = center, width = "{width|fill}") {
                  style """
                                        .inputgroup {
                                            border_width: 1;
                                            border_color: #80808059;
                                            border_radius: 6;
                                        }
                                        .inputgroup-addon { color: #80868d; size: 13; }
                                        inputgroup textinput,
                                        inputgroup maskedinput,
                                        inputgroup comboedit,
                                        InputGroup textinput,
                                        InputGroup maskedinput,
                                        InputGroup comboedit {
                                            background: #00000000;
                                            border_width: 0;
                                        }
                                    """
                  template(if = "{slot_prefix|false}", equals = true) {
                    container(padding = "0 4 0 8") {
                      slot(name = prefix)
                    }
                  }
                  template(else_if = "{prefix|}", not_equals = "") {
                    container(padding = "0 2 0 10") {
                      text(class = "inputgroup-addon {addon_class}", content = @prefix)
                    }
                  }
                  slot
                  template(if = "{action|}", not_equals = "") {
                    button(class = @action_class, text = @action, on_click = "app:{on_action}", border_radius = "0 5 5 0", padding = "5 14")
                  }
                  template(else_if = "{slot_suffix|false}", equals = true) {
                    container(padding = "0 8 0 4") {
                      slot(name = suffix)
                    }
                  }
                  template(else_if = "{suffix|}", not_equals = "") {
                    container(padding = "0 10 0 2") {
                      text(class = "inputgroup-addon {addon_class}", content = @suffix)
                    }
                  }
                }
            "##
            .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Recipiente puro.
    }
}
