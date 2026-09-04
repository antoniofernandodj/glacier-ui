# glacier-ui

**Glacier** é um motor de UI declarativa para Rust: você descreve a interface em
**XML** e o motor a renderiza com [`iced`](https://iced.rs). O comportamento pode
morar em **Rust** (o trait `Component`) ou **dentro do próprio template**, num
bloco `<script>` em **[Luau](https://luau.org)** interpretado em tempo de execução
— com **hot-reload**, **data binding**, **componentes**, **navegação**,
**formulários reativos**, **stylesheets `.gss`** (CSS-like), **rede assíncrona**
(`fetch`/SSE/WebSocket), **toasts** e **diálogos**.

```xml
<!-- examples/contador/contador.gv -->
<Container padding="20" alignX="Center" alignY="Center" width="fill" height="fill" background="#2E3440">
    <Column spacing="20" align="Center">
        <Text content="Valor do Contador: {contador}" size="28" bold="true" color="#ECEFF4" />
        <Row spacing="15" align="Center">
            <Button text="Diminuir" on_click="decrementar" color="#BF616A" padding="10 20" />
            <Button text="Aumentar" on_click="incrementar" color="#A3BE8C" padding="10 20" />
        </Row>
    </Column>
</Container>
```

Duas formas de dar comportamento a esse XML — escolha por caso de uso:

```rust
// 1) Em Rust: um Component tipado, com estado próprio.
impl Component for Contador {
    fn name(&self) -> &str { "contador" }
    fn template(&self) -> Template { Template::File("examples/contador/contador.gv".into()) }
    fn init(&mut self, ctx: &mut Context) { ctx.set("contador", self.valor.to_string()); }
    fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            _ => return,
        }
        ctx.set("contador", self.valor.to_string());
    }
}
```

```lua
-- 2) No próprio template, num <script> Luau (sem recompilar):
function init()        ctx.contador = ctx.contador or 0 end
function incrementar() ctx.contador = ctx.contador + 1 end
function decrementar() ctx.contador = ctx.contador - 1 end
```

---

## Sumário

- [glacier-ui](#glacier-ui)
  - [Sumário](#sumário)
  - [Por que Glacier](#por-que-glacier)
  - [Instalação](#instalação)
    - [Começando do zero: `glacier new`](#começando-do-zero-glacier-new)
    - [Como dependência](#como-dependência)
  - [Conceitos e arquitetura](#conceitos-e-arquitetura)
  - [Início rápido](#início-rápido)
    - [Ligando ao `iced`: `GlacierApp::bootstrap`](#ligando-ao-iced-glacierappbootstrap)
  - [Referência de tags](#referência-de-tags)
    - [Layout](#layout)
    - [Conteúdo e controles](#conteúdo-e-controles)
    - [Estruturais (composição, fluxo, recursos)](#estruturais-composição-fluxo-recursos)
  - [Cabeçalho: `<screen>`, `<component>` e `<resources>`](#cabeçalho-screen-component-e-resources)
    - [`<props>`: o contrato do componente](#props-o-contrato-do-componente)
    - [`spread`: o objeto inteiro de uma vez](#spread-o-objeto-inteiro-de-uma-vez)
  - [Atributos de layout e estilo](#atributos-de-layout-e-estilo)
  - [Data binding](#data-binding)
  - [Controle de fluxo](#controle-de-fluxo)
  - [Inputs de texto](#inputs-de-texto)
  - [Formulários (Reactive Forms)](#formulários-reactive-forms)
  - [Navegação entre telas](#navegação-entre-telas)
  - [Componentes e composição](#componentes-e-composição)
  - [Comportamento em `<script>` Luau](#comportamento-em-script-luau)
    - [`fetch`: rede async via corrotina](#fetch-rede-async-via-corrotina)
    - [`require`: módulos Luau](#require-módulos-luau)
    - [Timers: `after` e `every`](#timers-after-e-every)
    - [`storage`: persistência local](#storage-persistência-local)
    - [`viewport`, `toast`, `confirm`, `navigate`](#viewport-toast-confirm-navigate)
    - [Erros visíveis: `on_error`](#erros-visíveis-on_error)
    - [Streams: SSE e WebSocket](#streams-sse-e-websocket)
  - [Estilos `.gss`](#estilos-gss)
    - [Pseudo-estados: `:hover` / `:focus` / `:active` / `:disabled`](#pseudo-estados-hover--focus--active--disabled)
  - [`<link rel="…">` e temas](#link-rel-e-temas)
  - [Toasts e diálogos (em Rust)](#toasts-e-diálogos-em-rust)
  - [Drag-and-drop: listas reordenáveis](#drag-and-drop-listas-reordenáveis)
  - [Ações built-in](#ações-built-in)
  - [Hot-reload](#hot-reload)
  - [Rede e async em Rust](#rede-e-async-em-rust)
  - [Referência da API](#referência-da-api)
    - [`GlacierUI`](#glacierui)
    - [`EngineMessage`](#enginemessage)
    - [Tipos de apoio (re-exportados na raiz do crate)](#tipos-de-apoio-re-exportados-na-raiz-do-crate)
    - [Globais da camada Luau](#globais-da-camada-luau)
  - [Exemplos](#exemplos)
  - [Publicação no crates.io](#publicação-no-cratesio)
  - [Licença](#licença)

---

## Por que Glacier

- **Declarativo de verdade** — a UI é um arquivo XML, não uma árvore de chamadas Rust.
- **Comportamento onde couber melhor** — em Rust (tipado, com estado forte) ou em Luau dentro do `<script>` (interpretado, sem recompilar).
- **Hot-reload** — edite o XML, os estilos `.gss`, os dados JSON, o tema ou a lógica Luau com a app rodando e veja a mudança na hora; só a lógica em Rust exige recompilar.
- **Data binding por placeholders** — `{chave}` em qualquer atributo, resolvido contra um contexto de estado compartilhado.
- **Componentes** — encapsulam UI + comportamento + estado num único tipo Rust, compostos por `<import>`, referência por nome ou `children()`.
- **Assíncrono sem travar a UI** — `fetch` (HTTP), `sse`/`websocket` (streams) na camada Luau; `ctx.perform` e `Component::subscription` na camada Rust.
- **Estilos reutilizáveis** — classes `.gss` (CSS-like) globais ou com escopo por componente, com a precedência do CSS e pseudo-estados (`:hover`/`:focus`/`:active`/`:disabled`).
- **Renderiza com `iced`** — widgets nativos, multiplataforma, tema configurável.

---

## Instalação

### Começando do zero: `glacier new`

Um projeto glacier tem um `Cargo.toml`, um `src/main.rs`, um `.gv` com
cabeçalho, um `.gss`, um `.luaurc` e uma árvore de scripts Luau. Montar isso à
mão, lendo este README arquivo por arquivo, é a parte mais chata de começar — a
CLI faz um questionário e entrega tudo já ligado e rodando:

```bash
cargo install glacier-cli
glacier new
```

```
? Nome do projeto (meu-app) painel
? Qual preset?
  › 1  App completo      janela sem decoração, tema + .gss, componentes, navegação, fetch
    2  Mínimo            uma tela, um .gss e um bloco de script Luau
    3  Multi-janela      open_window/broadcast, bandeja, instância única
    4  Componente Rust   o trait Component, com estado tipado
? Instalar as extensões de VS Code (realce e ir-para-definição em .gv/.gss)? [S/n]
```

Ele mostra um resumo e **só então** escreve: até a confirmação, nada foi criado.
`glacier install-extensions` instala só as extensões de VS Code (sem precisar de
Node — o `.vsix` é empacotado na hora). Ver
[`crates/glacier-cli`](crates/glacier-cli).

### Como dependência

O motor é um crate só, **`glacier-ui`**.

```bash
cargo add glacier-ui
```

As dependências vêm junto: `iced 0.14`, `roxmltree`, `image`, `serde_json`,
`regex`, `mlua` (com **Luau** vendorizado — compilado do fonte, sem precisar de
Lua/Luau no sistema), `hyper` + `rustls` para `fetch`, e `tokio-tungstenite` para
WebSocket. O `iced` é re-exportado em `glacier_ui::iced`, então a sua `main`
pode nem listar `iced` como dependência direta. Requer Rust **edition 2024**
(≥ 1.85).

Os exemplos do repositório (`examples/`) não são compilados por padrão: são 31,
cada um linka o motor inteiro, e juntos passavam de 15 GiB em `target/`. Para
rodar um deles, comente o `autoexamples = false` do `Cargo.toml` da raiz:

```bash
cargo run --example contador
```

---

## Conceitos e arquitetura

| Peça | Papel |
|---|---|
| **Template XML** | descreve a árvore de UI (layout, texto, botões, imagens, …). |
| **Contexto** | mapa `String -> String` com o estado; templates leem dele via `{chave}`. |
| **`GlacierUI`** | o motor: registra templates/componentes/estilos, avalia o contexto e renderiza para `iced`. |
| **`Component`** | tipo Rust que junta **UI** (template) + **comportamento** (reação a ações) + **estado** próprio. |
| **`<script>` Luau** | comportamento embutido no template, alternativa interpretada ao `Component`. |
| **`EngineMessage`** | mensagens que o `iced` entrega ao motor (cliques, inputs, navegação, reload, efeitos). |
| **Stylesheet `.gss`** | classes de estilo reutilizáveis (CSS-like), globais ou por componente. |

O fluxo de cada frame de estado:

```
XML  ──parse──▶  AST  ──avalia (contexto + estilos + includes + if/for-each)──▶  AST resolvido  ──render──▶  widgets iced
                                                   ▲                                                            │
                                                   └────────── ação vira EngineMessage, roteada ao Component ◀──┘
```

A integração com o `iced` segue o padrão `application(init, update, view)`: o
`update` da app só repassa a mensagem para `motor.dispatch(...)`, e o `view`
chama `motor.render_current()`.

---

## Início rápido

Um app é uma casca fina em volta de um `GlacierUI`: registra os componentes,
repassa mensagens a `dispatch` e renderiza com `render_current`. Toda a lógica
vive nos componentes (Rust) ou nos `<script>` (Luau).

```rust
use glacier_ui::{GlacierUI, EngineMessage, Component, Context, Template};
use iced::{Element, Task, widget::text, Color};

struct Contador { valor: i32 }

impl Component for Contador {
    fn name(&self) -> &str { "contador" }
    fn template(&self) -> Template { Template::File("examples/contador/contador.gv".into()) }
    fn init(&mut self, ctx: &mut Context) { ctx.set("contador", self.valor.to_string()); }
    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        match action {
            "incrementar" => self.valor += 1,
            "decrementar" => self.valor -= 1,
            _ => return,
        }
        ctx.set("contador", self.valor.to_string());
    }
}

struct App { motor: GlacierUI }

impl App {
    fn new() -> (Self, Task<EngineMessage>) {
        let mut motor = GlacierUI::new();
        motor.register(Box::new(Contador { valor: 0 })).unwrap();
        motor.set_initial_screen("contador");
        (Self { motor }, Task::none())
    }
    fn update(&mut self, msg: EngineMessage) -> Task<EngineMessage> { self.motor.dispatch(&msg) }
    fn view(&self) -> Element<'_, EngineMessage> {
        self.motor.render_current()
            .unwrap_or_else(|e| text(e).color(Color::from_rgb(1.0, 0.0, 0.0)).into())
    }
    fn subscription(&self) -> iced::Subscription<EngineMessage> {
        GlacierUI::reload_subscription(std::time::Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("Contador")
        .run()
}
```

Para um comportamento embutido no template, troque `register(Box::new(...))` por
`register_component("contador", "caminho/para/contador.gv")`: se o template tiver
um `<script>`, o motor liga a lógica **Luau** automaticamente (ver
[`examples/contador_macro`](examples/contador_macro)).

### Ligando ao `iced`: `GlacierApp::bootstrap`

Para não repetir `iced::application(App::init, App::update, App::view).subscription(...)`
na mão, implemente o trait `GlacierApp` e chame `App::bootstrap()` — ele pré-liga
os quatro métodos e devolve o builder do `iced` (ainda aceita `.title`, `.theme`,
`.window`, …):

```rust
use glacier_ui::{EngineMessage, GlacierUI, GlacierApp};
use iced::{Element, Subscription, Task};

struct App { motor: GlacierUI }

impl GlacierApp for App {
    type Message = EngineMessage;
    fn init() -> (Self, Task<EngineMessage>) { /* ... */ }
    fn update(&mut self, msg: EngineMessage) -> Task<EngineMessage> { self.motor.dispatch(&msg) }
    fn view(&self) -> Element<'_, EngineMessage> { /* ... */ }
    fn subscription(&self) -> Subscription<EngineMessage> {
        GlacierUI::reload_subscription(std::time::Duration::from_millis(500))
    }
}

fn main() -> iced::Result {
    App::bootstrap().title("Glacier - navegação via script Lua").run()
}
```

Veja [`examples/navegacao_luau`](examples/navegacao_luau).

---

## Referência de tags

Todas as tags aceitam variações de caixa e nomes em inglês **ou** português.

### Layout

| Tag | Aliases | Descrição |
|---|---|---|
| `<Container>` | `container` | caixa única (1 filho lógico); base para cartões/painéis. |
| `<Column>` | `column` | empilha os filhos verticalmente. |
| `<Row>` | `row` | dispõe os filhos horizontalmente. |
| `<Scrollable>` | `Scroll`, `Rolagem` | viewport rolável de 1 filho; `direction`: `vertical` (padrão), `horizontal`, `both`. |
| `<Rule>` | `Divider`, `Divisoria` | linha separadora; `direction`: `horizontal` (padrão) ou `vertical`. |

### Conteúdo e controles

| Tag | Aliases | Atributos próprios |
|---|---|---|
| `<Text>` | `text` | `content`/`texto`, `size`/`tamanho`, `bold`/`negrito`, `color`/`cor`, `textAlign` |
| `<Button>` | `button`, `Botao` | `text`/`texto`, `on_click`/`aoClicar`, `navigateTo`/`irPara`, `navigateBack`/`voltar`, `color`/`cor` |
| `<TextInput>` | `Input`, `EntradaTexto` | `placeholder`/`dica`, `value`/`valor`, `onChange`/`aoMudar`, `secure`/`password` (mascara), `formControl` (liga a um `FormControl`) |
| `<TextArea>` | `TextEditor`, `Editor`, `AreaTexto` | `placeholder`/`dica`, `value`/`valor`, `onChange`/`aoMudar` (editor multilinha) |
| `<Form>` | `Formulario` | `onSubmit`/`aoSubmeter`, `name`/`nome` — renderiza como `<Column>` |
| `<Select>` | `Dropdown`, `PickList`, `ComboBox`, `Seletor` | `options`/`items` (chave com array JSON), `value`/`valor`, `onChange`/`onSelect`, `placeholder`, `labelField` (padrão `label`), `valueField` (padrão `value`) |
| `<Image>` | `Imagem` | `source`/`src`/`caminho`, `clip="Circle"` (corte circular) |
| `<Svg>` | `Icon`, `Icone` | `source`/`src`, `color`/`cor` (tinge o ícone vetorial) |
| `<Checkbox>` | `Check` | `label`, `checked`/`value` (chave de contexto), `onToggle`/`onChange`, `tristate` (cicla `false → mixed → true`, como o `Qt::CheckState`) |
| `<Toggle>` | `Toggler`, `Switch` | `label`, `checked`/`value`, `onToggle`/`onChange` — a bolinha desliza animada (200ms) |
| `<ProgressBar>` | `Progress`, `BarraProgresso` | `value`/`valor` (chave de contexto numérica), `min`/`max` (padrão `0`/`100`), `vertical`, `showValue` (percentual centralizado), `color`/`cor` (preenchimento; o `background` genérico é o trilho) |
| `<Spinner>` | `BusyIndicator`, `IndicadorOcupado`, `Carregando` | indicador **indeterminado** (`QProgressBar` com `setRange(0,0)`); `color`/`cor` (padrão: `primary` do tema); `width`/`height` define o diâmetro (padrão 24px). Gira sozinho — nenhum estado no contexto |
| `<Reveal>` | `Collapse`, `Revelar`, `Sanfona` | abre e fecha o conteúdo **animando a altura** (o que transborda é recortado): `open`/`aberto` (valor, não chave: `true`/`false` ou um `{var}`), `duration`/`duracao` em ms (padrão `180`; `0` desliga), `axis="x"` (anima a **largura** em vez da altura — é o que a gaveta do `<drawer>` usa). O filho fica na árvore fechado ou aberto — é o que o `<accordion>`/`<toolbox>` usa por dentro |
| `<DateEdit>` · `<TimeEdit>` · `<DateTimeEdit>` | `DatePicker`, `TimePicker`, `EditorData`, `EditorHora` | edição por **seções** (`QDateTimeEdit`): `value`/`valor` (chave), `onChange`/`aoMudar` (vazio = o widget grava sozinho), `seconds`/`segundos`, `format="br"` (só a exibição — a chave é sempre ISO), `calendarPopup="true"` (põe um botão 📅 que abre a grade do `<calendar>` **ancorada ao campo**, com `today`/`min`/`max` repassados a ela). A tag decide quais seções aparecem |
| `<Calendar>` · `<MonthYearPicker>` · `<DateRangePicker>` | `Calendario`, `SeletorMesAno`, `SeletorIntervalo` | a **grade** (`QCalendarWidget`): `value`/`valor` (chave; `start`/`end` no intervalo), `onChange`, `today`/`hoje` (realce — **prop, não relógio**: `date.today()`), `min`/`max`, `mode` (`day`/`month`/`year`, a escada de drill-up), `first_day="monday"`, `months="2"`, `month`/`mes_visivel` (chave que dirige o mês visível), `month_names`/`day_names`. A tag decide o que um clique grava |
| `<MaskedInput>` | `EntradaMascarada`, `Mascara` | `QLineEdit` com `setInputMask`: guarda **cru** na chave e exibe mascarado. `value`/`valor`, `mask`/`mascara` (gramática `#` dígito · `A` letra · `*` alfanumérico + literais, ou um preset: `cpf`, `cnpj`, `telefone`, `cep`, `placa`, `date`, `hora`, `cartao`), `onChange` (recebe o **cru**), `placeholder` (default: a máscara com `_`) |
| `<Pagination>` | `Paginacao` | `« ‹ 1 … 4 [5] 6 … 20 › »`: `value`/`valor` (chave com a página, base 1), `total`/`paginas`, `window`/`janela` (quantos números, default `5`), `ends="false"` (esconde `«`/`»`), `onChange`. Um total de 0 ou 1 esconde o widget |
| `<Popover>` · `<Popup>` | `Painel` | um painel que **flutua sobre a tela**: `value`/`valor` (chave do aberto/fechado), `placement`/`posicao` (`bottom` default · `top` · `left` · `right`), `align`/`alinhamento` (`start` · `center` · `end`), `offset`/`folga`, `panel_width` (um número ou `anchor`), `dismiss="false"`, `trigger="none"`, `onClose`. O filho marcado `slot="anchor"` é o **gatilho** (fica no fluxo); o resto é o painel. `<popup>` é a mesma primitiva **sem âncora**, centrada na janela. Abre no clique do gatilho, fecha no clique fora e no Esc — sem uma linha de app |
| `<Autocomplete>` | `Completer`, `AutoCompletar` | o campo que **filtra enquanto se digita** (`QCompleter`): `value`/`valor` (chave com o texto), `items`/`itens` (chave com o array de candidatos — strings ou `{id,label}`), `placeholder`, `min_chars` (default `1`; `0` abre ao focar), `max_items` (default `8`), `filter="false"` (quem filtra é o app), `onChange`, `onSelect` (recebe o `id`). Recorta sem acento e sem caixa; ▲▼ navegam, Enter aceita, Esc desiste |
| `<Grid>` | `Grade` | a grade com **colunas medidas** (`QGridLayout`): `columns`/`colunas` — um número (`"3"`, três colunas medidas) ou uma trilha por palavra (`"140 fill 80"`) —, `spacing` (vão horizontal) e `row_spacing` (vertical; sem ele, o `spacing`). A largura de uma coluna é o **máximo das células dela** |
| `<Flow>` | `Wrap`, `Fluxo` | a fileira que **quebra linha** quando não cabe: `spacing`, `row_spacing`, `align_y`. É o que um campo de tags/chips pede |
| `<TableView>` · `<TableHeader>` | `Tabela`, `CabecalhoTabela` | a tabela (`QTableView`/`QHeaderView`): `items`/`linhas` (chave com o array), `columns`/`colunas` (chave com `{key,label,width,align}` — ou uma spec de trilhas), `value` (linha escolhida), `mode="multi"`, `sort`/`ordem` (chave com `"coluna asc"`), `widths`/`larguras` (chave; a presença dela põe as **alças de arrasto**), `height` (a janela de rolagem), `onSelect`, `onSort`. Cabeçalho e corpo são a **mesma grade**, e a ordenação é numérica quando os dois lados são número |
| `<TreeView>` | `Arvore` | a árvore (`QTreeView`): `items` (array JSON aninhado com `{id,label,items}`), `value` (o nó escolhido — o **caminho** dele), `open`/`abertos` (o **conjunto** de caminhos abertos, `"raiz,raiz/src"`), `indent`/`recuo`, `onSelect` |
| `<ColumnView>` | `Colunas`, `Miller` | a navegação Miller do Finder: `items` (a mesma árvore), `value`/`caminho`, `column_width`, `onSelect` |
| `<Rating>` | `Nota`, `Estrelas` | a nota por estrelas: `value`/`valor` (chave), `max` (default `5`), `filled`/`empty_icon` (glifos, default `★`/`☆`), `size`, `color`, `readonly`, `onChange`. Prévia no hover; clicar na estrela já marcada zera |

### Estruturais (composição, fluxo, recursos)

| Tag | Aliases | Descrição |
|---|---|---|
| `<import>` | `Import`, `Importar` | declara um componente carregado de um arquivo: `name`/`nome`/`as`, `from`/`de`/`src`. |
| `<Include>` | `Incluir` | inclui outro template: `src`/`fonte`; demais atributos viram props. |
| `<NomeDoComponente .../>` | — | qualquer tag desconhecida referencia um componente por nome; atributos viram props. |
| `<ForEach>` | `For` | repete os filhos por item: `items`/`itens`, `var`/`variavel`. |
| `<if>` | `Se` | renderiza condicionalmente: `cond`, `equals`, `notEquals`. |
| `<else>` | `Senao` | renderiza quando o `<if>` imediatamente anterior foi falso. |
| `<template>` | `Gabarito` | `<ForEach>`/`<if>`/`<ElseIf>`/`<else>` sob um nome só — a flavour depende do atributo presente (`for-each`/`items`, `else`, `else-if`, `if`/`cond`; nenhum deles agrupa os filhos incondicionalmente). Ver "Controle de fluxo". |
| `<screen>` | `Screen`, `tela`, `Tela` | cabeçalho da tela: metadados da janela (`title`, `size`, `min-size`, `resizable`) com os recursos e o layout dentro. Ver "Cabeçalho". |
| `<props>` / `<prop>` | — | contrato de props de um `<component>`. Ver "Cabeçalho". |
| `<component>` | `Component`, `componente`, `Componente` | mesma casca do `<screen>` para um `.gv` que é **pedaço** de tela (importado por outro), sem os atributos de janela. |
| `<resources>` | `Resources`, `recursos`, `Recursos` | dentro do `<screen>`/`<component>`, agrupa o que não desenha: `<style>`, `<script>`, `<link>`, `<import>` e `<component name="…">`. |
| `<component name="…">` | `Componente` | **dentro do `<resources>`**, declara um componente na própria tela, sem arquivo: `<props>` + layout, a mesma casca de um `.gv`. Ver "Cabeçalho". |
| `<link>` | `Link` | carrega um recurso: stylesheet, componente, dados ou tema. |
| `<style>` | `Style` | classes `.gss` inline (global por padrão ou `scoped="true"`), ou externa com `href`. |
| `<script>` | — | comportamento Luau embutido (inline ou `src="arquivo.luau"`). |

---

## Cabeçalho: `<screen>`, `<component>` e `<resources>`

Um template pode declarar, no próprio arquivo, **o que a janela é** — e separar
o que não desenha do que desenha:

```xml
<screen title="Detalhe do serviço" size="960 700" min-size="640 480">
    <resources>
        <style scoped="true">
            .card { padding: 18; background: #161B22; }
        </style>
        <script src="detalhe.luau"></script>
    </resources>

    <container class="card">
        <text content="{servico}" />
    </container>
</screen>
```

Dentro do `<screen>`, o `<resources>` guarda o que a tela **precisa** (estilo,
script, `<link>`, `<import>`) e o resto é o layout. Sem ele, um `<style>` fica
lado a lado com os widgets, no mesmo nível de indentação de algo que aparece na
tela — que é justamente o que este cabeçalho existe para evitar.

O `<resources>` é **opcional**: num arquivo com uma ou duas declarações, elas
podem ficar soltas dentro do `<screen>` mesmo, que o efeito é o mesmo.

```xml
<screen title="Sobre" size="480 320">
    <style scoped="true"> … </style>

    <column> … </column>
</screen>
```

| Atributo | Aliases | O que faz |
|---|---|---|
| `title` | `titulo` | título da barra da janela |
| `size` | `tamanho` | tamanho inicial: `"960 700"`, `"960x700"` ou `"960, 700"` |
| `min-size` | `minSize`, `min_size`, `tamanho-minimo` | tamanho mínimo |
| `resizable` | `redimensionavel` | `"false"` trava o redimensionamento |

`size` usa um par de números (como o `padding`) em vez de `width`/`height`
separados porque esses dois nomes já querem dizer outra coisa no vocabulário de
layout (`fill`, `shrink`, `fill 2`).

As regras:

- **O cabeçalho é obrigatório num arquivo** (desde a 0.61). Todo `.gv` começa com
  `<screen>` (uma janela) ou `<component>` (o resto), e ele envolve o arquivo
  inteiro — `<resources>`, `<props>` e o layout vão dentro. Um arquivo sem
  cabeçalho não carrega, e o erro ensina a forma. A exceção é markup **inline**
  (`Template::Inline`, o que os builtins da própria lib usam): ali não há arquivo
  nem janela a que um cabeçalho se aplique, e ele segue sendo um fragmento — a
  regra distingue pela origem, não pelo conteúdo.
- **O template ganha do builder.** `GlacierDaemon::title()`/`main_size()` passam
  a ser o valor de quando o `.gv` não diz nada. Um campo não declarado não
  opina: só o que está escrito no arquivo sobrepõe.
- **O título acompanha a navegação.** Ir para uma tela que declara `title` troca
  o título da janela; ir para uma que não declara devolve o título base.
- **O tamanho é de quem abre a janela.** Navegar nunca redimensiona, e salvar o
  arquivo (hot-reload) só redimensiona quando o número mudou no `<screen>` — do
  contrário cada `Ctrl+S` desfaria o arrasto que você acabou de dar no canto da
  janela.
- **A geometria lembrada ganha do `size`.** Num app com
  `remember_window_geometry`, o tamanho declarado é o de *primeira* abertura: o
  tamanho em que o usuário deixou a janela vence, senão abrir o app desfaria o
  redimensionamento dele todo boot. O `min-size` do template continua valendo
  como piso dessa geometria.
- **Janela-filha herda do arquivo.** `open_window({ file = "detalhe.gv" })` usa o
  título e o tamanho declarados lá dentro, sem repeti-los na chamada — que
  continua podendo sobrepor os dois quando sabe algo que o arquivo não sabe
  (`title = "Editando nginx"`).
- **Componente importado usa `<component>`.** Um `.gv` trazido por `<import>` é
  um pedaço de tela, não uma janela; `title`/`size` ali não teriam a quem se
  aplicar, e o `<component>` os recusa em vez de ignorá-los.

### `<component>`: a mesma casca para quem não é janela

Nem todo `.gv` é uma tela. Um arquivo importado por outro (`<import>`) é um
pedaço de tela — um card, um item de menu, um badge — e ali `title`/`size` não
teriam a quem se aplicar. Para esses, a raiz é `<component>` (apelido:
`<componente>`):

```xml
<component>
    <resources>
        <style>
            .stat_card { background: #161B22; padding: 16 22; }
        </style>
    </resources>

    <column class="stat_card">
        <text class="stat_num" content="{value}" />
    </column>
</component>
```

É o mesmo agrupamento do `<screen>`, com uma diferença deliberada: **o
`<component>` não leva atributo nenhum**. Escrever `title=` nele é erro de
parse, com a explicação junto (`title/size descrevem uma JANELA, e um
<component> não é uma`) — em vez de aceitar em silêncio um atributo que nunca
teria efeito. As props de um componente continuam vindo de quem o usa
(`<MeuCard prop="…" />`), não do arquivo.

E, porque o cabeçalho é a parte do template que **não desenha nada**, um engano
nele não teria sintoma nenhum — a tela abriria igual, só que sem o que você
escreveu. Por isso ele erra alto, com linha, coluna e o trecho ofensor:

```
erro de XML — views/detalhe.gv:1:9: atributo 'titel' desconhecido no <screen>
  |
1 | <screen titel="Detalhe" size="960 700">
  |         ^
  = dica: o cabeçalho aceita title, size, min-size e resizable (apelidos: titulo, tamanho, tamanho-minimo, redimensionavel)
```

São erros de parse (o template não carrega): arquivo sem cabeçalho; cabeçalho
que não envolve o arquivo inteiro (escrito como *irmão* do layout, ou aninhado
no meio dele); atributo desconhecido no `<screen>` ou no `<resources>`; qualquer
atributo no `<component>` de raiz (o declarado no `<resources>` aceita só o
`name`, e sem ele é erro); `size`/`min-size` que não seja um par de números;
`resizable` que não seja booleano; um widget dentro do `<resources>`; um
`<resources>`/`<props>` fora de um cabeçalho; um `<props>` num `<screen>` (uma
janela não tem quem lhe passe props); `<prop>` sem `name` ou repetido.

### `<component name="…">`: declarar um componente na própria tela

A terceira forma de ter um componente. As outras duas trazem de um arquivo:

```xml
<import name="LinhaLog" from="linha_log.gv" />
<link rel="component" href="linha_log.gv" as="LinhaLog" />
```

Esta declara ali mesmo, dentro do `<resources>` — o componente **é** o que está
escrito entre as tags:

```xml
<screen title="Serviços" size="900 700">
    <resources>
        <component name="LinhaLog">
            <props>
                <prop name="hora" />
                <prop name="texto" />
                <prop name="nivel" default="info" />
            </props>
            <row spacing="12" align_y="center">
                <text content="{hora}" color="#6C7086" size="11" />
                <badge badge_text="{nivel}" />
                <text content="{texto}" />
            </row>
        </component>
    </resources>

    <column>
        <LinhaLog for-each="log" var="l" hora="{l.hora}" texto="{l.texto}" nivel="{l.nivel}" />
    </column>
</screen>
```

**Por que ela existe:** a maior parte dos componentes de uma tela é pequena e só
serve àquela tela — a linha de um item, o cabeçalho de um cartão, um rótulo com
um `<badge>` do lado. Obrigar cada um a virar arquivo troca três linhas de
markup por um arquivo, um caminho relativo e um `<import>`, e espalha por seis
arquivos o que se lê melhor num. É a mesma razão de existir um `<style>` inline
ao lado do `<link rel="stylesheet">`: a forma curta para o que é local, o
arquivo para o que é compartilhado.

**A casca é a mesma, de propósito.** O que vai entre `<component name="X">` e
`</component>` é *byte a byte* o que iria num `.gv` próprio: `<props>` e depois
o layout. A única diferença é o `name` — no arquivo o nome vem do `<import>` que
o traz; aqui ele precisa ser dito. Promover uma declaração a arquivo (ou o
contrário) é recortar e colar, sem reescrever uma linha.

O que ele **não** tem:

- **`<script>` próprio.** Não há arquivo contra o qual resolver um
  `src`/`require` — a mesma limitação do markup inline dos builtins. Na prática
  é o comportamento desejado: as ações escritas dentro de um componente local
  caem no `update` da **tela que o declarou**, que é de quem elas são. O id do
  item viaja dentro da ação, como no `<SpinBox>`: `on_click="detalhar:{s.id}"`.
- **Escopo.** O nome entra no mesmo espaço de nomes de tudo o mais
  (`<import>`, builtins, `register`), com a mesma regra: declara se o nome está
  livre **ou** se hoje ele guarda um builtin da lib. Um componente registrado
  pelo app vence a declaração local.

Componentes locais se compõem entre si e convivem com `<import>` no mesmo
`<resources>`. Ver [`examples/componentes_locais`](examples/componentes_locais),
que põe as duas formas lado a lado no mesmo arquivo.

### `<props>`: o contrato do componente

Um `<component>` pode declarar as props que aceita, e a declaração passa a ser
verificada no ponto de **uso**:

```xml
<component>
    <props>
        <prop name="nome" />
        <prop name="cor" default="#89B4FA" />
    </props>

    <text content="{nome}" color="{cor}" />
</component>
```

- prop passada e não declarada é erro, citando as que existem;
- prop declarada **sem** `default` é obrigatória; com `default`, o valor entra
  quando quem chama omite;
- um `<props>` vazio é um contrato ("não aceito prop nenhuma"), não a ausência
  de um;
- **sem `<props>`, nada é checado** — declarar é opcional.

O motivo de isto ser uma feature e não um comentário no topo do arquivo: as props
entram como uma **camada** sobre o contexto de quem usa, e um lookup que falha na
camada cai para o contexto de baixo. Sem contrato, `<Cartao nomee="Alice" />` não
renderiza vazio — renderiza o `nome` que existir no contexto global, e o typo
fica invisível até alguém reparar no valor errado na tela.

#### `spread`: o objeto inteiro de uma vez

Um card em lista costuma receber um atributo por campo, e **todos** são
mapeamentos identidade — o nome à esquerda igual ao campo à direita:

```xml
<ServiceCard for-each="linhas" var="c"
    id="{c.id}" nome="{c.nome}" porta="{c.porta}" cpu="{c.cpu}" mem="{c.mem}" />
```

`spread` passa o item inteiro no lugar dessa parede:

```xml
<ServiceCard for-each="linhas" var="c" spread="{c}" />
```

Cada campo do objeto cai na prop declarada de mesmo nome. **Dentro do componente
nada muda** — continua `{id}`, `{nome}`, `{porta}` —, e é isso que preserva o
contrato: não existe uma prop-objeto `card` cujo `{card.nmae}` renderizaria vazio
em silêncio. As regras:

- só as props que o `<props>` **declara** entram; campo sobrando no objeto é
  ignorado (o dado quase sempre carrega mais do que o componente usa);
- atributo escrito à mão **ganha** do spread — `spread="{c}" cor="#F00"` sobrepõe
  aquele campo;
- campo ausente cai no `default` do `<prop>`; sem default, é `MissingProp` —
  o contrato ganha alcance, pegando também a obrigatória que o **dado** não
  trouxe, não só a que o markup esqueceu;
- sem `<props>`, não há o que filtrar: todo campo do objeto entra na camada;
- valor **vazio** (a chave ainda não carregou) vale como "nenhum campo"; um
  escalar ou uma lista, aí sim, é erro;
- uma lista aninhada atravessa como JSON e volta a ser lista num `for-each` de
  dentro (`spread="{c}"` com `c.tags` → `<text for-each="tags" var="t">`).

Apelido em português: `espalhar`.

---

## Atributos de layout e estilo

Disponíveis em **qualquer** tag:

| Atributo | Aliases | Valores |
|---|---|---|
| `width` | `largura`, `w` | `fill`, `shrink` ou número (px) |
| `height` | `altura`, `h` | `fill`, `shrink` ou número (px) |
| `padding` | `espacamento_interno` | `"10"`, `"10 20"` (vert. horiz.) ou `"10 20 30 40"` (top right bottom left) |
| `alignX` | `align_x`, `align` | `start`, `center`, `end` |
| `alignY` | `align_y` | `start`, `center`, `end` |
| `spacing` | `espacamento` | número (espaço entre filhos de `Row`/`Column`) |
| `background` | `bg`, `fundo` | cor hex |
| `gradient` | `gradiente` | `"#a #b"` (cima→baixo) ou `"<ângulo> #a #b [#c …]"`; vence `background` |
| `borderRadius` | `border_radius`, `raio_borda` | número |
| `borderWidth` | `border_width` | número |
| `borderColor` | `border_color` | cor hex |
| `class` | `classe` | classes `.gss` separadas por espaço |
| `font` | `fonte`, `font-family` | `mono`/`monospace`/`code` ou `bold` — em `Text`/`Button` |
| `onPress` | `aoPressionar` | ação no **pressionar** (envolve em `mouse_area`); viabiliza `onPress="window:drag"` |
| `onDoubleClick` | `aoClicarDuplo` | ação no **duplo-clique** (ex.: `window:maximize` na barra de título) |
| `cursor` | `cursorIcon` | `pointer`, `text`, `grab`, `grabbing`, `move`, `crosshair`, `wait`, `not-allowed`, `resize-h/v/ne/nw`, … |
| `hidden` | `oculto` | `true`/`false` — remove do layout (não ocupa espaço) |
| `disabled` | `desabilitado` | `true`/`false` — desativa a interação de `Button`/`TextInput`/`Checkbox`/`Toggle` |

- **Eixos:** o eixo cruzado de uma `Column` é o `alignX`; o de uma `Row` é o `alignY`.
- **Cores:** hex `#RRGGBB` ou `#RRGGBBAA`.
- **`fill` só "enche"** se todo container pai até ele também for `width=fill` (o default da maioria dos widgets é `shrink`).

---

## Data binding

Qualquer valor de atributo pode conter placeholders `{chave}`, substituídos pelos
valores do contexto durante a avaliação:

```xml
<Text content="Olá, {user_name}!" color="{cor_texto}" />
<Container background="{painel_bg}"> ... </Container>
```

O componente publica valores com `ctx.set("user_name", "Clara")` (Rust) ou
`ctx.user_name = "Clara"` (Luau) — ou `motor.define_data("user_name", "Clara")`
por fora. Sempre que o contexto muda, o motor reavalia os templates e a UI
reflete o novo valor. **Chaves ausentes viram string vazia.** O estado é
compartilhado entre todas as telas.

---

## Controle de fluxo

A forma recomendada são **atributos diretiva** aplicados em qualquer elemento
(estilo Vue/Angular). A sintaxe antiga de tags-invólucro (`<if>`, `<else>`,
`<ForEach>`) continua suportada por retrocompatibilidade.

**Condicional** — `if` renderiza truthy (`true`/`1`/`yes`/`on`/`sim`); `else`
(pelado) se conecta ao `if` anterior; `equals`/`notEquals` comparam explicitamente:

```xml
<Column if="{logado}"><Text content="Bem-vindo!" /></Column>
<Column else><Text content="Por favor, conecte-se." /></Column>

<Text content="Painel Admin"  if="{papel}" equals="admin" />
<Text content="Acesso Comum"  if="{papel}" notEquals="admin" />
```

> *XML estrito:* atributos pelados como `else` não são válidos no padrão; o
> Glacier faz um pré-processamento transparente convertendo `else` → `else=""`.

Mais quatro comparadores, todos sobre o mesmo `if`/`else-if`:

| Atributo | Casa quando |
|---|---|
| `one_of="a b c"` (`equals_any`) | o valor da chave é **um dos** tokens escritos no markup |
| `contains="rede"` (`contem`, `has`) | o **valor da chave é uma lista** (`"geral,rede"`) que tem esse item — o simétrico do `one_of` |
| `empty` / `not_empty` (pelados) | a chave é (ou não é) um array JSON de zero elementos |

`contains` é o que dá ao motor o **conjunto nomeado**: várias seções de um
accordion abertas, uma seleção múltipla, um filtro por tags — tudo numa chave
de texto que o app nomeia, sem estado por instância. Os separadores aceitos são
vírgula, ponto-e-vírgula e espaço, os três ao mesmo tempo, e o item comparado
também interpola:

```xml
<!-- ctx.abertas = "rede,disco" -->
<template for-each="secoes" var="s">
    <Column if="{abertas}" contains="{s.id}"> … </Column>
</template>
```

**Loop** — `for-each` itera sobre um **array JSON** do contexto; `var` nomeia a
variável (padrão `item`). Objetos viram `{u.campo}`; escalares ficam em `{u}`:

```xml
<CartaoUsuario for-each="usuarios" var="u"
    nome="{u.nome}" cargo="{u.cargo}" cor="{u.cor}" />
```

```rust
ctx.set("usuarios", serde_json::json!([
    { "nome": "Clara",  "cargo": "Engenheira", "cor": "#89B4FA" },
    { "nome": "Sophia", "cargo": "Designer",   "cor": "#F5C2E7" },
]).to_string());
```

Combinados no mesmo elemento, `for-each` tem precedência: desenrola o loop
primeiro e o `if` filtra cada item gerado no contexto local. Veja
[`examples/condicional`](examples/condicional) e [`examples/lista`](examples/lista).

**Agrupar sem wrapper (`<template>`)** — nem a forma-atributo nem `for-each`
num elemento comum resolvem "quero 2+ nós irmãos por condição/iteração, sem
um `<Row>`/`<Column>` extra por baixo" — `if`/`for-each` num elemento SEMPRE
produzem aquele elemento (um nó), nunca uma lista solta. Para isso, use
`<template>` — o mesmo nome e a mesma ideia do `<template v-if>`/`<template
x-for>` do Vue/Alpine (que hoje já são o alvo de uma futura transpilação
deste dialeto para o browser):

```xml
<template if="{aba}" equals="detalhes">
    <Text content="Detalhes" />
    <Text content="{descricao}" />
</template>
<template else-if="{aba}" equals="historico">
    <Text content="Histórico" />
</template>
<template else>
    <Text content="Selecione uma aba" />
</template>

<template for-each="itens" var="i">
    <Text content="{i.nome}" />
    <Text content="{i.detalhe}" />
</template>
```

Cada `<Text>` acima sai como irmão direto do pai de `<template>` — não há
nó `<template>` nenhum na árvore renderizada. É a mesma mecânica que
`<If>`/`<ForEach>` (tags legadas) já tinham; `<template>` só lhes dá um
nome único e comum aos dois lados (se um dia a `<template>`-condição
precisar filtrar itens de um `<template for-each>`, aninhe-a no CORPO do
loop — as duas diretivas não se combinam na mesma tag).

---

## Inputs de texto

`<TextInput>` faz binding bidirecional: `value` aponta para a chave exibida e
`onChange` dispara uma ação com o novo texto a cada tecla. No `update`, o texto
chega em `value: Option<&str>`:

```xml
<TextInput placeholder="Seu nome..." value="user_name" onChange="mudar_nome" width="fill" padding="10" />
```

```rust
fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
    if action == "mudar_nome" {
        if let Some(v) = value { ctx.set("user_name", v); }
    }
}
```

Veja [`examples/perfil`](examples/perfil) e [`examples/navegacao`](examples/navegacao).

---

## Formulários (Reactive Forms)

Inspirado no Angular Reactive Forms: `FormBuilder` declara os `FormControl`s
(nome, valor inicial, validadores) do lado Rust; o componente guarda o `Form` no
seu estado; e o template liga cada input a um controle pelo atributo `formControl`
— o motor cuida do resto (o texto vai para o controle certo, Enter submete e
avança para o próximo campo).

```rust
let form = FormBuilder::new("login")
    .control(FormControl::new("username", "").required().min_length(3))
    .control(FormControl::new("password", "").required().min_length(6))
    // A lógica de submissão fica junto dos controles.
    .on_submit(|form, ctx| {
        if form.is_valid() {
            ctx.set("status", format!("Bem-vindo, {}!", form.value("username")));
        } else {
            form.validate();                 // marca também campos não tocados
            form.errors_to_context(ctx, "erro_");
        }
    })
    .build();
```

```xml
<Form onSubmit="entrar" name="login" width="fill">
    <TextInput formControl="username" placeholder="usuário" width="fill" />
    <TextInput formControl="password" placeholder="senha" secure="true" width="fill" />
    <Button text="Entrar" on_click="entrar" />
</Form>
```

`TextInput formControl="username"` sem `value`/`onChange` usa o nome do controle
para os dois. No `update`, `Form::has_control` reconhece a ação de campo sem um
`match` por campo — e a **submissão** vai por um método próprio, `on_form_submit`,
então atualização de campo e submissão nunca competem pelo mesmo `match`:

```rust
fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
    if self.form.has_control(action) {
        self.form.set_value(action, value.unwrap_or_default());
        self.form.sync_to_context(ctx);
    }
}
fn on_form_submit(&mut self, _action: &str, ctx: &mut Context) {
    self.form.submit(ctx);   // roda a closure registrada em .on_submit(...)
}
```

Validadores: `.required()`, `.min_length(n)`, `.max_length(n)`,
`.pattern(regex)`, `.validator(|v| Ok(()) | Err(msg))`. `Form::errors_to_context`
publica o primeiro erro de cada campo (`"{prefixo}{nome}"`) para exibir inline com
`Text "{erro_username}"`. Enter em qualquer campo dispara o `onSubmit` **e** avança
o foco — dá para preencher e enviar o formulário sem tocar no mouse. Veja
[`examples/formulario_login`](examples/formulario_login).

---

## Navegação entre telas

Cada tela é um componente registrado. Há três formas de trocar de tela:

**1. Declarativa (atributos no XML):**

```xml
<Button text="Ver perfil" navigateTo="perfil" />
<Button text="Voltar" navigateBack="true" />
```

**2. Imperativa (Rust):** de dentro do `update`, via `ctx.navigate_to(...)` /
`ctx.navigate_back()`; ou no motor, `motor.navigate_to(...)`.

**3. Decidida por script (Luau):** `navigate(tela)` / `navigate_back()` — o
script decide se navega (ex.: só depois de validar o login):

```lua
function entrar()
    if ctx.usuario == "admin" and ctx.senha == "123" then
        navigate("dashboard_luau")
    else
        ctx.erro = "Usuário ou senha inválidos."
    end
end
```

O motor mantém uma **pilha de histórico**: `navigateTo` empilha a tela atual;
`navigateBack` volta. O estado de contexto é compartilhado entre telas. Veja
[`examples/navegacao`](examples/navegacao) (declarativa) e
[`examples/navegacao_luau`](examples/navegacao_luau) (via script).

---

## Componentes e composição

**Composição a nível de template** — duas formas equivalentes; os atributos viram
**props** interpoladas no contexto local do filho:

```xml
<import name="PerfilCard" from="examples/perfil/perfil_card.gv" />
<PerfilCard nome="{user_name}" cargo="{user_role}" />

<!-- ou -->
<Include src="perfil_card.gv" nome="{user_name}" />
```

**O trait `Component`** — encapsula UI + comportamento + estado:

```rust
pub trait Component {
    fn name(&self) -> &str;                  // nome (registro + roteamento)
    fn template(&self) -> Template;          // Template::File(path) | Template::Inline(xml)
    fn init(&mut self, ctx: &mut Context) {} // estado inicial (opcional)
    fn children(&self) -> Vec<Box<dyn Component>> { Vec::new() } // sub-componentes (opcional)
    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context);
    fn on_form_submit(&mut self, action: &str, ctx: &mut Context) {} // onSubmit de um <Form>
    fn subscription(&self) -> iced::Subscription<EngineMessage> { /* streams/rede */ }
}
```

**Componentes aninhados e roteamento** — um `Component` pode **possuir** outros
via `children()`. Ao registrar o pai, o motor registra os filhos em cascata, e as
ações que saem da UI de um filho são roteadas para o `update` **do filho**. O motor
prefixa as ações da subárvore com o nome do componente
(`incrementar` → `CartaoContador::incrementar`); no `dispatch`:

- prefixo de um componente com comportamento → vai para ele;
- ação sem prefixo, ou prefixo só de UI → cai na **tela ativa** (fallback) — é o
  que mantém includes puramente visuais funcionando.

> **Limite conhecido:** um filho referenciado N vezes (ex.: dentro de `<ForEach>`)
> compartilha um único objeto e um único `update`.

**`ContextVar`** — açúcar para declarar variáveis legíveis em vez de chaves soltas:

```rust
ctx.set_var(&ContextVar::new("user_name", "Clara Silva"));
```

Veja [`examples/aninhado`](examples/aninhado), [`examples/lista`](examples/lista)
e [`examples/perfil`](examples/perfil).

---

## Comportamento em `<script>` Luau

Dá para colocar o comportamento **dentro do próprio template**, num bloco
`<script>` com funções **Luau**. Ao registrar com `register_component`, o motor
detecta o `<script>`, carrega o script e roteia cada ação
(`on_click`/`onChange`/`onSubmit`) para a função de mesmo nome — tudo
**interpretado em tempo de execução**, sem recompilar.

```xml
<Button text="+" on_click="incrementar" />
<Button text="-" on_click="decrementar" />

<script>
function init()        ctx.contador = ctx.contador or 0 end
function incrementar() ctx.contador = ctx.contador + 1 end
function decrementar() ctx.contador = ctx.contador - 1 end
</script>
```

```rust
motor.register_component("contador", "examples/contador_macro/contador_macro.gv")?;
```

Como funciona:

- cada função Luau vira uma ação homônima (casada com `on_click`/`onChange`/`onSubmit`);
- o **contexto** do motor é a tabela global `ctx`: ler `ctx.contador` devolve o valor atual, atribuir grava de volta. Luau coage strings numéricas, então `ctx.contador + 1` sobre `"0"` volta `"1"`. Atribuir `ctx.x = nil` **remove** a chave;
- atribuir uma **tabela** a `ctx.x` serializa via `json.encode` automaticamente;
- ações de `onChange` recebem o texto digitado como 1º argumento **e** na global `value`;
- `init()` (opcional) semeia o estado inicial.

**Arquivo externo** — aponte para um `.luau` separado com `src` (resolvido
relativo ao diretório do template):

```xml
<script src="contador_externo.luau"></script>
```

Veja [`examples/contador_macro`](examples/contador_macro) (inline) e
[`examples/contador_externo`](examples/contador_externo) (externo).

### `fetch`: rede async via corrotina

`fetch(url, opts)` faz HTTP/HTTPS (via `hyper` + `rustls`). Ela **suspende a
corrotina** da ação e retoma quando a resposta chega — a UI **não trava**, mas o
código fica com cara de `await`, linear:

```lua
function buscar()
    ctx.status = "carregando..."                     -- já aparece na tela
    local res = fetch("https://api.ipify.org?format=json") -- suspende aqui
    if res.ok then
        ctx.resultado = res.body                     -- retomou com a resposta
        ctx.status = "ok (" .. res.status .. ")"
    else
        ctx.status = "falhou"
    end
end
```

O retorno é `{ ok, status, body, error }`. O 2º argumento `opts` é opcional:
`{ method = "POST", body = "...", headers = { ["Authorization"] = "..." } }`.
Veja [`examples/fetch_luau`](examples/fetch_luau).

### `require`: módulos Luau

Extraia lógica em **bibliotecas** `.luau` e importe com `require` — encapsuladas e
reutilizáveis entre componentes:

```lua
-- net/http_client.luau
local Client = {}
Client.__index = Client
function Client.new(base) return setmetatable({ base = base, headers = {} }, Client) end
function Client:get(path) return fetch(self.base .. path, { headers = self.headers }) end
return Client
```

```lua
local http = require("net/http_client")
local api  = http.new("https://api.exemplo")
function carregar() local res = api:get("/dados"); if res.ok then ctx.dados = res.body end end
```

`require("a/b")` procura `a/b.luau` (e `a/b/init.luau`) **nesta ordem**: (1) o
diretório do template; (2) um subdir `lib/`; (3) cada caminho em
`GLACIER_LUAU_PATH` (separados por `:`). O módulo roda no **mesmo** interpretador,
então enxerga `fetch` e as globais; é carregado **uma vez** e cacheado. Veja
[`examples/imports_luau`](examples/imports_luau).

### Timers: `after` e `every`

- **`after(ms, fn)`** — temporizador de **disparo único** (setTimeout). Não suspende: agenda `fn` (função ou nome de global) e devolve um handle cancelável.
- **`every(ms, fn)`** — temporizador **repetitivo** (setInterval), construído sobre `after`: reagenda a si mesmo a cada disparo, com `:cancel()` estável entre repetições.

```lua
-- dispara uma vez após 3s, cancelável antes disso
local t = after(3000, "tempo_esgotado")
t:cancel()

-- repete a cada 1s até cancelar
cronometro = every(1000, "tique")
function tique() ctx.tiques = ctx.tiques + 1 end
function parar() cronometro:cancel() end
```

Veja [`examples/robustez_luau`](examples/robustez_luau).

### `storage`: persistência local

`storage.get/set/remove` guardam JSON em disco por componente, sobrevivendo a
reiniciar o processo:

```lua
function init()   ctx.rascunho = storage.get("rascunho") or "" end
function salvar() storage.set("rascunho", ctx.rascunho) end
```

### `date`: data e hora sobre strings ISO

Os campos `<dateedit>`/`<timeedit>`/`<datetimeedit>` e as grades
`<calendar>`/`<monthyearpicker>`/`<daterangepicker>` gravam **sempre em ISO** —
`YYYY-MM-DD`, `HH:MM[:SS]`, ou os dois separados por espaço. O global `date`
opera nesse mesmo formato: recebe string, devolve string, e por isso nada de
"objeto data" vaza para uma chave de contexto (que é sempre texto).

Não há dependência nova no motor: `today`/`now`/`time` saem do `os.date` que o
próprio Luau já tem (hora **local**), e o resto é aritmética civil.

```lua
function init()
    ctx.entrada = date.today()                          -- "2026-09-01"
    ctx.saida   = date.add(ctx.entrada, { days = 2 })   -- "2026-09-03"
    ctx.aviso   = date.now()                            -- "2026-09-01 14:30"
end

function recalcular()
    -- Dias de CALENDÁRIO (a hora não entra) — a conta de uma diária, um prazo.
    ctx.noites = date.diff(ctx.saida, ctx.entrada)
    -- A chave continua ISO; isto é só o que vai para a tela.
    ctx.rotulo = date.format(ctx.entrada, "DD/MM/YYYY")
end
```

| Função | Devolve |
|---|---|
| `date.today()` | hoje, `YYYY-MM-DD` |
| `date.now(segundos?)` | agora, `YYYY-MM-DD HH:MM[:SS]` |
| `date.time(segundos?)` | hora do relógio, `HH:MM[:SS]` |
| `date.parse(iso)` | `{ year, month, day, hour, min, sec }` ou `nil` |
| `date.valid(iso)` | booleano |
| `date.weekday(iso)` | `1`–`7`, **1 = domingo** (a base do `os.date("*t").wday`) |
| `date.date_of(iso)` / `date.time_of(iso)` | uma seção do valor, ou `nil` se ela não existe |
| `date.days_in_month(ano, mes)` | `28`–`31`, com bissexto |
| `date.compare(a, b)` | `-1` / `0` / `1` |
| `date.is_before(a, b)` / `date.is_after(a, b)` | booleano |
| `date.add(iso, delta)` | ISO na **mesma forma** da entrada |
| `date.diff(a, b)` / `date.diff_seconds(a, b)` | dias de calendário / segundos |
| `date.format(iso, fmt)` | texto (`YYYY` `YY` `MM` `DD` `HH` `mm` `SS`) |
| `date.epoch(iso)` | segundos desde 1970 |
| `date.from_epoch(secs, utc?)` | `YYYY-MM-DD HH:MM:SS` |
| `date.to_local(iso)` / `date.to_utc(iso)` | o mesmo instante na outra hora de parede |

Três detalhes que economizam bugs:

- **Comparar é pelo instante, não pelo texto.** ISO ordena como string, mas só
  entre valores da **mesma forma**: `"2026-09-10 08:00" > "2026-09-10"` é
  verdadeiro só porque a string é mais longa, ainda que os dois sejam o mesmo
  dia. `date.is_after` e `date.compare` tiram essa pegadinha da frente — uma
  data pura vale a meia-noite dela.
- **`add` preserva a forma.** Somar um dia a um `YYYY-MM-DD` não inventa um
  `00:00` no fim; somar uma hora a um `HH:MM` vira dentro do dia, porque não há
  data para onde transbordar. `months`/`years` andam pelo **calendário** e
  grudam no fim do mês (31/01 + 1 mês = 28/02, como o `QDateEdit` ao trocar a
  seção do mês); `days`/`hours`/`minutes`/`seconds` são duração pura.
- **O parse aqui é estrito.** Entrada inválida — inclusive uma data que não
  existe, como `2026-02-31` — devolve `nil`. É o oposto do parse do widget, que
  é tolerante de propósito para não renderizar quebrado enquanto a pessoa
  digita; um script pode escolher o que fazer com o `nil`, um widget no meio de
  um quadro não pode.

O relógio é lido **na chamada**: uma tela que precisa andar sozinha combina com
`every` — `every(1000, function() ctx.agora = date.now(true) end)`.

#### Fuso: RFC 3339 na entrada, hora local na tela

Um valor **sem** fuso é hora local — é o que `today`/`now` devolvem e o que os
campos de edição guardam. Um valor **com** fuso é aceito em toda entrada:
`2026-07-06T12:34:56Z`, `...-03:00`, `...-0300`, com fração de segundo opcional
(aceita e descartada). O offset viaja junto com o valor, então `add` o preserva
e `format` desenha as componentes como estão escritas.

Deslocar o instante é **explícito** — só `to_local` e `to_utc` fazem isso:

```lua
-- o que o backend mandou -> o que a tela mostra
ctx.inicio = date.format(date.to_local(dep.started_at), "DD/MM HH:mm")

-- e a volta, para mandar de novo
local agora_utc = date.format(date.to_utc(date.now(true)), "YYYY-MM-DDTHH:mm:SSZ")
```

A exceção é a comparação: `compare`/`diff`/`diff_seconds` trazem um valor com
fuso para a hora local antes de comparar, para que um `...Z` do backend e um
`date.today()` da tela sejam comparáveis direto. Isso lê o instante, não muda
nenhum valor.

> **`os.time` do Luau não é o do Lua.** Ele usa `timegm`, e não `mktime`: uma
> tabela de componentes é lida como **UTC**. Por isso o truque clássico de achar
> o offset local — `os.difftime(t, os.time(os.date("!*t", t)))` — devolve
> **zero** aqui, sem erro nenhum. O que funciona é `os.time(os.date("*t", t)) - t`,
> e é o que o `date` usa por dentro. Se você tem código que faz a conta na mão,
> vale conferir.

Veja [`examples/data_hora_luau`](examples/data_hora_luau).

### `viewport`, `toast`, `confirm`, `navigate`

- **`viewport()`** → `{ width, height }` em px lógicos (tamanho atual da janela).
- **`toast(opts)`** — notificação efêmera; `opts` = string ou `{ message, kind?, title? }` (kind: `info`/`success`/`warning`/`error`).
- **`confirm(opts)`** — diálogo modal; o botão de confirmação despacha `opts.confirm_action`.
- **`navigate(tela)` / `navigate_back()`** — navegação (ver [Navegação](#navegação-entre-telas)).

Nenhuma delas suspende a corrotina — o motor aplica o efeito e retoma na hora.

### Múltiplas janelas: `open_window`, `broadcast`, `close_window`

No modelo multi-janela (runner `GlacierDaemon`, sobre `iced::daemon`) cada
janela é um `GlacierUI` **independente** — contexto e estado isolados. Estas três
funções do prelúdio coordenam janelas (nenhuma suspende a corrotina):

- **`open_window(opts)`** — abre uma janela nova. `opts` = string (caminho de
  template) ou tabela:
  ```lua
  open_window("telas/detalhe.gv")
  open_window({
      file = "telas/detalhe.gv",   -- ou component = "nome_registrado"
      title = "Detalhe", width = 460, height = 340,
      data = { url = ctx.api_url, token = ctx.api_token },  -- semeia o contexto da nova janela
  })
  ```
  Os pares de `data` são gravados no contexto do motor da nova janela **antes**
  do `init` — é como passar parâmetros para ela.
- **`broadcast(event, payload)`** — envia uma mensagem para as **outras** janelas
  (não para a própria). `payload` é opcional; uma tabela é serializada em JSON. A
  janela receptora trata em `on_broadcast(event, payload)` (função global), com o
  `payload` já decodificado de volta numa tabela:
  ```lua
  -- janela A (emissora):
  broadcast("project_created", { id = "42", name = "api" })
  -- janela B (receptora):
  function on_broadcast(event, payload)
      if event == "project_created" then ctx.ultimo = payload.name end
  end
  ```
- **`close_window()`** — fecha a **própria** janela (o motor isolado não conhece
  o próprio `window::Id`; quem fecha é o daemon). Comum logo após um `broadcast`,
  no padrão "janela auxiliar devolve um resultado e some".

Do lado Rust, os equivalentes são `Context::open_window` / `Context::broadcast`
/ `Context::close_window` e o método `Component::on_broadcast`. Veja
[`examples/janelas_glacier`](examples/janelas_glacier).

### Erros visíveis: `on_error`

Um erro de runtime no script vira **visível** em vez de sumir num `eprintln!`.
Defina `on_error(msg)` opcional para controlar a mensagem ao usuário (e guardar o
erro técnico); sem ele, o motor promove a mensagem crua a um **toast** automático:

```lua
function on_error(msg)
    ctx.ultimo_erro = msg
    toast({ title = "Erro no script", message = "Algo deu errado.", kind = "error" })
end
```

### Streams: SSE e WebSocket

Ao contrário do `fetch` (one-shot), `sse` e `websocket` são streams de **vida
longa**: NÃO suspendem — registram o stream e devolvem um handle na hora. Cada
evento chama o callback correspondente em `opts` (`on_open`, `on_message`,
`on_error`, `on_close`), que escreve em `ctx` como qualquer ação:

```lua
sse_conn = sse("https://sse.dev/test", {
    on_open    = function() ctx.sse_status = "aberto" end,
    on_message = function(data) ctx.sse_msg = data end,
    on_close   = function() sse_conn = nil end,
})
function fechar() sse_conn:close() end

ws_conn = websocket("wss://echo.websocket.org", {
    on_message = function(data) ctx.ws_msg = data end,
})
ws_conn:send("ping")   -- envia pela conexão viva
```

O callback é uma **função** — closure, upvalue e método de tabela funcionam, e
o handler não precisa ser global nem ter nome:

```lua
local function assinar(canal, destino)
    return sse("https://ex/" .. canal, {
        -- `destino` é um upvalue: a mesma função serve a vários canais.
        on_message = function(data) ctx[destino] = data end,
    })
end
```

Um **nome de função global** (`on_message = "sse_recebeu"`) também é aceito,
como atalho. Prefira a função: o nome obriga o handler a ser global, não fecha
sobre nada, e um nome errado falha em silêncio — o evento chega e não chama
ninguém.

**Importante:** os streams viram `iced::Subscription`s produzidas por
`GlacierUI::subscription`. O `subscription()` do app precisa incluir
`self.motor.subscription()` — sem isso, nenhuma conexão é aberta. Veja
[`examples/stream_lua`](examples/stream_lua).

---

## Estilos `.gss`

Um `.gss` (*glacier stylesheet*) é um arquivo CSS-like que tira estilos repetidos
da markup e os agrupa em **classes**. Aplique com `class="..."`:

```gss
/* Comentários // de linha e de bloco. '#' nunca é comentário (cores ficam intactas). */
.card  { padding: 24; background: #1E1E2E; border-radius: 16; border-width: 1; border-color: #313244; align-x: Center; }
.title { size: 26; bold: true; color: #CDD6F4; }
```

```xml
<Container class="card"><Text class="title" content="Olá" /></Container>
```

**Precedência (igual à do CSS):**

1. um **atributo inline** no nó sempre vence tudo;
2. um seletor de **id** (`#nome`) vence a classe;
3. classes aplicam da **esquerda para a direita** (`class="a b"` → `b` sobrepõe `a`);
4. um seletor de **tag** (`Button`, `Card`) é o de **menor** especificidade — abaixo de classe/id/inline;
5. estilos **globais** primeiro, depois os **com escopo** do componente.

Especificidade, do mais fraco ao mais forte: **tag < classe < id < inline**.

**Seletor de id.** Além de `.classe`, um bloco `#nome { }` casa o atributo
`id="nome"` do nó e é aplicado **por cima** das classes (mas ainda por baixo do
inline). Bom para estilizar *um* elemento sem inventar uma classe descartável.
Aceita pseudo-estados (`#salvar:hover { }`) e vale dentro de `@media`. A
unicidade não é exigida — vários nós podem compartilhar o mesmo `id`.

```gss
.title  { color: #CDD6F4; }
#hero   { color: #F38BA8; }   /* vence a .title onde id="hero" */
```

```xml
<Text id="hero" class="title" content="Olá" />
```

**Seletor de tag.** Um bloco `Tag { }` casa elementos pela **tag da markup**, com
a **menor** especificidade (abaixo de classe/id/inline) — bom para *defaults*.
Casa por dois caminhos: o **tipo builtin** do nó (`Button`, `Column`, `Text`, …)
e o **nome de um componente** no seu uso (`<Card/>`). Como o componente é
inlinado, um `Card {}` é aplicado como base na **raiz** do template dele. O nome
é normalizado para minúsculo (`Button {}` == `button {}`) e aceita pseudo-estados
(`Button:hover { }`) e `@media`.

```gss
Button { border-radius: 8; }   /* default de todo Button (builtin) */
Card   { padding: 24; }        /* default de todo uso de <Card> (componente) */
```

> Cuidado com o alcance: uma regra de tag **global** atinge o elemento em
> **todos** os componentes (não é opt-in como a classe). Prefira id/classe quando
> quiser mirar um caso específico. Um componente de template **multi-raiz**
> (`Fragment`) não recebe o underlay de `Card {}` — use uma raiz única.

#### `class` no **uso** de um componente (0.69)

O outro extremo da escada. `Card {}` acima é o *default* de todo uso; a `class`
escrita **num** uso mira só aquele, e também aplica na **raiz** expandida:

```gv
<Card class="destaque" />
```

```gss
.destaque { background: #3B1F1F; }
```

A escada completa, do mais fraco ao mais forte:

```
tag de componente (Card {})  <  tag builtin  <  classe do template  <
classe do USO  <  id do template  <  inline do template
```

Ou seja: **a classe escrita no uso vence as classes do template, e perde para os
atributos inline do template.** É a intuição do CSS — a classe do autor do
componente é um *default*, o atributo que ele cravou inline é uma *decisão*.

> Antes da 0.69 escrever `class` num componente era um **no-op silencioso**: a
> classe era lida, viajava no mapa de props e não pintava nada, sem erro nem
> aviso. Se você contornou isso embrulhando o componente numa `<Column
> class="…">`, o embrulho pode sair.

Ela aplica **só na raiz**. Para estilizar um nó específico lá dentro, o
componente expõe uma prop com nome próprio — e **todo builtin da lib tem as
suas** desde a 0.89:

```gv
<listview items="servicos" value="qual" selected="{qual}"
          item_class="linha" selected_class="linha_ativa" />

<card title="Servidor" class="destaque" title_class="titulo_card" />
```

O sufixo é sempre `_class` e o prefixo nomeia o alvo (`field_class`,
`item_class`, `title_class`, `bar_class`, `head_class`…). Três regras que a
biblioteca inteira segue, e que valem para um componente seu:

1. **A classe injetada entra depois da classe da lib no mesmo nó**, então ela
   redefine o que declara e herda o resto — inclusive os `:hover`.
2. **Num par base/refinamento, o refinamento vem por último**:
   `item_class` primeiro, `selected_class` depois, e o segundo vence no item
   selecionado.
3. **Nó de raiz não ganha prop** — o `class` do uso já o alcança. É por isso que
   o `<toolbutton>`, cuja raiz *é* o `<Button>`, não tem `button_class`.

A tabela por widget está em [`BUILTINS.md`](BUILTINS.md).

> **Um atributo inline que resolve para vazio cai na classe** (0.89). Um
> `background="{bg}"` no template de um componente vencia a classe mesmo quando
> a prop não vinha — o campo virava `Some("")` e o widget saía sem fundo nenhum.
> É o que permite um template aceitar a cor por prop **e** ter um default por
> classe; o corolário para quem escreve um componente é que o default de uma cor
> vai numa classe, não num `{prop|#aabbcc}` (que resolve sempre, e resolvendo
> sempre vence toda classe).

**Propriedades reconhecidas:** `width`/`w`, `height`/`h`, `padding`, `spacing`,
`align-x`/`align-y`, `background`/`bg`, `border-radius`, `border-width`,
`border-color`, `color`, `text-color`, `size`, `bold`, `hidden`.

Carregue por código (`motor.load_stylesheet("styles/app.gss")` — sempre **global**)
ou declare no template. Um `<style>` inline é **global** por padrão, ou **com
escopo** ao componente com `scoped="true"` — a única forma de escopar um `.gss`:

```xml
<style>
    .card  { padding: 24; background: #1E1E2E; border-radius: 16; }
</style>
<style scoped="true">
    .only_here { color: red; }
</style>
```

Veja [`examples/estilos`](examples/estilos) (arquivo + `<link>`) e
[`examples/estilos_inline`](examples/estilos_inline) (bloco `<style>`).

### Pseudo-estados: `:hover` / `:focus` / `:active` / `:disabled`

Uma classe pode declarar overlays por pseudo-estado — cada bloco sobrescreve só os
campos que declara, por cima da regra base (igual ao CSS):

```gss
.btn          { background: #313244; text-color: #CDD6F4; border-radius: 8; }
.btn:hover    { background: #45475A; }
.btn:active   { background: #1E1E2E; }
.btn:disabled { background: #181825; text-color: #6C7086; }
```

Cada pseudo-estado é mapeado para o `Status` nativo do widget do iced — nada de
rastrear hover manualmente. **Cobertura atual:**

- **`Button`** — `:hover`/`:active`/`:disabled` completos (requer uma `color` base na classe).
- **`TextInput`** — `:hover`/`:focus`/`:disabled` completos.
- **`Select`** — só `:hover` (o `pick_list` do iced não tem `Status::Disabled`).
- **`Checkbox`/`Toggle`** — só o atributo `disabled` (usam o visual padrão do tema).

Veja [`examples/pseudo_estados`](examples/pseudo_estados).

---

## `<link rel="…">` e temas

O `<link>` declara um recurso externo; `rel` escolhe o tipo:

| `rel` | O que faz | Atributos |
|---|---|---|
| `stylesheet` (padrão) | carrega um `.gss` **global** | `href` |
| `import` / `component` | carrega outro template (igual a `<import>`) | `href`, `as`/`name` |
| `data` | faz merge de um JSON no contexto | `href`, `as`/`name` |
| `theme` | aplica uma paleta como `iced::Theme` | `href` |

```xml
<link rel="stylesheet" href="styles/estilos.gss" />
<link rel="import" href="templates/perfil_card.gv" as="PerfilCard" />
<link rel="data" href="data/equipe.json" as="app" />   <!-- {app.campo}, <ForEach items="app.lista"> -->
<link rel="theme" href="styles/theme.json" />
```

**Tema** — um JSON de cores hex aplicado como `iced::Theme`:

```json
{ "name": "Mocha", "background": "#181825", "text": "#CDD6F4",
  "primary": "#89B4FA", "success": "#A6E3A1", "danger": "#F38BA8" }
```

Ligue-o na `application` via `motor.theme()` (devolve `Theme::Dark` se nenhum foi
carregado) — também resolve o "fundo branco" padrão do `iced`:

```rust
iced::application(App::new, App::update, App::view)
    .theme(|app| app.motor.theme())
    .run()
```

Como os `<import>`, os `<link>`/`<style>`/`<script>` podem ficar no topo do
arquivo, como irmãos da raiz, e não renderizam nada.

---

## Estilos builtin (QStyle-like)

Quatro estilos prontos em [`glacier_ui::style`](src/style.rs) — o análogo dos
`QStyle` do Qt (`Fusion`, `windowsvista`, …): **`FROST`** (claro nativo),
**`FUSION`** (claro cinza), **`FUSION_DARK`** (escuro azul) e **`PHANTOM`**
(escuro grafite). Cada um é uma `const Style` com **paleta** (vira o
`iced::Theme` do app) + **GSS de regras de tag** (`Button { }`, `Select { }`,
com pseudo-estados), instalado como *underlay* — abaixo de qualquer `.gss` do
app, então classes, ids, atributos inline e `<link rel="theme">` continuam
vencendo, exatamente como um stylesheet vence o QStyle no Qt.

```rust
use glacier_ui::{style, GlacierDaemon};

GlacierDaemon::new()
    .style(style::FUSION_DARK)   // default de TODAS as janelas (QApplication::setStyle)
    .main(|motor| { /* como sempre */ })
    .run()
```

Num app de janela única, `motor.set_style(&style::FUSION)?`. Para trocar em
runtime (o combo "Style:" do Widget Gallery do Qt), duas ações builtin — nenhum
código de componente envolvido:

```xml
<Button text="Escuro" on_click="style:fusion-dark" />
<Select options="estilos" value="glacier_style" onChange="style:set" />
```

O nome do estilo ativo fica no contexto em `glacier_style`
(`style::CONTEXT_KEY`), e o GSS de cada estilo publica a paleta como variáveis
(`var(--primary)`, `var(--surface)`, `var(--border)`, …) para o `.gss` do app.
Um app também pode declarar o próprio `const Style { … }` e passá-lo aos mesmos
pontos. Demo completa: `cargo run --example galeria_estilos`.

---

## Toasts e diálogos (em Rust)

**Toasts** — notificações efêmeras empilhadas no canto, dispensadas sozinhas após
alguns segundos (ou pelo "×"):

```rust
ctx.show_toast(ToastSpec::success("Serviço publicado."));
ctx.show_toast(ToastSpec::warning("Fica 10s.").with_title("Custom").with_duration(Duration::from_secs(10)));
```

Requer `GlacierUI::toast_subscription(...)` no `subscription()` do app — sem ele,
os toasts só fecham no "×". Veja [`examples/toasts`](examples/toasts).

**Diálogos** — modais estilo `QMessageBox` (informação, aviso, erro, pergunta,
confirmação), sobrepostos pelo motor:

```rust
ctx.show_dialog(DialogSpec::error("Falha no deploy", "Porta 8080 já em uso."));
ctx.show_dialog(
    DialogSpec::confirm("Excluir projeto", "Essa ação não pode ser desfeita.")
        .with_detail("3 serviços serão removidos.")
        .with_button(DialogButton::discard("excluir_confirmado")),
);
```

Os botões despacham ações (`"ok"`, `"yes"`, `"no"`, `"cancel"`, ou a ação
customizada) roteadas ao `update` — o motor já fechou o diálogo antes. Veja
[`examples/dialogs`](examples/dialogs). (Da camada Luau, use `confirm(opts)`.)

---

## Drag-and-drop: listas reordenáveis

Um `<ForEach>` com `onReorder` + `reorderKey` vira uma lista reordenável por
arrasto: arraste pelo elemento marcado `dragHandle="true"`. Ao soltar, `onReorder`
entrega a nova ordem (array JSON dos valores de `reorderKey`). Durante o arrasto,
o item agarrado recebe `{t.__dragging} = "true"` para destacá-lo:

```xml
<ForEach items="tarefas" var="t" onReorder="reordenar" reorderKey="id">
    <Row if="{t.__dragging}" equals="true" background="#434C5E" borderColor="#88C0D0" ...>
        <Text content="⋮⋮" dragHandle="true" cursor="grabbing" />
        <Text content="{t.nome}" width="fill" />
    </Row>
    <Row else="true" background="#3B4252" ...>
        <Text content="⋮⋮" dragHandle="true" cursor="grab" />
        <Text content="{t.nome}" width="fill" />
    </Row>
</ForEach>
```

Requer `self.motor.subscription()` no `subscription()` do app (carrega o listener
global de "soltar o mouse" que encerra o drag). Veja
[`examples/lista_reordenavel`](examples/lista_reordenavel).

---

## Ações built-in

Algumas ações de `on_click`/`onPress` são tratadas pelo motor, sem código no
componente:

| Ação | Efeito |
|---|---|
| `clipboard:<chave>` | copia o valor de contexto `<chave>` para a área de transferência |
| `window:minimize` | minimiza a janela |
| `window:maximize` | alterna maximizar/restaurar (alias `window:toggle_maximize`) |
| `window:close` | fecha a janela |
| `window:drag` | inicia o arraste — use em `onPress` de uma região da barra de título |
| `window:resize:<dir>` | inicia o redimensionamento — `<dir>` ∈ `n,s,e,w,ne,nw,se,sw` |

Permitem montar uma barra de título customizada para uma janela sem decorações
(`decorations: false` nas `window::Settings`):

```xml
<Row width="fill" onPress="window:drag"><Text content="Meu App" /></Row>
<Button text="—" on_click="window:minimize" />
<Button text="✕" on_click="window:close" />
```

---

## Hot-reload

Recursos carregados de arquivo são recarregados quando mudam em disco: **templates**
(inclusive `<import>`/`<link>` novos), **stylesheets `.gss`**, **dados**
(`<link rel="data">`), **tema** e a **lógica Luau** de um `<script src>`. Ligue a
subscription:

```rust
fn subscription(&self) -> iced::Subscription<EngineMessage> {
    GlacierUI::reload_subscription(std::time::Duration::from_millis(500))
}
```

Edite o XML, o `.gss`, o JSON, o tema ou o `.luau` e veja a UI atualizar sem
recompilar. Só a lógica em Rust exige um novo build.

---

## Rede e async em Rust

Além da camada Luau, um `Component` pode disparar I/O por **efeitos** e receber
fluxos por **subscriptions**.

**Efeitos pontuais** — dentro do `update`, `ctx.perform(future)`. Ao completar,
os pares `(chave, valor)` são mesclados no contexto e a UI reavalia. O
`EffectOutcome` também carrega um **toast** opcional:

```rust
fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
    if action == "salvar" {
        ctx.perform(async {
            match salvar_no_servidor().await {
                Ok(_)  => EffectOutcome::data(vec![("salvo".into(), "true".into())])
                    .with_toast(ToastSpec::success("Salvo.")),
                Err(e) => EffectOutcome::toast(ToastSpec::error(format!("Falha: {e}"))),
            }
        });
    }
}
```

Para isso, `dispatch` devolve `iced::Task<EngineMessage>` — repasse-a no `update`
da app (`self.motor.dispatch(&msg)`).

**Fluxos contínuos** — implemente `Component::subscription` devolvendo uma
`iced::Subscription` que emita `EngineMessage::ContextPatch(pares)`. O motor agrega
tudo em `GlacierUI::subscription`, que você liga à app. Cada item recebido mescla
no contexto e reavalia — sem escrever `match` de mensagens.

```rust
fn subscription(&self) -> iced::Subscription<EngineMessage> {
    Subscription::batch([
        self.motor.subscription(),                                    // rede/streams dos componentes
        GlacierUI::reload_subscription(Duration::from_millis(500)),   // hot-reload
    ])
}
```

---

## Referência da API

### `GlacierUI`

| Método | Descrição |
|---|---|
| `new()` | cria um motor vazio. |
| `register(Box<dyn Component>)` | registra um componente (UI + comportamento + `children()` em cascata). |
| `register_component(name, path)` | registra de um arquivo; liga o comportamento Luau se houver `<script>`. |
| `load_stylesheet(path)` | carrega/recarrega um `.gss` **global** e reavalia tudo. |
| `theme()` | o `iced::Theme` do `<link rel="theme">`, ou `Theme::Dark`. |
| `dispatch(&EngineMessage)` | roteia a mensagem, aplica navegação/reload/patch e devolve uma `iced::Task` com os efeitos. |
| `subscription()` | agrega as `Component::subscription` (rede, streams, drag) numa `iced::Subscription`. |
| `set_initial_screen(name)` | define a tela ativa inicial e limpa o histórico. |
| `navigate_to(name)` / `navigate_back()` | navegação imperativa. |
| `render_current()` / `render(name)` | renderiza a tela ativa / um componente. |
| `define_data(k, v)` / `get_data(k)` | manipulam o contexto por fora. |
| `reevaluate_all()` / `check_reload()` | reavalia tudo / recarrega arquivos alterados. |
| `reload_subscription(period)` / `toast_subscription(period)` | subscriptions de hot-reload / expiração de toasts. |

### `EngineMessage`

```rust
pub enum EngineMessage {
    UiClick(String),                                   // on_click
    UiInputChanged { action: String, value: String },  // onChange
    Navigate(String),                                  // navigateTo
    NavigateBack,                                      // navigateBack
    FileChanged(String),                               // tick do hot-reload
    ContextPatch(Vec<(String, String)>),               // subscriptions -> contexto
    EffectOutcome(EffectOutcome),                      // efeito async: patch + toast
    UiSubmit { action: String, next_focus: Option<String> }, // Enter num formControl
    // ... DragStart/DragHover/DragEnd (drag-and-drop), UiEditorAction (TextArea),
    //     LuauStream / LuauTimer (streams e timers da camada Luau)
}
```

### Tipos de apoio (re-exportados na raiz do crate)

- `GlacierApp` — trait com `bootstrap()` (atalho para `iced::application`).
- `Template::File(String)` | `Template::Inline(String)`.
- `Context` — `get`, `set`, `set_var`, `navigate_to`, `navigate_back`, `perform`, `show_toast`, `show_dialog`, `close_dialog`.
- `EffectOutcome` — `::data(...)` / `::toast(...)` / `.with_toast(...)`.
- `ContextVar::new(key, value)` · `Nav::To(String)` | `Nav::Back`.
- `FormBuilder` / `Form` / `FormControl` / `Validator` (ver [Formulários](#formulários-reactive-forms)).
- `DialogSpec` / `DialogButton` / `DialogIcon` / `ButtonRole` (ver [Diálogos](#toasts-e-diálogos-em-rust)).
- `ToastSpec` / `ToastKind`.
- `iced` re-exportado como `glacier_ui::iced` (e `Element`, `Task`, `Subscription`, `Font`, `Point`, `Size`, `window`).

### Globais da camada Luau

| Global | Assinatura | Suspende? |
|---|---|---|
| `ctx` | tabela = contexto do motor (ler/escrever `{chave}`) | — |
| `value` | texto do `onChange` (1º arg das ações de input) | — |
| `fetch(url, opts?)` | HTTP → `{ ok, status, body, error }` | **sim** |
| `sse(url, opts)` | abre SSE, devolve handle `{ :close() }` | não |
| `websocket(url, opts)` | abre WS, devolve handle `{ :send(t), :close() }` | não |
| `after(ms, fn)` | timer único, devolve handle `{ :cancel() }` | não |
| `every(ms, fn)` | timer repetitivo, devolve handle `{ :cancel() }` | não |
| `viewport()` | `{ width, height }` | não |
| `toast(opts)` | notificação efêmera | não |
| `confirm(opts)` | diálogo modal | não |
| `navigate(tela)` / `navigate_back()` | navegação | não |
| `storage.get/set/remove` | persistência local em JSON | não |
| `date.today/now/add/diff/format/…` | data e hora sobre strings ISO | não |
| `json.encode/decode/array` | (de)serialização JSON | não |
| `require(mod)` | importa uma biblioteca `.luau` | não |
| `on_error(msg)` | hook opcional de erro de script | — |

---

## Exemplos

Todos em [`examples/`](examples), rodáveis com `cargo run --example <nome>`.

| Exemplo | Demonstra |
|---|---|
| `contador` | `Component` básico com estado e cliques (Rust). |
| `contador_macro` | comportamento embutido via `<script>` Luau + `<style>` inline. |
| `contador_externo` | `<script src="...luau">` externo; `onChange` num input define o passo. |
| `perfil` | inputs, `<import>` de um cartão, `Image` circular e `ContextVar`. |
| `lista` | `<ForEach>` sobre JSON com um componente (`<import>`) por item. |
| `lista_reordenavel` | drag-and-drop: `onReorder`/`reorderKey`/`dragHandle`. |
| `condicional` | `<if>`/`<else>` (truthy e comparação). |
| `aninhado` | componente dentro de outro via `children()`, roteamento por namespace. |
| `navegacao` | múltiplas telas, histórico e `navigateTo`/`navigateBack` declarativos. |
| `navegacao_luau` | navegação decidida pelo script (`navigate` após validar); `GlacierApp::bootstrap`. |
| `formulario_login` | `Form`/`FormBuilder`/`FormControl`: validação, Enter para submeter/avançar. |
| `estilos` | `.gss` de arquivo (global + escopado via `<link>`), classes e tema. |
| `estilos_inline` | classes `.gss` inline e escopadas via bloco `<style>`. |
| `pseudo_estados` | `:hover`/`:focus`/`:active`/`:disabled` em Button/TextInput/Select. |
| `galeria_estilos` | estilos builtin (`style::FUSION`, …) com troca em runtime via `style:set`. |
| `dialogs` | diálogos modais estilo QMessageBox (Rust). |
| `toasts` | toasts info/sucesso/aviso/erro, com título e duração customizados. |
| `fetch_luau` | chamada HTTP (`fetch`) do Luau, async via corrotina. |
| `imports_luau` | `require` de bibliotecas Luau (client de rede + utilitários). |
| `robustez_luau` | timers (`after`/`every`), `storage`, `viewport`, tabelas em `ctx`, `on_error`. |
| `stream_lua` | streams de vida longa: SSE + WebSocket a partir do Luau. |
| `spinbox` | o builtin `<SpinBox/>`: campo numérico com degraus, nas duas formas do Qt. |
| `timepicker` | `<dateedit>`/`<timeedit>`/`<datetimeedit>`: edição por seções, sem uma linha de código do app. |
| `data_hora_luau` | os mesmos campos com `onChange`, **inteiramente controlados por Luau** — validação e regras no script (sobre o global `date`), zero lógica em Rust. |
| `componentes_locais` | `<component name="…">` no `<resources>`: declarar um componente na própria tela, com a forma de arquivo (`<import>`) ao lado para comparar. |
| `onda6` | a grade: `grid` (colunas medidas), `flow`, `tableview`/`tableheader` (ordenação, seleção simples e múltipla, colunas arrastáveis), `treeview` e `columnview`. Uma medição, seis widgets. |
| `onda6_luau` | a **mesma tela** em Luau — e é onde a diferença entre as duas linguagens mais aparece: as três estruturas que a tela passa aos widgets (linhas, colunas e uma árvore de três níveis) são tabelas literais aqui e `serde_json::json!` do outro lado. |
| `onda5` | o conteúdo que sai da tela e entra no widget: `tabs` (barra **mais** página), `popover`/`popup`, `autocomplete`, `drawer` e o `calendarPopup` do `dateedit`. |
| `onda5_luau` | a **mesma tela** sem `impl Component`. Mostra que nenhum dos seis widgets pede script — e que `date.today()` é uma linha onde o Rust precisa de vinte. |
| `onda4` | os widgets que têm **função**: `pagination`, `listview` (seleção simples e múltipla), `accordion`/`toolbox`, `buttonbox`, `maskedinput`, `rating` e o `decimals` do `spinbox`. O `.gv` não tem uma cor — tudo em `app.gss`, inclusive os nós de dentro dos builtins. |
| `onda4_luau` | a **mesma tela**, sem `impl Component`: o `main.rs` só registra o `.gv` e todo o comportamento vive em `scripts/app.luau`. Lado a lado com o `onda4`, mostra que nenhum dos sete widgets pede script. |
| `onda3` | o calendário: `<calendar>`, `<monthyearpicker>` e `<daterangepicker>` são a **mesma** primitiva — e a prop `today` saindo de `date.today()`. |
| `onda2` | os recipientes que o `<slot/>` destrancou: `groupbox`, `frame`, `card`, `toolbutton`, `toolbar`/`statusbar` e `tabbar`. |
| `onda1` | `slider`, `space`, `radio`/`radiogroup` e `avatar` — e a diferença entre primitiva (o app grava a chave) e builtin (o widget grava). |

---

## Publicação no crates.io

```bash
cargo login             # token de https://crates.io/settings/tokens
cargo publish --dry-run # valida o empacotamento
cargo publish           # publica glacier-ui
```

---

## Licença

Licenciado sob **MIT OR Apache-2.0**, à sua escolha.
