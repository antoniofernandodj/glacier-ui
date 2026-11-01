//! Quais atributos são **ligação** (o nome de uma chave do contexto) e não dado.
//!
//! No markup, `value="progresso"` e `value="{progresso}"` parecem a mesma coisa e
//! são opostas: a primeira entrega ao widget o **nome** da chave (ele lê e grava
//! nela), a segunda entrega o **valor** que a chave tem agora — e o widget passa a
//! procurar uma chave chamada "42". O XML sozinho não distingue um do outro, e o
//! erro falha em silêncio (ver "Quatro armadilhas que falham EM SILÊNCIO" no
//! `PRIMITIVAS.md`).
//!
//! A saída é marcar a ligação **no nome do atributo**, como o `:value` do Vue:
//!
//! ```gv
//! progressbar(:value = progresso)
//! // a ligação: a chave `progresso`
//! text(content = "{progresso}%")
//! // o dado: o valor dela
//! ```
//!
//! Este módulo é a lista do que é ligação. Quem a consulta: o parser do `.gv`
//! (que avisa quando uma ligação chega sem o `:`), o dessugarador do `.gv`
//! (que recusa) e as ferramentas de migração.

/// `(tags, atributos)`: em cada uma das `tags` (e seus apelidos), os `atributos`
/// são o nome de uma chave. Os nomes são comparados sem caixa e sem `-`/`_`.
type Linha = (&'static [&'static str], &'static [&'static str]);

/// Os primitivos (todo campo `*_var` do `parser.rs`) e os builtins cujas props são
/// repassadas como nome de chave.
const TABELA: &[Linha] = &[
    // — primitivos —
    (&["textinput", "input"], &["value"]),
    (&["textarea", "texteditor"], &["value"]),
    (&["maskedinput"], &["value"]),
    (
        &["checkbox", "check"],
        &["checked", "value"],
    ),
    (
        &["toggle", "toggler"],
        &["checked", "value"],
    ),
    (&["progressbar", "progress"], &["value"]),
    (&["timeedit", "timepicker"], &["value"]),
    (
        &["calendar"],
        &[
            "value",
            "start",
            "end",
            "month",
            "mesvisivel",
        ],
    ),
    (
        &["pagination"],
        &["value", "page"],
    ),
    (&["wizardnav"], &["value", "step"]),
    (
        &["colorwheel"],
        &["value", "color"],
    ),
    (&["rating"], &["value"]),
    (
        &["popover"],
        &["value", "open"],
    ),
    (
        &["autocomplete", "completer"],
        &["value", "items", "options"],
    ),
    (
        &["tableview"],
        &[
            "items",
            "rows",
            "columns",
            "cols",
            "value",
            "selected",
            "sort",
            "widths",
            "resize",
        ],
    ),
    (
        &["treeview"],
        &[
            "items", "nodes", "value", "selected", "open", "expanded",
        ],
    ),
    (
        &["columnview"],
        &[
            "items", "nodes", "value", "path", ],
    ),
    (&["dial", "knob"], &["value"]),
    (&["gauge"], &["value"]),
    (&["lcdnumber", "lcd"], &["value"]),
    (
        &["linechart"],
        &["items", "data", "series"],
    ),
    (
        &["barchart"],
        &["items", "data"],
    ),
    (&["piechart", "donut"], &["items", "data"]),
    (
        &["splitter"],
        &["sizes", "value"],
    ),
    (
        &["dock", "dockwidget"],
        &[
            "mode", "value", "size", "sizes", "floatx",
            "floaty", "x", "y",
        ],
    ),
    (&["mdisubwindow"], &["x", "y", "w", "h"]),
    (
        &["swipeview"],
        &["value", "index"],
    ),
    (
        &["rangeslider"],
        &["start", "from", "end", "to"],
    ),
    (&["tumbler"], &["value"]),
    (
        &["rubberband"],
        &["selection", "value", "selected"],
    ),
    (&["shortcutinput", "keysequenceedit"], &["value"]),
    (
        &["radio", "radiobutton"],
        &[
            "group",
            "checked",
            "selected",
            ],
    ),
    (&["slider"], &["value"]),
    (
        &["select", "dropdown", "comboedit", "editablecombo"],
        &[
            "value",
            "selected",
            "items",
            "options",
            "source",
            ],
    ),
    (&["menuitem"], &["checked"]),
    (
        &["menu", "contextmenu"],
        &["items", "options"],
    ),
    // — qualquer tag: a repetição lê o NOME da chave com a lista —
    (&["*"], &["for_each", "foreach", "each", "repeat"]),
    // — builtins (as props que viram nome de chave por dentro) —
    (&["tabbar", "tabs"], &["value", "items"]),
    (&["listview"], &["value", "items"]),
    (&["radiogroup"], &["value", "items"]),
    (&["fontselect"], &["value", "items"]),
    (&["spinbox", "accordion", "drawer", "wizard"], &["value"]),
];

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '-' | '_'))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Se o atributo `attr` da tag `tag` é uma ligação. `attr` sem o `:` da frente.
pub fn is_binding(tag: &str, attr: &str) -> bool {
    let (tag, attr) = (norm(tag), norm(attr));
    TABELA.iter().any(|(tags, attrs)| {
        (tags.contains(&tag.as_str()) || tags.contains(&"*")) && attrs.contains(&attr.as_str())
    })
}

/// Os atributos de ligação de uma tag, sem normalizar (para ferramentas que
/// listam a tabela inteira).
pub fn table() -> impl Iterator<Item = (&'static [&'static str], &'static [&'static str])> {
    TABELA.iter().copied()
}
