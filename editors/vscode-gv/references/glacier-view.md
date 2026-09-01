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

### `<Slider>` (`<Deslizante>`)
Cursor arrastável numa faixa — o `QSlider`. Como o `<TextInput>`, **não grava a chave sozinho**: dispara `onChange` com o valor novo e quem grava é o app.

```gv
<slider value="volume" min="0" max="100" onChange="ajustar" width="320" />
<slider value="brilho" min="0" max="1" step="0.05" onChange="mudar" />
<slider value="graves" min="-10" max="10" default="0" onChange="eq" />
```

| prop | default | o que faz |
| --- | --- | --- |
| `value` | — | **nome** da chave de contexto com o número |
| `min` / `max` | `0` / `100` | a faixa |
| `step` | `1` | granularidade do arraste. As **casas decimais da saída saem daqui**: `step="0.05"` grava `0.60`, não `0.6000000238418579` |
| `default` | — | valor para onde o **duplo clique** devolve o cursor |
| `onRelease` | — | ação disparada só ao SOLTAR, para quem não quer efeito colateral por pixel arrastado |
| `shiftStep` | — | passo fino com Shift segurado |
| `vertical` | `false` | usa o `vertical_slider` (peça uma `height`) |
| `color` | — | cor do trilho preenchido e do cursor |

- **`disabled` deixa inerte, mas não esmaece**: o `slider::Status` do iced 0.14 não tem `Disabled`. O cursor não se move porque a chave não muda.

### `<Radio>` (`<RadioButton>`, `<Opcao>`)
Uma opção de um grupo mutuamente exclusivo — o `QRadioButton`. O grupo **é a chave**, não um nó pai: `group` é o **nome** da chave (como o `checked` do `<Checkbox>`), e a opção fica marcada quando o valor guardado ali é igual ao `value` dela.

```gv
<radio label="Grátis" value="free" group="plano" onChange="escolher" />
<radio label="Pro"    value="pro"  group="plano" onChange="escolher" />
```

| prop | o que faz |
| --- | --- |
| `label` | o rótulo ao lado da bolinha |
| `value` | o valor que **esta** opção representa |
| `group` | **nome** da chave que guarda a escolha. Sem `{}`: escrever `group="{plano}"` passa o *valor* no lugar do nome, e aí nenhuma opção casa e o grupo inteiro aparece desmarcado |
| `onChange` | ação disparada no clique, com o `value` da opção junto |

- Como o `<Checkbox>`, **não grava sozinho** — quem grava é o app. Para o caso comum sem handler nenhum, use o builtin `<RadioGroup>`.

### `<Space>` (`<Espaco>`, `<Spacer>`)
Espaço vazio — o `QSpacerItem`. Sem `width`/`height` é `Fill` nos dois eixos (o espaçador **flexível**, que empurra o resto para a borda); com eles, um vão fixo.

```gv
<row><text content="esquerda" /><space /><text content="direita" /></row>
<row><text content="a" /><space width="80" /><text content="b" /></row>
```

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

### `<slot>` (`<conteudo>`)
O buraco que o conteúdo escrito **entre as tags** de um componente preenche: `<groupbox>…</groupbox>` renderiza esse `…` onde o template do `<groupbox>` escreveu `<slot/>`.

```gv
<!-- no template do componente -->
<container><column><slot/></column></container>
```

- O conteúdo é avaliado no contexto e com o **dono de quem escreveu**: um `on_click="salvar"` escrito dentro de um `<groupbox>` chega no handler da **tela**, não do widget. Não escreva `app:` nele.
- Os filhos do próprio `<slot>` são o **conteúdo de reserva**, usado quando quem chama não escreve nada dentro da tag. Esses são do componente e enxergam as props dele.

**Mais de um buraco:** `<slot name="footer"/>` no template, e o atributo `slot` no uso. O que não for etiquetado vai para o `<slot/>` anônimo.

