// Glacier View (.gv) language support.
//
// Two navigation providers, both driven by the same tag scanner (`iterTags`):
//
//   * DocumentLink — underlined, Ctrl/Cmd+clickable targets for every reference
//     a template makes to another file:
//       <script src="app.luau">          -> the .luau file
//       on_click="foo"                   -> `function foo()` inside that .luau
//                                           (or inside the inline <script>)
//       <link rel="stylesheet" href=…>   -> the .gss sheet
//       <style href="…">                 -> idem
//       <link rel="import" href=…>,
//       <import from=…>, <Include src=…> -> the imported template
//       <link rel="theme|data" href=…>   -> the JSON file
//       <Image src=…>, <Svg src=…>       -> the asset
//       <PerfilCard/>                    -> the .gv/.xml declaring the component
//       navigateTo="perfil"              -> the screen's template
//       {sem_perguntas}, value="nome"    -> where that context key is written
//       {c.titulo} under var="c"         -> the loop variable's declaration
//
//     Anything backed by code resolves **script first**: a handler with no Lua
//     behind it (the template's behaviour lives in a Rust `Component`) falls
//     through to the `"foo" =>` arm of that component's `update`, and a context
//     key with no `ctx.key = …` in Lua to its `ctx.set("key", …)` in Rust.
//
//   * Definition (F12) — the same targets, plus the bundled syntax reference for
//     native/builtin tags.
//
// Intentionally simple and dependency-free (plain JS, no build step). Meant to
// grow: hovers, diagnostics for unknown handlers, completion, etc.

const vscode = require("vscode");
const fs = require("fs");
const path = require("path");

// Canonical native tag -> all recognised spellings (from src/parser.rs). Used to
// tell a native/builtin widget apart from an app component, and to anchor the
// native tag into the bundled reference doc.
const NATIVE_TAGS = {
  Container: ["container"],
  Column: ["column"],
  Row: ["row"],
  Text: ["text", "span"],
  Button: ["button", "botao"],
  TextInput: ["textinput", "input", "entradatexto", "entrada_texto"],
  TextArea: ["textarea", "texteditor", "editor", "areatexto", "area_texto"],
  Image: ["image", "imagem"],
  Svg: ["svg", "icon", "icone"],
  Scrollable: ["scrollable", "scroll", "rolagem"],
  Checkbox: ["checkbox", "check"],
  Toggle: ["toggle", "toggler", "switch"],
  Rule: ["rule", "divider", "divisoria", "hr"],
  ProgressBar: ["progressbar", "progress", "barraprogresso", "barra_progresso"],
  Spinner: [
    "spinner", "busyindicator", "busy_indicator",
    "indicadorocupado", "indicador_ocupado", "carregando",
  ],
  Select: ["select", "dropdown", "picklist", "combobox", "combo", "seletor"],
  ComboEdit: [
    "comboedit", "editablecombo", "editableselect", "comboeditavel",
    "combo_editavel",
  ],
  Form: ["form", "formulario"],
  Include: ["include", "incluir"],
  Import: ["import", "importar"],
  ForEach: ["foreach", "for"],
  If: ["if", "se"],
  ElseIf: ["elseif", "else-if", "senaose", "senao-se"],
  Else: ["else", "senao"],
  Template: ["template", "gabarito"],
  Link: ["link"],
  Style: ["style", "stylesheet"],
  Script: ["script"],
  Badge: ["badge"],
  TimePicker: ["timepicker"],
  Screen: ["screen", "tela"],
  ComponentRoot: ["component", "componente"],
  Resources: ["resources", "recursos"],
  Props: ["props"],
  Prop: ["prop"],
};

// Lowercased spelling -> canonical name.
const NATIVE_LOOKUP = {};
for (const [canon, variants] of Object.entries(NATIVE_TAGS)) {
  NATIVE_LOOKUP[canon.toLowerCase()] = canon;
  for (const v of variants) NATIVE_LOOKUP[v.toLowerCase()] = canon;
}

// Canonical tag -> attributes whose value is a path to another file, in the
// order src/parser.rs reads them (`get_attr` takes the first one present).
const PATH_ATTRS = {
  Script: ["src", "from"],
  Link: ["href", "src", "from", "caminho"],
  Style: ["href", "src", "from", "caminho"],
  Import: ["from", "de", "src", "path", "caminho"],
  Include: ["src", "fonte"],
  Image: ["source", "src", "origem", "caminho"],
  Svg: ["source", "src", "origem", "caminho"],
};

// Every action attribute spelling from src/parser.rs. The value names the
// handler that runs it — a Lua function or a Rust `update` arm.
const ACTION_ATTRS = new Set(
  [
    "onClick", "on_click", "on-click", "aoClicar", "ao_clicar",
    "onPress", "on_press", "on-press", "aoPressionar", "ao_pressionar",
    "onDoubleClick", "on_double_click", "on-double-click", "aoClicarDuplo",
    "onChange", "on_change", "on-change", "aoMudar", "ao_mudar",
    "onToggle", "on_toggle", "on-toggle",
    "onSelect", "on_select", "on-select", "aoSelecionar", "ao_selecionar",
    "onSubmit", "on_submit", "on-submit", "aoSubmeter", "ao_submeter",
    "onReorder", "on_reorder", "on-reorder", "aoReordenar",
    "onOpen", "on_open", "on-open",
    "onMessage", "on_message", "on-message",
    "onError", "on_error", "on-error",
    "onClose", "on_close", "on-close",
  ].map((s) => s.toLowerCase())
);

