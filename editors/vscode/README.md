# Glacier GSS — VS Code extension

Language support for **Glacier Stylesheets** (`.gss`), the CSS-like styling
format used by `glacier-ui`.

## Features

- **Syntax highlighting** tuned to the GSS grammar:
  - Class selectors (`.card`), id selectors (`#save`, higher specificity than
    a class), tag selectors (`button`, `card` — a builtin kind or a component
    name, lowest specificity), and pseudo-states (`:hover`, `:focus`, `:active`,
    `:pressed`, `:disabled`, `:invalid`) on any of them, the structural
    `:first-child`, `:last-child`, `:only-child`, `:nth-child(an+b)` and the
    functional `:not(…)`/`:is(…)` (glacier-ui 0.169) — unknown pseudo-states are
    flagged.
  - Compound selectors and combinators (glacier-ui 0.122): `row.p-2 > text.title`,
    `.list text`, `*`. The sibling combinators (`+`, `~`) and attribute
    selectors (`[x]`) are flagged, since the engine rejects them.
  - `:root { --token: value; }` design tokens and `var(--token, fallback)`
    references, and the `@theme { background: …; primary: …; }` palette.
  - Nesting with `&` (glacier-ui 0.169): `.btn { &:hover { … } &--primary { … } .icon { … } }`.
  - `@media (min-width: 600) and (max-width: 900) { … }` responsive blocks.
  - Known properties are highlighted as such (including `box_shadow`,
    `font_weight`, `font_style`, `line_height` and `display`); typos (unknown
    properties) are marked so they stand out. The engine skips them with a
    `file:line` warning and a suggestion, and with `app(gss_strict = true)` (or
    `glacier check --strict`) they are an error.
  - Hex colors (`#RRGGBB`/`#RRGGBBAA`), numbers with optional `px`, and value
    keywords (`fill`, `Center`, `none`, `true`, …).
  - `//` line comments and `/* … */` block comments.
- **Go to Definition and hover on `var(--token)`** — F12 (or hover) on a token jumps to its
  `--token: value` declaration, preferring the one in `:root` and the open file, then any `.gss`
  in the workspace.
- **Snippets**: `class`, `id`, `tag`, `hover`, `root`, `media`, `var`, `card`.
- **A CSS-style file icon** so `.gss` files read as stylesheets in the explorer.

## Install (local / development)

```bash
cd editors/vscode
npm install -g @vscode/vsce   # once
vsce package                  # produces glacier-gss-0.4.0.vsix
code --install-extension glacier-gss-0.4.0.vsix
```

Or press **F5** in VS Code with this folder open to launch an Extension
Development Host.

## Grammar reference

The highlighting mirrors `src/stylesheet.rs`. Property names are `snake_case`
only (the same names as the `.gv` attributes); custom properties and `@media`
features keep the CSS hyphen (`--accent`, `max-width`). The full grammar is in
`docs/LINGUAGEM.md` of glacier-ui.
