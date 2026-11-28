# CLAUDE.md

Instruções para quem trabalha neste repositório — pessoa ou agente.

O `docs/arquivados/PLANO_WIDGETS.md` (os planos concluídos moram em `docs/arquivados/`) é o planejamento de longo prazo; `BUILTINS.md` e
`PRIMITIVAS.md` dizem como escrever um widget; `DIALOGS.md`, `ANIMACOES.md` e
`TROUBLESHOOTING.md` cobrem os assuntos que o nome anuncia. Este arquivo é só a
convenção de escrita, e ela vale para **todo `.gv` e todo `.gss`** do
repositório — os que já existem e os que vierem.

## A convenção de markup (`.gv`)

### 1. Tags do motor em minúsculas; componentes do app com o nome registrado

```gv
// não
column(spacing = 10) {
  textinput(on_change = salvar, min_size = "200 100")
}
// sim
column {
  textinput(on_change = salvar, min_size = "200 100")
}
```

Vale para **primitivas e builtins**: o motor só conhece a grafia em minúsculas
(`textinput`, `min_size`); qualquer outra é `UnknownComponent` ou um atributo
ignorado com aviso.

**A exceção é obrigatória, não estilística:** a tag de um componente do app é
resolvida pelo **nome com que ele foi registrado**, e a busca é sensível a
caixa. `MeuCartao` funciona; `meucartao` é `UnknownComponent`.

Isso dá à convenção uma propriedade que vale de graça: numa tela, **`CamelCase`
significa "componente deste app"** e minúscula significa "widget do motor". Dá
para saber o que é seu só de olhar.

```gv
column(class = lista) {  // motor
  CartaoServico(nome = api)  // do app
}
```

### 2. O texto de um `text` é o primeiro argumento, não o atributo `content`

```gv
// não
text(content = "Serviços ativos")
// sim
text("Serviços ativos")
```

Interpolação continua valendo (`text("Olá, {usuario}")`). O
atributo `content` segue funcionando e não vai sumir; só não é a forma que se
escreve aqui.

### 3. Estilo mora no `.gss`, markup é estrutura

Cor, tamanho, espaçamento, padding e largura de caixa saem do `.gv`. No lugar
deles, uma **classe com nome de papel**:

```gv
// não
text("Último salvamento", size = 12, color = #7F849C)
// sim
text("Último salvamento", class = rotulo)
```

```gss
.rotulo { size: 12; color: var(--fraco); }
```

**Três coisas continuam inline, porque não são estilo:**

- **valor dirigido por dado** — `background = @cor` é o dado sendo mostrado, não
  decoração;
- **medida com significado de widget** — o `size` de um `colorwheel` é o
  diâmetro da roda, o `width` de um `spinbox` é a largura do **campo** (ver a
  armadilha no `PRIMITIVAS.md`). Movê-los para o `.gss` os confunde com largura
  de caixa;
- **estrutura e ação** — `value`, `items`, `slot`, `on_click`, `steps`.

### 4. Indentação de dois espaços

Nos dois arquivos, `.gv` e `.gss`.

## A mesma regra no `.gss`

Propriedades em `snake_case`: `border_radius`, `border_width`, `border_color`,
`align_x`, `align_y`, `text_align`, `text_color`, `max_width`, `max_height`,
`font_family`. O motor só aceita a forma com sublinhado, a mesma dos atributos do `.gv`.

Uma exceção, e ela é obrigatória: dentro de `@media` o nome é uma **feature do
CSS**, não uma propriedade do motor, e só a forma com hífen é aceita.

```gss
@media (max-width: 720) { … }   /* sim */
@media (max_width: 720) { … }   /* erro: Unsupported @media feature */
```

Cores nomeadas em `:root`, usadas por `var(--nome)` — nunca um hexadecimal
repetido em cinco regras.

## A linguagem: expressões, duas formas, cor (0.169+)

### Desde a 0.170 (as frentes do `TODO.md`)
- **O `:` decide o valor** (0.170): `:attr = expr` é uma via, `::attr = @chave` é
  o vínculo nos dois sentidos (não escreva o trio `form_control` + `:value` +
  `on_change = field:x`), `:class.ativa = expr` é a classe condicional,
  `class = [a, b]` e `size = [1280, 820]` são listas, `@x ?? "padrão"` substitui
  `{x|padrão}`. As formas antigas ainda rodam e são avisadas pelo `check`.