```gv
<card title="Servidor">
    <text content="uptime 31 dias" />
    <template slot="footer">
        <button text="Reiniciar" on_click="reiniciar" />
    </template>
</card>
```

- `<template slot="…">` agrupa vários nós; para um nó só, o atributo direto (`<button slot="footer" …/>`) evita o embrulho.
- Vários blocos com o mesmo nome se concatenam na ordem escrita; o conteúdo anônimo preserva a ordem de documento mesmo com um bloco nomeado no meio dele.
- Dentro do componente, `{slot_<nome>}` vale `true` quando aquele slot foi preenchido — é o que permite decorar uma região opcional (a linha divisória que só existe quando existe rodapé).
- O nome é **fixo**, resolvido no template: `<slot name="{aba}"/>` (nome vindo do contexto) ainda não existe.

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

### `<Avatar>`
Foto circular com as **iniciais como reserva** quando não há imagem — ocupa o mesmo espaço nos dois casos, para não quebrar o alinhamento de uma lista.

```gv
<avatar src="fotos/ana.png" size="56" />
<avatar initials="AF" bg="#89B4FA" fg="#11111B" />
```

| prop | default | o que faz |
| --- | --- | --- |
| `src` | — | caminho/URL da imagem; presente, vence as iniciais |
| `initials` | `?` | 1–2 letras da reserva |
| `size` | `40` | diâmetro em px |
| `bg` / `fg` | `#8080803d` / `#cdd6f4` | cores do círculo de iniciais |

### `<RadioGroup>`
Um grupo de opções exclusivas montado de uma coleção do contexto — o `QButtonGroup`. Ao contrário da primitiva `<radio>`, ele **grava a chave sozinho**: nenhum handler no app.

```gv
<radiogroup value="plano" items="planos" />
<radiogroup value="plano" items="planos" layout="row" />
```

| prop | default | o que faz |
| --- | --- | --- |
| `items` | — (**obrigatória**) | **nome** da chave com o array de `{id, label}` |
| `value` | — (**obrigatória**) | **nome** da chave que guarda o `id` escolhido — lida para marcar, escrita no clique |
| `layout` | `column` | `column` ou `row` (o `Qt::Orientation` do grupo) |
| `spacing` | `8` | espaço entre as opções |

- Não precisa de um `active` como o `<tabbar>`: aqui quem resolve a marcação é a primitiva `<radio>`, em Rust, onde ler a chave cujo nome está numa prop é uma linha.

---

## Builtins que embrulham conteúdo (`<slot/>`)

Os cinco abaixo renderizam o que se escreve **entre as tags** deles. A ação de dentro pertence a quem a escreveu — um `on_click="salvar"` dentro de um `<groupbox>` chega no handler da tela.

### `<GroupBox>`
Moldura com título — o `QGroupBox`.

```gv
<groupbox title="Rede">
    <checkbox label="Usar proxy" checked="usar_proxy" />
    <button text="Salvar" on_click="salvar" />
</groupbox>
```

| prop | default | o que faz |
| --- | --- | --- |
| `title` | vazio | rótulo do grupo; vazio = sem cabeçalho (sobra a moldura pura) |
| `slot="actions"` | — | controles à direita da linha do título — onde vai o `<checkbox>` que faz o papel do `QGroupBox::setCheckable` |
| `flat` | `false` | `true` = título + linha, sem caixa (o `QGroupBox::flat`) |
| `padding` / `spacing` | `12` / `8` | espaço interno e entre os filhos |
| `title_size` | `13` | corpo do título |
| `width` | `fill` | largura do conjunto |

- **Aparência**: classes `.groupbox-frame` e `.groupbox-title`, redefiníveis numa `.gss` do app.

### `<Frame>`
A moldura sozinha, sem título — o `QFrame`.