// A component name paired with the template file it renders, as written on the
// Rust/Lua side. Nothing may sit between the two literals but plain expression
// text — no quote, no `;`, no brace — so the pair belongs to one statement.
const NAMED_TEMPLATE_RE = /"([A-Za-z_][\w-]*)"[^";{}]{0,160}"([^"]*\.(?:gv|xml))"/g;

// Attributes naming a screen to navigate to (src/parser.rs). The value is a
// registered component's name, so it resolves like a component tag.
const NAV_ATTRS = new Set(
  ["navigateTo", "navigate_to", "navigate-to", "irPara", "ir_para"].map((s) =>
    s.toLowerCase()
  )
);

// Attributes whose bare value is a context key rather than a literal (from the
// `get_attr` lists in src/parser.rs). Everything else reaches the context
// through `{interpolation}`, which is matched separately.
const BINDING_ATTRS = new Set(
  [
    "value", "valor", "value_var", "selected", "selecionado",
    "checked", "marcado",
    "items", "itens", "options", "opcoes", "source", "origem",
    "cond", "condition", "when", "quando", "condicao",
    "if", "se", "else-if", "elseIf", "else_if", "senaoSe", "senao_se",
    "for-each", "forEach", "foreach", "each", "repeat",
  ].map((a) => a.toLowerCase())
);

// `{chave}` / `{chave|default}` / `{chave.campo}` — the interpolation the eval
// engine resolves against the context. Only the key (first segment) is linked.
const INTERPOLATION_RE = /\{([A-Za-z_]\w*)((?:\.\w+)*)(\|[^{}]*)?\}/g;

// Actions the engine handles by itself in `GlacierUI::dispatch` — the word
// before the `:` is a namespace, never a function. `"key"` marks the ones whose
// suffix is a **context key** (so it links to where that key is written);
// `"engine"` marks the ones whose suffix is a command the engine interprets.
const BUILTIN_ACTIONS = {
  "clipboard:": "key",
  "open:": "key",
  "textarea_end:": "key",
  "textarea_top:": "key",
  "window:": "engine",
  "style:": "engine",
};

/** `{ prefix, kind, arg }` when `value` is an engine built-in, else null. */
function builtinAction(value) {
  for (const [prefix, kind] of Object.entries(BUILTIN_ACTIONS)) {
    if (value.startsWith(prefix)) {
      return { prefix, kind, arg: value.slice(prefix.length) };
    }
  }
  return null;
}

// The reference doc heading that documents the built-in actions above.
const BUILTIN_ACTIONS_HEADING = /^##+\s*A(?:ç|c)(?:ões|oes) built-in/m;

// `on*`/`ao*` attributes that name something other than a handler.
const NOT_ACTION_ATTRS = new Set(["oneof", "one_of", "one-of"]);

/**
 * Whether `name` is an attribute whose value is a handler name. Beyond the
 * spellings the parser knows, any `on…`/`ao…` attribute counts: a component
 * takes its actions as plain props (`<TimePicker on_pick="abrir_modal"/>`
 * forwards `{on_pick}` into a nested `on_click`), so the set is open-ended. A
 * false positive costs nothing — a name that resolves to no function is simply
 * not linked.
 */
function isActionAttr(name) {
  const lower = name.toLowerCase();
  if (ACTION_ATTRS.has(lower)) return true;
  if (NOT_ACTION_ATTRS.has(lower)) return false;
  return /^(?:on|ao)[_-]?[a-z][\w-]*$/.test(lower);
}

// What a workspace scan looks at, and what it never looks at.
const TEMPLATE_GLOB = "**/*.{gv,xml}";
const CODE_GLOB = "**/*.{rs,lua,luau}";
const EXCLUDE_GLOB = "**/{target,node_modules,.git,dist,out}/**";

function escapeRe(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Convert a char offset in `text` to a zero-based vscode.Position. */
function offsetToPosition(text, offset) {
  let line = 0;
  let last = 0;
  for (let i = 0; i < offset; i++) {
    if (text.charCodeAt(i) === 10) {
      line++;
      last = i + 1;
    }
  }
  return new vscode.Position(line, offset - last);
}

/**
 * A file URI carrying the position VS Code should reveal. The link opener reads
 * a `L<line>,<col>` fragment (1-based), so a DocumentLink can land on a precise
 * line the way a Location does.
 */
function uriAt(fsPath, position) {
  const uri = fsPath instanceof vscode.Uri ? fsPath : vscode.Uri.file(fsPath);
  if (!position) return uri;
  return uri.with({ fragment: `L${position.line + 1},${position.character + 1}` });
}

// ---------------------------------------------------------------------------
// File reading (cached by mtime — the providers run on every edit)
// ---------------------------------------------------------------------------

const fileCache = new Map(); // fsPath -> { mtimeMs, size, text }

/** Read a file as UTF-8, reusing the last read while its mtime/size hold. */
function readFileCached(fsPath) {
  let stat;
  try {
    stat = fs.statSync(fsPath);
  } catch (_) {
    fileCache.delete(fsPath);
    return null;
  }
  const hit = fileCache.get(fsPath);
  if (hit && hit.mtimeMs === stat.mtimeMs && hit.size === stat.size) return hit.text;
  let text;
  try {
    text = fs.readFileSync(fsPath, "utf8");
  } catch (_) {
    return null;
  }
  fileCache.set(fsPath, { mtimeMs: stat.mtimeMs, size: stat.size, text });
  return text;
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/**
 * Resolve a path written in a template to a file on disk, mirroring the engine:
 * a relative path is tried against the declaring file's own directory first
 * (`GlacierUI::resolve_import_href`, and `<script src>` in `luau::resolve_script`)
 * and then against the asset root — in practice the workspace folder, since the
 * engine resolves the fallback against the process CWD. Returns null when the
 * path is interpolated, remote, or simply doesn't exist.
 */
function resolveAssetPath(documentUri, raw) {
  if (!raw) return null;
  const clean = raw.trim();
  if (!clean || clean.includes("{")) return null; // `src="{icone}.svg"` — dynamic
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(clean)) return null; // http://, file://…
  if (path.isAbsolute(clean)) return fs.existsSync(clean) ? clean : null;

  const roots = [];
  if (documentUri && documentUri.scheme === "file") {
    roots.push(path.dirname(documentUri.fsPath));
  }
  for (const folder of vscode.workspace.workspaceFolders || []) {
    roots.push(folder.uri.fsPath);
  }
  for (const root of roots) {
    const candidate = path.resolve(root, clean);
    if (fs.existsSync(candidate)) return candidate;
  }
  return null;
}

// ---------------------------------------------------------------------------
// Markup scanning
// ---------------------------------------------------------------------------

/** Char ranges covered by `<!-- … -->`, so tags quoted in a comment are ignored. */
function commentRanges(text) {
  const out = [];
  const re = /<!--[\s\S]*?-->/g;
  let m;
  while ((m = re.exec(text)) !== null) out.push([m.index, m.index + m[0].length]);
  return out;
}

function inRanges(ranges, offset) {
  for (const [start, end] of ranges) {
    if (offset >= start && offset < end) return true;
  }
  return false;
}

/**
 * Char ranges of embedded Lua/GSS bodies. A `{…}` in there is a Lua table or a
 * GSS block, never a context interpolation.
 */
function embeddedRanges(text) {
  const out = [];
  const re = /<(script|style)\b[^>]*>([\s\S]*?)<\/\1\s*>/gi;
  let m;
  while ((m = re.exec(text)) !== null) {
    const bodyStart = m.index + m[0].indexOf(">") + 1;
    out.push([bodyStart, bodyStart + m[2].length]);
  }
  return out;
}

/**
 * Yield every element tag in `text` as
 * `{ name, nameStart, attrsText, attrsStart, attrsEnd, closing, selfClosing }`
 * (offsets are absolute char offsets into `text`).
 *
 * The body of `<script>`/`<style>` is skipped: Lua's `a < b` and GSS's `>` would
 * otherwise be scanned as markup.
 */
function* iterTags(text, comments) {
  const ranges = comments || commentRanges(text);
  const tagRe = /<(\/?)([A-Za-z_][\w.:-]*)((?:"[^"]*"|'[^']*'|[^>"'])*)>/g;
  let lowerText = null; // only built if an embedded body has to be skipped
  let m;
  let skipUntil = 0;
  while ((m = tagRe.exec(text)) !== null) {
    if (m.index < skipUntil || inRanges(ranges, m.index)) continue;
    const closing = m[1] === "/";
    const name = m[2];
    // The `/` of a self-closing tag lands at the end of the attribute chunk.
    const selfClosing = /\/\s*$/.test(m[3]);
    const attrsText = selfClosing ? m[3].replace(/\/\s*$/, "") : m[3];
    const attrsStart = m.index + 1 + m[1].length + name.length;

    yield {
      name,
      nameStart: m.index + 1 + m[1].length,
      attrsText,
      attrsStart,
      attrsEnd: attrsStart + attrsText.length,
      closing,
      selfClosing,
    };

    // Jump over an embedded Lua/GSS body.
    const lower = name.toLowerCase();
    if (!closing && !selfClosing && (lower === "script" || lower === "style")) {
      if (lowerText === null) lowerText = text.toLowerCase();
      const close = lowerText.indexOf(`</${lower}`, tagRe.lastIndex);
      if (close >= 0) skipUntil = close;
    }
  }
}

/**
 * Yield every `name="value"` of a tag's attribute chunk as
 * `{ name, value, start, end }` (offsets absolute, given `attrsStart`).
 */
function* iterAttrs(attrsText, attrsStart) {
  const re = /([A-Za-z_][\w.:-]*)\s*=\s*("[^"]*"|'[^']*')/g;
  let m;
  while ((m = re.exec(attrsText)) !== null) {
    const quoted = m[2];
    const start = attrsStart + m.index + m[0].length - quoted.length + 1;
    yield {
      name: m[1],
      value: quoted.slice(1, -1),
      start,
      end: start + quoted.length - 2,
    };
  }
}

/** The value of the first attribute of `tag` named in `names`, or undefined. */
function attrValue(tag, names) {
  const wanted = names.map((n) => n.toLowerCase());
  for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
    if (wanted.includes(attr.name.toLowerCase())) return attr.value;
  }
  return undefined;
}

// ---------------------------------------------------------------------------
// Action handlers
// ---------------------------------------------------------------------------

/** Find `function <name>(` inside `text`; return the char offset of `name` or -1. */
function findLuaFunctionOffset(text, name) {
  const re = new RegExp("function\\s+(" + escapeRe(name) + ")\\s*\\(", "g");
  const m = re.exec(text);
  if (m) return m.index + m[0].indexOf(m[1]);
  // Also `foo = function(...)` style.
  const re2 = new RegExp(
    "(?:^|[^\\w.])(" + escapeRe(name) + ")\\s*=\\s*function\\b", "g"
  );
  const m2 = re2.exec(text);
  if (m2) return m2.index + m2[0].indexOf(m2[1]);
  return -1;
}

/** Module names a Luau chunk pulls in with `require("…")`. */
function luauRequires(text) {
  const out = [];
  const re = /\brequire\s*\(\s*["']([^"']+)["']\s*\)/g;
  let m;
  while ((m = re.exec(text)) !== null) out.push(m[1]);
  return out;
}

/**
 * Resolve a `require("…")` to a file, mirroring `luau::resolve_module`: the
 * name takes `.` or `/` as the package separator (leading `./`/`../` kept as
 * navigation), and each root is tried as `rel.luau`, `rel/init.luau`, then the
 * same in `.lua`. Roots are the requiring file's directory, its `lib/`, and
 * whatever `GLACIER_LUAU_PATH` adds — the same list `luau::module_roots` builds.
 */
function resolveLuauModule(fromFsPath, modname) {
  let prefix = "";
  let rest = modname;
  for (;;) {
    if (rest.startsWith("../")) {
      prefix += "../";
      rest = rest.slice(3);
    } else if (rest.startsWith("./")) {
      rest = rest.slice(2);
    } else break;
  }
  const rel = prefix + rest.replace(/\./g, "/");

  const base = path.dirname(fromFsPath);
  const roots = [base, path.join(base, "lib")];
  for (const extra of (process.env.GLACIER_LUAU_PATH || "").split(":")) {
    if (extra) roots.push(extra);
  }
  for (const ext of ["luau", "lua"]) {
    for (const root of roots) {
      const file = path.resolve(root, `${rel}.${ext}`);
      if (fs.existsSync(file)) return file;
      const init = path.resolve(root, rel, `init.${ext}`);
      if (fs.existsSync(init)) return init;
    }
  }
  return null;
}

// A pathological require graph shouldn't stall the provider.
const MAX_SCRIPT_SOURCES = 60;

/**
 * All the Lua a template can reach: the inline `<script>` bodies, the file of
 * every `<script src>`, and — breadth-first — every module those `require`,
 * transitively. A helper library is where a big app actually writes the context
 * (`E.erro()` doing `ctx.erro = …` from `lib/entrevista.luau`), so stopping at
 * the template's own script would miss most of it.
 *
 * Returns `[{ uri, text, bodyStart, full, anchor }]`, `bodyStart` being the
 * offset the body sits at inside `full` (0 for a file) and `anchor` the path a
 * `require` inside it resolves against. Nearest first: the template's own
 * script wins over anything it pulls in.
 */
function scriptSources(documentUri, text) {
  const out = [];
  const anchorPath = documentUri.scheme === "file" ? documentUri.fsPath : "";

  const inlineRe =
    /<script(?![^>]*\bsrc\s*=)(?![^>]*\bfrom\s*=)[^>]*>([\s\S]*?)<\/script>/gi;
  let m;
  while ((m = inlineRe.exec(text)) !== null) {
    out.push({
      uri: documentUri,
      text: m[1],
      bodyStart: m.index + m[0].indexOf(m[1]),
      full: text,
      // An inline script has no file of its own; the engine anchors its
      // `require`s on the template (see `LuauComponent::from_file`).
      anchor: anchorPath,
    });
  }

  for (const tag of iterTags(text)) {
    if (tag.closing || tag.name.toLowerCase() !== "script") continue;
    const p = resolveAssetPath(documentUri, attrValue(tag, PATH_ATTRS.Script));
    if (!p) continue;
    const content = readFileCached(p);
    if (content === null) continue;
    out.push({ uri: vscode.Uri.file(p), text: content, bodyStart: 0, full: content, anchor: p });
  }

  const seen = new Set(out.map((s) => s.anchor).filter(Boolean));
  for (let i = 0; i < out.length && out.length < MAX_SCRIPT_SOURCES; i++) {
    const src = out[i];
    if (!src.anchor) continue;
    for (const modname of luauRequires(src.text)) {
      const p = resolveLuauModule(src.anchor, modname);
      if (!p || seen.has(p)) continue;
      seen.add(p);
      const content = readFileCached(p);
      if (content === null) continue;
      out.push({ uri: vscode.Uri.file(p), text: content, bodyStart: 0, full: content, anchor: p });
    }
  }
  return out;
}

/**
 * Where the context key `key` is written on the Rust side —
 * `ctx.set("key", …)`, `motor.define_data("key", …)` — or, for a file that
 * mixes both, `ctx.key = …`. Returns the char offset of the key, or -1. (The
 * Lua side goes through `contextWritesIn`, which extracts every key at once.)
 */
function findContextKeyOffset(text, key) {
  const k = escapeRe(key);
  const re = new RegExp(
    "ctx\\s*\\.\\s*(" + k + ")\\s*=(?!=)" +
      "|ctx\\s*\\[\\s*[\"'](" + k + ")[\"']\\s*\\]\\s*=(?!=)" +
      "|(?:set|define_data)\\s*\\(\\s*\"(" + k + ")\"",
    "g"
  );
  const m = re.exec(text);
  if (!m) return -1;
  const name = m[1] || m[2] || m[3];
  return m.index + m[0].lastIndexOf(name);
}

/**
 * Every context key the template's Lua writes, mapped to where it writes it.
 * Built in one pass over the whole script graph — a template reads dozens of
 * keys, and scanning each source once beats scanning it once per key. Nearest
 * source and first write win, so the template's own script beats a module it
 * pulls in.
 */
function contextWritesIn(sources) {
  const map = new Map();
  for (const src of sources) {
    for (const hit of collectContextWrites(src.text, 0)) {
      if (map.has(hit.name)) continue;
      map.set(
        hit.name,
        new vscode.Location(src.uri, offsetToPosition(src.full, src.bodyStart + hit.offset))
      );
    }
  }
  return map;
}

/** The context key `key`, looked up in the template's own Lua. */
function resolveContextKeyInLua(sources, key) {
  return contextWritesIn(sources).get(key) || null;
}

/**
 * The handler names an action value can resolve to, most specific first: the
 * value as written, then — for the engine's `nome:sufixo` convention — the part
 * before the first `:`. `onToggle="escolher_tipo:roadmap"` runs
 * `escolher_tipo("roadmap", value)` when no `escolher_tipo:roadmap` exists
 * (`LuauComponent::run_inner`), so the link belongs on `escolher_tipo`.
 */
function handlerCandidates(value) {
  const colon = value.indexOf(":");
  return colon > 0 ? [value, value.slice(0, colon)] : [value];
}

/**
 * Where the Lua handler `name` is defined, given the template's `sources` (see
 * `scriptSources`). Empty when the template has no such function — a Rust-side
 * handler, or a typo; see `workspaceIndex().handlers`.
 */
function resolveLuaHandler(sources, name) {
  const out = [];
  for (const src of sources) {
    const off = findLuaFunctionOffset(src.text, name);
    if (off < 0) continue;
    out.push(
      new vscode.Location(src.uri, offsetToPosition(src.full, src.bodyStart + off))
    );
  }
  return out;
}

/**
 * The `{ start, end }` char offsets of the `{ … }` body that begins at or after
 * `from`, matching braces while skipping string/char literals and comments.
 */
function braceBlock(text, from) {
  const open = text.indexOf("{", from);
  if (open < 0) return null;
  let depth = 0;
  for (let i = open; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      // Skip a string literal (a `{` inside `"{contador}"` must not count).
      i++;
      while (i < text.length && text[i] !== '"') {
        if (text[i] === "\\") i++;
        i++;
      }
      continue;
    }
    if (c === "'") {
      // A char literal is skipped; a lone tick is a lifetime, not a quote.
      const lit = /^'(?:\\.|[^\\'])'/.exec(text.slice(i, i + 6));
      if (lit) i += lit[0].length - 1;
      continue;
    }
    if (c === "/" && text[i + 1] === "/") {
      const nl = text.indexOf("\n", i);
      i = nl < 0 ? text.length : nl;
      continue;
    }
    if (c === "/" && text[i + 1] === "*") {
      const close = text.indexOf("*/", i + 2);
      i = close < 0 ? text.length : close + 1;
      continue;
    }
    if (c === "{") depth++;
    else if (c === "}" && --depth === 0) return { start: open, end: i + 1 };
  }
  return null;
}

/**
 * Action names dispatched by a Rust `Component::update` — the `"nome" =>` arms
 * of its `match action`. A template with no `<script>` gets its handlers from
 * Rust, and those are what an action attribute names there.
 *
 * Each hit carries the `Template::File("…")` of the `impl Component` block it
 * came from, so a handler can be tied to the template that actually declares it
 * instead of to any homonym elsewhere in the workspace (see `rustHandlerFor`).
 */
function collectRustHandlers(text) {
  const out = [];

  const scanArms = (body, bodyStart, template) => {
    const fnRe = /fn\s+update\s*\(/g;
    let f;
    while ((f = fnRe.exec(body)) !== null) {
      const block = braceBlock(body, f.index);
      if (!block) continue;
      // `"nome" =>` / `"a" | "b" =>` arms, the `if action == "nome"` form, and
      // the `nome:sufixo` convention as Rust reads it — `strip_prefix("nome:")`.
      const armRe =
        /"([A-Za-z_][\w-]*):?"\s*(?:\||=>)|==\s*"([A-Za-z_][\w-]*):?"|(?:starts_with|strip_prefix)\s*\(\s*"([A-Za-z_][\w-]*):"/g;
      const arms = body.slice(block.start, block.end);
      let a;
      while ((a = armRe.exec(arms)) !== null) {
        const name = a[1] || a[2] || a[3];
        const at = a.index + a[0].indexOf('"' + name) + 1;
        out.push({ name, offset: bodyStart + block.start + at, template });
      }
      fnRe.lastIndex = block.end;
    }
  };

  const implRe = /impl\b[^{;]*?\bComponent\s+for\s+[\w:<>]+/g;
  const covered = [];
  let m;
  while ((m = implRe.exec(text)) !== null) {
    const block = braceBlock(text, m.index);
    if (!block) continue;
    const body = text.slice(block.start, block.end);
    const tpl = /Template::File\s*\(\s*"([^"]+)"/.exec(body);
    scanArms(body, block.start, tpl ? tpl[1] : null);
    covered.push([block.start, block.end]);
    implRe.lastIndex = block.end;
  }

  // `fn update` outside an `impl Component` block (a helper, a macro body):
  // still indexed, just without a template to tie it to.
  const strayRe = /fn\s+update\s*\(/g;
  while ((m = strayRe.exec(text)) !== null) {
    if (covered.some(([s, e]) => m.index >= s && m.index < e)) continue;
    const block = braceBlock(text, m.index);
    if (!block) continue;
    scanArms(text.slice(m.index, block.end), m.index, null);
    strayRe.lastIndex = block.end;
  }

  return out;
}

// ---------------------------------------------------------------------------
// Component resolution
// ---------------------------------------------------------------------------

/** `PerfilCard` -> the keys it may be indexed under (`perfilcard`, `perfil_card`). */
function componentKeys(name) {
  const lower = name.toLowerCase();
  const snake = name.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
  return [...new Set([lower, snake, lower.replace(/[_-]/g, "")])];
}

/**
 * Components declared by the document itself: `<import name="X" from="…">` and
 * `<link rel="import" href="…" as="X">`. Returns a Map of lowercased name ->
 * fsPath (the name defaults to the file stem, like the engine's `file_stem`
 * fallback in `process_links`).
 */
function localImports(documentUri, text) {
  const map = new Map();
  for (const tag of iterTags(text)) {
    if (tag.closing) continue;
    const canon = NATIVE_LOOKUP[tag.name.toLowerCase()];
    let href;
    let name;
    if (canon === "Import") {
      href = attrValue(tag, PATH_ATTRS.Import);
      name = attrValue(tag, ["name", "nome", "as"]);
    } else if (canon === "Link") {
      const rel = (attrValue(tag, ["rel", "tipo"]) || "stylesheet").toLowerCase();
      if (rel !== "import" && rel !== "component") continue;
      href = attrValue(tag, PATH_ATTRS.Link);
      name = attrValue(tag, ["as", "name", "nome"]);
    } else {
      continue;
    }
    const p = resolveAssetPath(documentUri, href);
    if (!p) continue;
    const key = (name || path.basename(p).replace(/\.[^.]+$/, "")).toLowerCase();
    if (key) map.set(key, p);
  }
  return map;
}


// ---------------------------------------------------------------------------
// Props declaradas (<props> no cabeçalho de um <component>)
// ---------------------------------------------------------------------------

const PROPS_BLOCK_RE = /<props\s*>([\s\S]*?)<\/props\s*>/i;

/**
 * O contrato de props de um `.gv`, ou `null` quando o arquivo não declara um.
 *
 * `null` e `[]` querem dizer coisas diferentes, e a distinção é a mesma do
 * motor: sem `<props>` nada é checado; um `<props>` vazio declara que o
 * componente não aceita prop nenhuma.
 */
function declaredProps(fsPath) {
  const text = readFileCached(fsPath);
  if (!text) return null;
  const block = PROPS_BLOCK_RE.exec(text);
  if (!block) return null;
  const props = [];
  const re = /<prop\b([^>]*)>/gi;
  let m;
  while ((m = re.exec(block[1])) !== null) {
    const attrs = {};
    for (const attr of iterAttrs(m[1], 0)) attrs[attr.name.toLowerCase()] = attr.value;
    const name = attrs.name || attrs.nome;
    if (!name) continue;
    const padrao = attrs.default ?? attrs.padrao ?? attrs["padrão"];
    props.push({ name, default: padrao });
  }
  return props;
}

/** As props declaradas pelo componente que a tag `name` referencia. */
async function propsForTag(document, tagName) {
  if (NATIVE_LOOKUP[tagName.toLowerCase()]) return null;
  const index = await workspaceIndex();
  const imports = localImports(document.uri, document.getText());
  const fsPath = lookupComponent(index, imports, tagName, document.uri);
  return fsPath ? declaredProps(fsPath) : null;
}

// ---------------------------------------------------------------------------
// Workspace index (component names + Rust action handlers)
// ---------------------------------------------------------------------------

let indexPromise = null;

/** The workspace index, built on first use and kept until a file changes. */
function workspaceIndex() {
  if (!indexPromise) indexPromise = buildWorkspaceIndex();
  return indexPromise;
}

function invalidateWorkspaceIndex() {
  indexPromise = null;
}

function indexAdd(map, key, value, front) {
  if (!key) return;
  const list = map.get(key) || [];
  if (!list.some((v) => v.fsPath === value.fsPath && v.offset === value.offset)) {
    if (front) list.unshift(value);
    else list.push(value);
  }
  map.set(key, list);
}

/**
 * Index what only a workspace-wide scan can answer:
 *
 * `components` — name key -> declaring file, from
 *   1. template file names (`perfil_card.gv` answers both `perfil_card` and
 *      `PerfilCard`);
 *   2. `<import name="X" from="…">` / `<link rel="import" as="X">` in any
 *      template (a component registered by one screen is visible to all);
 *   3. a name paired with a template path on the Rust/Lua side, in whatever
 *      shape the registration takes (`register_component`, a struct literal,
 *      a constructor call).
 *
 * `handlers` — action name -> the `"nome" =>` arm of a Rust `update`.
 */
/** Every `ctx.chave = …` / `ctx["chave"] = …` written in a Luau chunk. */
function collectContextWrites(text, baseOffset) {
  const out = [];
  const re =
    /ctx\s*\.\s*([A-Za-z_]\w*)\s*=(?!=)|ctx\s*\[\s*["']([A-Za-z_]\w*)["']\s*\]\s*=(?!=)/g;
  let m;
  while ((m = re.exec(text)) !== null) {
    const name = m[1] || m[2];
    out.push({ name, offset: baseOffset + m.index + m[0].lastIndexOf(name) });
  }
  return out;
}

async function buildWorkspaceIndex() {
  const components = new Map();
  const handlers = new Map();
  // Context keys written anywhere in the workspace's Lua. The engine keeps a
  // single context for the whole app, so a key a screen reads is often written
  // by a sibling screen's script.
  const luaKeys = new Map();
  // template fsPath -> the .rs files whose `impl Component` renders it, and
  // directory -> the .rs files sitting in it (the fallback when the template
  // path is built at runtime instead of written as a literal).
  const rustByTemplate = new Map();
  const rustByDir = new Map();

  const templates = await vscode.workspace.findFiles(TEMPLATE_GLOB, EXCLUDE_GLOB, 2000);
  const byBasename = new Map(); // "inicio.gv" -> its full paths
  for (const uri of templates) {
    const file = path.basename(uri.fsPath);
    const list = byBasename.get(file) || [];
    list.push(uri.fsPath);
    byBasename.set(file, list);
    for (const key of componentKeys(file.replace(/\.[^.]+$/, ""))) {
      indexAdd(components, key, { fsPath: uri.fsPath });
    }
  }
  const inlineScriptRe = /<script(?![^>]*\bsrc\s*=)[^>]*>([\s\S]*?)<\/script>/gi;
  for (const uri of templates) {
    const text = readFileCached(uri.fsPath);
    if (text === null) continue;
    if (/<\s*(import|importar|link)\b/i.test(text)) {
      for (const [key, p] of localImports(uri, text)) {
        indexAdd(components, key, { fsPath: p }, true);
      }
    }
    inlineScriptRe.lastIndex = 0;
    let sc;
    while ((sc = inlineScriptRe.exec(text)) !== null) {
      const bodyStart = sc.index + sc[0].indexOf(sc[1]);
      for (const hit of collectContextWrites(sc[1], bodyStart)) {
        indexAdd(luaKeys, hit.name, {
          fsPath: uri.fsPath,
          offset: hit.offset,
          position: offsetToPosition(text, hit.offset),
        });
      }
    }
  }

  const code = await vscode.workspace.findFiles(CODE_GLOB, EXCLUDE_GLOB, 500);
  for (const uri of code) {
    const text = readFileCached(uri.fsPath);
    if (text === null) continue;

    // A component name paired with the template it renders, in whatever shape
    // the registration takes: `register_component("Nome", "tela.gv")`,
    // `Tela { nome: "perfil", template: "nav_perfil.gv" }`, `new("x", "x.gv")`.
    // The two literals must sit in the same expression — the gap may not cross
    // a quote, a `;`, or a block boundary, which is what keeps an unrelated
    // string from being paired with a path further down the file.
    NAMED_TEMPLATE_RE.lastIndex = 0;
    let m;
    while ((m = NAMED_TEMPLATE_RE.exec(text)) !== null) {
      const p = resolveAssetPath(uri, m[2]);
      if (!p) continue;
      for (const key of componentKeys(m[1])) {
        indexAdd(components, key, { fsPath: p }, true);
      }
    }

    if (!uri.fsPath.endsWith(".rs")) {
      for (const hit of collectContextWrites(text, 0)) {
        indexAdd(luaKeys, hit.name, {
          fsPath: uri.fsPath,
          offset: hit.offset,
          position: offsetToPosition(text, hit.offset),
        });
      }
      continue;
    }

    // Any template path written in the file ties it to that template —
    // `Template::File("ui/inicio.gv")`, `register_component("inicio", …)`, a
    // struct literal, anything. A file that names the template is the file
    // that stands behind it.
    const tplRe = /"([^"]*\.(?:gv|xml))"/g;
    let t;
    while ((t = tplRe.exec(text)) !== null) {
      const resolved = resolveAssetPath(uri, t[1]);
      // The literal is often just a file name, with the directory computed at
      // runtime (`ui_dir().join(file)`), so fall back to matching by name.
      const targets = resolved
        ? [resolved]
        : byBasename.get(path.basename(t[1])) || [];
      for (const target of targets) {
        indexAdd(rustByTemplate, target, { fsPath: uri.fsPath });
      }
    }
    indexAdd(rustByDir, path.dirname(uri.fsPath), { fsPath: uri.fsPath });

    if (text.includes("fn update")) {
      for (const hit of collectRustHandlers(text)) {
        indexAdd(handlers, hit.name, {
          fsPath: uri.fsPath,
          offset: hit.offset,
          position: offsetToPosition(text, hit.offset),
          template: hit.template,
        });
      }
    }
  }
  return { components, handlers, luaKeys, rustByTemplate, rustByDir };
}

/** The `.rs` files that back `documentUri` — its renderer, or its neighbours. */
function rustFilesFor(index, documentUri) {
  const tied = index.rustByTemplate.get(documentUri.fsPath);
  if (tied && tied.length) return tied.map((v) => v.fsPath);
  const near = index.rustByDir.get(path.dirname(documentUri.fsPath)) || [];
  return near.map((v) => v.fsPath);
}

/**
 * The context key `key` as written by some **other** script in the workspace —
 * the engine keeps one context for the whole app, so a screen commonly reads a
 * key a sibling screen writes. Among candidates, the one sharing the most path
 * with the document wins.
 */
function resolveContextKeyInWorkspaceLua(index, documentUri, key) {
  const hits = index.luaKeys.get(key) || [];
  if (!hits.length) return null;
  const dir = documentUri.scheme === "file" ? path.dirname(documentUri.fsPath) : "";
  let best = hits[0];
  let bestDepth = -1;
  for (const hit of hits) {
    const depth = dir ? sharedPathDepth(dir, path.dirname(hit.fsPath)) : 0;
    if (depth > bestDepth) {
      best = hit;
      bestDepth = depth;
    }
  }
  return new vscode.Location(vscode.Uri.file(best.fsPath), best.position);
}

/** The context key `key`, looked up in the Rust that backs this template. */
function resolveContextKeyInRust(index, documentUri, key) {
  for (const fsPath of rustFilesFor(index, documentUri)) {
    const text = readFileCached(fsPath);
    if (text === null) continue;
    const off = findContextKeyOffset(text, key);
    if (off < 0) continue;
    return new vscode.Location(vscode.Uri.file(fsPath), offsetToPosition(text, off));
  }
  return null;
}

/**
 * The Rust `update` arm that handles `name` **for this template**. Handler names
 * repeat across a workspace (every `contador` has an `incrementar`), so a hit
 * only counts when the `impl Component` block it came from declares this very
 * template, or at least lives beside it. An ambiguous name gets no link — a
 * wrong jump is worse than none.
 */
function rustHandlerFor(index, documentUri, name) {
  const hits = index.handlers.get(name) || [];
  if (!hits.length) return null;
  const target = documentUri.fsPath;
  const dir = path.dirname(target);

  const declares = hits.find((h) => {
    if (!h.template) return false;
    const p = resolveAssetPath(vscode.Uri.file(h.fsPath), h.template);
    return p === target;
  });
  if (declares) return declares;

  // No `Template::File` to go by: accept a neighbour, which is how the examples
  // (and most single-screen apps) are laid out.
  const untied = hits.filter((h) => !h.template);
  return untied.find((h) => path.dirname(h.fsPath) === dir) || null;
}

/** How many leading path segments `a` and `b` share. */
function sharedPathDepth(a, b) {
  const x = a.split(path.sep);
  const y = b.split(path.sep);
  let n = 0;
  while (n < x.length && n < y.length && x[n] === y[n]) n++;
  return n;
}

/**
 * The declaring file of component `tag`, or null. The document's own `<import>`
 * wins; then, among the workspace candidates, the nearest one — a name like
 * `perfil` can be both `examples/perfil/perfil.gv` and the screen registered as
 * `perfil` next door, and the one sharing the most path with the document is
 * the one that screen means.
 */
function lookupComponent(index, imports, tag, documentUri) {
  const direct = imports.get(tag.toLowerCase());
  if (direct) return direct;

  const candidates = [];
  for (const key of componentKeys(tag)) {
    for (const hit of index.components.get(key) || []) {
      if (!candidates.includes(hit.fsPath)) candidates.push(hit.fsPath);
    }
  }
  if (!candidates.length) return null;
  if (candidates.length === 1 || !documentUri || documentUri.scheme !== "file") {
    return candidates[0];
  }

  const dir = path.dirname(documentUri.fsPath);
  let best = candidates[0];
  let bestDepth = -1;
  for (const candidate of candidates) {
    const depth = sharedPathDepth(dir, path.dirname(candidate));
    if (depth > bestDepth) {
      best = candidate;
      bestDepth = depth;
    }
  }
  return best;
}

// Set in `activate`: the providers need the bundled reference doc.
let extensionPath = "";

/** The bundled reference doc position matching `pattern`, or its first line. */
function referenceLocation(pattern) {
  const ref = path.join(extensionPath, "references", "glacier-view.md");
  const text = readFileCached(ref);
  if (text === null) return null;
  const m = pattern.exec(text);
  return new vscode.Location(
    vscode.Uri.file(ref),
    m ? offsetToPosition(text, m.index) : new vscode.Position(0, 0)
  );
}

/** Jump a native tag to its heading in the bundled reference doc. */
function resolveNative(canonical) {
  const ref = path.join(extensionPath, "references", "glacier-view.md");
  const text = readFileCached(ref);
  if (text === null) return [];
  // The heading that names the tag, or — for a spelling documented as an alias,
  // like `<Else>` under the `<If>` heading — the first heading mentioning it.
  const own = new RegExp("^#+\\s*`?<?" + escapeRe(canonical) + "\\b", "mi");
  const alias = new RegExp("^#+.*<" + escapeRe(canonical) + ">", "mi");
  const m = own.exec(text) || alias.exec(text);
  const pos = m ? offsetToPosition(text, m.index) : new vscode.Position(0, 0);
  return [new vscode.Location(vscode.Uri.file(ref), pos)];
}

// ---------------------------------------------------------------------------
// Providers
// ---------------------------------------------------------------------------

/** Tooltip for a link that opens a file, by what kind of file it is. */
function pathTooltip(fsPath) {
  switch (path.extname(fsPath).toLowerCase()) {
    case ".luau":
    case ".lua":
      return "Open Lua script";
    case ".gss":
    case ".css":
      return "Open GSS stylesheet";
    case ".gv":
    case ".xml":
      return "Open template";
    case ".json":
      return "Open JSON";
    default:
      return "Open " + path.basename(fsPath);
  }
}

/**
 * The loop variables a template declares: `<ForEach var="c">`, `<template
 * for-each="…" var="c">`. Returns `[{ name, start, end }]` in document order —
 * a `{c.titulo}` refers to one of these, not to a context key.
 */
function loopVars(text, comments) {
  const out = [];
  for (const tag of iterTags(text, comments)) {
    if (tag.closing) continue;
    for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
      const lower = attr.name.toLowerCase();
      if (lower !== "var" && lower !== "variavel") continue;
      const name = attr.value.trim();
      if (!/^[A-Za-z_]\w*$/.test(name)) continue;
      const start = attr.start + attr.value.indexOf(name);
      out.push({ name, start, end: start + name.length });
    }
  }
  return out;
}

/** The declaration of loop variable `name` closest above `offset`, or null. */
function loopVarFor(vars, name, offset) {
  let best = null;
  for (const v of vars) {
    if (v.name !== name || v.start > offset) continue;
    if (!best || v.start > best.start) best = v;
  }
  return best;
}

/**
 * Every file reference the document makes, as an underlined Ctrl/Cmd+clickable
 * link: `<script src>`, action handlers, `<link>`/`<style>` sheets, imports and
 * includes, assets, and component tags. References that resolve to nothing (a
 * missing file, a handler that exists nowhere) are dropped rather than linked
 * to a wrong target — the absent underline is itself the hint.
 */
async function provideDocumentLinks(document) {
  const text = document.getText();
  const comments = commentRanges(text);
  const imports = localImports(document.uri, text);
  const scripts = scriptSources(document.uri, text);
  // Context keys the template's own Lua writes — one pass, reused below.
  const luaWrites = contextWritesIn(scripts);
  const links = [];
  // Resolved against the workspace index, which is only built if needed.
  const pendingComponents = [];
  const pendingHandlers = [];
  const pendingKeys = [];

  const range = (start, end) =>
    new vscode.Range(document.positionAt(start), document.positionAt(end));

  for (const tag of iterTags(text, comments)) {
    if (tag.closing) continue;
    const canon = NATIVE_LOOKUP[tag.name.toLowerCase()];

    // 1. Path attributes: <script src>, <link href>, <style href>, <import
    //    from>, <Include src>, <Image src>, <Svg src>.
    const pathAttrs = canon ? PATH_ATTRS[canon] : undefined;
    if (pathAttrs) {
      const wanted = pathAttrs.map((n) => n.toLowerCase());
      for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
        if (!wanted.includes(attr.name.toLowerCase())) continue;
        const p = resolveAssetPath(document.uri, attr.value);
        if (p) {
          const link = new vscode.DocumentLink(range(attr.start, attr.end), uriAt(p));
          link.tooltip = pathTooltip(p);
          links.push(link);
        }
        break; // `get_attr` takes the first spelling present
      }
    }

    // 2. Action attributes. Three shapes, in order:
    //    - an engine built-in (`clipboard:chave`, `window:close`): the suffix
    //      is a context key, or the whole thing is a command the engine runs;
    //    - `foo:arg`: the engine calls `foo(arg, value)`, so the link goes on
    //      the `foo` half;
    //    - `foo`: a plain handler.
    for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
      if (!isActionAttr(attr.name)) continue;
      const value = attr.value.trim();
      if (!value || value.includes("{")) continue;
      const start = attr.start + attr.value.indexOf(value);

      const builtin = builtinAction(value);
      if (builtin) {
        // A command the engine interprets: nothing in the workspace defines it,
        // so the link explains it instead.
        if (builtin.kind !== "key" || !builtin.arg) {
          const doc = referenceLocation(BUILTIN_ACTIONS_HEADING);
          if (doc) {
            const link = new vscode.DocumentLink(
              range(start, start + value.length),
              uriAt(doc.uri, doc.range.start)
            );
            link.tooltip = "Built-in action — open the reference";
            links.push(link);
          }
          continue;
        }
        // `clipboard:obra_pasta` — the suffix names a context key.
        const keyStart = start + builtin.prefix.length;
        const link = new vscode.DocumentLink(
          range(keyStart, keyStart + builtin.arg.length)
        );
        const lua = luaWrites.get(builtin.arg);
        if (lua) {
          link.target = uriAt(lua.uri, lua.range.start);
          link.tooltip = `Go to context key "${builtin.arg}"`;
        } else {
          pendingKeys.push({ link, key: builtin.arg, builtin: { value, start, range } });
        }
        links.push(link);
        continue;
      }

      let link = null;
      for (const name of handlerCandidates(value)) {
        const [lua] = resolveLuaHandler(scripts, name);
        if (!lua) continue;
        link = new vscode.DocumentLink(range(start, start + name.length));
        link.target = uriAt(lua.uri, lua.range.start);
        link.tooltip = `Go to function ${name}()`;
        break;
      }
      if (!link) {
        // Nothing in Lua: let the workspace index try the Rust side, still
        // narrowing to the name half if the value carries a suffix.
        link = new vscode.DocumentLink(range(start, start + value.length));
        pendingHandlers.push({ link, value, start, range });
      }
      links.push(link);
    }

    // 3. Binding attributes: value="user_name", items="tarefas", … name a
    //    context key straight, without `{}`.
    for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
      if (!BINDING_ATTRS.has(attr.name.toLowerCase())) continue;
      const key = attr.value.trim();
      if (!key || !/^[A-Za-z_]\w*$/.test(key)) continue; // literal or `{…}`
      const start = attr.start + attr.value.indexOf(key);
      const link = new vscode.DocumentLink(range(start, start + key.length));
      link.tooltip = `Go to context key "${key}"`;
      const lua = luaWrites.get(key);
      if (lua) link.target = uriAt(lua.uri, lua.range.start);
      else pendingKeys.push({ link, key });
      links.push(link);
    }

    // 4. Navigation attributes: navigateTo="perfil" -> the screen's template.
    for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
      if (!NAV_ATTRS.has(attr.name.toLowerCase())) continue;
      const name = attr.value.trim();
      if (!name || name.includes("{")) continue;
      const link = new vscode.DocumentLink(range(attr.start, attr.end));
      link.tooltip = "Open screen";
      const direct = imports.get(name.toLowerCase());
      if (direct) link.target = uriAt(direct);
      else pendingComponents.push({ link, name });
      links.push(link);
    }

    // 5. Component tag: <PerfilCard/> -> the file declaring it.
    if (!canon) {
      const link = new vscode.DocumentLink(
        range(tag.nameStart, tag.nameStart + tag.name.length)
      );
      link.tooltip = "Open component";
      const direct = imports.get(tag.name.toLowerCase());
      if (direct) link.target = uriAt(direct);
      else pendingComponents.push({ link, name: tag.name });
      links.push(link);
    }
  }

  // 6. `{chave}` / `{chave|default}` anywhere in the markup — the same context
  //    key, reached through interpolation. Lua and GSS bodies are skipped:
  //    their braces are code, not interpolation.
  const skip = comments.concat(embeddedRanges(text));
  const vars = loopVars(text, comments);
  INTERPOLATION_RE.lastIndex = 0;
  let interp;
  while ((interp = INTERPOLATION_RE.exec(text)) !== null) {
    if (inRanges(skip, interp.index)) continue;
    const key = interp[1];
    const start = interp.index + 1;
    const link = new vscode.DocumentLink(range(start, start + key.length));

    // A loop variable shadows the context inside its subtree, so it answers
    // first — and it answers within the template, not from a script.
    const loop = loopVarFor(vars, key, interp.index);
    if (loop) {
      link.tooltip = `Go to loop variable "${key}"`;
      link.target = uriAt(document.uri, document.positionAt(loop.start));
      links.push(link);
      continue;
    }

    link.tooltip = `Go to context key "${key}"`;
    const lua = luaWrites.get(key);
    if (lua) link.target = uriAt(lua.uri, lua.range.start);
    else pendingKeys.push({ link, key });
    links.push(link);
  }

  if (pendingComponents.length || pendingHandlers.length || pendingKeys.length) {
    const index = await workspaceIndex();
    for (const { link, name } of pendingComponents) {
      const p = lookupComponent(index, imports, name, document.uri);
      if (p) link.target = uriAt(p);
    }
    const rustKey = new Map(); // key -> Location | null, resolved once
    for (const { link, key, builtin } of pendingKeys) {
      if (!rustKey.has(key)) {
        rustKey.set(
          key,
          resolveContextKeyInRust(index, document.uri, key) ||
            resolveContextKeyInWorkspaceLua(index, document.uri, key)
        );
      }
      const rust = rustKey.get(key);
      if (rust) {
        link.target = uriAt(rust.uri, rust.range.start);
        link.tooltip = `Go to context key "${key}"`;
        continue;
      }
      // Written nowhere we can see. A plain `{chave}` just stays unlinked; a
      // built-in action still has something to say, so it points at the doc.
      if (!builtin) continue;
      const doc = referenceLocation(BUILTIN_ACTIONS_HEADING);
      if (!doc) continue;
      link.range = builtin.range(builtin.start, builtin.start + builtin.value.length);
      link.target = uriAt(doc.uri, doc.range.start);
      link.tooltip = "Built-in action — open the reference";
    }

    for (const pending of pendingHandlers) {
      for (const name of handlerCandidates(pending.value)) {
        const hit = rustHandlerFor(index, document.uri, name);
        if (!hit) continue;
        pending.link.range = pending.range(pending.start, pending.start + name.length);
        pending.link.target = uriAt(hit.fsPath, hit.position);
        pending.link.tooltip = `Go to handler "${name}" (Rust)`;
        break;
      }
    }
  }

  return links.filter((link) => link.target);
}