- **Ação é sempre chamada**: `on_click = salvar()`, `window.minimize()`,
  `field("nome")`. `salvar` sem parênteses é depreciado; o `check` confere nome,
  aridade e tipo contra o `--!strict` do `.luau`.
- **Tipos de verdade**: `ctx.x = true` é lido como bool pelo template (`if @x`,
  `@n > 3`, `@lista is empty`) — não escreva `@x == "true"` nem `json.encode`
  para falar com a view. `app(truthy = typed)` é a regra nova de verdade.
- **`state { … }`** declara o estado local de um `component`; `@app.x` é o global
  dito com todas as letras. Um componente usado num arquivo precisa de `link(rel =
  import, …)` naquele arquivo.
- **Botão ocupado**: `app(busy_buttons = true, busy_release = toast)` cobre submit,
  reentrada e `busy.release()`; `spinner(if = @__busy.salvar)`.
- **`glacier fmt` reindenta** (2 espaços, comentários e `"""` intactos,
  idempotente) e `fmt --check` é porta de CI.


A gramática inteira está em **`docs/LINGUAGEM.md`**; o porquê, em
`docs/PLANO_LINGUAGEM_GV_GSS.md`. O que muda a escrita do dia a dia:

- **Condição é expressão**: `if @a && !@b`, `if = @n > 0`, `@tipo in ["a", "b"]`,
  `@lista is empty`. Não escreva `equals =`/`not_equals =`/`one_of =` (depreciados;
  `glacier fmt --migrate` reescreve).
- **Duas formas por conceito**, cada uma com seu papel: a curta num nó (`if =`,
  `each = @l as v`, e `else` **logo depois** do irmão com `if =` — senão é erro) e a de
  bloco (`if … { } else { }`, `each @l as v { }`). `template(if/:for_each)` saiu.
- **Classe condicional** `class:x--on = expr` em vez de repetir o nó para trocar uma
  classe; **atributo com ternário** (`src = @a ? "x" : "y"`).
- **Ação com argumentos**: `on_click = salvar(@item.id, true)`. O `handler:arg` está
  depreciado (fica para os embutidos: `window:drag`, `field:x`).
- **i18n curto**: `text(t("tela.chave"))`.
- **Props com tipo**: `prop(name = ativo, type = bool, default = false)`.
- **`.gss`**: aninhamento com `&` (`&:hover`, `&--primario`), `:not()`, `:is()`,
  `font_weight`, `line_height`, `box_shadow`, `@theme` (a paleta num lugar só),
  `display: none` (não `hidden`). Propriedade em `snake_case`; variável e `@media` com
  hífen.
- **Cor**: em `app(gss_color = css)`, `background` é fundo e `color` é conteúdo em
  todo widget. Sem isso vale o sentido antigo (o `color` de um `button` é o fundo).

Antes de dizer que um `.gv`/`.gss` está certo: `glacier check --strict`; antes de dizer
que uma tela desenha: `glacier test --layout`.

## O que essa convenção protege

Uma propriedade de estilo que o motor não conhece é **ignorada com aviso**, não
é erro. Um `border-bottom:` (que não existe — a borda do `UiNode` é dos quatro
lados) some em silêncio no meio de um `.gss` grande. Concentrar o estilo num
arquivo só, com nomes de papel, é o que torna esse aviso fácil de ver e a
correção fácil de fazer num lugar só.

O mesmo vale do lado do markup: quanto menos atributo por tag, mais óbvio fica
quando um deles **não é** o que parece — que é a família de bugs mais cara deste
motor (ver "Quatro armadilhas que falham EM SILÊNCIO" no `PRIMITIVAS.md`).

## Antes de dizer que funciona

Este repositório já produziu, mais de uma vez, um widget que passava em todos os
testes e aparecia **vazio na tela**. O padrão é sempre o mesmo: o teste escreve
uma chave de contexto e lê a mesma chave de volta, o que passa mesmo com o
widget quebrado.

Um teste de widget precisa olhar a **árvore avaliada**:

- o ramo que deveria existir está lá? (uma escada de `if`/`else` que não casa
  não é erro, é um buraco);
- o `value_var` do nó é o **nome de chave** que você escreveu, ou o valor já
  interpolado? (`value = @x` num `progressbar` faz o widget procurar uma
  chave chamada "42");
- a largura declarada é um número, ou um `fill` dentro de um pai `shrink`?

E, quando der, rode o exemplo: `cargo run --example <nome>`. Nesta máquina o
Vulkan da GPU integrada está quebrado — use `WGPU_BACKEND=gl` antes de culpar o
código.
