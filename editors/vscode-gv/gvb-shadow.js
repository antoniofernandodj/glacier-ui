"use strict";
// `.gvb` → o `.gva` equivalente (XML), com um mapa de volta para o texto de origem.
//
// A extensão inteira (links, F12, diagnósticos) sabe ler XML. Em vez de ensinar
// cada provedor a ler o markup de blocos, o `.gvb` é dessugarado aqui para o XML
// que ele significa — o mesmo que `src/gvb.rs` entrega ao motor — e cada
// caractere do XML guarda o deslocamento do caractere do `.gvb` de que veio. Um
// provedor que acha `salvar` em `on_click="salvar"` devolve um deslocamento do
// XML; `map[deslocamento]` é onde isso está no arquivo que a pessoa vê.
//
// É uma leitura **tolerante**, ao contrário da do motor: o arquivo está sendo
// digitado, quase sempre incompleto, e um `{` aberto ou um `nome =` sem valor não
// pode apagar os links do resto da tela. O que não se entende é pulado até a
// próxima linha. Quem manda no que é erro é o motor (`src/gvb.rs`).

const IDENT = /[\p{L}\p{N}_-]/u;
const isIdent = (c) => c !== undefined && IDENT.test(c);
const isWs = (c) => c === " " || c === "\t" || c === "\n" || c === "\r";

class Parser {
  constructor(s) {
    this.s = s;
    this.i = 0;
    this.root = { tag: "", attrs: [], text: null, children: [] };
  }

  peek(n = 0) {
    return this.s[this.i + n];
  }

  starts(str) {
    return this.s.startsWith(str, this.i);
  }

  skipWs() {
    for (;;) {
      const c = this.peek();
      if (c === undefined) return;
      if (isWs(c)) this.i++;
      else if (this.starts("//")) {
        while (this.i < this.s.length && this.s[this.i] !== "\n") this.i++;
      } else if (this.starts("/*")) {
        const e = this.s.indexOf("*/", this.i + 2);
        this.i = e < 0 ? this.s.length : e + 2;
      } else return;
    }
  }

  ident() {
    const off = this.i;
    while (isIdent(this.peek())) this.i++;
    return { text: this.s.slice(off, this.i), off };
  }

  skipInline() {
    while (this.peek() === " " || this.peek() === "\t") this.i++;
  }

  wordIs(w) {
    return this.starts(w) && !isIdent(this.s[this.i + w.length]);
  }

  /** `"…"` ou `"""…"""` → { raw, off (do 1º caractere do conteúdo), triple }. */
  string() {
    if (this.starts('"""')) {
      this.i += 3;
      const off = this.i;
      const e = this.s.indexOf('"""', off);
      const end = e < 0 ? this.s.length : e;
      this.i = e < 0 ? end : end + 3;
      return { raw: this.s.slice(off, end), off, triple: true };
    }
    this.i++;
    const off = this.i;
    while (this.i < this.s.length) {
      const c = this.s[this.i];
      if (c === "\\" && (this.s[this.i + 1] === '"' || this.s[this.i + 1] === "\\")) {
        this.i += 2;
        continue;
      }
      if (c === '"') break;
      this.i++;
    }
    const raw = this.s.slice(off, this.i);
    if (this.s[this.i] === '"') this.i++;
    return { raw, off, triple: false };
  }

  /** O valor de um `nome =`: entre aspas, ou nu até o espaço, a `,` ou o `)`. */
  value() {
    const c = this.peek();
    if (c === '"') return { kind: "src", ...this.string() };
    if (c === undefined || c === "," || c === ")") throw new Error("valor");
    const off = this.i;
    while (this.i < this.s.length) {
      const d = this.s[this.i];
      if (isWs(d) || d === "," || d === ")" || this.starts("//")) break;
      this.i++;
    }
    return { kind: "src", raw: this.s.slice(off, this.i), off, triple: true };
  }

