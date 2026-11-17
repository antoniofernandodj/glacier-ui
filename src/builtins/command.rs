/// A **paleta de comandos**: um campo de busca sobre uma lista de ações, que
/// filtra enquanto se digita — o `Command` do gpui-kit, o Ctrl+Shift+P do VS
/// Code, o ⌘K do macOS.
///
/// ```gv
/// command(:items = comandos, :value = busca, filter = @busca, on_select = executar)
/// ```
///
/// ```lua
/// ctx.comandos = json.encode(json.array({
///     { id = "novo",   label = "Novo projeto",  hint = "cria do zero", shortcut = "Ctrl+N" },
///     { id = "abrir",  label = "Abrir pasta",   shortcut = "Ctrl+O" },
///     { id = "tema",   label = "Alternar tema", keywords = "escuro claro dark" },
/// }))
///
/// function executar(id: string): ()
///     ctx.busca = ""
///     -- … o que o comando `id` faz
/// end
/// ```
///
/// # O par `value` + `filter`
///
/// É o par `value` + `active` do `tabbar`: `value` é o **nome** da chave que a
/// digitação escreve (o `update` daqui grava sozinho, sem uma linha de Luau);
/// `filter` é o **valor** dessa chave, que o template lê para filtrar. O
/// template não consegue ler o valor de uma chave cujo nome está numa prop (a
/// indireção `{{value}}` que o interpolador não tem), daí os dois.
///
/// # O filtro
///
/// Cada item aparece quando **todas** as palavras digitadas estão, sem
/// diferenciar maiúsculas, no `label`, no `hint` ou nas `keywords` dele — a
/// condição `matches=` do motor, que nasceu para este widget e vale em
/// qualquer lista. Campo vazio mostra tudo.
///
/// # A escolha é da tela
///
/// Clicar num item dispara `on_select` **na tela**, com o `id` como
/// argumento (`executar("novo")`), pelo prefixo `app:`. O widget não fecha
/// nada nem limpa a busca: quem abriu a paleta (um `dialog`, um `popover`, um
/// `reveal`) é quem a fecha, e limpar a busca é uma linha (`ctx.busca = ""`).
///
/// # Props
///
/// - `items`       — **obrigatória**: nome da chave com o array de
///   `{id, label, hint?, shortcut?, keywords?}`. `shortcut` sai desenhado num
///   `kbd` à direita; `keywords` só entra na busca, não aparece.
/// - `value`       — **obrigatória**: nome da chave do texto da busca.
/// - `filter`      — o valor atual da chave `value` (`filter = @busca`).
///   Sem ele, a lista não filtra.
/// - `on_select`   — **obrigatória**: ação da tela que recebe o `id`.
/// - `placeholder` — dica do campo. Default: `Digite um comando…`.
/// - `max_height`  — altura máxima da lista, que rola além dela.
///   Default: `320`.
/// - `width`       — Default: `fill`.
/// - `input_class` / `item_class` / `label_class` / `hint_class` — classes
///   do campo, de cada item, do rótulo e da dica (`class` no uso pinta a
///   coluna inteira, a raiz).
///
/// # Limites conhecidos
///
/// - **Sem navegação por setas nem Enter.** O `textinput` não tem
///   `on_submit`, e um `update` não consegue despachar a ação do app; a
///   escolha é por clique (ou Tab até o item e Espaço). O campo nasce com
///   `autofocus`, então abrir a paleta e digitar funciona sem mouse.
/// - **Sem "nenhum resultado".** O template não sabe quantos itens o filtro
///   deixou passar. Quem quer a mensagem calcula na tela.
use crate::component::{Component, Context, Template};

pub struct Command;

impl Component for Command {
    fn name(&self) -> &str {
        "Command"
    }

    fn template(&self) -> Template {
        // `on_change = "edit:{value}"` cai no `update` daqui (prefixo de dono);
        // `on_click = "app:{on_select}:{cmd.id}"` pula o dono e vai para a tela
        // — os dois padrões do BUILTINS.md ("widget que age" e "widget que
        // delega") no mesmo template.
        //
        // A condição junta os três campos num texto só: `matches` procura as
        // palavras em qualquer lugar dele. `{cmd.hint|}`/`{cmd.keywords|}`
        // garantem vazio, não o marcador literal, quando o item não os tem.
        Template::Inline(
            r##"
                column(spacing = 6, width = "{width|fill}") {
                  style """
                                        .command-item {
                                            color: #00000000;
                                            text_color: #cdd6f4;
                                            border_width: 0;
                                            border_radius: 6;
                                        }
                                        .command-item:hover { color: #8080802e; }
                                        .command-hint { color: #80868d; }
                                    """
                  textinput(class = @input_class, :value = "{value}", on_change = "edit:{value}", placeholder = "{placeholder|Digite um comando…}", autofocus = true, width = fill)
                  scrollable(max_height = "{max_height|320}", width = fill) {
                    column(spacing = 2, width = fill) {
                      template(:for_each = "{items}", var = cmd) {
                        template(if = "{cmd.label} {cmd.hint|} {cmd.keywords|}", matches = @filter) {
                          button(class = "command-item {item_class}", on_click = "app:{on_select}:{cmd.id}", padding = "7 10", width = fill) {
                            row(spacing = 10, align_y = center, width = fill) {
                              text(class = @label_class, content = @cmd.label, size = 13)
                              template(if = "{cmd.hint|}", not_equals = "") {
                                text(class = "command-hint {hint_class}", content = @cmd.hint, size = 12)
                              }
                              space(width = fill)
                              template(if = "{cmd.shortcut|}", not_equals = "") {
                                kbd(keys = @cmd.shortcut, size = 11)
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
            "##
            .to_string(),
        )
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // `edit:busca` — a chave que o app nomeou; o texto vem no `value`.
        let Some(("edit", chave)) = action.split_once(':') else {
            return;
        };
        let chave = chave.trim();
        // Sem `value` não há onde escrever (mesma regra do `tabbar`).
        if chave.is_empty() {
            return;
        }
        ctx.set(chave, value.unwrap_or_default().to_string());
    }
}
