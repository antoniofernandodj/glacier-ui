# Texto rico, links, seleção e Markdown

Fase 1 do `PLANO_SUBSTITUIBILIDADE.md`. Exemplos (a mesma tela, em duas formas):

| estado em Rust | estado em Luau |
|---|---|
| `texto_rico` | `texto_rico_luau` |

O de Rust aceita `--arvore` (imprime a árvore avaliada). No de Luau o estado vive em
`scripts/app.luau` e as telas são filhas do `app`, sem `Component`.

## `prose`: texto de leitura em blocos

Estilo **dentro** do parágrafo existe só dentro de um `prose`. O `text` continua
simples, de propósito: `b`, `i`, `code`, `u`, `s`, `span`, `a`, `p` e os
blocos abaixo **não existem** fora dele — usar um num `text` dá erro na avaliação,
apontando o `prose`, em vez de perder as palavras.

Um `prose` é uma lista de **blocos**:

```gv
prose {
  h2("Manchete")
  p {
    span("Olá,")
    b("{usuario}")
    span("! Veja")
    code(".gss")
    span("e")
    a("o site", href = "https://…")
    span(".")
  }
  blockquote {
    p("Foi uma decisão difícil.")
    footer("Ana Souza")
  }
  figure {
    img(src = "fotos/praia.png", alt = Praia)
    figcaption {
      span("A praia ao")
      b("amanhecer")
      span(".")
    }
  }
  ul {
    li("um")
    li("dois")
  }
}
```

| Bloco | Contém |
|---|---|
| `p` | texto e as tags inline |
| `h1`…`h6` | texto e as tags inline |
| `blockquote` | `p`s e, opcional, um `footer`/`cite` com a autoria |
| `ul`, `ol` | `li`s (texto e tags inline) |
| `figure` | um `img` (imagem **local**) e, opcional, um `figcaption` (texto rico) |
| `img` | uma imagem sem legenda |
| `pre` | texto cru, em fonte monoespaçada |
| `hr` | uma régua |

| Tag inline | Efeito |
|---|---|
| `b` `strong` | negrito |
| `i` `em` | itálico |
| `code` | fonte monoespaçada |
| `u` | sublinhado |
| `s` `del` | tachado |
| `span color="#…"` | cor do trecho |
| `a` | link |

Aninham (`b { i("…") }`). `{interpolação}` vale em cada trecho.

**Forma obrigatória** (erro de carga, com linha e coluna, se violada): o `prose` só
contém os blocos da primeira tabela — texto solto ou uma tag de estilo direto nele é
erro —, e um bloco de texto só contém texto e as tags inline (nenhum outro elemento, e um
`p` não entra em outro `p`). O espaço entre trechos é preservado como no HTML.

## `prose(syntax, content)`: o texto vem de fora

Um `prose` com `content` não tem filhos: o texto vem de uma chave do contexto — um
arquivo lido, o resultado de um `fetch`, uma linha de banco — e é lido **na avaliação**, na
sintaxe pedida:

```gv
prose(syntax = html, content = @artigo)
prose(syntax = gv, content = @artigo)  // o mesmo que omitir `syntax`
prose(syntax = markdown, content = @artigo)
```

`syntax` aceita `gv`, `html` e `markdown`. **Omitido, vale `gv`.** `content`
junto com filhos é erro de carga; um `syntax` que não existe também.

O valor entra **literal**: um `{x}` ou um `b` dentro dele não é reinterpretado como
modelo do app (o `@` do `.gv` também não vira ligação — um e-mail `a@b.com` fica
`a@b.com`).

| Sintaxe | Como é lida |
|---|---|
| `gv` | o `.gv` dessugarado, **sem interpolação**, com a gramática de blocos de cima. Estrito: fora dela é erro. |
| `html` | parser próprio e **tolerante**: `p`/`li` sem fechar, entidades (`&amp;`, `&nbsp;`, `&ccedil;`…), comentários, atributos sem aspas. `script`, `style`, `<iframe>`, `form` e afins somem com o conteúdo; `<div>`/`<section>`/`<article>` são transparentes; tag desconhecida perde a tag e fica o texto. |
| `markdown` | o renderizador de `markdown` (abaixo). |

### Conteúdo vindo de dado não é de confiança

O texto de um banco ou de uma API não foi escrito por quem escreveu o app. Por isso, em
`content`:

- **só `href` vira link, e só `http`, `https` e `mailto`.** `javascript:`, `file:` e
  qualquer protocolo registrado por outro programa não viram link (o texto fica);
