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

### `<DateEdit>` · `<TimeEdit>` · `<DateTimeEdit>`
Os três campos de data/hora do Qt (`QDateEdit`, `QTimeEdit`, `QDateTimeEdit`) — **uma primitiva só**; a tag decide quais seções aparecem.

Edição por **seções**: clique numa (ano, mês, dia, hora, minuto, segundo) e ela ganha o realce da paleta; as setas ▴▾ passam a mexer **naquela** seção. Um controle cobre o valor inteiro — não há prop de passo.

```gv
<dateedit value="nascimento" format="br" />
<timeedit value="alarme" seconds="true" />
<datetimeedit value="agendado" onChange="validar" />
```

| tag | seções | o que grava na chave |
| --- | --- | --- |
| `<timeedit>` (`<timepicker>`) | hora, minuto \[, segundo\] | `HH:MM[:SS]` |
| `<dateedit>` (`<datepicker>`) | ano, mês, dia | `YYYY-MM-DD` |
| `<datetimeedit>` | as duas | `YYYY-MM-DD HH:MM[:SS]` |

| prop | default | o que faz |
| --- | --- | --- |
| `value` | — | **nome** da chave de contexto com o valor |
| `seconds` | `false` | acrescenta a seção de segundos |
| `format` | `iso` | `br` exibe `DD/MM/YYYY`. **Só a exibição** — a chave continua ISO |
| `onChange` | — | ver abaixo |
| `width` | natural | largura do campo |

- **A chave é sempre ISO**, mesmo com `format="br"`. É o que um backend espera — e o que faz `a < b` entre duas chaves ser a comparação cronológica, sem parse.
- **Sem `onChange`, o widget grava a chave sozinho** (nenhuma linha do lado do app). **Com `onChange`, ele só avisa** e quem grava é o handler — o mesmo contrato do `<TextInput>`, e é o que permite validar ou recusar um valor. Ver `examples/data_hora_luau`.
- **Cada seção vira dentro de si**: mexer no minuto não empurra a hora (o `wrapping` do `QAbstractSpinBox`). O ano satura em vez de virar.
- **Teclado** (0.70), com uma seção selecionada: `▲`/`▼` dão o passo, `←`/`→` trocam de seção, e `0`–`9` digitam nela. Digitar `0930` numa hora atravessa hora e minuto sozinho — a seção avança quando enche ou quando nenhum próximo algarismo caberia. Um `<TextInput>` focado captura os algarismos e o `←`/`→`, que por isso não chegam ao widget; `▲`/`▼`, que o campo de texto não usa, ainda alcançam uma seção selecionada e não largada por um clique.
- **O calendário é respeitado**: 31/01 subindo o mês vira 28/02, ou 29 em ano bissexto.
- **Não dá para digitar** no campo: a interação é clicar na seção + setas.

### `<Calendar>` · `<MonthYearPicker>` · `<DateRangePicker>`
A **grade** do Qt (`QCalendarWidget`) — e, pelas mesmas linhas de render, o seletor de mês/ano e o de intervalo. **Uma primitiva só**; a tag decide o que um clique grava.

```gv
<calendar value="entrada" today="{hoje}" />
<calendar value="entrada" onChange="validar_entrada" min="{hoje}" />
<monthyearpicker value="competencia" />
<daterangepicker start="entrada" end="saida" months="2" today="{hoje}" />
```

| tag | o que grava | chave |
| --- | --- | --- |
| `<calendar>` (`<calendario>`) | um dia | `YYYY-MM-DD` |
| `<monthyearpicker>` (`<seletormesano>`) | um mês | `YYYY-MM` |
| `<daterangepicker>` (`<seletorintervalo>`) | duas datas, em **duas chaves** | `start` e `end` |