| prop | default | o que faz |
| --- | --- | --- |
| `shape` | `box` | `box` (contorno), `filled` (contraste, o `QFrame::Panel`) ou `none` |
| `background` | — | cor do `filled` por instância; omitida, vem da classe `.frame-filled` |
| `padding` / `spacing` | `12` / `8` | idem `<GroupBox>` |

- Sem `Raised`/`Sunken`: o motor não tem campo de sombra.

### `<Card>`
Superfície de um item, com cabeçalho de título e subtítulo independentes.

```gv
<card title="Servidor" subtitle="produção" width="250">
    <text content="uptime 31 dias" />
</card>
```

| prop | default | o que faz |
| --- | --- | --- |
| `title` / `subtitle` | vazio | o cabeçalho aparece se **um dos dois** existir |
| `slot="footer"` | — | faixa de ações no pé, com linha divisória; só se paga quando preenchida |
| `padding` / `spacing` | `16` / `12` | espaço interno e entre os filhos |
| `width` | `fill` | numa grade, dê uma largura fixa a cada cartão |

```gv
<card title="Servidor">
    <text content="uptime 31 dias" />
    <template slot="footer"><button text="Reiniciar" on_click="reiniciar" /></template>
</card>
```

### `<ToolBar>` e `<ToolButton>`
A faixa de ações e o botão-ícone dela — o `QToolBar` e o `QToolButton`.

```gv
<toolbar>
    <toolbutton icon="📄" text="Novo" layout="beside" tooltip="Novo" on_click="novo" />
    <rule direction="vertical" />
    <toolbutton icon="🗑" tooltip="Excluir" on_click="excluir" />
</toolbar>
```

| prop de `<ToolButton>` | default | o que faz |
| --- | --- | --- |
| `on_click` | — | **ação do app** (o widget delega) |
| `icon` / `icon_src` | `●` | glifo, ou caminho de um `.svg` (que vence o glifo) |
| `text` | vazio | rótulo, usado por `layout="beside"`/`"under"` |
| `layout` | `icon` | as três formas do `Qt::ToolButtonStyle` |
| `icon_size` / `text_size` | `16` / `12` | corpos |
| `tooltip` | vazio | num botão só-ícone, é ela que diz o que ele faz |

`<ToolBar>` aceita `padding` (`6 8`), `spacing` (`4`), `divider` (`true`) e `width` (`fill`).

### `<StatusBar>`
O rodapé de status — o `QStatusBar`. A prop `message` é a zona da **esquerda** (o `showMessage`); o conteúdo do slot são os permanentes da **direita** (o `addPermanentWidget`).

```gv
<statusbar message="{status}">
    <badge badge_text="3 erros" badge_bg="#F38BA8" />
</statusbar>
```

Props: `message`, `padding` (`4 10`), `spacing` (`10`), `size` (`12`), `divider` (`true`), `width` (`fill`).

### `<TabBar>`
A fileira de abas — o `QTabBar`. Só a **barra**: o empilhado de páginas continua sendo `se`/`senao` na tela, porque cada página precisaria do seu próprio slot nomeado.

```gv
<tabbar value="aba" active="{aba}" items="abas" />
<template if="{aba}" equals="geral"> … </template>
```

| prop | default | o que faz |
| --- | --- | --- |
| `items` | — (**obrigatória**) | **nome** da chave com o array de `{id, label}` |
| `value` | — (**obrigatória**) | **nome** da chave que recebe o `id` clicado |
| `active` | — | o **valor atual** dessa chave, para o destaque. Sem ele a barra aparece inteira apagada |
| `padding` / `spacing` / `size` | `7 14` / `2` / `13` | área de clique, vão entre abas, corpo do rótulo |

- `value` e `active` andam em par porque quem decide o destaque aqui é o **template**, que não consegue ler o valor da chave cujo nome está numa prop.

---

Componentes do **app** são qualquer tag desconhecida (ex.: `<PerfilCard/>`),
resolvida pelo nome — declarada em outro `.gv`, por `<import from>`, ou por
`register_component("Nome", "caminho")` no Rust.
