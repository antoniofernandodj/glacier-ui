# Glacier View (`.gv`) — referência de sintaxe

Markup XML do glacier-ui. Uma tela é uma árvore de tags; atributos configuram cada
widget; `{var}` interpola o contexto; `<script>` carrega o comportamento em Lua;
`<style>` carrega GSS.

## Estrutura

```gv
<Container padding="24" background="#2E3440">
  <Column spacing="16" align="Center">
    <Text content="Olá {nome}" size="22" />
    <Button text="Clique" on_click="fazer_algo" />
  </Column>
</Container>

<script>
function fazer_algo()
  ctx.nome = "mundo"
end
</script>
```

- **Interpolação**: `content="Valor: {contador}"` — lê `ctx.contador`. `{k|default}` usa `default` quando a chave falta.
- **Ações**: `on_click`, `onClick`, `on_change`/`onChange`, `on_toggle`, `on_submit`, `on_reorder`, `on_open`/`on_message`/`on_error`/`on_close`, mais as variantes `ao_*`/`aoX`. O valor é o **nome de uma função** definida no `<script>` (inline ou `src=`).
- **Comportamento**: `<script>…</script>` (Lua inline) ou `<script src="arquivo.luau"></script>` (externo, relativo ao template).
- **Estilo**: `<style>…</style>` (GSS inline) ou `<link rel="stylesheet" href="app.gss"/>`. Ver a extensão *Glacier GSS*.
- **Cabeçalho** (obrigatório em arquivo, desde a 0.61): `<screen title="…" size="960 700">` (uma janela) ou `<component>` (o resto) como raiz, envolvendo o arquivo inteiro — um `<resources>` agrupando `<style>`/`<script>`/`<link>`/`<import>`, um `<props>` opcional e o layout depois. Ver `<Screen>` abaixo.

---

## Widgets primitivos

### `<Container>`
Caixa única com padding/fundo/borda. Aceita um filho. Atributos: `padding`, `background`, `width`, `height`, `alignX`, `alignY`, `border-radius`, `border-width`, `border-color`.

### `<Column>`
Empilha filhos na vertical. Atributos: `spacing`, `align` (eixo cruzado = X), `width`, `height`.

### `<Row>`
Empilha filhos na horizontal. Atributos: `spacing`, `align` (eixo cruzado = Y).

### `<Text>` (`<Span>`)
Texto. Conteúdo via `content="…"` ou filho `<Text>…</Text>`. Atributos: `size`, `bold`, `color`.

### `<Button>` (`<Botao>`)
Botão. Atributos: `text`, `on_click`, `navigateTo`, `navigateBack`, `color`.

### `<TextInput>` (`<Input>`)
Campo de texto de uma linha. Atributos: `value`, `placeholder`, `onChange`, `secure`, `formControl`.

### `<TextArea>` (`<Editor>`)
Editor multilinha. Atributos: `value`, `placeholder`, `onChange`.

### `<Image>` (`<Imagem>`)
Imagem. Atributos: `source`/`src`, `clip="Circle"`.

### `<Svg>` (`<Icon>`)
Ícone/SVG. Atributos: `source`/`src`, `color`.

### `<Scrollable>` (`<Scroll>`)
Área rolável. Atributos: `direction` (`vertical`/`horizontal`).

### `<Checkbox>` (`<Check>`)
Caixa de seleção. Atributos: `label`, `checked`, `onToggle`.

### `<Toggle>` (`<Switch>`)
Interruptor. Atributos: `label`, `checked`, `onToggle`.

### `<Rule>` (`<Divider>`)
Divisória. Atributos: `direction`.

### `<ProgressBar>` (`<Progress>`, `<BarraProgresso>`)
Barra de progresso. Atributos: `value` (chave de contexto com o valor), `min` (0), `max` (100), `vertical`, `showValue`, `color`.

### `<Spinner>` (`<BusyIndicator>`, `<Carregando>`)
Indicador de atividade indeterminada. Atributo: `color`.

### `<Select>` (`<Dropdown>`, `<ComboBox>`)
Seletor. Atributos: `options`, `value`, `onChange`, `placeholder`, `labelField`, `valueField`, `color`.

### `<ComboEdit>` (`<EditableCombo>`, `<ComboEditavel>`)
Combo editável: campo de texto com lista de sugestões. Atributos: `options`, `value`, `placeholder`, `onChange`, `onSelect`.