| prop | default | o que faz |
| --- | --- | --- |
| `value` | — | **nome** da chave com o dia (ou o **início**, no intervalo) |
| `end` | — | nome da chave com o fim. Só no intervalo |
| `onChange` | — | vazio = o widget grava sozinho; preenchido = delega. No intervalo, o valor entregue é `"<início> <fim>"` (o fim vem vazio no primeiro clique) |
| `today` | — | a data de hoje, para o realce. **Prop, não relógio**: é `date.today()` numa linha de Luau. Sem ela, nenhum dia fica destacado |
| `min` / `max` | — | limites; os dias fora saem inertes |
| `mode` | `day` | `day` · `month` · `year` — onde o clique **para de navegar e passa a gravar** |
| `month` | — | chave que dirige o mês visível. Sem ela, ele mora em `__cal_<chave>`, do motor, e o app não configura nada |
| `first_day` | `sunday` | `monday` gira o cabeçalho de sete iniciais |
| `months` | `1` | quantas grades desenhar lado a lado |
| `month_names` / `day_names` | pt-BR | rótulos, separados por espaço (12 e 7). Os dias vão **sempre a partir de domingo**; `first_day` gira a lista sozinho |

- **A escada de drill-up**: clicar no **título** sobe um degrau (dia → mês → ano), como o `QCalendarWidget`. Descer escolhe o mês/ano **visível**, sem tocar na chave — `mode` é que diz onde o clique passa a gravar.
- **O intervalo grava duas chaves separadas**, não `"a/b"` numa só: é o que deixa o `date.diff` do Luau ler as duas direto. O primeiro clique marca o início; o segundo fecha. Entre um e outro, a faixa sob o cursor é pintada (chave global `__cal_hover`).
- **As células dos meses vizinhos são inertes** — as setas `‹ ›` navegam.
- **Nenhuma dependência de datas**: dia da semana é `days_from_civil`, oito linhas ao lado do `dias_no_mes`.

### `<MaskedInput>` (`<EntradaMascarada>`, `<Mascara>`)
`QLineEdit` com `setInputMask`: guarda o valor **cru** na chave e exibe mascarado — a mesma separação valor/exibição do `<dateedit>`.

```gv
<maskedinput value="cpf" mask="cpf" />
<maskedinput value="placa" mask="AAA#*##" />
```

| símbolo | aceita |
| --- | --- |
| `#` | dígito |
| `A` | letra |
| `*` | letra ou dígito |

Qualquer outro caractere é **literal**. Presets: `cpf`, `cnpj`, `telefone`/`phone`, `cep`, `placa`, `date`/`data`, `hora`, `cartao`/`card`.

| prop | default | o que faz |
| --- | --- | --- |
| `value` | — | **nome** da chave com o valor cru |
| `mask` | — | a máscara, ou um preset |
| `onChange` | — | vazio = grava sozinho; preenchido = delega, **com o cru** |
| `placeholder` | a máscara com `_` | a dica |

- **A chave guarda `"12345678901"`**, não `"123.456.789-01"`: é o que um backend espera e o que compara sem surpresa.
- **Limite conhecido**: apagar um separador do **meio** da string não faz nada visível (a tecla seguinte remove o dígito). Corrigir exigiria a posição do cursor, que o `on_input` do iced não entrega. No fim da string — onde se digita — apagar funciona sempre.

### `<Pagination>` (`<Paginacao>`)
`« ‹ 1 … 4 [5] 6 … 20 › »`. A janela de números anda com a página e gruda nas pontas; as setas ficam **inertes** no limite.

```gv
<pagination value="pagina" total="{total_paginas}" />
<pagination value="pagina" total="{total_paginas}" window="3" ends="false" onChange="repaginar" />
```

| prop | default | o que faz |
| --- | --- | --- |
| `value` | — | **nome** da chave com a página atual (base 1) |
| `total` | `0` | total de **páginas**. `0` ou `1` esconde o widget |
| `window` | `5` | quantos números aparecem em volta do atual |
| `ends` | `true` | mostra `«`/`»` (primeira/última) |
| `onChange` | — | vazio = grava sozinho; preenchido = delega |

O widget conta **páginas**, não itens: recortar a lista é do app, porque só o app sabe o que é um item.

### `<Rating>` (`<Nota>`, `<Estrelas>`)
A nota por estrelas, com pré-visualização ao passar o mouse.

```gv
<rating value="nota" />
<rating value="nota" max="10" size="15" color="#F9E2AF" />
<rating value="media" readonly="true" />
```

