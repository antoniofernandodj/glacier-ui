/// A tecla desenhada: o atalho de teclado mostrado como uma "tampinha" — o
/// `<kbd>` do HTML, o `Kbd` do gpui-kit. Puramente apresentacional.
///
/// ```gv
/// row {
///   text("Abrir a paleta:")
///   kbd("Ctrl+K")
/// }
/// ```
///
/// # Uma tampinha por combinação
///
/// `kbd("Ctrl+Shift+P")` desenha **uma** tampinha com o texto inteiro, que é a
/// forma compacta do gpui-kit e do macOS (`⌘K`). Quem quer uma tampinha por
/// tecla escreve uma por tecla — `kbd("Ctrl") text("+") kbd("K")`: o template
/// não tem como partir uma string, e partir no Rust esconderia do markup o que
/// ele desenha.
///
/// # Fora e dentro de um `prose`
///
/// Dentro de um `prose`, `kbd` continua sendo o que sempre foi lá: estilo
/// inline de código no meio do parágrafo (`src/prose.rs`). Este builtin é a
/// tag **fora** do `prose` — onde, antes dele, `kbd` era `UnknownComponent`.
///
/// # Props
///
/// - `keys` — o texto da tecla; é o primeiro argumento sem nome
///   (`kbd("Esc")`). Default: vazio.
/// - `size` — corpo do texto. Default: `12`.
/// - `text_class` — classe do texto de dentro (`class` no uso pinta a
///   tampinha, que é a raiz).
///
/// # Aparência
///
/// `.kbd-cap` (a tampinha) e `.kbd-text` (o texto) na folha global do
/// template, em cinza translúcido: clareia sobre tema escuro e escurece sobre
/// claro, sem saber qual está ativo — o mesmo truque do `spinbox`.
use crate::component::{Component, Context, Template};

pub struct Kbd;

impl Component for Kbd {
    fn name(&self) -> &str {
        "Kbd"
    }

    fn template(&self) -> Template {
        // A borda é a da caixa inteira (o motor não tem borda por lado), então
        // o "relevo" da tecla de verdade vira uma moldura fina + fundo leve.
        Template::Inline(
            r##"
                container(class = kbd-cap, padding = "2 6") {
                  style """
                                        .kbd-cap {
                                            background: #8080801f;
                                            border_width: 1;
                                            border_color: #80808059;
                                            border_radius: 4;
                                        }
                                        .kbd-text { font_family: monospace; }
                                    """
                  text(class = "kbd-text {text_class}", content = @keys, size = "{size|12}")
                }
            "##
            .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional.
    }
}