- **`on_click` e `navigate_to` são recusados.** Sem isso, um registro com
  `on_click="app::apagar"` dispararia uma ação do app. Em `gv` é um erro visível;
  em `html` o atributo é ignorado;
- **imagem só de caminho local** (relativo, sem esquema, sem `..`). `http://`, `data:` e
  caminho absoluto aparecem como `[imagem recusada: alt]`;
- o conteúdo tem teto de **2 MiB**, e o aninhamento do HTML, de 100 níveis.

Um conteúdo que não dá para ler **não derruba a tela**: o `prose` mostra o erro, em
vermelho (`prose (gv): linha 3: <div> direto no prose …`), e o resto segue de pé.
`selectable` vale só para `prose` escrito no markup, não para `content`.

Exemplos (o mesmo artigo — manchete, citação, **figura com legenda**, lista, links — em
HTML, gv e Markdown, lido de arquivos em `examples/conteudo_prose`): `prose_artigo`
(Rust) e `prose_artigo_luau` (`fetch("file://…")`). `cargo run --example prose_check` roda o parser de blocos contra
entradas difíceis (HTML sujo de CMS, links perigosos, `@` em e-mail, `.gv` quebrado) sem
janela.

### No `.gv`

O `.gv` não mistura texto com elementos: o texto é sempre um atributo. Nas tags que
**carregam** texto (`text`, `markdown`, `b`, `i`, `code`, `u`, `s`, `a`, `span`…) o
**primeiro argumento sem nome** vale como `content`. `prose`, `p`, `ul`, `ol`, `blockquote` e `figure` são só contêineres (o texto vive nos filhos
deles) e **não têm texto sem nome**; títulos (`h1`…`h6`), `li`, `figcaption`, `footer` e
`cite` têm:

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
vírgula (`a("o site", href = …)`). O mesmo vale para `text("Olá", class = titulo)` e para o bloco que preserva as quebras
de linha: `markdown(l"""…""", class = md)` (o `"""` simples colapsaria as quebras, e o
Markdown precisa delas).

- O texto sem nome vem **primeiro** na lista, e não pode coexistir com `content = …`
  na mesma tag (erro: "tem o texto duas vezes").