| prop | default | o que faz |
| --- | --- | --- |
| `value` | — | **nome** da chave com a nota |
| `max` | `5` | quantos alvos desenhar (preso em 1–20) |
| `filled` / `empty_icon` | `★` / `☆` | os glifos |
| `size` | `20` | corpo do glifo |
| `color` | primária do tema | cor do glifo cheio |
| `readonly` | `false` | desenha e não aceita clique nem hover — o `Rating` de uma **lista** |
| `onChange` | — | vazio = grava sozinho; preenchido = delega |

Clicar na estrela já marcada **zera** a nota.

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
Condicional. `<If>` aceita `cond`, `equals`, `notEquals`, `one_of`, `contains`, `empty`/`not_empty`.

- `one_of="a b c"` (`equals_any`) — o valor da chave é **um dos** tokens do markup.
- `contains="rede"` (`contem`, `has`) — o **simétrico**: a lista está na chave (`"geral,rede"`) e o item está no markup. É o **conjunto nomeado** — várias seções de um accordion abertas, uma seleção múltipla, um filtro por tags — sem estado por instância. Separadores: vírgula, ponto-e-vírgula ou espaço, os três ao mesmo tempo; o item também interpola (`contains="{s.id}"`).
- `empty` / `not_empty` (pelados) — a chave é (ou não é) um array JSON de zero elementos.

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

> **`class` num componente** (0.69). Escrever `class` na tag de um componente —
> builtin da lib ou do seu app — estiliza a **raiz expandida** do template dele.
> Antes da 0.69 isso era um **no-op silencioso**: a classe era lida, viajava no
> mapa de props e não pintava nada, sem erro nem aviso.
>
> A escada de especificidade, do mais fraco ao mais forte:
>
> ```
> seletor de tag do componente  <  tag builtin  <  classe do template  <
> classe do USO  <  id do template  <  inline do template
> ```
>
> Em uma frase: **a classe escrita no uso vence as classes do template, e perde
> para os atributos inline do template.** É a intuição do CSS — classe é
> default do autor, inline é decisão dele.
>
> Ela aplica **só na raiz**. Estilizar um nó específico lá dentro continua sendo
> decisão do componente, que expõe uma prop com nome próprio para isso — como o
> `field_class` do `<SpinBox>` abaixo.

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
| `step` | `1` | passo de cada clique |
| `decimals` | do `step` | casas decimais da saída — é o que fecha o `QDoubleSpinBox`. **Sem ela** as casas saem do `step` como escrito (`step="0.25"` → 2 casas), o que acerta quase sempre e erra em `step="1"` sobre um preço: `10`, não `10.00` |
| `layout` | `stacked` | `stacked`: as setinhas `▴▾` empilhadas e coladas à direita do campo (o `QSpinBox` clássico). `inline`: `−  campo  +`, degraus nas pontas (o `SpinBox` do Qt Quick Controls) |
| `width` | `72` | largura do campo |
| `placeholder` | vazio | dica quando a chave está vazia |
| `field_class` | vazio | classe aplicada **ao campo de dentro**, não ao widget inteiro. Ver a nota abaixo sobre por que não se chama `class` |
| `form_control` | vazio | repassado ao campo de dentro: dá a ele um id de foco estável e liga o **Enter** da `<Form>` que o envolve (submeter + avançar para o próximo controle). **Tab não depende disto** — a travessia por Tab é um listener global do motor e já alcança qualquer widget focável |
| `dec_text` / `inc_text` | `▾`/`▴` (stacked), `−`/`+` (inline) | glifos dos degraus |
| `glyph_size` | `11` (stacked), `15` (inline) | corpo do glifo |

