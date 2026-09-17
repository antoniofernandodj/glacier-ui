# `.gvb` — o markup de blocos

**Rascunho.** Nada disto roda: o motor lê `.gv` (XML) e só. O que existe hoje é
o realce de sintaxe na extensão de VS Code (`glacier-view-block`) e o
`rascunhos/onda9.gvb`, que é o `examples/onda9/app.gv` convertido inteiro — a
tela mais densa do repositório, escolhida de propósito para a proposta não ser
medida num exemplo fácil.

O alvo é o **desconforto de escrever**, não o de rodar. Por isso a regra que
manda em tudo: **o `.gvb` não muda uma vírgula da semântica do motor.** Ele
dessugara para a mesma `UiNode` que o `.gv` produz, e toda diferença aqui é de
grafia.

---

## Por que não o XML

No `examples/onda9/app.gv`, 297 linhas:

- **88 fechamentos** que repetem o nome da tag;
- **24 `&lt;`/`&gt;`**, porque a prosa dos comentários e dos `<text>` cita tags
  o tempo todo e o XML não deixa;
- **82 `class="…"`**, uma string por elemento para dizer o papel dele;
- 12 blocos de `<!-- -->`, onde `--` é proibido no meio;
- 8 linhas passando de 80 colunas, sem jeito de quebrar.

O mesmo arquivo em `.gvb`: 324 linhas, 12711 bytes (−8%), **nenhuma linha acima
de 80 colunas**. Ganha-se quebra livre e perde-se em número de linhas — 52 delas
são só `}`. É a troca consciente da proposta, e a seção final explica por que ela
compensa.

---

## As regras

### 1. Bloco é `{ }`, e a quebra de linha não significa nada

Não existe fechamento com nome repetido, e não existe indentação significativa.
Um valor é sempre consumido pelo `nome:` que vem antes dele, então um
identificador solto só pode ser um irmão novo — o que faz a lista de atributos
quebrar onde você quiser, sem vírgula nem contrabarra:

```
rangeslider {
  start: preco_min end: preco_max
  min: 0 max: 1000 step: 10 size: 300
  on_release: lacou
}
```

A indentação é de **dois espaços**, como no `.gv` e no `.gss`, mas por convenção:
recortar e colar entre níveis é feiúra, nunca bug.

### 2. Antes das chaves vai o SELETOR; dentro vai todo o resto

O que precede o `{` não é "nome mais um atributo": é a mesma grafia que o `.gss`
usa para selecionar — tag, classes e id.

```
text.nota "…"                 /* ↔  .nota { … }  no .gss */
container.center-xy.fill#exportar { … }
```

Atributo **nunca** fica fora das chaves. Sem filhos, o bloco cabe numa linha:

```
shortcut { key: ctrl+s on_press: salvar }
```

Com filhos, as propriedades vão no topo, uma linha em branco, e os filhos
depois:

```
splitter.divide {
  sizes: painel_h min: 120 handle: 6

  column.painel { … }
}
```

A regra em uma frase: **cabeçalho ou bloco, nunca metade e metade.** É o que
impede o `{` de ficar órfão no fim de uma lista de atributos quebrada em três
linhas.

### 3. Classes encadeiam, e a ordem importa

```
column.bloco.bloco-cresce        /* ↔  class="bloco bloco-cresce" */
```

Em CSS a ordem das classes no atributo não diz nada. Aqui diz: `resolve_classes`
(`src/stylesheet.rs:375`) aplica **da esquerda para a direita, a última
sobrescrevendo a anterior**. A corrente expõe essa cascata na direção em que ela
de fato resolve, em vez de escondê-la dentro de uma string.

Os três níveis seguem o do motor: **tag < classe < id**.

Quando o valor é **dado** e não nome literal, a abreviação não serve e volta-se
ao atributo por extenso:

```
button.btn { class: "@estado" }    /* .btn literal + a variação do contexto */
button { id: @c.id }
```

### 4. `CamelCase` é componente do app; minúscula é widget do motor

Idêntico ao `.gv`, e pela mesma razão obrigatória: a tag de um componente é
resolvida pelo nome com que ele foi registrado, e a busca é sensível a caixa.
`MeuCartao` funciona; `meucartao` é `UnknownComponent`.

### 5. `@chave` é DADO; nome nu é NOME DE CHAVE

Esta é a regra que mais rende, e não é economia de tecla. No `.gv` a diferença
entre passar o nome de uma chave e passar o valor dela são dois caracteres
dentro das aspas:

