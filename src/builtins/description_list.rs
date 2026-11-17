/// A lista de **pares rótulo → valor**: a ficha de um item ("Região: sa-east-1",
/// "Criado em: 12/11"), o `DescriptionList` do gpui-kit, o `<dl>` do HTML.
///
/// ```gv
/// descriptionlist(:items = detalhes)
/// ```
///
/// ```lua
/// ctx.detalhes = json.encode(json.array({
///     { label = "Imagem",  value = "nginx:1.27" },
///     { label = "Porta",   value = "8080" },
///     { label = "Réplicas", value = "3" },
/// }))
/// ```
///
/// # Por que não um `grid`
///
/// Dá para escrever com um `grid` ou uma `column` de `row`s — e é o que este
/// builtin faz por dentro. O que ele acrescenta é o que toda ficha repete e
/// todo mundo erra: a coluna de rótulos com **largura fixa** (sem ela, cada
/// linha alinha o valor num lugar diferente), o rótulo apagado e o valor
/// forte, e o `layout = vertical` para telas estreitas.
///
/// # As linhas vêm de uma coleção do contexto
///
/// `items` é o **nome de uma chave** com um array JSON de `{label, value}` —
/// a mesma convenção do `tabbar` e do `for_each` (uma lista literal no
/// atributo não existe). Um valor ausente ou vazio aparece como `—`, para a
/// linha não sumir nem desalinhar.
///
/// Não há linha divisória entre os pares: o `for_each` não expõe a posição
/// do item, e um `rule` por item deixaria um sobrando depois do último. Quem
/// quer separação usa `spacing` maior ou uma `row_class` com fundo.
///
/// # Props
///
/// - `items`       — **obrigatória**: nome da chave com o array de
///   `{label, value}`.
/// - `layout`      — `horizontal` (default: rótulo à esquerda, valor à
///   direita) ou `vertical` (rótulo em cima do valor).
/// - `label_width` — largura da coluna de rótulos no `horizontal`.
///   Default: `140`.
/// - `spacing`     — espaço entre as linhas. Default: `8`.
/// - `size`        — corpo do texto. Default: `13`.
/// - `width`       — Default: `fill`.
/// - `row_class` / `label_class` / `value_class` — classes da linha, do
///   rótulo e do valor (`class` no uso pinta a coluna inteira, a raiz).
///
/// # Aparência
///
/// `.dl-label` (apagado) e `.dl-value` na folha global do template.
use crate::component::{Component, Context, Template};

pub struct DescriptionList;

impl Component for DescriptionList {
    fn name(&self) -> &str {
        "DescriptionList"
    }

    fn template(&self) -> Template {
        // O `if`/`else` do layout fica DENTRO do corpo do `for_each` (as duas
        // diretivas não compõem num nó só — ver `tab_bar.rs`). O `—` do valor
        // vazio é um `if` e não um `{par.value|—}`: o default inline só cobre a
        // chave AUSENTE, e `"value": ""` é chave presente.
        Template::Inline(
            r##"
                column(spacing = "{spacing|8}", width = "{width|fill}") {
                  style """
                                        .dl-label { color: #80868d; }
                                    """
                  template(:for_each = "{items}", var = par) {
                    template(if = "{layout|horizontal}", equals = vertical) {
                      column(class = @row_class, spacing = 2, width = fill) {
                        text(class = "dl-label {label_class}", content = @par.label, size = "{size|13}")
                        template(if = "{par.value|}", not_equals = "") {
                          text(class = "dl-value {value_class}", content = @par.value, size = "{size|13}")
                        }
                        template(else) {
                          text(class = "dl-value {value_class}", content = "—", size = "{size|13}")
                        }
                      }
                    }
                    template(else) {
                      row(class = @row_class, spacing = 12, width = fill) {
                        text(class = "dl-label {label_class}", content = @par.label, size = "{size|13}", width = "{label_width|140}")
                        template(if = "{par.value|}", not_equals = "") {
                          text(class = "dl-value {value_class}", content = @par.value, size = "{size|13}", width = fill)
                        }
                        template(else) {
                          text(class = "dl-value {value_class}", content = "—", size = "{size|13}", width = fill)
                        }
                      }
                    }
                  }
                }
            "##
            .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional.
    }
}