/**
 * Classify what the cursor is on:
 *   { kind: "action", name }         — an action attribute's value
 *   { kind: "key", name }            — a context key: `{chave}` or a binding
 *   { kind: "path", value }          — a path attribute's value
 *   { kind: "tag", name, canonical } — a tag name
 * or null.
 */
function classify(document, position) {
  const text = document.getText();
  const offset = document.offsetAt(position);

  // `{chave}` wins wherever it sits — attribute value or loose text.
  const skip = commentRanges(text).concat(embeddedRanges(text));
  INTERPOLATION_RE.lastIndex = 0;
  let interp;
  while ((interp = INTERPOLATION_RE.exec(text)) !== null) {
    if (inRanges(skip, interp.index)) continue;
    if (offset > interp.index && offset < interp.index + interp[0].length) {
      return { kind: "key", name: interp[1], offset: interp.index };
    }
  }

  for (const tag of iterTags(text)) {
    if (offset < tag.nameStart) break; // tags come in document order
    const canon = NATIVE_LOOKUP[tag.name.toLowerCase()];

    if (offset >= tag.nameStart && offset <= tag.nameStart + tag.name.length) {
      return { kind: "tag", name: tag.name, canonical: canon };
    }
    if (offset < tag.attrsStart || offset > tag.attrsEnd) continue;

    const pathAttrs = (canon ? PATH_ATTRS[canon] : undefined) || [];
    const wanted = pathAttrs.map((n) => n.toLowerCase());
    for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
      if (offset < attr.start || offset > attr.end) continue;
      const lower = attr.name.toLowerCase();
      if (isActionAttr(lower)) {
        const name = attr.value.trim();
        return name ? { kind: "action", name } : null;
      }
      if (BINDING_ATTRS.has(lower)) {
        const key = attr.value.trim();
        return /^[A-Za-z_]\w*$/.test(key) ? { kind: "key", name: key } : null;
      }
      if (NAV_ATTRS.has(lower)) {
        const name = attr.value.trim();
        return name ? { kind: "tag", name, canonical: undefined } : null;
      }
      if (wanted.includes(lower)) return { kind: "path", value: attr.value };
      return null;
    }
  }
  return null;
}