### `<Form>` (`<Formulario>`)
Formulário. Atributos: `onSubmit`, `name`. Envolve `formControl`s.

---

## Controle de fluxo e composição

### `<ForEach>` (`<For>`)
Repete o corpo por item. Atributos: `items`, `var`.

### `<If>` (`<Se>`) / `<Else>` (`<Senao>`)
Condicional. `<If>` aceita `cond`, `equals`, `notEquals`.

### `<ElseIf>` (`<SenaoSe>`)
Ramo intermediário entre um `<If>` e o `<Else>`. Mesmos atributos de condição do `<If>`.

### `<template>` (`<gabarito>`)
Tag única que unifica repetição e condição: com `for-each`/`items` repete como `<ForEach>`; com `if`/`equals`/`one-of`/… condiciona como `<If>`. Não desenha caixa nenhuma — só emite os filhos.

### `<Include>` (`<Incluir>`)
Inclui outro template. Atributo: `src`; demais atributos viram props.

### `<Import>` (`<Importar>`)
Registra um componente por nome. Atributos: `name`/`as`, `from`.

---

## Recursos externos

### `<Script>`
Comportamento em Lua: `<script>…</script>` (inline) ou `<script src="arquivo.luau"></script>` (externo, resolvido relativo ao template). O valor de cada ação é o nome de uma função definida aqui.

### `<Link>`
Recurso externo declarado no próprio template. `rel` escolhe o tipo: `stylesheet` (padrão, um `.gss` global), `import`/`component` (outro template, nomeado por `as`/`name`), `data` (JSON no contexto, sob a chave `as`/`name`) e `theme` (paleta JSON). Atributo do caminho: `href`.

### `<Style>`
`<style>…</style>` é GSS inline — global por padrão, restrito ao componente com `scoped="true"`. `<style href="…">` equivale a `<link rel="stylesheet">`.

---

## Cabeçalho da tela

### `<Screen>` (`<tela>`)
Raiz que declara os metadados da **janela** e separa o que não desenha do que desenha. Atributos: `title`/`titulo`, `size`/`tamanho` (`"960 700"`, `"960x700"`), `min-size`/`minSize`, `resizable`/`redimensionavel`. O template ganha do builder Rust; o título acompanha a navegação entre telas; o tamanho só é reaplicado no hot-reload quando o número muda no arquivo. Um `.gv` que é pedaço de tela usa `<Component>`, não este.

### `<Component>` (`<componente>`)
A mesma casca do `<Screen>` para um `.gv` que é pedaço de tela (importado por outro template), não janela. Agrupa declarações igual e **não leva atributo nenhum** — `title`/`size` ali seriam promessa sem efeito, e viram erro de parse com a explicação.

### `<Resources>` (`<recursos>`)
Dentro do cabeçalho, agrupa o que a tela precisa e não aparece: `<style>`, `<script>`, `<link>`, `<import>`. O que estiver fora dele (ainda dentro do cabeçalho) é o layout. É opcional — com uma ou duas declarações, elas podem ficar soltas dentro do cabeçalho.

### `<Props>` / `<Prop>`
O contrato de um `<Component>`: quais props ele aceita. `<prop name="label" />` é obrigatória; `<prop name="cor" default="#89B4FA" />` é opcional e o default entra quando quem chama omite. Passar uma prop não declarada é erro (a extensão marca no editor e completa os nomes ao digitar dentro da tag). Declarar é opcional: sem `<props>` nada é checado. Um `<props>` vazio é um contrato — "não aceito prop nenhuma". Não vale num `<Screen>`: ninguém *usa* uma janela, ela é aberta.

O cabeçalho não desenha nada, então engano ali vira **erro de parse** (com linha/coluna) em vez de silêncio: arquivo sem cabeçalho, cabeçalho que não envolve o arquivo inteiro (irmão do layout ou aninhado nele), atributo desconhecido, `size`/`min-size` que não seja par de números, `resizable` não booleano, widget dentro do `<Resources>`, `<Resources>`/`<Props>` fora de um cabeçalho, `<Prop>` sem `name` ou repetido.

---

## Ações built-in

Ações tratadas pelo próprio motor (`GlacierUI::dispatch`), sem código no
componente. O que vem depois do `:` é uma **chave de contexto** nas quatro
primeiras, e um comando nas demais.

