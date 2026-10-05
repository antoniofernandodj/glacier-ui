# `.gvb` — o markup de blocos

**Rascunho.** Nada disto roda: o motor lê `.gv` (XML) e só. O que existe hoje é
o realce de sintaxe na extensão de VS Code (`glacier-view-block`) e o
`rascunhos/onda9.gvb`, que é o `examples/gva/onda9/app.gva` convertido inteiro — a
tela mais densa do repositório, escolhida de propósito para a proposta não ser
medida num exemplo fácil.

O alvo é o **desconforto de escrever**, não o de rodar. Por isso a regra que
manda em tudo: **o `.gvb` não muda uma vírgula da semântica do motor.** Ele
dessugara para a mesma `UiNode` que o `.gv` produz, e toda diferença aqui é de
grafia.

---

## Por que não o XML

No `examples/gva/onda9/app.gva`, 297 linhas:

- **88 fechamentos** que repetem o nome da tag;
- **24 `&lt;`/`&gt;`**, porque a prosa dos comentários e dos `<text>` cita tags
  o tempo todo e o XML não deixa;
- **82 `class="…"`**, uma string por elemento para dizer o papel dele (no
  `.gvb` continuam sendo 82, só que como `class = nome`);
- 12 blocos de `<!-- -->`, onde `--` é proibido no meio;
- 8 linhas passando de 80 colunas, sem jeito de quebrar.

O mesmo arquivo em `.gvb`: 389 linhas, 13933 bytes (+0,4%), **nenhuma linha acima
de 80 colunas**. Ganha-se quebra livre e perde-se em número de linhas — 52 delas
são só `}`, e `class = nome` custa mais que `.nome` custaria. É a troca consciente da proposta, e a seção final explica por que ela
compensa.

---

## As regras

### 1. Filhos são `{ }`, atributos são `( )`, e a quebra de linha não significa nada

Não existe fechamento com nome repetido, e não existe indentação significativa.
Os atributos são `nome = valor` separados por **vírgula** (a última é opcional),
e é a vírgula — não o espaço — que decide onde um acaba, então a lista quebra
onde você quiser, sem contrabarra:

```
rangeslider(
  start = preco_min, end = preco_max,
  min = 0, max = 1000, step = 10, size = 300,
  on_release = lacou,
)
```

Era `tag { nome: valor … }` (atributos soltos dentro das chaves, separados só por
espaço). Foi trocado porque a chave fazia dois trabalhos — atributos e filhos
no mesmo bloco — e porque a chamada `tag(a = 1, b = 2)` já é lida assim em
qualquer linguagem: o que está nos parênteses descreve a tag, o que está nas
chaves está dentro dela. Escrever `nome: valor` entre chaves é erro, com a dica.

A indentação é de **dois espaços**, como no `.gv` e no `.gss`, mas por convenção:
recortar e colar entre níveis é feiúra, nunca bug.

### 2. A forma é `tag(atributos) { filhos }`

Cada parte é opcional, e **atributo nunca fica fora dos parênteses**. Não existe
`tag.classe` nem `tag#id`: o ponto e o cerquilha não têm significado ali
(escrevê-los é um erro, com a dica). A forma `objeto.atributo` das linguagens
orientadas a objeto é a razão da escolha: lá o ponto **acessa** algo que o objeto
tem, e numa abreviação de seletor ele significaria "etiqueta", que é uma leitura
que cada pessoa precisa aprender.

```
text(class = nota, content = "…")
container(class = "center-xy fill", id = exportar) { … }
```

Sem filhos, não há chaves:

```
shortcut(key = ctrl+s, on_press = salvar)
```

Com filhos, o cabeçalho fecha no `)` e as chaves trazem só eles:

```
splitter(class = divide, sizes = painel_h, min = 120, handle = 6) {
  column(class = painel) { … }
}
```

Quando o cabeçalho não cabe em 80 colunas, ele quebra em grupos de linha e o `)`
fecha sozinho, sem deixar um `{` órfão no fim de uma lista de três linhas:

```
toolbutton(
  icon = "📄", text = Novo, layout = beside, tooltip = "Novo documento",
  on_click = novo,
) {
  …
}
```

### 3. `class` e `id` são atributos como os outros