/** Definition(s) for what the cursor is on — the F12 side of the same links. */
async function resolveDefinition(document, position) {
  const hit = classify(document, position);
  if (!hit) return undefined;
  const text = document.getText();

  if (hit.kind === "action") {
    const sources = scriptSources(document.uri, text);
    const builtin = builtinAction(hit.name);
    if (builtin) {
      if (builtin.kind === "key" && builtin.arg) {
        const lua = resolveContextKeyInLua(sources, builtin.arg);
        if (lua) return [lua];
        const index = await workspaceIndex();
        const rust = resolveContextKeyInRust(index, document.uri, builtin.arg);
        if (rust) return [rust];
      }
      const doc = referenceLocation(BUILTIN_ACTIONS_HEADING);
      return doc ? [doc] : undefined;
    }
    for (const name of handlerCandidates(hit.name)) {
      const lua = resolveLuaHandler(sources, name);
      if (lua.length) return lua;
    }
    const index = await workspaceIndex();
    for (const name of handlerCandidates(hit.name)) {
      const rust = rustHandlerFor(index, document.uri, name);
      if (rust) return [new vscode.Location(vscode.Uri.file(rust.fsPath), rust.position)];
    }
    return undefined;
  }

  if (hit.kind === "key") {
    if (hit.offset !== undefined) {
      const loop = loopVarFor(loopVars(text), hit.name, hit.offset);
      if (loop) {
        return [new vscode.Location(document.uri, document.positionAt(loop.start))];
      }
    }
    const lua = resolveContextKeyInLua(scriptSources(document.uri, text), hit.name);
    if (lua) return [lua];
    const index = await workspaceIndex();
    const found =
      resolveContextKeyInRust(index, document.uri, hit.name) ||
      resolveContextKeyInWorkspaceLua(index, document.uri, hit.name);
    return found ? [found] : undefined;
  }

  if (hit.kind === "path") {
    const p = resolveAssetPath(document.uri, hit.value);
    return p ? [new vscode.Location(vscode.Uri.file(p), new vscode.Position(0, 0))] : undefined;
  }

  // tag
  if (hit.canonical) return resolveNative(hit.canonical);
  const index = await workspaceIndex();
  const p = lookupComponent(index, localImports(document.uri, text), hit.name, document.uri);
  return p ? [new vscode.Location(vscode.Uri.file(p), new vscode.Position(0, 0))] : undefined;
}


