"use strict";
// A aspa dupla no `.gvb`, decidida por quem sabe onde o cursor está.
//
// O par `"` do `language-configuration` é cego: fecha toda aspa digitada, e não
// sabe que três em sequência são um `"""`, que dentro de um `"""…"""` uma aspa é
// só uma aspa, nem que depois de `//` não há string. Aqui a decisão é uma função
// pura (`decideQuote`), testável sem editor; `extension.js` só a executa.

/** Quantos `"""` existem antes de `offset`: ímpar = o cursor está num corpo de `"""`. */
function inTripleBody(text, offset) {
  return (text.slice(0, offset).match(/"{3}/g) || []).length % 2 === 1;
}

/**
 * O que fazer ao digitar `"` com o cursor em `offset` de `text`:
 *  - "pair"      `"` → `"|"` (abre uma string);
 *  - "overtype"  o `"` seguinte é o fecho que já está lá: só passa por cima;
 *  - "triple"    `""|` → `"""|"""` (a terceira aspa de seguida);
 *  - "plain"     só insere a aspa (dentro de um corpo de `"""`, fechando uma
 *                string, depois de `//`, colada numa palavra).
 */
function decideQuote(text, offset) {
  if (inTripleBody(text, offset)) return "plain";
  const lineStart = text.lastIndexOf("\n", offset - 1) + 1;
  const lineEnd = text.indexOf("\n", offset);
  const before = text.slice(lineStart, offset);
  const after = text.slice(offset, lineEnd < 0 ? text.length : lineEnd);

  // Dentro de uma string desta linha? (escape `\"` incluído; `//` fora de string
  // é comentário, e aí aspa nenhuma abre string.)
  let inStr = false;
  for (let i = 0; i < before.length; i++) {
    const c = before[i];
    if (inStr) {
      if (c === "\\") i++;
      else if (c === '"') inStr = false;
    } else if (c === '"') inStr = true;
    else if (c === "/" && before[i + 1] === "/") return "plain";
  }
  if (inStr) return after.startsWith('"') ? "overtype" : "plain";

  // Fora de string. Duas aspas coladas antes do cursor são uma string vazia
  // acabada de fechar: a terceira vira um `"""`.
  if (before.endsWith('""') && !after.startsWith('"')) return "triple";

  const prev = before.slice(-1);
  const next = after.slice(0, 1);
  const word = /[\p{L}\p{N}_]/u;
  if (prev && word.test(prev)) return "plain";
  if (next && word.test(next)) return "plain";
  return "pair";
}

/** A edição para uma decisão: `{ insert, at, cursor }` (deslocamentos absolutos), ou null para "plain". */
function quoteEdit(kind, offset) {
  switch (kind) {
    case "pair":
      return { insert: '""', at: offset, cursor: offset + 1 };
    case "triple":
      return { insert: '""""', at: offset, cursor: offset + 1 };
    case "overtype":
      return { insert: "", at: offset, cursor: offset + 1 };
    default:
      return null;
  }
}

module.exports = { decideQuote, quoteEdit, inTripleBody };