| Ação | Efeito |
| --- | --- |
| `clipboard:<chave>` | copia o valor de contexto `<chave>` para a área de transferência |
| `open:<alvo>` | abre no navegador do SO — `<alvo>` é uma chave de contexto ou, se ela não existir, a própria URL |
| `textarea_end:<binding>` | rola o `<TextArea>` de `binding` até o fim (e leva o cursor pro fim) |
| `textarea_top:<binding>` | o par do anterior: rola até o topo |
| `window:minimize` / `window:maximize` / `window:close` | controles da janela (`window:toggle_maximize` é alias de `maximize`) |
| `window:drag` | inicia o arraste — use no `onPress` de uma região da barra de título |
| `window:resize:<dir>` | inicia o redimensionamento; `<dir>` ∈ `n,s,e,w,ne,nw,se,sw` |
| `style:<nome>` | troca o estilo builtin ativo (ver `src/style.rs`); `style:set` é reservado para a forma `<Select onChange="style:set">` |

Qualquer outro valor de ação é o nome de uma função: exata, ou `nome:sufixo` —
sem uma função `nome:sufixo`, o motor chama `nome(sufixo, value)`.

---

## Builtins (registrados pela lib)

### `<Badge>`
Rótulo/etiqueta ("pílula") embutido. Props: `badge_text` (`Badge`), `badge_bg` (`#89B4FA`), `badge_fg` (`#11111B`), `badge_size` (`13`). Ver `BUILTINS.md` para estender.

### `<SpinBox>`
Campo numérico com os degraus de somar/subtrair — o `QSpinBox` do Qt. Clicar soma ou subtrai `step`, saturando em `min`/`max`; a aritmética roda no widget, em Rust, **sem código do lado do app**.

```gv
<SpinBox value="quantidade" min="1" max="99" />
<SpinBox value="preco" min="0" max="10" step="0.25" width="90" />
<SpinBox value="zoom" min="25" max="400" step="25" layout="inline" />
```

| prop | default | o que faz |
| --- | --- | --- |
| `value` | — (**obrigatória**) | **nome** da chave de contexto onde o número mora. É o que torna duas instâncias independentes: o widget não guarda estado, escreve na chave que o app nomeia |
| `min` / `max` | `0` / `100` | limites; o clique satura neles (a faixa padrão do Qt) |
| `step` | `1` | passo de cada clique. As **casas decimais da saída saem daqui**: `step="0.25"` formata com 2 casas — é o `QDoubleSpinBox` sem um segundo widget |
| `layout` | `stacked` | `stacked`: as setinhas `▴▾` empilhadas e coladas à direita do campo (o `QSpinBox` clássico). `inline`: `−  campo  +`, degraus nas pontas (o `SpinBox` do Qt Quick Controls) |
| `width` | `72` | largura do campo |
| `placeholder` | vazio | dica quando a chave está vazia |
| `dec_text` / `inc_text` | `▾`/`▴` (stacked), `−`/`+` (inline) | glifos dos degraus |
| `glyph_size` | `11` (stacked), `15` (inline) | corpo do glifo |

- **Chave vazia**: o primeiro clique inicializa no `min` (não em `min + step`).
- **Digitação** entra filtrada (só dígitos, um `-` à frente e um `.`) e **sem saturar** — como o `QSpinBox`, que só valida ao terminar a edição; o clique seguinte satura.
- **Sem `on_change` para o app**: o `onChange` do campo interno é usado pelo próprio widget para filtrar a digitação. Quem precisa reagir lê a chave de `value`.
- **Aparência**: os degraus são pintados por uma folha GSS que o próprio widget declara, na classe `.spinbox-step` (com `:hover`/`:active`). Ela é instalada antes de qualquer `.gss` do app, então redefinir `.spinbox-step` numa folha sua vence e repinta os degraus.

### `<TimePicker>`
Campo de hora (`HH:MM`) com botão de seleção. Props: `value` (chave de contexto), `on_change`, `on_pick`, `placeholder`, `width`, `pick_icon`.

---

Componentes do **app** são qualquer tag desconhecida (ex.: `<PerfilCard/>`),
resolvida pelo nome — declarada em outro `.gv`, por `<import from>`, ou por
`register_component("Nome", "caminho")` no Rust.