// ---------------------------------------------------------------------------
// Completar e diagnosticar props
// ---------------------------------------------------------------------------

/** A tag de abertura que contém `offset`, se houver. */
function openTagAt(text, offset) {
  for (const tag of iterTags(text)) {
    if (tag.closing) continue;
    if (offset > tag.attrsStart && offset <= tag.attrsEnd + 1) return tag;
  }
  return null;
}

const provideCompletion = {
  async provideCompletionItems(document, position) {
    const text = document.getText();
    const tag = openTagAt(text, document.offsetAt(position));
    if (!tag) return null;
    const props = await propsForTag(document, tag.name);
    if (!props || !props.length) return null;
    const jaEscritas = new Set(
      [...iterAttrs(tag.attrsText, tag.attrsStart)].map((a) => a.name.toLowerCase())
    );
    return props
      .filter((p) => !jaEscritas.has(p.name.toLowerCase()))
      .map((p) => {
        const item = new vscode.CompletionItem(p.name, vscode.CompletionItemKind.Property);
        item.detail =
          p.default === undefined
            ? `${tag.name} — obrigatória`
            : `${tag.name} — opcional (default: ${p.default})`;
        item.insertText = new vscode.SnippetString(`${p.name}="$1"`);
        // Obrigatórias primeiro: são as que, faltando, quebram o render.
        item.sortText = (p.default === undefined ? "0" : "1") + p.name;
        return item;
      });
  },
};