```
column(class = "bloco bloco-cresce")   /* ↔  class="bloco bloco-cresce" */
button(class = "btn @estado")          /* a variação vem do contexto */
button(id = @c.id)
```

Uma classe só dispensa as aspas (`class = nota`). Em CSS a ordem das classes não
diz nada, mas aqui diz: `resolve_classes` (`src/stylesheet.rs:375`) aplica **da
esquerda para a direita, a última sobrescrevendo a anterior**. Os três níveis
seguem o do motor: **tag < classe < id**.

### 4. `CamelCase` é componente do app; minúscula é widget do motor

Idêntico ao `.gv`, e pela mesma razão obrigatória: a tag de um componente é
resolvida pelo nome com que ele foi registrado, e a busca é sensível a caixa.
`MeuCartao` funciona; `meucartao` é `UnknownComponent`.

### 5. Texto, valor e ligação: `x`, `@x` e `:attr = x`

Esta é a regra que mais rende, e não é economia de tecla. Há três coisas que um
atributo pode receber, e no `.gv` duas delas se escreviam quase igual, dentro das
aspas:

```xml
value="x"      <!-- o nome da chave… ou o texto "x"? o motor decide pelo widget -->
value="{x}"    <!-- o valor dela -->
```

É a armadilha que o `PRIMITIVAS.md` chama de a família de bug mais cara deste
motor (`value="{x}"` num `<progressbar>` faz o widget procurar uma chave chamada
"42"). O estado da tela são gavetas com etiqueta: a **chave** é a etiqueta, o
**valor** é o que está dentro. O `.gvb` dá uma grafia a cada coisa:

```
align = center           /* texto: sempre literal */
content = @preco_min     /* o VALOR da chave, mostrado e atualizado */
:value = preco_min       /* a LIGAÇÃO: o nome da chave (o widget lê e grava) */
```

A marca da ligação fica no **nome do atributo** (`:value`, como no Vue), porque é
o atributo que é uma ligação, e não o valor que ele recebe. É a mesma no `.gva`
(`<progressbar :value="x" />`). Antes de ler o XML, o motor troca `:value` por
`bind_value`, dois atributos de nome diferente, e por isso consegue cobrar a
marca: no `.gvb`, um atributo
que é ligação (a lista está em `src/bindings.rs`) **sem o `:` é erro**, com a
linha; no `.gva` é um aviso. `on_click = salvar` não leva marca: é o nome de uma
função.

Formas completas: `@{chave}` quando o nome encosta em letra (`"@{preco}reais"`),
e `@@` para um arroba literal. O sigilo do valor **não** é `$` de propósito —
`R$ $preco` num arquivo brasileiro é ilegível.

O outro efeito é que `{` e `}` passam a ter um trabalho só. `else if @aba ==
"teclado" {` não tem chave nenhuma disputando sentido com a do bloco.

### 6. Texto é atributo, sempre

O texto de um `<text>` é o atributo `content`, e o de um `<button>`, `text`.
Não existe texto solto depois do cabeçalho — escrevê-lo é erro, com a dica —,
então um elemento é sempre `tag(tudo o que o descreve) { filhos }`, e o rótulo
não fica pendurado depois do `)`:

```
text(class = titulo, content = "Onda 9 — o ponteiro preso")
button(on_click = ir_monitor, color = #5E81AC, padding = "10 18", text = "Ir")
```

Prosa longa vai em `"""`, desindentada e com o espaço em branco colapsado pela
mesma regra de hoje (`UiNode::normalize_text`), então quebrar a linha onde der é
de graça:

```
text(
  class = nota,
  content = """
    Não existe página -1 nem página 3: o `Alvo::Indice` do grip prende o
    resultado em [0, n-1], como a `Alvo::Trilha` prende a largura no piso.
  """,
)
```

Como o valor não é XML, `<splitter>` se escreve `<splitter>`: os 24 `&lt;` do
arquivo somem. A exceção é o corpo cru de `script`/`style` (`style """…"""`),
que é código, não texto, e fica depois do cabeçalho.

Uma diferença com o `.gva`: `<button>Salvar</button>` vira, no motor, um `Text`
dentro do botão; `text = "Salvar"` é o atalho que o motor já tem para o rótulo
simples. O `Text` filho só importa para quem estiliza o rótulo por seletor de tag.

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
if "api" in @marcados { badge(badge_text = "api no laço") }
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
