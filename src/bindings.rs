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
//! ```xml
//! <progressbar :value="progresso" />      <!-- a ligação: a chave `progresso` -->
//! <text content="{progresso}%" />         <!-- o dado: o valor dela -->
//! ```
//!
//! Este módulo é a lista do que é ligação. Quem a consulta: o parser do `.gva`
//! (que avisa quando uma ligação chega sem o `:`), o dessugarador do `.gvb`
//! (que recusa) e as ferramentas de migração.

/// `(tags, atributos)`: em cada uma das `tags` (e seus apelidos), os `atributos`
/// são o nome de uma chave. Os nomes são comparados sem caixa e sem `-`/`_`.
type Linha = (&'static [&'static str], &'static [&'static str]);

/// Os primitivos (todo campo `*_var` do `parser.rs`) e os builtins cujas props são
/// repassadas como nome de chave.
const TABELA: &[Linha] = &[
    // — primitivos —
    (&["textinput", "input"], &["value", "valor"]),
    (&["textarea", "texteditor"], &["value", "valor"]),
    (&["maskedinput", "entradamascarada"], &["value", "valor"]),
    (
        &["checkbox", "check"],
        &["checked", "value", "valor", "marcado"],
    ),
    (
        &["toggle", "toggler"],
        &["checked", "value", "valor", "marcado"],
    ),
    (&["progressbar", "progress"], &["value", "valor"]),
    (&["timeedit", "timepicker"], &["value", "valor"]),
    (
        &["calendar", "calendario"],
        &[
            "value",
            "valor",
            "start",
            "inicio",
            "end",
            "fim",
            "final",
            "month",
            "mesvisivel",
        ],
    ),
    (
        &["pagination", "paginacao"],
        &["value", "valor", "page", "pagina"],
    ),
    (&["wizardnav"], &["value", "valor", "step", "passo"]),
    (
        &["colorwheel", "rodadecor"],
        &["value", "valor", "color", "cor"],
    ),
    (&["rating", "nota"], &["value", "valor"]),
    (
        &["popover", "painel"],
        &["value", "valor", "open", "aberto"],
    ),
    (
        &["autocomplete", "completer"],
        &["value", "valor", "items", "itens", "options", "opcoes"],
    ),
    (
        &["tableview", "tabela"],
        &[
            "items",
            "itens",
            "rows",
            "linhas",
            "columns",
            "colunas",
            "cols",
            "value",
            "valor",
            "selected",
            "selecionada",
            "sort",
            "ordem",
            "ordenacao",
            "widths",
            "larguras",
            "resize",
        ],
    ),
    (
        &["treeview", "arvore"],
        &[
            "items", "itens", "nodes", "nos", "value", "valor", "selected", "open", "aberto",
            "abertos", "expanded",
        ],
    ),
    (
        &["columnview", "colunas"],
        &[
            "items", "itens", "nodes", "nos", "value", "valor", "path", "caminho",
        ],
    ),
    (&["dial", "knob"], &["value", "valor"]),
    (&["gauge", "medidor"], &["value", "valor"]),
    (&["lcdnumber", "lcd"], &["value", "valor"]),
    (
        &["linechart", "graficolinha"],
        &["items", "itens", "data", "dados", "series"],
    ),
    (
        &["barchart", "graficobarras"],
        &["items", "itens", "data", "dados"],
    ),
    (&["piechart", "donut"], &["items", "itens", "data", "dados"]),
    (
        &["splitter", "divisor"],
        &["sizes", "tamanhos", "value", "valor"],
    ),
    (
        &["dock", "dockwidget"],
        &[
            "mode", "modo", "value", "valor", "size", "sizes", "tamanho", "tamanhos", "floatx",
            "floaty", "x", "y",
        ],
    ),
    (&["mdisubwindow", "janelainterna"], &["x", "y", "w", "h"]),
    (
        &["swipeview", "carrossel"],
        &["value", "valor", "index", "indice"],
    ),
    (
        &["rangeslider", "faixadupla"],
        &["start", "inicio", "from", "end", "fim", "to"],
    ),
    (&["tumbler", "roleta"], &["value", "valor"]),
    (
        &["rubberband", "laco"],
        &["selection", "selecao", "value", "valor", "selected"],
    ),
    (&["shortcutinput", "keysequenceedit"], &["value", "valor"]),
    (
        &["radio", "radiobutton"],
        &[
            "group",
            "grupo",
            "checked",
            "marcado",
            "selected",
            "selecionado",
        ],
    ),
    (&["slider", "deslizante"], &["value", "valor"]),
    (
        &["select", "dropdown", "comboedit", "editablecombo"],
        &[
            "value",
            "valor",
            "selected",
            "selecionado",
            "items",
            "itens",
            "options",
            "opcoes",
            "source",
            "origem",
        ],
    ),
    (&["menuitem", "itemmenu"], &["checked", "marcado"]),
    (
        &["menu", "cardapio", "contextmenu", "menucontexto"],
        &["items", "itens", "options", "opcoes"],
    ),
    // — qualquer tag: a repetição lê o NOME da chave com a lista —
    (&["*"], &["foreach", "each", "repeat"]),
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
