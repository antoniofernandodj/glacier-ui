# Texto rico, links e Markdown

Fase 2 do `PLANO_SUBSTITUIBILIDADE.md`. Exemplo: `examples/gva/texto_rico`
(`--arvore` imprime a árvore avaliada).

## Texto inline misto

Um `<text>` com filhos inline vira um parágrafo de trechos estilizados:

```xml
<text>
  Olá, <b>{usuario}</b>! Veja <code>.gss</code> e <a href="https://…">o site</a>.
</text>
```

| Tag | Efeito |
|---|---|
| `b` `strong` | negrito |
| `i` `em` | itálico |
| `code` | fonte monoespaçada |
| `u` | sublinhado |
| `s` `del` | tachado |
| `span color="#…"` | cor do trecho |
| `a` | link (azul, sublinhado) |

Aninham (`<b><i>…</i></b>`). `{interpolação}` vale em cada trecho. O espaço entre
trechos é preservado como no HTML. Um `<text>` **sem** filho inline continua o
`Text` de sempre. No `.gvb` o texto é atributo: `b(content = "isto")`.

## `<a>`

Dentro de um `<text>` ou avulso. Destinos, em ordem de precedência:

- `href` — abre no navegador do SO (a ação built-in `open:`);
- `navigate_to` — troca de tela;
- `on_click` — ação do app (namespaceada pelo componente dono, como num `<button>`).

## `<markdown>`

CommonMark: títulos, ênfase, negrito, tachado, código inline e em bloco, links,
listas (aninhadas, numeradas), citação e linha horizontal. O texto é o filho — com a
indentação comum removida e as quebras **preservadas** — ou o atributo `content`.
`{chave}` é interpolado: um bloco de código com chaves literais precisa vir por
`content` de uma chave do contexto.

## O que NÃO está feito

- **Seleção e cópia de texto estático.** O `text`/`rich_text` do iced 0.14 não tem
  seleção. O único caminho dentro do iced é um `text_editor` somente-leitura, que
  perde quebra por palavra e trechos estilizados e exige estado por nó. Fica como
  trabalho de upstream.
- **Foco de teclado nos links** (Tab/Enter): clique apenas. Depende da Fase 3.
- **Tabelas e imagens no Markdown**, e HTML cru (ignorado).
- Os links não têm estado de hover.