/**
 * Marca no editor as mesmas violações que o motor recusaria no parse: prop
 * passada que o `<props>` não declara, e prop obrigatória faltando.
 */
async function refreshDiagnostics(document, collection) {
  if (document.languageId !== "glacier-view") return;
  const text = document.getText();
  const out = [];
  for (const tag of iterTags(text)) {
    if (tag.closing) continue;
    let props;
    try {
      props = await propsForTag(document, tag.name);
    } catch {
      continue;
    }
    if (!props) continue;
    const nomes = new Set(props.map((p) => p.name.toLowerCase()));
    const passadas = new Set();
    for (const attr of iterAttrs(tag.attrsText, tag.attrsStart)) {
      passadas.add(attr.name.toLowerCase());
      if (nomes.has(attr.name.toLowerCase())) continue;
      const range = new vscode.Range(
        document.positionAt(attr.start - attr.name.length - 2),
        document.positionAt(attr.end + 1)
      );
      const lista = props.map((p) => p.name).join(", ") || "nenhuma";
      out.push(
        new vscode.Diagnostic(
          range,
          `<${tag.name}> não aceita a prop '${attr.name}' — o <props> dele declara: ${lista}`,
          vscode.DiagnosticSeverity.Error
        )
      );
    }
    for (const p of props) {
      if (p.default !== undefined || passadas.has(p.name.toLowerCase())) continue;
      out.push(
        new vscode.Diagnostic(
          new vscode.Range(
            document.positionAt(tag.nameStart),
            document.positionAt(tag.nameStart + tag.name.length)
          ),
          `<${tag.name}> precisa da prop '${p.name}': declarada sem default, então é obrigatória`,
          vscode.DiagnosticSeverity.Error
        )
      );
    }
  }
  collection.set(document.uri, out);
}