  refName() {
    const off = this.i;
    for (;;) {
      const c = this.peek();
      const n = this.peek(1);
      if (c !== undefined && (/[\p{L}\p{N}_]/u.test(c) || (c === "." && n !== undefined && /[\p{L}\p{N}_]/u.test(n)))) this.i++;
      else break;
    }
    return { name: this.s.slice(off, this.i), off };
  }

  operand() {
    const c = this.peek();
    if (c === "@") {
      const at = this.i++;
      const braced = this.peek() === "{";
      if (braced) this.i++;
      const r = this.refName();
      if (braced && this.peek() === "}") this.i++;
      if (!r.name) throw new Error("ref");
      return { kind: "ref", name: r.name, off: r.off, at };
    }
    if (c === '"') return { kind: "src", ...this.string() };
    if (isIdent(c)) {
      const id = this.ident();
      return { kind: "src", raw: id.text, off: id.off, triple: true };
    }
    throw new Error("operando");
  }

  /** Atributos de um ramo `if`, já como { name, nameOff, v }. */
  condition(key, at) {
    const mk = (name, v) => ({ name, nameOff: at, v });
    const first = this.operand();
    this.skipInline();
    const asRef = (o) => {
      if (o.kind !== "ref") throw new Error("esquerda");
      return o;
    };
    if (this.wordIs("in")) {
      this.i += 2;
      this.skipInline();
      const target = asRef(this.operand());
      return [mk(key, target), mk("contains", first)];
    }
    if (this.wordIs("is")) {
      this.i += 2;
      this.skipInline();
      if (this.wordIs("empty")) this.i += 5;
      return [mk(key, asRef(first)), mk("empty", { kind: "lit", text: "true", off: at })];
    }
    const op = this.starts("==") ? "equals" : this.starts("!=") ? "not_equals" : null;
    if (op) {
      this.i += 2;
      this.skipInline();
      return [mk(key, asRef(first)), mk(op, this.operand())];
    }
    return [mk(key, asRef(first))];
  }

  /** Os atributos de `( nome = valor, … )` em `node` (o `(` sob o cursor). Tolerante: para no `)` ou numa linha que não se entende. */
  attrs(node) {
    this.i++; // `(`
    for (;;) {
      this.skipWs();
      const c = this.peek();
      if (c === undefined) return;
      if (c === ")") {
        this.i++;
        return;
      }
      if (c === ",") {
        this.i++;
        continue;
      }
      if (!isIdent(c) && c !== ":") {
        // o que está sendo digitado: pula a linha e segue (um `{` ou `}` encerra)
        if (c === "{" || c === "}") return;
        while (this.i < this.s.length && this.s[this.i] !== "\n") this.i++;
        continue;
      }
      // `:nome` é uma ligação: o `:` entra no nome, e o XML gerado o leva junto.
      const bound = c === ":";
      if (bound) this.i++;
      const w = this.ident();
      if (bound) {
        w.text = ":" + w.text;
        w.off -= 1;
      }
      this.skipWs();
      if (this.peek() !== "=") continue;
      this.i++;
      this.skipWs();
      try {
        node.attrs.push({ name: w.text, nameOff: w.off, v: this.value() });
      } catch (_) {
        // `nome =` sem valor ainda: o atributo fica de fora
      }
    }
  }

  /** O conteúdo de um bloco: os filhos de `node`, até o `}` (não consumido). */
  items(node, inBlock) {
    for (;;) {
      this.skipWs();
      const c = this.peek();
      if (c === undefined) return;
      if (c === "}") {
        if (inBlock) return;
        this.i++;
        continue;
      }
      const before = this.i;
      try {
        this.item(node);
      } catch (_) {
        // pula o resto da linha: o que está sendo digitado não entra
        while (this.i < this.s.length && this.s[this.i] !== "\n") this.i++;
      }
      if (this.i === before) this.i++;
    }
  }