- **Chave vazia**: o primeiro clique inicializa no `min` (não em `min + step`).
- **Digitação** entra filtrada (só dígitos, um `-` à frente e um `.`) e **sem saturar** — como o `QSpinBox`, que só valida ao terminar a edição; o clique seguinte satura.
- **Sem `on_change` para o app**: o `onChange` do campo interno é usado pelo próprio widget para filtrar a digitação. Quem precisa reagir lê a chave de `value`.
- **Aparência**: os degraus são pintados por uma folha GSS que o próprio widget declara, na classe `.spinbox-step` (com `:hover`/`:active`). Ela é instalada antes de qualquer `.gss` do app, então redefinir `.spinbox-step` numa folha sua vence e repinta os degraus.
- **`class` × `field_class`** (0.69): `class` estiliza a **raiz do widget** — a `Row` inteira, campo mais degraus, que é o que estilizar um `QSpinBox` significa no Qt. `field_class` estiliza **só o `<TextInput>` de dentro**. As duas são legítimas e diferentes; por isso têm nomes diferentes, em vez de um só ambíguo.

```gv
<SpinBox value="qtd" min="1" max="9"
         class="moldura"          <!-- a Row: campo + degraus -->
         field_class="campo_num"  <!-- só o campo -->
         form_control="qtd" />    <!-- só o campo -->
```


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

### `<ListView>`
`QListWidget`: a lista rolável cujo item escolhido mora numa chave — o `<tabbar>` na vertical, com scroll.

```gv
<listview items="servicos" value="servico" selected="{servico}" height="240" />
<listview items="servicos" value="marcados" selected="{marcados}" mode="multi" />
```

| `mode` | a chave guarda | o destaque testa |
| --- | --- | --- |
| `single` (default) | um id (`"api"`) | `equals` |
| `multi` | um **conjunto** (`"api,db"`) | `contains` |

Props: `items` (chave com o array de `{id, label, sub?}`), `value` (nome da chave), `selected` (valor atual), `mode`, `height` (default `240`), `width`, `spacing`, `padding`, `size`, `virtualize` (altura declarada da linha, para listas longas).

`value` e `selected` andam em par pelo mesmo motivo do `<tabbar>`: o template não lê a chave cujo *nome* está numa prop.

### `<Accordion>` / `<AccordionItem>` · `<ToolBox>` / `<ToolBoxItem>`
Os dois modos do mesmo widget: o accordion guarda um **conjunto** de seções abertas, o tool box guarda **uma**.

```gv
<accordion>
    <accordionitem title="Rede" value="abertas" open="{abertas}" id="rede">
        <input value="host" />
    </accordionitem>
    <accordionitem title="Disco" value="abertas" open="{abertas}" id="disco"> … </accordionitem>
</accordion>

<toolbox>
    <toolboxitem title="Medidas" value="secao" open="{secao}" id="medidas"> … </toolboxitem>
</toolbox>
```

Props do item: `title`, `sub` (segunda linha, opcional), `value` (**nome** da chave), `open` (valor atual), `id` (esta seção), `padding`, `spacing`.

- **Uma tag por seção**, e não uma coleção, porque o **conteúdo** de cada uma é diferente — e nomes de slot são fixos no template do componente. É a mesma forma do `QToolBox::addItem(widget, "Título")`.
- No `<toolboxitem>`, clicar na seção já aberta a **fecha** (a chave vira vazia).
- A moldura de fora (`<accordion>`/`<toolbox>`) é só uma `<Column>`: o item funciona sozinho.

### `<ButtonBox>`
`QDialogButtonBox`: a fileira de botões de um formulário, com **papéis** e a ordem decidida pela plataforma.

```gv
<buttonbox accept="Salvar" on_accept="salvar" reject="Cancelar" on_reject="cancelar"
           destructive="Excluir" on_destructive="excluir">
    <button text="Ajuda" on_click="ajuda" padding="8 16" />
</buttonbox>
```

| papel | quando | aparência |
| --- | --- | --- |
| `accept` | OK, Salvar, Sim | destaque |
| `reject` | Cancelar, Não, Fechar | discreto |
| `destructive` | Excluir, Descartar | perigo, e **longe** dos outros |

- **A ordem é da plataforma**: GNOME/macOS põem o afirmativo por último, Windows primeiro. O widget escolhe em Rust, por alvo de compilação; `order="gnome"`/`"windows"` força.
- **Um botão sem rótulo não aparece** — uma caixa só com `accept` é um botão só.
- O `<slot/>` e o destrutivo ficam à **esquerda**, separados das ações por um `<Space/>`.

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