function activate(context) {
  const selector = { language: "glacier-view" };

  extensionPath = context.extensionPath;
  const definitionProvider = { provideDefinition: resolveDefinition };

  const watcher = vscode.workspace.createFileSystemWatcher("**/*.{gv,xml,rs,lua,luau}");
  watcher.onDidCreate(invalidateWorkspaceIndex);
  watcher.onDidDelete(invalidateWorkspaceIndex);
  watcher.onDidChange(invalidateWorkspaceIndex);

  const props = vscode.languages.createDiagnosticCollection("glacier-view-props");
  const revalida = (doc) => refreshDiagnostics(doc, props).catch(() => {});
  vscode.workspace.textDocuments.forEach(revalida);

  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider(selector, definitionProvider),
    vscode.languages.registerDocumentLinkProvider(selector, { provideDocumentLinks }),
    // O ` ` fecha o caso de digitar a prop logo após o nome da tag.
    vscode.languages.registerCompletionItemProvider(selector, provideCompletion, " "),
    props,
    vscode.workspace.onDidOpenTextDocument(revalida),
    vscode.workspace.onDidChangeTextDocument((e) => revalida(e.document)),
    vscode.workspace.onDidCloseTextDocument((doc) => props.delete(doc.uri)),
    watcher,
    vscode.workspace.onDidChangeWorkspaceFolders(invalidateWorkspaceIndex)
  );
}

function deactivate() {}

module.exports = { activate, deactivate };
