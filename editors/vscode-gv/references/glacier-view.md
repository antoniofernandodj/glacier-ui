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
Rótulo/etiqueta embutido de `src/builtins.rs`. Ver `BUILTINS.md` para estender.

### `<TimePicker>`
Campo de hora (`HH:MM`) com botão de seleção. Props: `value` (chave de contexto), `on_change`, `on_pick`, `placeholder`, `width`, `pick_icon`.

---

Componentes do **app** são qualquer tag desconhecida (ex.: `<PerfilCard/>`),
resolvida pelo nome — declarada em outro `.gv`, por `<import from>`, ou por
`register_component("Nome", "caminho")` no Rust.
