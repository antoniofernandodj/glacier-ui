# CLAUDE.md

Instruções para quem trabalha neste repositório — pessoa ou agente.

O `PLANO_WIDGETS.md` é o planejamento de longo prazo; `BUILTINS.md` e
`PRIMITIVAS.md` dizem como escrever um widget; `DIALOGS.md`, `ANIMACOES.md` e
`TROUBLESHOOTING.md` cobrem os assuntos que o nome anuncia. Este arquivo é só a
convenção de escrita, e ela vale para **todo `.gv` e todo `.gss`** do
repositório — os que já existem e os que vierem.

## A convenção de markup (`.gv`)

### 1. Tags do motor em minúsculas; componentes do app com o nome registrado

```xml
<!-- não -->
<Column spacing="10"><TextInput onChange="salvar" minSize="200 100" /></Column>

<!-- sim -->
<column><textinput on_change="salvar" min_size="200 100" /></column>
```

Vale para **primitivas e builtins** — o motor aceita `CamelCase`, `camelCase` e
`kebab-case` como apelidos, e é por isso que a regra precisa estar escrita: nada
quebra se você misturar, e um arquivo com quatro grafias da mesma coisa é o
resultado natural de não decidir.

**A exceção é obrigatória, não estilística:** a tag de um componente do app é
resolvida pelo **nome com que ele foi registrado**, e a busca é sensível a
caixa. `<MeuCartao/>` funciona; `<meucartao/>` é `UnknownComponent`.

Isso dá à convenção uma propriedade que vale de graça: numa tela, **`CamelCase`
significa "componente deste app"** e minúscula significa "widget do motor". Dá
para saber o que é seu só de olhar.

```xml
<column class="lista">      <!-- motor -->
  <CartaoServico nome="api" />  <!-- do app -->
</column>
```

### 2. O texto de um `<text>` é FILHO, não atributo

```xml
<!-- não -->
<text content="Serviços ativos" />

<!-- sim -->
<text>Serviços ativos</text>
```

Interpolação continua valendo no filho (`<text>Olá, {usuario}</text>`). O
atributo `content` segue funcionando e não vai sumir; só não é a forma que se
escreve aqui.

### 3. Estilo mora no `.gss`, markup é estrutura

Cor, tamanho, espaçamento, padding e largura de caixa saem do `.gv`. No lugar
deles, uma **classe com nome de papel**:

```xml
<!-- não -->
<text size="12" color="#7F849C">Último salvamento</text>

<!-- sim -->
<text class="rotulo">Último salvamento</text>
```

```gss
.rotulo { size: 12; color: var(--fraco); }
```

**Três coisas continuam inline, porque não são estilo:**

- **valor dirigido por dado** — `background="{cor}"` é o dado sendo mostrado, não
  decoração;
- **medida com significado de widget** — o `size` de um `<colorwheel>` é o
  diâmetro da roda, o `width` de um `<spinbox>` é a largura do **campo** (ver a
  armadilha no `PRIMITIVAS.md`). Movê-los para o `.gss` os confunde com largura
  de caixa;
- **estrutura e ação** — `value`, `items`, `slot`, `on_click`, `steps`.

### 4. Indentação de dois espaços

Nos dois arquivos, `.gv` e `.gss`.

## A mesma regra no `.gss`

Propriedades em `snake_case`: `border_radius`, `border_width`, `border_color`,
`align_x`, `align_y`, `text_align`, `text_color`, `max_width`, `max_height`,
`font_family`. O motor aceita a forma com hífen como apelido; a forma escrita
aqui é a com sublinhado, para casar com os atributos do `.gv`.

Uma exceção, e ela é obrigatória: dentro de `@media` o nome é uma **feature do
CSS**, não uma propriedade do motor, e só a forma com hífen é aceita.

```gss
@media (max-width: 720) { … }   /* sim */
@media (max_width: 720) { … }   /* erro: Unsupported @media feature */
```

Cores nomeadas em `:root`, usadas por `var(--nome)` — nunca um hexadecimal
repetido em cinco regras.

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

- o ramo que deveria existir está lá? (uma escada de `se`/`senao` que não casa
  não é erro, é um buraco);
- o `value_var` do nó é o **nome de chave** que você escreveu, ou o valor já
  interpolado? (`value="{x}"` num `<progressbar>` faz o widget procurar uma
  chave chamada "42");
- a largura declarada é um número, ou um `fill` dentro de um pai `shrink`?

E, quando der, rode o exemplo: `cargo run --example <nome>`. Nesta máquina o
Vulkan da GPU integrada está quebrado — use `WGPU_BACKEND=gl` antes de culpar o
código.
