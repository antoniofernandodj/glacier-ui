// Glacier GSS — go-to-definition e hover de `var(--token)`.
//
// O realce e os snippets vivem na gramática; isto é a única parte em JS: um
// `var(--fraco)` leva à declaração `--fraco: …` (de preferência a do `:root`),
// no arquivo aberto primeiro e depois em qualquer `.gss` do workspace — o motor
// também junta as folhas num lugar só. Sem dependência e sem build.

const vscode = require("vscode");
const fs = require("fs");

const TOKEN_RE = /--[A-Za-z0-9_-]+/g;

/** O mesmo texto com cada `/* … *\/` trocado por espaços (offsets preservados). */
function maskComments(text) {
  return text.replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " "));
}

/** O token `--nome` sob o offset, ou null. */
function tokenAt(text, offset) {
  TOKEN_RE.lastIndex = 0;
  let m;
  while ((m = TOKEN_RE.exec(text)) !== null) {
    if (offset >= m.index && offset <= m.index + m[0].length) {
      return { name: m[0], start: m.index, end: m.index + m[0].length };
    }
  }
  return null;
}

/**
 * Cada declaração `--nome: valor` do texto, `{ offset, value, root }`. `root`
 * é verdadeiro quando o bloco que a contém é um `:root`.
 */
function declarationsIn(text, name) {
  const masked = maskComments(text);
  const re = new RegExp(`(^|[\\s{;])(${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})\\s*:\\s*([^;}]*)`, "g");
  const out = [];
  let m;
  while ((m = re.exec(masked)) !== null) {
    const offset = m.index + m[1].length;
    const abre = masked.lastIndexOf("{", offset);
    const seletor = abre < 0 ? "" : masked.slice(masked.lastIndexOf("}", abre) + 1, abre).trim();
    out.push({ offset, value: m[3].trim(), root: /^:root\b/.test(seletor) });
  }
  return out;
}

/** Acha a declaração: arquivo aberto primeiro, depois o workspace; `:root` antes de outro bloco. */
async function findDeclaration(document, name) {
  const candidatos = [];
  const aqui = declarationsIn(document.getText(), name);
  for (const d of aqui) candidatos.push({ uri: document.uri, text: document.getText(), ...d, local: true });
  const arquivos = await vscode.workspace.findFiles("**/*.gss", "**/{node_modules,target,.git}/**", 200);
  for (const uri of arquivos) {
    if (uri.fsPath === document.uri.fsPath) continue;
    let text;
    try {
      text = fs.readFileSync(uri.fsPath, "utf8");
    } catch (_) {
      continue;
    }
    for (const d of declarationsIn(text, name)) candidatos.push({ uri, text, ...d, local: false });
  }
  candidatos.sort((a, b) => Number(b.root) - Number(a.root) || Number(b.local) - Number(a.local));
  return candidatos[0];
}

function positionIn(text, offset) {
  const antes = text.slice(0, offset).split("\n");
  return new vscode.Position(antes.length - 1, antes[antes.length - 1].length);
}

async function resolve(document, position) {
  const text = document.getText();
  const tok = tokenAt(text, document.offsetAt(position));
  if (!tok) return null;
  const decl = await findDeclaration(document, tok.name);
  return decl ? { tok, decl } : null;
}

function activate(context) {
  const selector = { language: "gss" };
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider(selector, {
      provideDefinition: async (document, position) => {
        const r = await resolve(document, position);
        if (!r) return undefined;
        const p = positionIn(r.decl.text, r.decl.offset);
        // Estar em cima da própria declaração não leva a lugar nenhum.
        if (r.decl.uri.fsPath === document.uri.fsPath && p.line === position.line) return undefined;
        return new vscode.Location(r.decl.uri, p);
      },
    }),
    vscode.languages.registerHoverProvider(selector, {
      provideHover: async (document, position) => {
        const r = await resolve(document, position);
        if (!r) return undefined;
        const md = new vscode.MarkdownString();
        md.appendCodeblock(`${r.tok.name}: ${r.decl.value}`, "css");
        if (/^#[0-9a-fA-F]{3,8}$/.test(r.decl.value)) md.appendMarkdown(`\n\ncor \`${r.decl.value}\``);
        return new vscode.Hover(md);
      },
    })
  );
}

function deactivate() {}

module.exports = { activate, deactivate, declarationsIn, tokenAt };