  block(node) {
    // `{` já visto
    this.i++;
    this.items(node, true);
    node.closeOff = this.i;
    if (this.peek() === "}") this.i++;
  }

  item(parent) {
    if (!isIdent(this.peek())) throw new Error("item");
    const w = this.ident();
    if (this.peek() === ":") throw new Error("atributo fora dos parênteses");
    if (w.text === "if") return this.ifChain(parent, w.off);
    if (w.text === "each") return this.each(parent, w.off);
    if (w.text === "else") throw new Error("else");
    this.element(parent, w);
  }

  element(parent, w) {
    const node = {
      tag: w.text,
      tagOff: w.off,
      attrs: [],
      text: null,
      children: [],
      closeOff: this.i,
      rawTag: w.text === "script" || w.text === "style",
    };
    parent.children.push(node);
    let save = this.i;
    this.skipWs();
    if (this.peek() === "(") {
      this.attrs(node);
      node.closeOff = this.i;
      save = this.i;
      this.skipWs();
    }
    if (this.peek() === '"') {
      node.text = { ...this.string(), raw_body: node.rawTag };
      node.closeOff = this.i;
      save = this.i;
      this.skipWs();
    }
    if (this.peek() === "{") this.block(node);
    else this.i = save;
  }

  branch(parent, key, at) {
    const node = { tag: "template", tagOff: at, attrs: [], text: null, children: [], closeOff: at };
    parent.children.push(node);
    this.skipWs();
    node.attrs.push(...this.condition(key, at));
    this.skipWs();
    if (this.peek() === "{") this.block(node);
    return node;
  }

  ifChain(parent, at) {
    this.branch(parent, "if", at);
    for (;;) {
      const save = this.i;
      this.skipWs();
      if (!this.wordIs("else")) {
        this.i = save;
        return;
      }
      const l = this.i;
      this.i += 4;
      this.skipWs();
      if (this.wordIs("if")) {
        this.i += 2;
        this.branch(parent, "else_if", l);
        continue;
      }
      const node = { tag: "template", tagOff: l, attrs: [{ name: "else", nameOff: l, v: { kind: "lit", text: "", off: l } }], text: null, children: [], closeOff: l };
      parent.children.push(node);
      if (this.peek() === "{") this.block(node);
      return;
    }
  }

  each(parent, at) {
    this.skipWs();
    const items = this.operand();
    if (items.kind !== "ref") throw new Error("each");
    this.skipWs();
    if (!this.wordIs("as")) throw new Error("as");
    this.i += 2;
    this.skipWs();
    const v = this.ident();
    const node = {
      tag: "foreach",
      tagOff: at,
      attrs: [
        { name: "items", nameOff: at, v: { kind: "lit", text: items.name, off: items.off, map: true } },
        { name: "var", nameOff: at, v: { kind: "lit", text: v.text, off: v.off, map: true } },
      ],
      text: null,
      children: [],
      closeOff: at,
    };
    parent.children.push(node);
    this.skipWs();
    if (this.peek() === "(") {
      this.attrs(node);
      this.skipWs();
    }
    if (this.peek() === "{") this.block(node);
  }
}

// ── emissão ──────────────────────────────────────────────────────────────

class Out {
  constructor() {
    this.xml = [];
    this.map = [];
  }

  /** Cada caractere de `str` aponta para `off` (um ponto só). */
  at(str, off) {
    for (let i = 0; i < str.length; i++) {
      this.xml.push(str[i]);
      this.map.push(off);
    }
  }

  /** Cada caractere de `str` aponta para `off + posição` (1:1 com a fonte). */
  seq(str, off) {
    for (let k = 0; k < str.length; k++) {
      this.xml.push(str[k]);
      this.map.push(off + k);
    }
  }

  esc(ch, off, attr) {
    const e = ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch === ">" ? "&gt;" : ch === '"' && attr ? "&quot;" : null;
    this.at(e || ch, off);
  }

