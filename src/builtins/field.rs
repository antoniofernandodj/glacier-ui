/// A **casca de um campo de formulário**: rótulo, marca de obrigatório, o
/// controle, a ajuda e a mensagem de erro — o `Field` do gpui-kit.
///
/// ```gv
/// form(on_submit = salvar) {
///   field(label = Nome, required = true, error = @erro_nome) {
///     input(form_control = nome, rules = required)
///   }
///   field(label = Descrição, description = "Aparece na lista de projetos") {
///     input(form_control = descricao)
///   }
/// }
/// ```
///
/// # O que ele faz, e o que não faz
///
/// O `form` do motor já valida, publica a mensagem de cada campo em
/// `{erro_<campo>}` e acende o `:invalid`. O que ele não fazia era **desenhar**
/// o campo: cada tela repetia `column > text(rótulo) > controle > text(erro, if
/// …)` — e errava a condição do erro de jeitos que não dão erro nenhum (ler a
/// chave com o prefixo errado, testar a string com `not_empty`, que só vale
/// para array JSON). O `field` é essa repetição escrita uma vez, com a condição
/// certa.
///
/// Ele **não** valida nem lê o `form_control` do controle de dentro: o erro
/// chega por prop (`error = @erro_nome`). O `field` não enxerga os atributos do
/// conteúdo do `slot` — e é por isso que a prop existe.
///
/// # Ajuda e erro
///
/// `description` aparece sob o controle; quando `error` tem texto, a mensagem
/// de erro aparece **no lugar** da ajuda (as duas juntas empurram o formulário
/// a cada tecla). Erro vazio = sem linha.
///
/// # Rótulo em cima ou ao lado
///
/// `label_position = top` (default) põe o rótulo acima do controle;
/// `label_position = left` põe ao lado, numa coluna de `label_width` (default
/// `140`) — vários `field` seguidos assim alinham os controles numa linha só,
/// que é o formulário de preferências do desktop.
///
/// # O canto do rótulo: `slot(name = aside)`
///
/// Algo à direita do rótulo, na mesma linha (um "seguro", um link "esqueci a
/// senha", um contador de caracteres):
///
/// ```gv
/// field(label = Token) {
///   template(slot = aside) { text("seguro", class = fraco) }
///   input(form_control = token, secure = true)
/// }
/// ```
///
/// Só no `label_position = top`.
///
/// # Props
///
/// - `label`          — o rótulo. Vazio = sem linha de rótulo.
/// - `required`       — `true` desenha um `*` depois do rótulo. Só a marca: a
///   regra continua no controle (`rules = required`).
/// - `description`    — a ajuda sob o controle.
/// - `error`          — a mensagem de erro (`@erro_<campo>`); vazia = nada.
/// - `label_position` — `top` (default) ou `left`.
/// - `label_width`    — largura do rótulo no `left`. Default: `140`.
/// - `spacing`        — espaço entre rótulo, controle e ajuda. Default: `6`.
/// - `width`          — Default: `fill`.
/// - `label_class` / `description_class` / `error_class` — classes dos textos
///   de dentro (`class` no uso pinta a coluna inteira, a raiz).
///
/// # Aparência
///
/// `.field-label`, `.field-required`, `.field-description` e `.field-error`
/// na folha global do template — uma folha do app que redefina essas classes
/// vence por ordem.
use crate::component::{Component, Context, Template};

pub struct Field;

impl Component for Field {
    fn name(&self) -> &str {
        "Field"
    }

    fn template(&self) -> Template {
        // O `slot` aparece nos dois ramos do layout: só um ramo é expandido, e
        // o conteúdo entra uma vez. A escada ajuda/erro também se repete nos
        // dois — o `left` a põe sob o controle, na coluna da direita, para ela
        // alinhar com o controle e não com o rótulo.
        Template::Inline(
            r##"
                column(spacing = "{spacing|6}", width = "{width|fill}") {
                  style """
                                        .field-label { size: 13; }
                                        .field-required { color: #F38BA8; }
                                        .field-description { size: 12; color: #80868d; }
                                        .field-error { size: 12; color: #F38BA8; }
                                    """
                  template(if = "{label_position|top}", equals = left) {
                    row(spacing = 12, width = fill, align_y = center) {
                      row(spacing = 2, width = "{label_width|140}") {
                        text(class = "field-label {label_class}", content = @label)
                        template(if = "{required|false}", equals = true) {
                          text(class = field-required, content = "*")
                        }
                      }
                      column(spacing = "{spacing|6}", width = fill) {
                        slot
                        template(if = "{error|}", not_equals = "") {
                          text(class = "field-error {error_class}", content = @error)
                        }
                        template(else_if = "{description|}", not_equals = "") {
                          text(class = "field-description {description_class}", content = @description)
                        }
                      }
                    }
                  }
                  template(else) {
                    template(if = "{label|}{slot_aside|}", not_equals = "") {
                      row(spacing = 2, width = fill, align_y = center) {
                        text(class = "field-label {label_class}", content = @label)
                        template(if = "{required|false}", equals = true) {
                          text(class = field-required, content = "*")
                        }
                        template(if = "{slot_aside|false}", equals = true) {
                          space(width = fill)
                          slot(name = aside)
                        }
                      }
                    }
                    slot
                    template(if = "{error|}", not_equals = "") {
                      text(class = "field-error {error_class}", content = @error)
                    }
                    template(else_if = "{description|}", not_equals = "") {
                      text(class = "field-description {description_class}", content = @description)
                    }
                  }
                }
            "##
            .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Recipiente puro: o controle de dentro é da tela.
    }
}
