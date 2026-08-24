# Glacier View — VS Code extension

Language support for **Glacier View** (`.gv`), the XML markup of `glacier-ui`.

## Features

- **Syntax highlighting** for Glacier View markup:
  - Tags (primitives, builtins, and app components), attributes, `{interpolation}`
    with `{key|default}`, and hex colors.
  - Action attributes (`on_click`, `onChange`, `on_toggle`, …) are highlighted
    distinctly, and their handler names are scoped as functions.
  - **Embedded Lua** inside `<script>…</script>`.
  - **Embedded GSS** inside `<style>…</style>` (needs the *Glacier GSS* extension
    for full GSS colors).
- **Links** — every file a template names is underlined and Ctrl/Cmd+clickable,
  and answers **Go to Definition** (F12) too:

  | in the `.gv` | jumps to |
  | --- | --- |
  | `<script src="app.luau">` | the `.luau` file |
  | `on_click="salvar"` | `function salvar()` in that `.luau`, or in the inline `<script>` |
  | `onToggle="escolher_tipo:roadmap"` | `function escolher_tipo(…)` — the `nome:sufixo` convention links on the name half |
  | `on_click="clipboard:obra_pasta"` | where that context key is written (`ctx.obra_pasta = …`, `ctx.set("obra_pasta", …)`) — same for `open:`, `textarea_end:`, `textarea_top:` |
  | `on_click="window:close"`, `style:…` | the built-in actions table in the bundled reference |
  | `hidden="{sem_perguntas}"`, `{status}`, `{modelo\|padrão}` | where that context key is written |
  | `value="user_name"`, `items="tarefas"`, `checked="marcado"`, `cond=…` | idem: a binding attribute names a context key |
  | `{c.titulo}` under `for-each="…" var="c"` | the `var="c"` that declares the loop variable |
  | `on_click="salvar"` with no `<script>` | the `"salvar" =>` arm of the Rust `Component::update` |
  | `<link rel="stylesheet" href="app.gss">`, `<style href=…>` | the GSS sheet |
  | `<link rel="import" href=…>`, `<import from=…>`, `<Include src=…>` | the imported template |
  | `<link rel="theme">` / `<link rel="data">` | the JSON file |
  | `<Image src=…>`, `<Svg src=…>` | the asset |
  | `<PerfilCard/>` | the `.gv`/`.xml` declaring the component |
  | `navigateTo="perfil"` | the screen's template |
  | `<Button/>`, `<Badge/>`, … (native/builtin) | the bundled syntax reference |

  Context keys resolve **script first**, following the same chain the engine
  builds: the template's own `<script>` (inline or `src`), then every module it
  `require`s — transitively, resolved like `luau::resolve_module`, so a
  `ctx.erro = …` inside `lib/entrevista.luau` is found. Only a key written
  nowhere in that graph falls through to the Rust behind the template, and then
  to any other script in the workspace (one app, one context: a screen commonly
  reads what a sibling screen wrote). Paths resolve the way the engine resolves
  them: relative to the declaring file first, then to the workspace root. A reference that resolves to nothing — a
  missing file, a handler defined nowhere — is deliberately left unlinked, so a
  typo shows up as a missing underline.

Starts simple, meant to grow (hovers, unknown-handler diagnostics, completion).

## Install (local)

```bash
cd editors/vscode-gv
make install     # packages the .vsix and installs it into VS Code
```

Then open any `.gv` file. To try it on the existing `.xml` templates, either
rename them to `.gv` or right-click → *Change Language Mode* → *Glacier View*.