```xml
value="x"      <!-- o nome da chave -->
value="{x}"    <!-- o valor dela -->
```

É a armadilha que o `PRIMITIVAS.md` chama de a família de bug mais cara deste
motor (`value="{x}"` num `<progressbar>` faz o widget procurar uma chave chamada
"42"). No `.gvb` ela vira uma marca visível no meio da linha:

```
value: preco_min      /* o nome da chave */
value: @preco_min     /* o valor dela   */
```

Formas completas: `@{chave}` quando o nome encosta em letra (`"@{preco}reais"`),
e `@@` para um arroba literal. O sigilo **não** é `$` de propósito — `R$ $preco`
num arquivo brasileiro é ilegível.

O outro efeito é que `{` e `}` passam a ter um trabalho só. `else if @aba ==
"teclado" {` não tem chave nenhuma disputando sentido com a do bloco.

### 6. Texto é o corpo, não atributo

Uma string solta depois do seletor é o filho de texto:

```
text.titulo "Onda 9 — o ponteiro preso"
text.valor "R$ @preco_min — R$ @preco_max"
```

Prosa longa vai em `"""`, desindentada e com o espaço em branco colapsado pela
mesma regra de hoje (`UiNode::normalize_text`), então quebrar a linha onde der é
de graça:

```
text.nota """
  Não existe página -1 nem página 3: o `Alvo::Indice` do grip prende o
  resultado em [0, n-1], como a `Alvo::Trilha` prende a largura no piso.
"""
```

O texto fica **fora** das chaves porque é conteúdo, não propriedade — é a única
coisa além do seletor que fica. E, como o bloco não é XML, `<splitter>` se
escreve `<splitter>`: os 24 `&lt;` do arquivo somem.

`content:` existe no motor, mas é justamente a forma que a regra 2 do
`CLAUDE.md` manda não escrever.

### 7. Comentário é `//` ou `/* … */`

Os dois convivem. Podem citar tags à vontade, e o `--` não é proibido em lugar
nenhum.

### 8. Condicional é notação, não semântica nova

Mapeamento fechado com o que o parser já lê:

| `.gvb` | `.gv` |
|---|---|
| `if @aba == "paineis" { … }` | `<template if="{aba}" equals="paineis">` |
| `if @aba != "paineis" { … }` | `not_equals` |
| `if "api" in @marcados { … }` | `contains="api"` |
| `if @marcados is empty { … }` | `empty="true"` |
| `if @ligado { … }` | truthy |
| `} else if … {` / `} else {` | `<elseif>` / `<else>` |
| `each @servicos as s { … }` | `<foreach items="servicos">` |

A chave é o que torna o encadeamento natural, e o caso trivial cabe numa linha:

```
if "api" in @marcados { badge { badge_text: "api no laço" } }
```

---

## Duas armadilhas

**Valor sem aspas com `//` dentro vira comentário.** `href: https://x` perde
tudo a partir da segunda barra. URL se escreve entre aspas.

**Vírgula dentro de um valor pede aspas**, porque ela é o separador dos
conjuntos que o `contains` lê:

```
items: "norte,nordeste,centro-oeste,sudeste,sul"
```

---

## O que falta para rodar

**Dessugarar para XML preservando o número de linha**, e chamar o
`parse_xml_with_source` que já existe (`src/parser.rs:6318`). Cada linha `.gvb`
emite na mesma linha do XML gerado — é o truque que o `protect_style_bodies`
(`src/parser.rs:6577`) já usa para posicionar erro de `.gss` inline. Com isso,
`eval`, builtins, diagnósticos e os 115 `.gv` do repositório não são tocados, e
o `.gvb` nasce com as mensagens de erro apontando para o lugar certo. Um parser
direto para `UiNode` pode vir depois, se valer.

Junto disso: um `glacier fmt --to-gvb` (a conversão é mecânica nos dois
sentidos) e o `language-configuration` — este já existe.

---

## Por que blocos, e não indentação

A primeira versão desta proposta era indentada, sem delimitador nenhum, e
ganhava: 224 linhas contra as 324 desta. Perdeu por um eixo só, e é o eixo que
importa mais que o tamanho: **`.gv` gerado ou editado por ferramenta.** Se
`glacier fmt` tem de ser um formatador determinístico, se script ou agente vai
emitir tela, se colar um bloco de outro arquivo não pode quebrar nada — então a
insensibilidade a espaço em branco vale as 100 linhas, porque com chaves uma
indentação corrompida é feiúra e sem elas é bug.
