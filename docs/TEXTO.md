# Texto rico, links, seleção e Markdown

Fase 2 do `PLANO_SUBSTITUIBILIDADE.md`. Exemplo: `examples/gva/texto_rico`
(`--arvore` imprime a árvore avaliada).

## `<prose>`: texto inline misto

Estilo **dentro** do parágrafo existe só dentro de um `<prose>`. O `<text>` continua
simples, de propósito: `<b>`, `<i>`, `<code>`, `<u>`, `<s>`, `<span>`, `<a>` e `<p>`
**não existem** fora dele — usar um num `<text>` dá erro na avaliação, apontando o
`<prose>`, em vez de perder as palavras.

```xml
<prose>
  <p>Olá, <b>{usuario}</b>! Veja <code>.gss</code> e <a href="https://…">o site</a>.</p>
  <p>Segundo parágrafo, com <i>itálico</i> e <s>tachado</s>.</p>
</prose>
```

| Tag | Efeito |
|---|---|
| `p` | parágrafo (uma linha em branco entre eles) |
| `b` `strong` | negrito |
| `i` `em` | itálico |
| `code` | fonte monoespaçada |
| `u` | sublinhado |
| `s` `del` | tachado |
| `span color="#…"` | cor do trecho |
| `a` | link |

Aninham (`<b><i>…</i></b>`). `{interpolação}` vale em cada trecho. Texto direto no
`<prose>`, sem `<p>`, é um parágrafo só. O espaço entre trechos é preservado como no
HTML. No `.gvb` o texto é atributo: `b(content = "isto")`.

### No `.gvb`

O `.gvb` não mistura texto com elementos: o texto é sempre um atributo. Nas tags que
**carregam** texto (`text`, `markdown`, `b`, `i`, `code`, `u`, `s`, `a`, `span`…) o
**primeiro argumento sem nome** vale como `content`. `prose` e `p` são só contêineres
(o texto vive nos filhos deles) e **não têm `content`**:

```
prose(class = corpo) {
  p {
    span("Olá, ") b("{usuario}") span("! Veja ")
    a("o site", href = "https://exemplo.com")
    b { i("negrito itálico") }
  }
  p {
    s("tachado")
  }
}
```

`b("x")` é `b(content = "x")`; o resto da lista segue `nome = valor`, depois de uma
vírgula (`a("o site", href = …)`). O mesmo vale para `text("Olá", class = titulo)`.

- O texto sem nome vem **primeiro** na lista, e não pode coexistir com `content = …`
  na mesma tag (erro: "tem o texto duas vezes").
- Fora das tags de texto continua erro (`button("Salvar")` → "não aceita texto sem
  nome"): em `button` o texto é `text`, em `textinput` é `placeholder`, e `nome = valor`
  diz qual. Estender é uma linha na tabela `posicional` de `src/gvb.rs`.
- **O espaço é do texto**, não do formato: `"Olá, "` leva o espaço dentro das aspas, e a
  quebra de linha entre duas tags não conta. (Dentro de `prose` e das tags inline o
  dessugar põe essa quebra num comentário XML — mantém a contagem de linhas dos erros e
  não vira espaço.)

Conferido com `gvb_check`: o `.gvb` acima e o `.gva` equivalente dão a mesma árvore.

## `<a>`

Só dentro de um `<prose>`. Destinos, em ordem de precedência:

- `href` — abre no **navegador do SO** (a ação built-in `open:`); o app não carrega
  nada da URL;
- `navigate_to` — troca de tela;
- `on_click` — ação do app (namespaceada pelo componente dono, como num `<button>`).

Hover: o link fica azul e **sublinha sob o cursor** (e o cursor do mouse muda); o
sublinhado permanente foi retirado para o hover ser visível.

## Texto selecionável

`selectable="true"` num `<text>` ou `<prose>` o torna selecionável e copiável
(arrastar o mouse, Ctrl+C, menu de contexto do botão direito). Por baixo é um
`<textarea readonly>` sem borda nem fundo, com o buffer vindo do próprio texto.

```xml
<text selectable="true">Selecione e copie isto.</text>
```

**O que se perde** — um `text_editor` não tem: trechos estilizados e links. Um
`<prose selectable>` vira texto simples. Quebra por palavra, tamanho, cor e negrito
da classe `.gss` continuam valendo. Dois nós com o mesmo texto dividem o buffer (e a
seleção). Não foi conferido visualmente que a altura encolhe ao conteúdo.

## `<markdown>`

CommonMark: títulos, ênfase, negrito, tachado, código inline e em bloco, links,
listas (aninhadas, numeradas), citação, linha horizontal e **tabelas**. O texto é o
filho — com a indentação comum removida e as quebras **preservadas** — ou o atributo
`content`. `{chave}` é interpolado: um bloco de código com chaves literais precisa
vir por `content` de uma chave do contexto.

**Imagens**: só caminho **local**, relativo, lido pela fonte de assets do app
(`![alt](pasta/figura.png)`). `http://`, `https://`, `data:`, caminho absoluto e `..`
são recusados e a imagem vira `[imagem recusada: alt]`. O Markdown costuma vir de dado
externo (um `fetch`), e ele não escolhe o que o app lê nem busca coisa na rede.

## O que NÃO está feito

- **Foco de teclado nos links** (Tab até um link, Enter para ativar). O
  `rich_text` do iced não é focável. Exige um widget próprio que embrulhe o
  `rich_text`, implemente `Focusable`, trate Tab/Enter e guarde qual link está em
  foco fora do `view` (o `view` reconstrói os trechos a cada quadro). Não foi
  feito porque não dá para verificá-lo aqui: esta máquina não tem como injetar
  teclado numa janela, e um widget de foco sem teste quebraria **todo** `<prose>`.
- **Links no texto selecionável** (ver acima) e no Markdown além do clique.
- HTML cru no Markdown é ignorado.