  /** O conteúdo de `src` (string do `.gvb`), com `@x` → `{x}` e o escape de XML. */
  src(v, attr, raw) {
    const s = v.raw;
    for (let i = 0; i < s.length; i++) {
      const off = v.off + i;
      const c = s[i];
      if (raw) {
        this.at(c, off);
        continue;
      }
      if (c === "\\" && !v.triple && (s[i + 1] === '"' || s[i + 1] === "\\")) {
        i++;
        this.esc(s[i], v.off + i, attr);
        continue;
      }
      if (c !== "@") {
        this.esc(c, off, attr);
        continue;
      }
      const n = s[i + 1];
      if (n === "@") {
        this.at("@", off);
        i++;
      } else if (n === "{") {
        const e = s.indexOf("}", i + 2);
        if (e < 0) {
          this.at("@", off);
          continue;
        }
        this.at("{", off);
        for (let j = i + 2; j < e; j++) this.esc(s[j], v.off + j, attr);
        this.at("}", v.off + e);
        i = e;
      } else if (n !== undefined && /[\p{L}\p{N}_]/u.test(n)) {
        let j = i + 1;
        while (j < s.length && (/[\p{L}\p{N}_]/u.test(s[j]) || (s[j] === "." && j + 1 < s.length && /[\p{L}\p{N}_]/u.test(s[j + 1])))) j++;
        this.at("{", off);
        this.seq(s.slice(i + 1, j), v.off + i + 1);
        this.at("}", v.off + j);
        i = j - 1;
      } else this.at("@", off);
    }
  }

  attr(a, prevEnd) {
    // o espaço aponta para o fim do que veio antes (o nome da tag, o valor
    // anterior): é aí que um provedor que mede `nameStart + nome.length` termina
    this.at(" ", prevEnd);
    this.seq(a.name, a.nameOff);
    // `="` aponta para onde o valor começa, para um provedor que mede
    // `attr.start - nome.length - 2` cair no início do nome
    const v = a.v;
    const startOff = v.off !== undefined ? v.off : a.nameOff;
    this.at('="', startOff);
    if (v.kind === "ref") {
      this.at("{", v.at);
      this.seq(v.name, v.off);
      this.at("}", v.off + v.name.length);
    } else if (v.kind === "lit") {
      if (v.map) this.seq(v.text, v.off);
      else this.at(v.text, v.off);
    } else this.src(v, true, false);
    const endOff = v.kind === "src" ? v.off + v.raw.length : v.kind === "ref" ? v.off + v.name.length : (v.off || 0) + (v.text || "").length;
    this.at('"', endOff);
  }

  node(n) {
    this.at("<", n.tagOff);
    this.seq(n.tag, n.tagOff);
    let prevEnd = n.tagOff + n.tag.length;
    for (const a of n.attrs) {
      this.attr(a, prevEnd);
      const v = a.v;
      prevEnd = v.kind === "src" ? v.off + v.raw.length : v.kind === "ref" ? v.off + v.name.length : (v.off || 0) + (v.text || "").length;
    }
    const rawTag = n.rawTag;
    if (!n.text && n.children.length === 0 && !rawTag) {
      this.at("/>", n.closeOff);
      return;
    }
    this.at(">", n.tagOff);
    if (n.text) this.src(n.text, false, rawTag);
    for (const c of n.children) this.node(c);
    this.at("</", n.closeOff);
    this.seq(n.tag, n.tagOff);
    this.at(">", n.closeOff);
  }
}

/**
 * `src` (um `.gvb`) → `{ xml, map }`, com `map[i]` o deslocamento em `src` do
 * caractere `i` do `xml` (e `map[xml.length]` o fim de `src`).
 */
function toXml(src) {
  const p = new Parser(src);
  p.items(p.root, false);
  const out = new Out();
  for (const n of p.root.children) out.node(n);
  const map = Int32Array.from(out.map.concat([src.length]));
  return { xml: out.xml.join(""), map };
}

module.exports = { toXml };