- Fora das tags de texto continua erro (`button("Salvar")` → "não aceita texto sem
  nome"): em `button` o texto é `text`, em `textinput` é `placeholder`, e `nome = valor`
  diz qual. Estender é uma linha na tabela `posicional` de `src/gv.rs`.
- **O espaço é do texto**, não do formato: `"Olá, "` leva o espaço dentro das aspas, e a
  quebra de linha entre duas tags não conta. (Dentro de `prose` e das tags inline o
  dessugar põe essa quebra num comentário XML — mantém a contagem de linhas dos erros e
  não vira espaço.)

Os blocos no `.gv`:

```
prose {
  h2("Manchete")
  blockquote { p { span("Foi uma decisão difícil.") } footer("Ana Souza") }
  figure {
    img(src = "fotos/praia.png", alt = "Praia")
    figcaption { span("A praia ao ") b("amanhecer") span(".") }
  }
  ul { li("um") li { span("dois ") b("negrito") } }
}
```

## `a`

Só dentro de um `prose`. Destinos, em ordem de precedência:

- `href` — abre no **navegador do SO** (a ação built-in `open:`); o app não carrega
  nada da URL;
- `navigate_to` — troca de tela;
- `on_click` — ação do app (namespaceada pelo componente dono, como num `button`).

**Teclado** (`src/link_focus.rs`): um `prose` com link entra na ordem de Tab. Com o foco,
Tab / Shift+Tab andam de link em link dentro do parágrafo (e só saem dele depois do
último / antes do primeiro); **Enter** ou **Espaço** ativam o link (a mesma ação do
clique). O link em foco aparece com fundo azul claro e sublinhado. Clicar fora tira o
foco. Um `prose` sem link não é focável.

Como foi verificado, em duas camadas:

- a lógica (entrar no foco, andar, ativar, sair) roda sem janela em
  `tools/link_focus_check` (`cd tools/link_focus_check && cargo run`), com o `Rich` de
  verdade do iced e um renderizador nulo;
- o **desenho** foi conferido numa janela real, sob X11, com teclas injetadas por
  `tools/xtest.py` e a janela capturada: Tab destaca o primeiro link, Tab anda para o
  seguinte, Enter no link de ação incrementa o contador (0 → 1) e Tab no último link
  sai do parágrafo, sem destaque. O destaque é fundo azul claro com sublinhado.

Por baixo, o destaque é uma variante pronta do parágrafo por link (o `Rich` não revela
onde cada link está), então um parágrafo com N links monta N+1 cópias.

Hover: o link fica azul e **sublinha sob o cursor** (e o cursor do mouse muda); o
sublinhado permanente foi retirado para o hover ser visível.

## Texto selecionável

`selectable="true"` num `text` ou `prose` o torna selecionável e copiável
(arrastar o mouse, Ctrl+C, menu de contexto do botão direito). Por baixo é um
`textarea(readonly)` sem borda nem fundo, com o buffer vindo do próprio texto.

```gv
text("Selecione e copie isto.", selectable = true)
```

**O que se perde** — um `text_editor` não tem: trechos estilizados e links. Um
`prose(selectable)` vira texto simples. Quebra por palavra, tamanho, cor e negrito
da classe `.gss` continuam valendo. Dois nós com o mesmo texto dividem o buffer (e a
seleção). Conferido numa janela: arrastar o mouse destaca o trecho, a altura encolhe ao conteúdo e não há borda. **Copiar (Ctrl+C) não foi testado** — só a seleção.

## `markdown`

CommonMark: títulos, ênfase, negrito, tachado, código inline e em bloco, links,
listas (aninhadas, numeradas), citação, linha horizontal e **tabelas**. O texto é o
filho — com a indentação comum removida e as quebras **preservadas** — ou o atributo
`content`. `{chave}` é interpolado: um bloco de código com chaves literais precisa
vir por `content` de uma chave do contexto.

**Imagens**: só caminho **local**, relativo, lido pela fonte de assets do app
(`![alt](pasta/figura.png)`). `http://`, `https://`, `data:`, caminho absoluto e `..`
são recusados e a imagem vira `[imagem recusada: alt]`. O Markdown costuma vir de dado
externo (um `fetch`), e ele não escolhe o que o app lê nem busca coisa na rede.

## Um defeito do `cosmic-text` contornado aqui

Pontuação ou letras coladas a um trecho estilizado saíam no estilo do trecho:
`bAna</b>!` desenhava o `!` em negrito, `bAna</b>(x)` o `(x)` inteiro, `iit</i>ab`
o `ab` em itálico, e `**negrito**,` no `markdown` a vírgula em negrito.

A causa está no `cosmic-text` 0.15 (o motor de texto do iced), em `ShapeWord::build`: uma
"palavra" só de ASCII é moldada **de uma vez, com a fonte do primeiro caractere**,
ignorando a troca de estilo no meio. As palavras são unidades de quebra de linha
(UAX #14), então também ` !` e ` ...` — que não podem abrir uma linha — ficam presos à
palavra anterior. Uma palavra com um caractere não-ASCII cai no caminho completo, que
respeita a troca de estilo; por isso `Olá` e `itálico` nunca tiveram o problema.

**Contorno** (`precisa_juntar`, em `src/widget.rs`): onde a fonte muda sem uma quebra
legítima entre os trechos, o Glacier põe um `U+2060` (WORD JOINER) na frente do trecho
seguinte. Ele é invisível, não tem largura e não cria ponto de quebra, então o texto
quebra exatamente onde quebraria sem ele, mas a palavra deixa de ser "só ASCII". Vale
para o `prose` e para o `markdown`.

Verificado de duas formas: moldando direto com o `cosmic-text` (o peso por glifo vai de
700 para 400 no `!` e na vírgula, em todos os casos testados) e numa janela, ampliada.
Quando o `cosmic-text` corrigir o atalho, o contorno vira inofensivo e pode sair.

## O que NÃO está feito

- **Listas aninhadas e tabelas dentro do `prose`.** Os blocos acima são todos; `<table>` e
  `ul` dentro de `li` não entram (no HTML, o texto da tabela segue no fluxo; no `gv`
  é erro). O `markdown`, sim, tem lista aninhada e tabela.
- **O parser de HTML não é o do HTML5.** Cobre o que um texto de leitura usa (ver a tabela
  acima). `p` dentro de `p` sem fechar o primeiro funciona, mas um aninhamento torto de
  `bi` é resolvido do jeito simples (fecha até a tag que casa).
- **`selectable` com `content`** e, no `prose` estático, `selectable` perde estilo, links e
  figuras (vira texto simples).
- **Foco de teclado nos links do `markdown`** (só clique) e nos do texto selecionável
  (que nem tem link).
- Ao voltar com Shift+Tab para um parágrafo com links, o foco cai no **primeiro** link, não
  no último (a operação de foco do iced não diz de que lado o foco chegou). Com vários
  parágrafos, cada um é um ponto de parada do Tab.
- Imagem **remota** (`https://…`) nunca é carregada: de propósito, o Glacier não busca
  nada de uma URL além de dados pelo `fetch`.
