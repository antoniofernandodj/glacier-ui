# Plano de construção da biblioteca de widgets — rumo a um "Qt em Rust"

Este documento é o **planejamento de longo prazo** da biblioteca de widgets do
`glacier-ui`. A meta declarada é concorrer diretamente com o **Qt**: acumular,
ao longo dos anos, um catálogo vasto de widgets — de `Button` a `QDateTimeEdit`,
de diálogos de arquivo a árvores model/view — todos como **componentes Rust** que
carregam **estrutura** (template), **estilo** (`.gss`) e **comportamento**
(`update` + Luau).

É um documento vivo. Cada linha da tabela é um item de backlog; conforme um
widget nasce, seu status vira ✅ e ele ganha exemplo em `examples/` e doc curta.
A **fila de execução** — o que construir a seguir, em ordem — está na §6.2; a
§6.1 guarda a fila já cumprida, porque o *porquê* de cada item continua valendo,
e a §6.3 guarda o troco decorativo que não justifica abrir uma rodada.

Última revisão da fila: **2026-09-01**, sobre a 0.73. Ela passou a ordenar por
**função** — widgets que carregam lógica — em vez de por custo; o motivo está no
alto da §6.2.

**Grafia das tags:** todo widget aceita `CamelCase` e minúsculas coladas
(`<GroupBox/>` == `<groupbox/>`, `<ToolButton/>` == `<toolbutton/>`), a mesma
convenção que as primitivas do motor já tinham (`<textinput/>`,
`<progressbar/>`). Os exemplos usam a forma minúscula.

Relacionados: [`BUILTINS.md`](BUILTINS.md) (como escrever um builtin),
[`PRIMITIVAS.md`](PRIMITIVAS.md) (como escrever uma primitiva — inclui a
armadilha do `Length::Fill` no wrap de background/borda),
[`DIALOGS.md`](DIALOGS.md) (diálogos modais), [`ROADMAP.md`](ROADMAP.md)
(maturidade do motor).

---

## 1. Como um widget Qt vira um componente glacier-ui

O glacier-ui já tem três níveis (ver `BUILTINS.md`). Cada widget Qt do catálogo
abaixo é classificado em um deles, mais dois auxiliares:

| Nível | O que é | Onde vive | Analogia Qt |
|---|---|---|---|
| **Primitiva** | Nó nativo do motor, mapeado 1:1 a um widget do `iced` | `widget.rs` + `parser.rs` | folha atômica (`QPushButton`) |
| **Builtin** | `impl Component` que a lib auto-registra; template inline sobre primitivas | `src/builtins/` | widget composto de conveniência |
| **Componente** | Igual ao builtin, mas registrado pelo app | arquivos do app | widget custom do usuário |
| **Diálogo** | Transiente, construído em Rust, sobreposto via `Stack` | `dialogs.rs` | `QDialog`/`QMessageBox` |
| **Motor** | Capacidade de infraestrutura, não um widget | núcleo | `QWidget`/`QLayout`/model-view |

**Regra de decisão:**
- Mapeia direto a um widget do `iced 0.14`? → **Primitiva**.
- Dá para compor de primitivas com só props (sem estado próprio)? → **Builtin**.
- Precisa de estado por instância, canvas custom, ou model/view? → **Componente**
  + provavelmente **bloqueado por um item de Motor** (ver §3).

Base de referência: `iced 0.14` expõe hoje `button, text, text_input,
text_editor, checkbox, toggler, radio, slider, vertical_slider, progress_bar,
pick_list, combo_box, scrollable, container, column, row, space, rule, image,
svg, tooltip, canvas, markdown, qr_code, pane_grid, mouse_area, stack, pin,
hover, themer`. Tudo que **não** está nessa lista tem de ser construído por
composição ou via `canvas` — a coluna **Base iced** sinaliza isso.

### Legenda das tabelas

- **Nível**: `Prim` (primitiva) · `Built` (builtin) · `Comp` (componente) ·
  `Diál` (diálogo) · `Motor` (infra).
- **Estado?**: `—` apresentacional/prop-driven (usável N× hoje) · `◐` estado
  simples controlável por prop (valor + `on_change`) · `●` **exige estado por
  instância** (bloqueado, ver §3).
- **Base iced**: primitiva(s) do `iced` que sustentam o widget, ou `canvas`
  (desenho próprio) / `compõe` (só composição) / `stack` (overlay).
- **Prio**: `P0` fundação/próximo · `P1` alto valor, comum · `P2` importante,
  complexo · `P3` nicho/avançado.
- **Status**: ✅ existe · 🟡 parcial · ⬜ falta.

---

## 2. O catálogo (a "tabela gigante")

### 2.1 Botões e ações

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QPushButton | `Button` | Prim | button | — | P0 | ✅ | já existe (`on_click`, estilos GSS) |
| QToolButton | `ToolButton` | Built | button+svg | — | P1 | ✅ | botão-ícone com `autoRaise` (fundo só no hover); glifo ou `.svg`, e as três formas do `Qt::ToolButtonStyle` (`icon`/`beside`/`under`) |
| QRadioButton | `Radio` | Prim | radio | ◐ | P1 | ✅ | o grupo **é a chave**: `group="plano"` é o *nome* dela (convenção do `checked=`), e todo `<radio>` que aponta para a mesma chave é do mesmo grupo. Não grava sozinho (regra do `<Checkbox>`) — para isso, o `RadioGroup` |
| QCheckBox | `Checkbox` | Prim | checkbox | ◐ | P0 | ✅ | já existe |
| QCheckBox (tristate) | `Checkbox tristate` | Prim | checkbox | ◐ | P2 | ✅ | flag `tristate` no `<Checkbox>`; cicla `false → mixed → true` (a ordem do Qt) e desenha `−` no lugar do check, como `Qt::PartiallyChecked` |
| QCommandLinkButton | `CommandLink` | Built | button+col | — | P2 | ⬜ | título + descrição + seta. §6.3 |
| QDialogButtonBox | `ButtonBox` | Built | row+button | — | P1 | 🟡 | existe nos diálogos; expor como builtin de tela, com os **papéis** (`accept`/`reject`/`destructive`) e a ordem por plataforma decididos no widget. Onda 4 |
| (switch/QML Switch) | `Toggle`/`Toggler` | Prim | toggler | ◐ | P0 | ✅ | já existe |
| QML RoundButton | `RoundButton` | Built | button | — | P3 | ⬜ | border-radius total. §6.3 |
| QML DelayButton | `DelayButton` | Comp | button+canvas | ● | P3 | ⬜ | anel de progresso ao segurar |

### 2.2 Entradas de texto

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QLineEdit | `TextInput` | Prim | text_input | ◐ | P0 | ✅ | já existe |
| QLineEdit (password) | `TextInput password` | Prim | text_input | ◐ | P1 | ✅ | flag `secure`/`password`/`seguro`/`senha` no `<TextInput>`, sobre o `.secure()` do iced |
| QLineEdit (mask/validator) | `MaskedInput` | **Prim** | text_input | ◐ | P2 | ⬜ | máscara + validação (CPF, CNPJ, telefone, CEP). **Reclassificado de `Comp ●` para `Prim ◐`** pela lição do `DateEdit` (§3): a máscara é função pura da string aplicada no `on_input`, e o que a impedia de ser builtin era ler uma chave cujo *nome* vem de prop — indireção que primitiva não tem. Onda 4 |
| QTextEdit (rich) | `TextEditor` | Prim | text_editor | ● | P1 | ✅ | multi-linha; rich text é limitado |
| QPlainTextEdit | `PlainTextEditor` | Prim | text_editor | ● | P1 | 🟡 | variante sem formatação |
| QTextBrowser | `TextBrowser` | Built | markdown/scrollable | — | P2 | ⬜ | render read-only + links. §6.3 |
| QKeySequenceEdit | `ShortcutInput` | Comp | text_input | ● | P3 | ⬜ | captura combinação de teclas |
| QComboBox (editable) | `ComboEdit` | Prim | combo_box | ◐ | P1 | ✅ | `options`/`value`/`onChange`/`onSelect`/`placeholder` + `labelField`/`valueField` para listas de objetos (ver `examples/combo_edit`) |
| — (autocomplete) | `Autocomplete` | **Prim** | text_input+overlay | ◐ | P2 | ⬜ | a mesma tag do `Completer` (§2.12), vista do lado do campo. Onda 5 |

### 2.3 Entradas numéricas e de valor

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QSpinBox | `SpinBox` | Built | text_input+button | ◐ | P1 | ✅ | campo + degraus, `min`/`max`/`step`, `layout="stacked"` (as setinhas ▴▾ coladas no campo, o QSpinBox clássico) ou `"inline"` (`− campo +`, o SpinBox do Qt Quick); a aritmética roda no `update` em Rust — **reclassificado de `●`**: o número mora numa chave que o app nomeia (prop `value`) e a ação carrega essa chave, então N instâncias não colidem (ver `src/builtins/spin_box.rs`) |
| QDoubleSpinBox | `SpinBox decimals` | Built | text_input+button | ◐ | P1 | 🟡 | sai de graça do `SpinBox`: as casas decimais vêm do `step` (`step="0.25"` → 2 casas). Falta uma prop `decimals` explícita |
| QSlider | `Slider` | Prim | slider / vertical_slider | ◐ | P1 | ✅ | `min`/`max`/`step`, `vertical`, mais `default` (duplo clique), `on_release` e `shift_step`. Casas decimais da saída vêm do `step` como escrito. `disabled` deixa inerte, sem esmaecer: o `slider::Status` do iced 0.14 não tem `Disabled` |
| QML RangeSlider | `RangeSlider` | Comp | canvas | ● | P2 | ⬜ | dois cursores |
| QDial | `Dial` | Comp | canvas | ● | P2 | ⬜ | knob rotativo |
| QScrollBar | `ScrollBar` | Motor | scrollable | — | P2 | 🟡 | embutido no `scrollable`; expor avulso é raro |
| QProgressBar | `ProgressBar` | Prim | progress_bar | — | P1 | ✅ | `value`/`min`/`max`/`vertical`/`showValue`; `color` = preenchimento |
| QProgressBar (busy) | `Spinner`/`BusyIndicator` | Prim | fill_quad (sem canvas) | — | P1 | ✅ | indeterminado; fase de rotação no `tree::State` do widget — **não** exige estado por instância no contexto (reclassificado de `●`; ver `src/spinner.rs`) |
| QLCDNumber | `LcdNumber` | Comp | canvas | — | P3 | ⬜ | dígitos estilo display 7-segmentos |
| QML Tumbler | `Tumbler` | Comp | scrollable | ● | P3 | ⬜ | roleta de valores |
| QML Gauge / medidor | `Gauge` | Comp | canvas | ◐ | P2 | ⬜ | medidor circular/arco |
| — (nota por estrelas) | `Rating` | Built | row+button | ◐ | P2 | ⬜ | N estrelas numa chave nomeada (padrão `SpinBox`), com pré-visualização no hover. Citado na §3 como “nunca esteve bloqueado” e faltava na tabela. Onda 4 |

### 2.4 Seleção, listas e árvores (model/view)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QComboBox | `Select` / `Combo` | Prim | pick_list / combo_box | ◐ | P0 | ✅ | ambos existem |
| QFontComboBox | `FontSelect` | Comp | combo_box | ● | P3 | ⬜ | lista fontes do sistema |
| QListWidget | `ListView` | **Built** | scrollable+ForEach | ◐ | P1 | 🟡 | dá para fazer com `for`; falta a **seleção**, que é o padrão do `SpinBox` (coleção numa chave + item escolhido noutra, como o `TabBar` já faz). Multi-seleção é a mesma chave com uma lista, e aí pede o `contains` no condicional (§6.2). Onda 4 |
| QListView (model) | `ListView bind` | Motor+Comp | scrollable | ◐ | P2 | ⬜ | ligado a coleção do contexto — e a ligação **já existe** (`items="chave"`, a convenção do `<Menu>`/`<TabBar>`). Onda 6 |
| QTreeWidget/QTreeView | `TreeView` | **Prim** | column+recursão | ◐ | P2 | ⬜ | ~~expandir/recolher = estado por nó~~ — é um **conjunto nomeado** (`abertos="raiz,raiz/src"`) + o `contains` da Onda 4. Onda 6 |
| QTableWidget/QTableView | `TableView` | **Prim** | column+row | ◐ | P2 | ⬜ | **grande**: cabeçalho, seleção, sort, edição. A parte cara é a **medição de coluna**, que é a mesma do `Grid` — os dois saem juntos. Onda 6 |
| QHeaderView | `TableHeader` | **Prim** | row+button | ◐ | P2 | ⬜ | parte da TableView; sort/resize (o arrasto na família do `__drag_key`). Onda 6 |
| QColumnView | `ColumnView` | **Prim** | row+ListView | ◐ | P3 | ⬜ | navegação Miller (finder); quase de graça depois do `TreeView`. Onda 6 |
| QListWidgetItem etc. | (dados, não widget) | — | — | — | — | — | modelados como valores de contexto |
| QCompleter | `Completer` | **Prim** | overlay+ListView | ◐ | P2 | ⬜ | popup de sugestões (ver §2.12). Onda 5 |
| QML PageIndicator | `PageIndicator` | Built | row | ◐ | P2 | ⬜ | pontinhos de página — o irmão visual do `Pagination`, mesma chave |
| — (paginação) | `Pagination` | Built | row+button | ◐ | P1 | ⬜ | primeira/anterior/número/próxima/última, com a aritmética de página no `update`. Citado na §3 como “nunca esteve bloqueado” e faltava na tabela; é o companheiro obrigatório de `ListView`/`TableView`. Onda 4 |

### 2.5 Data e hora — **foco declarado do projeto**

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QCalendarWidget | `Calendar` | **Prim** | compõe | ◐ | P1 | ⬜ | grade de mês, navegação, dia selecionado. **Reclassificado de `Comp ●` para `Prim ◐`** — ver a nota abaixo da tabela; deixou de estar bloqueado. Onda 3 |
| QDateEdit | `DateEdit` / `DatePicker` | Prim | compõe | ◐ | P1 | ✅ | edição por **seções** (ano/mês/dia), com o realce da paleta na seção ativa e ▴▾ agindo sobre ela. Calendário respeitado: 31/01 + 1 mês satura em 28 ou 29. `format="br"` troca só a exibição — a chave é sempre ISO. Teclado completo na 0.70 (▲▼ na seção, ←→ para trocar, dígitos com avanço automático). Falta a variante `calendarPopup`, que espera o overlay ancorado (§3) |
| QTimeEdit | `TimeEdit` / `TimePicker` | Prim | compõe | ◐ | P1 | ✅ | as mesmas seções, para hora/minuto\[/segundo\]. Cada seção vira **dentro de si** (o `wrapping` do `QAbstractSpinBox`): mexer no minuto não empurra a hora |
| QDateTimeEdit | `DateTimeEdit` | Prim | compõe | ◐ | P1 | ✅ | as duas famílias de seção no mesmo campo. É **a mesma primitiva** dos dois acima — a tag só decide quais seções aparecem |
| — (range) | `DateRangePicker` | **Prim** | compõe | ◐ | P2 | ⬜ | intervalo início→fim: **a mesma primitiva** com `range`, duas chaves (`start`/`end`) e `months="2"` para as duas grades lado a lado. Onda 3 |
| — (mês/ano) | `MonthYearPicker` | **Prim** | compõe | ◐ | **P2** | ⬜ | seleção só de mês/ano: **a mesma primitiva** em `mode="month"` — a tela de drill-up que o `QCalendarWidget` abre ao clicar no título, promovida a tag. Grava `YYYY-MM`. Sobe de P3 porque sai junto do `Calendar`, não depois. Onda 3 |

> **A dependência de datas não foi necessária** para os três campos de edição: a
> aritmética que eles pedem é somar 1 numa seção e saber quantos dias tem o mês
> — bissexto incluído, regra do século incluída —, o que cabe em vinte linhas
> (`Instante`, em `src/widget.rs`). E não foi necessária **nem para o resto**: a
> 0.72 fechou a decisão `chrono` vs. `time` (§4) pela negativa, com o global
> `date` do prelúdio Luau cobrindo intervalo, dia da semana e formatação sobre
> strings ISO, sem crate nenhuma.
>
> Os três campos **também não precisaram de estado por instância** — ver a
> correção na §3.
>
> **E as três linhas restantes também não precisam** (revisão de 2026-09-01).
> Este documento afirmava, ainda na §3, que "o que de fato espera o estado por
> instância na §2.5 é o `Calendar`". É a mesma falha de nível que já tinha
> acontecido com o `TimePicker`, e a pergunta que a §3 mandou fazer —
> *"antes de declarar um widget bloqueado, pergunte se ele não é uma
> primitiva"* — não tinha sido feita aqui.
>
> Como **primitiva**, o `Calendar` não tem nenhum dos três bloqueios que se
> atribuíam a ele:
>
> | Bloqueio alegado | Por que some |
> |---|---|
> | estado de navegação (que mês estou vendo) por instância | é **um valor que se nomeia**, e o nome sai de graça: a identidade da instância já é o nome da chave que ela edita, então o mês visível mora em `__cal_<chave>` (do motor, sem o app configurar nada), e uma prop `month=` opcional deixa o app dirigir os dois calendários de um intervalo |
> | `Grid` (`QGridLayout`) como pré-requisito | só valeria para um **builtin**, cujo template é markup e precisaria de uma grade declarativa. Uma primitiva monta `Column` de `Row`s num laço em Rust — o `Grid` continua valendo por si, mas deixa de estar no caminho crítico da §2.5 |
> | a decisão `chrono` vs. `time` | fechada na 0.72 (§4). Dia da semana é `days_from_civil`, seis linhas ao lado do `dias_no_mes` que o `Instante` já tem |
>
> Sobra um item honesto: marcar **hoje** exige o offset local, que a `std` não
> dá. A saída é não pedir: `today="{hoje}"` como prop, preenchida com
> `date.today()` numa linha de Luau — que é exatamente para isso que o global
> `date` da 0.72 existe. Sem a prop, o calendário simplesmente não destaca
> nenhum dia, o que é degradação aceitável.
>
> O resultado é que a §2.5 inteira sai por **uma primitiva com três tags**, do
> mesmo jeito que `<dateedit>`/`<timeedit>`/`<datetimeedit>` são uma só. Ver a
> Onda 3 na §6.2.

### 2.6 Displays e indicadores (apresentacionais)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QLabel (texto) | `Text` | Prim | text | — | P0 | ✅ | já existe |
| QLabel (rich/link) | `Link` / `span` | Prim | text/rich | — | P1 | ✅ | `Link` e `span` existem |
| QLabel (imagem) | `Image` | Prim | image | — | P0 | ✅ | já existe |
| — (ícone SVG) | `Svg` / `icone` | Prim | svg | — | P0 | ✅ | já existe |
| — (pílula/rótulo) | `Badge` | Built | container+text | — | P1 | ✅ | builtin canônico |
| — (cartão) | `Card` | Built | container+col | — | P1 | ✅ | builtin de verdade a partir da 0.65: cabeçalho (título/subtítulo), corpo por `<slot/>` e rodapé por `<slot name="footer"/>` (0.67), que só se paga quando preenchido |
| — (avatar) | `Avatar` | Built | container+image | — | P1 | ✅ | foto circular ou iniciais como reserva, com cores por instância. Sem indicador de presença (pediria `Stack` dentro do builtin) |
| — (chip removível) | `Chip` | Built | row+button | — | P2 | ⬜ | badge com "×". §6.3 |
| — (separador) | `Divider` / `Rule` | Prim | rule | — | P0 | ✅ | `Rule` existe |
| QFrame | `Frame` | Built | container | — | P2 | ✅ | três formas: `box` (contorno), `filled` (contraste, o `QFrame::Panel`) e `none`. Sem `Raised`/`Sunken`: o `UiNode` não tem campo de sombra |
| — (skeleton) | `Skeleton` | Built | container | — | P2 | ⬜ | placeholder de carregamento. §6.3 |
| — (QR) | `QrCode` | Prim | qr_code | — | P3 | ⬜ | iced tem nativo. §6.3 |
| QGraphicsView | `Canvas` | Prim | canvas | ● | P3 | ⬜ | superfície de desenho livre |
| QOpenGLWidget | `Shader` | Prim | shader | ● | P3 | ⬜ | iced `shader` (wgpu) |
| — (toast) | `Toast` | Motor | stack | — | P1 | ✅ | já existe (`toasts.rs`) |

### 2.7 Containers e agrupadores

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QWidget/QFrame | `Container` | Prim | container | — | P0 | ✅ | já existe |
| QGroupBox | `GroupBox` | Built | container+text | — | P1 | ✅ | moldura com título + `flat="true"` (o `QGroupBox::flat`), e ações no cabeçalho por `<slot name="actions"/>` — onde vai o `<checkbox>` que faz o papel do `setCheckable` |
| QScrollArea | `Scrollable` / `rolagem` | Prim | scrollable | — | P0 | ✅ | já existe |
| QSplitter | `Splitter` / `PaneGrid` | Prim | pane_grid | ● | P2 | ⬜ | painéis redimensionáveis |
| QToolBox | `ToolBox` | **Built** | column+button | ◐ | P2 | ⬜ | seções empilhadas, **uma** aberta por vez — é o `TabBar` na vertical, mesma chave nomeada. Nunca esteve bloqueado. Onda 4 |
| — (accordion) | `Accordion` | **Built** | column+button | ◐ | P1 | ⬜ | itens abre/fecha, **várias** abertas ao mesmo tempo. ~~precisa estado por instância~~ — precisa de um **conjunto** numa chave nomeada e do `contains` no condicional (§6.2), que é bem menor. Onda 4 |
| QMdiArea/QMdiSubWindow | `MdiArea` | Comp | canvas/stack | ● | P3 | ⬜ | janelas MDI internas |
| QDockWidget | `Dock` | Comp | pane_grid | ● | P3 | ⬜ | painéis acopláveis |
| QSpacerItem | `Space` | Prim | space | — | P1 | ✅ | sem `width`/`height` é `Fill` nos dois eixos (o espaçador flexível); com eles, vão fixo. Duplicado na §2.11 por ser layout **e** container |

### 2.8 Navegação (abas, wizard, stacks)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QTabWidget/QTabBar | `TabBar` | Built | row+button | ◐ | P1 | 🟡 | a **barra** existe (0.65): abas de uma coleção do contexto, ativa numa chave que o app nomeia (padrão `SpinBox`). O empilhado de páginas continua sendo `se`/`senao`: o `QTabWidget` inteiro precisa de uma página por aba, e o slot nomeado da 0.67 tem **nome fixo** — falta o nome **dinâmico** (`<slot name="{aba}"/>`) — **Onda 5** |
| QStackedWidget | `Stack`/`StackView` | Comp | condicional (`se`) | ◐ | P1 | 🟡 | já dá com `se`; formalizar — sai junto do `Tabs` completo, mesmo mecanismo. Onda 5 |
| QWizard/QWizardPage | `Wizard` | Comp | Stack+ButtonBox | ● | P2 | ⬜ | passos com voltar/avançar/finalizar |
| QML SwipeView | `SwipeView` | Comp | stack | ● | P3 | ⬜ | páginas deslizáveis |
| QML Drawer | `Drawer` | **Built** | stack+animação | ◐ | P2 | ⬜ | painel lateral deslizante — `<slot/>` + chave nomeada + a animação que o motor já tem; sem bloqueio. Onda 5 |
| (roteamento de telas) | `navigate_to` | Motor | — | — | P0 | ✅ | navegação já existe |

### 2.9 Janela principal e barras

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QMainWindow | `Window`/`App` | Motor | app | ● | P0 | ✅ | app já é a janela |
| QMenuBar | `MenuBar` | Prim | row+overlay | — | P2 | ✅ | `<MenuBar>` + `<Menu>`; overlay próprio em `src/menu.rs` |
| QMenu | `Menu` | Prim | overlay próprio | — | P2 | ✅ | `<Menu>`/`<MenuItem>`/`<MenuSeparator>`, com ícone, item marcável, `disabled` e **submenus aninhados a profundidade arbitrária**; itens também por `items=` (coleção do contexto) |
| — (menu de contexto) | `ContextMenu` | Prim | mouse_area+overlay | — | P2 | ✅ | `<ContextMenu items="…">`, botão direito (ver `examples/menus`) |
| QToolBar | `ToolBar` | Built | row+ToolButton | — | P2 | ✅ | faixa de ações por `<slot/>` (aceita qualquer widget, como o `addWidget` do Qt), com `divider` opcional |
| QStatusBar | `StatusBar` | Built | row+text | — | P2 | ✅ | mensagem à esquerda (`showMessage`) e permanentes à direita por `<slot/>` (`addPermanentWidget`) |
| QSystemTrayIcon | `SystemTray` | Motor | (SO) | — | P3 | ✅ | `src/tray.rs` (feature `tray-icon`, thread dedicada): app sobrevive à última janela, menu de bandeja e interruptor de notificações (ver `examples/bandeja`) |
| QSizeGrip | — | Motor | — | — | P3 | ⬜ | canto de redimensionamento |

### 2.10 Diálogos (módulo `dialogs.rs`)

| Qt | Tag / API glacier-ui | Nível | Base | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QMessageBox (info) | `DialogSpec::information` | Diál | stack | — | P0 | ✅ | existe |
| QMessageBox (warning) | `DialogSpec::warning` | Diál | stack | — | P0 | ✅ | existe |
| QMessageBox (critical) | `DialogSpec::error` | Diál | stack | — | P0 | ✅ | existe |
| QMessageBox (question) | `DialogSpec::question` | Diál | stack | — | P0 | ✅ | existe |
| — (confirm) | `DialogSpec::confirm` | Diál | stack | — | P0 | ✅ | existe |
| QInputDialog | `InputDialog` | Diál | stack+TextInput | ● | P1 | ⬜ | pede texto/número/item |
| QProgressDialog | `ProgressDialog` | Diál | stack+ProgressBar | ● | P1 | ⬜ | progresso cancelável |
| QFileDialog (abrir arquivo) | `FileDialog::open` | Diál | **`rfd`** (nativo do SO) | — | P1 | ✅ | `src/file_dialog.rs`; Luau `open_file()`/`open_files()`, suspensivo como `confirm()`/`fetch()` (ver `examples/file_dialog`) |
| QFileDialog (salvar) | `FileDialog::save` | Diál | `rfd` | — | P1 | ✅ | Luau `save_file()` |
| QFileDialog (diretório) | `FileDialog::directory` | Diál | `rfd` | — | P1 | ✅ | Luau `pick_folder()` |
| QColorDialog | `ColorDialog` | Diál | stack+canvas | ● | P2 | ⬜ | roda/HSV/hex |
| QFontDialog | `FontDialog` | Diál | stack+lista | ● | P3 | ⬜ | escolher fonte/tamanho |
| QErrorMessage | (coberto por `error`) | Diál | — | — | — | ✅ | redundante |
| QPrintDialog/QPageSetup | `PrintDialog` | Diál | `rfd`/SO | ● | P3 | ⬜ | impressão (fora do escopo inicial) |

> **Decisão pendente (§4):** diálogos de arquivo/cor nativos (via crate `rfd`)
> vs. construídos em glacier-ui. `rfd` entrega já-funcional e nativo do SO; a
> versão própria dá controle total de estilo mas é cara. Sugestão: `rfd`
> primeiro (P1), versão própria estilizável depois (P3).

### 2.11 Layouts (nível Motor / `parser`)

| Qt | Equivalente glacier-ui | Nível | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|
| QHBoxLayout | `Row` / `row` | Prim | — | P0 | ✅ | existe |
| QVBoxLayout | `Column` / `column` | Prim | — | P0 | ✅ | existe |
| QGridLayout | `Grid` | Prim/Motor | — | P1 | ⬜ | grade linhas×colunas — o `iced` não tem, então é composição de `Row`/`Column` com medição. Essa medição é **a mesma** do `TableView`: os dois saem da mesma obra. Onda 6 |
| QFormLayout | `Form` / `formulario` | Prim | — | P1 | ✅ | `Form` existe |
| QStackedLayout | `se`/`senao` + Stack | Motor | ◐ | P1 | 🟡 | condicional existe |
| QSpacerItem | `Space` | Prim | — | P1 | ✅ | ver §2.7 |
| (flow layout) | `Flow`/`Wrap` | **Prim** | row+quebra | — | P2 | ⬜ | quebra automática de linha — a medição da Onda 6 num eixo só. Onda 6 |

### 2.12 Overlays, dicas e utilitários

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QToolTip | `tooltip=` | Prim | tooltip | — | P1 | ✅ | **atributo universal**, não tag: `tooltip`/`title`/`dica` em *qualquer* nó, com `tooltip_position` |
| QML ToolTip | (idem) | Prim | tooltip | — | P1 | ✅ | mesmo atributo |
| QWhatsThis | — | — | — | — | P3 | ⬜ | ajuda contextual (raro) |
| QCompleter | `Completer` | **Prim** | text_input+overlay | ◐ | P2 | ⬜ | sugestões enquanto digita, com ↑↓/Enter/Esc. Onda 5 |
| QML Popup | `Popup` | **Prim** | stack | ◐ | P2 | ⬜ | genérico, centrado na janela — a mesma primitiva do `Popover` sem âncora. Onda 5 |
| — (menu popover) | `Popover` | **Prim** | stack | ◐ | P2 | ⬜ | conteúdo flutuante ancorado, por `<slot/>`. Onda 5 |
| QSplashScreen | `SplashScreen` | Comp | stack | — | P3 | ⬜ | tela de abertura |
| QRubberBand | — | Comp | canvas | ● | P3 | ⬜ | retângulo de seleção |
| QShortcut/QAction | `Shortcut`/`Action` | Motor | subscription | ● | P2 | ⬜ | atalhos globais de teclado |
| QScroller | (no scrollable) | Motor | scrollable | — | P3 | 🟡 | rolagem por gesto |
| — (badge de notificação) | `NotificationDot` | Built | container | — | P2 | ⬜ | pontinho sobre ícone. §6.3 |

### 2.13 Gráficos e visualização (Qt Charts / DataVisualization)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QChartView (linha) | `LineChart` | Comp | canvas | — | P2 | ⬜ | avaliar crate `plotters`+iced |
| QChartView (barra) | `BarChart` | Comp | canvas | — | P2 | ⬜ | — |
| QChartView (pizza) | `PieChart` | Comp | canvas | — | P2 | ⬜ | — |
| QChartView (área/scatter) | `AreaChart`/`Scatter` | Comp | canvas | — | P3 | ⬜ | — |
| — (sparkline) | `Sparkline` | Comp | canvas | — | P2 | ⬜ | mini-gráfico inline |
| Q3D* (3D bars/scatter) | — | Comp | shader | ● | P3 | ⬜ | escopo distante (wgpu) |

---

## 3. Pré-requisitos do motor (o caminho crítico)

Boa parte da tabela está **bloqueada por infraestrutura**, não por esforço de
markup. A restrição dominante é a coluna **Estado? = ●**:

> **Estado por instância.** Hoje `ctx.set` grava num **único** contexto global —
> duas instâncias do mesmo widget com estado colidiriam (documentado em
> `BUILTINS.md` e no ROADMAP, Fase 2). É o desbloqueio de maior alavancagem:
> destrava `Calendar`, `DatePicker`, os pickers de data — o coração do "foco
> declarado" do projeto.

**Ressalva descoberta ao construir o `SpinBox` (0.63):** `●` estava marcando
duas coisas diferentes, e só uma delas bloqueia de fato.

| O estado é… | Bloqueia? | Porque |
|---|---|---|
| um **valor que o app nomeia** (quantidade, preço, aba ativa, página) | **não** | a chave entra por prop e a ação carrega a chave (`inc:qtd\|1\|99\|1`), então o `update` do builtin sabe onde escrever e duas instâncias com chaves diferentes são independentes |
| um estado **sem nome natural** (posição do cursor, buffer de digitação, fase de animação) | depende | ou vive no `tree::State` do widget nativo (foi assim no `Spinner`), ou espera o estado por instância de verdade |

Ou seja: `SpinBox`, `Pagination`, `Rating` e afins nunca estiveram bloqueados —
o que faltava era o padrão, não o motor. O mesmo tipo de correção que a linha
`QProgressBar (busy)` já tinha recebido. (Os dois estavam citados aqui e **não
existiam na tabela**; entraram na revisão de 2026-09-01, §2.3 e §2.4.)

**Extensão da mesma ideia (revisão de 2026-09-01): o conjunto nomeado.** Se um
valor que o app nomeia dispensa estado por instância, uma **lista** de valores
na mesma chave também dispensa — `abertas="rede,proxy"`, `selecao="3,7,9"`. É o
que separa o `Accordion` (várias seções abertas) do `ToolBox` (uma só), e o
`ListView` de seleção múltipla do de seleção simples. O `update` do builtin faz
o toggle de pertinência, que é aritmética de string.

O que falta para isso **não** é o estado por instância: é um `contains` no
condicional (`<template if="{abertas}" contains="rede">`), ao lado do `one_of`
que já existe — e que é exatamente o caso simétrico. Ver §6.2.

Há ainda um terceiro caso, descoberto ao ordenar a fila do §6: o estado **tem**
nome (`<Calendar value="data" month="mes_visivel"/>`), mas o widget precisa
**semear** esse contexto antes do primeiro clique — e `Component::init(&mut
self, ctx)` não recebe as props da instância, só o contexto global. Um
`Calendar` renderizaria vazio até alguém clicar.

> **Correção (0.68).** Este documento dizia que "a §2.5 inteira continua atrás
> do estado por instância". Errado — e a lição é sobre o **nível**, não sobre o
> motor. O terceiro caso só existe para um **builtin**, cujo template precisa
> semear e depois ler o contexto para desenhar. Uma **primitiva** não tem esse
> problema: ela lê e escreve na hora do render, em Rust.
>
> Foi o que destravou `DateEdit`/`TimeEdit`/`DateTimeEdit`. Eles nasceram
> builtin, e a forma correta (editar por **seções**, como o Qt) era impossível
> ali: o template exibiria o valor inteiro, e para desenhar `13` e `45`
> separados precisaria ler partes de uma chave cujo *nome* vem de uma prop — a
> indireção `{{value}}` que o interpolador não tem. Como primitiva, partir a
> string é uma linha.
>
> Sobrou só o estado de **foco** (qual seção está selecionada), que nem é por
> instância: é global por natureza — uma seção da tela inteira por vez —, e vive
> numa chave do motor com a identidade da instância no valor (`__timeedit` =
> `"inicio:h"`).
>
> ~~O que **de fato** espera o estado por instância na §2.5 é o `Calendar`: uma
> grade de mês tem estado de navegação (que mês estou vendo) que é dele, não do
> app.~~ **Errado também** — e pela mesma razão, uma revisão depois
> (2026-09-01): o `Calendar` é primitiva. O mês visível é um valor que se nomeia
> e o nome sai da própria chave editada (`__cal_<chave>`), como o `__timeedit`
> guarda a seção selecionada. A §2.5 saiu inteira de trás do estado por
> instância; a demonstração está no blockquote da §2.5.
>
> Regra prática que fica: **antes de declarar um widget bloqueado, pergunte se
> ele não é uma primitiva.** Ela já foi aplicada tarde demais duas vezes neste
> documento — vale aplicá-la a cada `●` que sobrou.

O que **de fato** travava `Tabs`, `Accordion`, `GroupBox`, `Frame`, `ToolBar` e
`StatusBar` era outro item, que não estava nesta lista — e que **caiu na 0.65**:

> ~~**Componente não aceita filhos.**~~ **Resolvido.** `NodeType::Slot` existe:
> `<slot/>` no template de um componente recebe o conteúdo escrito entre as tags
> do uso, com conteúdo de reserva quando não vem nada. O ponto fino é a
> **posse**: o conteúdo é avaliado no contexto e com o dono de *quem escreveu*,
> então `on_click="salvar"` dentro de um `<GroupBox>` chega na tela e não vira
> `GroupBox::salvar`. Ver `BUILTINS.md`.
>
> **Slot nomeado** (`<slot name="footer"/>`) veio logo depois, com nomes fixos:
> um componente abre quantas regiões quiser e quem usa etiqueta o conteúdo com
> `slot="footer"`. Mais o marcador `{slot_<nome>}`, que deixa o template decorar
> uma região opcional (a linha divisória que só existe quando existe rodapé).
>
> O que **ainda** falta é o nome **dinâmico** (`<slot name="{aba}"/>`, resolvido
> contra o contexto). É só isso que separa o `TabBar` de hoje de um `QTabWidget`
> inteiro — e o `Accordion` continua atrás do estado por instância, porque quer
> várias seções abertas ao mesmo tempo.

Ordem sugerida de habilitadores de Motor:

1. **Estado por instância** (`●` do segundo tipo, acima). ~~**P0.**~~ **P1** —
   rebaixado na revisão de 2026-09-01. Ele continua sendo o maior item da lista,
   mas deixou de ser o que separa o projeto do seu foco declarado: a §2.5 saiu
   de trás dele, e `Accordion`/`ToolBox`/`ListView` saem pelo conjunto nomeado.
   O que resta atrás dele é genuinamente estado sem nome — `TreeView` (um bit de
   expandido por nó, em árvore de profundidade arbitrária), `MdiArea`, `Dock`.
2. ~~**Filhos em componente (`<slot/>`)**~~ ✅ **feito na 0.65** — destravou
   `GroupBox`, `Frame`, `Card`, `ToolBar` e `StatusBar`, todos construídos na
   mesma leva (§6). O **slot nomeado com nomes fixos** saiu na 0.67, e com ele o
   `Card` ganhou rodapé e o `GroupBox` ganhou ações no cabeçalho. Sobra o nome
   **dinâmico**, que é o que falta para o `Tabs` completo. **P1.**
3. **`ctx.dispatch(acao)`** — repasse de evento **do lado Rust**: um `update`
   não consegue despachar outra ação, então um builtin que trata um evento para
   si não pode também repassá-lo. O caso declarativo (widget que só delega, como
   o `TimePicker`) já está resolvido pelo prefixo `app:` na 0.63 — ver
   `BUILTINS.md`. **P2.**
4. ~~**`Space`**~~ ✅ **(0.66)** **+ `Grid`** — sobrou o `Grid`, o layout que
   falta para telas densas. **P1.** Deixou de ser pré-requisito do `Calendar`
   (§2.5): quem monta grade em Rust não precisa de grade em markup. Continua
   valendo por si, para painéis densos escritos à mão.
5. **Sistema de overlay ancorado reutilizável** — `Stack` + posição relativa a
   um widget âncora. **Meio resolvido:** `src/menu.rs` já construiu um overlay
   ancorado com cascata de submenus, mas fechado sobre `MenuNode` — não é um
   mecanismo genérico. Generalizá-lo (ou trocá-lo por um
   `iced::advanced::{Widget, Overlay}` custom, o caminho que o próprio
   `menu.rs` documenta como o "certo") destrava `Popup`, `Popover`,
   `Completer` e o popup do `DatePicker`. Lição de `DIALOGS.md` já mapeou os
   cuidados (`Interaction::Idle` + `on_press` sempre presente). **P1** — é o
   habilitador B da **Onda 5** (§6.2).
6. **Contexto tipado / valor de data** — reduz `to_string()`/parse manual;
   necessário para pickers de data robustos. **P1.**
7. ~~**Binding a coleção (model/view)**~~ — **já existe**, e este item estava
   superdimensionado: a convenção `items="chave"` (array JSON numa chave) é o
   que o `<Menu>` e o `<TabBar>` usam desde a 0.65, e o que todo `for-each` do
   motor sempre leu. O que falta para `TableView`/`TreeView` é a **medição de
   coluna** — que o `Grid` compartilha e que nunca esteve nesta lista — mais as
   convenções de seleção e ordenação, que são o padrão do `SpinBox`. **Onda 6**
   (§6.2). **P2.**
8. **Canvas exposto como primitiva** — destrava `Dial`, `Gauge`,
   `ColorDialog`, `LcdNumber`, todos os gráficos. **P2.** (`Spinner` saiu
   desta lista: acabou não precisando de `canvas` nem de estado por
   instância — ver a nota da linha `QProgressBar (busy)` na tabela §2.3.)
9. **Subscriptions de teclado** — `Shortcut`/`Action` globais. **P2.** (O
   `<datetimeedit>` já lê teclado desde a 0.70, mas por um listener global que
   só age quando ninguém consumiu o evento — não é este item.)
10. **Virtualizar a lista** — avaliar só as linhas visíveis de um `for-each`
    dentro de um `<scrollable>`, em vez de todas. Pede a realimentação de
    rolagem (`on_scroll`), que o motor não plumba, e uma decisão sobre altura de
    linha. **P1**, e é o único teto de tamanho de lista que sobrou depois das
    releases de custo 0.74/0.75. Vai na **Onda 6** (§6.2), junto do `TableView`.
11. **`contains` no condicional** — `<template if="{abertas}" contains="rede">`,
    o simétrico do `one_of` que já existe. **P1**, e é o menor da lista: destrava
    `Accordion` e a seleção múltipla do `ListView`. Ver §6.2.

---

## 4. Decisões em aberto

- ~~**Diálogos nativos vs. próprios** (arquivo)~~ — **decidido e feito**: `rfd`
  nativo do SO, em `src/file_dialog.rs`, exposto ao Luau como
  `open_file`/`open_files`/`save_file`/`pick_folder`. A versão própria
  estilizável segue em P3. Cor e fonte continuam em aberto pela mesma pergunta.
- ~~**Dependência de datas**: `chrono` vs. `time` para o módulo de data/hora.~~
  — **respondida pela negativa na 0.72**: o motor não puxa nenhuma das duas. O
  lado Luau ganhou o global `date` (`src/luau/prelude.luau`), Luau puro sobre
  strings ISO, com `today`/`now` saindo do `os.date` que o dialeto já tem — o
  `io`/`os.execute` é que está fora do sandbox, o relógio não. O lado Rust já
  tinha o `Instante` (`src/widget.rs`), e a semântica dele é *anti*-calendário
  de propósito (cada seção vira dentro de si, sem carry), que é o oposto do que
  um `NaiveDateTime` faz — trocar seria desarmar a crate o tempo todo.

  Sobra **um** caso que a `std` não cobre: o *offset local* em Rust (a `std` só
  dá epoch UTC), necessário se o `Calendar` for marcar "hoje" sem receber a data
  por chave de contexto. **Resolvido pela negativa também** (revisão de
  2026-09-01): o `Calendar` recebe `today="{hoje}"` por prop, e o app preenche
  com `date.today()` — uma linha de Luau, com o global que a 0.72 criou
  exatamente para isso. Sem a prop, nenhum dia fica destacado. Se um dia a
  pergunta voltar, a resposta é **`chrono`**, não `time`:
  `time::OffsetDateTime::now_local()` devolve `Err` em processo multithread no
  Unix (o problema do `localtime_r`/`setenv`), e o motor é tokio — falharia
  justo no caso de uso.

  Uma correção de fato: o dia da semana não pede crate — é `days_from_civil`,
  seis linhas —, mas este documento dizia que ele já estava "em uso nos dois
  lados", e **não está**. Existe só no Luau (`date.weekday`, em
  `src/luau/prelude.luau`); o lado Rust tem o `dias_no_mes` e mais nada. As seis
  linhas entram junto do `Calendar`, ao lado dele em `src/widget.rs`.
- **Gráficos**: `canvas` na mão vs. integrar `plotters`.
- **Convenção de nomes**: manter aliases PT-BR (`botao`, `seletor`, `rolagem`…)
  para todo widget novo, ou só para o núcleo? (hoje o núcleo tem os dois.)
- **Rich text no `TextBrowser`/`QLabel`**: quanto do HTML/markdown do Qt vale
  reproduzir sobre o `markdown`/`rich` do iced.

---

## 5. Fases de construção (síntese priorizada)

Corte transversal da tabela por prioridade, na ordem que maximiza valor:

**Fase A — fechar o núcleo primitivo (P0/P1 sobre iced direto)**
~~`Radio`~~ ✅ · ~~`Slider`~~ ✅ · ~~`ProgressBar` (formalizar)~~ ✅ ·
~~`Tooltip`~~ ✅ · ~~`Space`~~ ✅ · `Grid` · ~~`password`/`secure` no
`TextInput`~~ ✅ · `QrCode`. Sobram dois: o `QrCode` (nativo do iced, barato) e
o `Grid`, que é o caro — o iced não tem grade.

**Fase B — destravar estado por instância (Motor P0)**
Sem markup novo; habilita a fase C inteira.

**Fase C — widgets compostos comuns (P1, ~~dependem de estado~~ dependem do
padrão da chave nomeada)**
`Tabs` (só a barra ✅; o empilhado espera **nome dinâmico de slot**) ·
`Accordion` (espera o `contains`) · ~~`SpinBox`~~ ✅ · ~~`GroupBox`~~ ✅ ·
`ListView` (com seleção) · `ToolBox` · `Pagination` · `Rating` ·
~~`ToolBar`/`StatusBar`~~ ✅ · ~~`Avatar`~~ ✅ ·
~~`Spinner`/`BusyIndicator`~~ ✅. É a **Onda 4** da §6.2: só o `Tabs` completo
continua atrás de um habilitador (o nome dinâmico de slot); o resto sai com o
motor como está.

**Fase D — data/hora (P1, foco declarado)**
~~`Calendar` → `DatePicker` → `TimePicker` → `DateTimePicker`. Depende de estado
+ overlay ancorado + valor de data.~~ **Metade feita, e a outra metade não
depende de nada disso.** Os três campos de edição saíram na 0.68 (teclado na
0.70). Sobram `Calendar`, `MonthYearPicker` e `DateRangePicker`, que são **uma
primitiva com três tags** e nenhum habilitador — é a **Onda 3** da §6.2. Só a
variante `calendarPopup` do `QDateEdit` continua atrás do overlay ancorado.

**Fase E — diálogos ricos (P1)**
~~`FileDialog` (open/save/directory via `rfd`)~~ ✅ · `InputDialog` ·
`ProgressDialog`.

**Fase F — overlays e menus (P2)**
~~`Menu` · `ContextMenu` · `MenuBar`~~ ✅ (overlay próprio em `src/menu.rs`) →
generalizar esse overlay → `Popover` · `Popup` · `Completer` · o `calendarPopup`
do `<dateedit>`. É a **Onda 5** da §6.2, junto do `Tabs` completo — que não é
overlay, mas responde a mesma pergunta (de quem é este conteúdo?).

**Fase G — model/view pesado (P2)**
`TableView` · `TableHeader` · `TreeView` · `ColumnView`, mais o `Grid` e o
`Flow`. É a **Onda 6**, e a releitura que ela traz é que o item caro não é o
"binding a coleção" (que já existe, via `items="chave"`) e sim a **medição de
coluna** — que o `Grid` e o `TableView` compartilham, e que ninguém tinha
catalogado como item de motor.

**Fase H — canvas e visualização (P2/P3)**
`Dial` · `Gauge` · `ColorDialog` · `LineChart`/`BarChart`/`PieChart` ·
`Sparkline`.

**Fase I — nicho/avançado (P3)**
`MdiArea` · `Dock` · `Wizard` · `SwipeView` · `Drawer` · `SystemTray` ·
`Shader`/3D · impressão.

---

### Resumo numérico

Contagem sobre as linhas que têm status, atualizada em 2026-09-01. Duas
ressalvas: a §2.4 tem uma linha (`QListWidgetItem`) que é dado, não widget, e
fica de fora; e o `Space` aparece duas vezes (§2.7 como container, §2.11 como
layout), então o total tem uma duplicata — 123 widgets distintos, não 124. O
total subiu de 122 porque `Pagination` e `Rating`, citados na §3 desde a 0.63
como exemplos de widget que nunca esteve bloqueado, não tinham linha.

| Categoria | Widgets catalogados | ✅ prontos | 🟡 parciais | ⬜ a fazer |
|---|---|---|---|---|
| Botões e ações | 10 | 6 | 1 | 3 |
| Entradas de texto | 9 | 4 | 1 | 4 |
| Numéricas/valor | 12 | 4 | 2 | 6 |
| Seleção/listas/árvores | 11 | 1 | 1 | 9 |
| Data e hora | 6 | 3 | 0 | 3 |
| Displays/indicadores | 15 | 10 | 0 | 5 |
| Containers | 9 | 4 | 0 | 5 |
| Navegação | 6 | 1 | 2 | 3 |
| Janela/barras | 8 | 7 | 0 | 1 |
| Diálogos | 14 | 9 | 0 | 5 |
| Layouts | 7 | 4 | 1 | 2 |
| Overlays/utilitários | 11 | 2 | 1 | 8 |
| Gráficos | 6 | 0 | 0 | 6 |
| **Total** | **124** | **55** | **9** | **60** |

O motor já entrega ~44% do catálogo Qt de superfície (34% antes da onda 2, 39%
antes da onda 1, 43% antes dos campos de data/hora — a queda de 45% para 44% é o
denominador que cresceu, não trabalho desfeito). O gargalo **também não é** o
punhado de habilitadores de Motor do §3, como este resumo afirmava: as ondas 3 e
4 da §6.2 somam quinze widgets e consomem **um** habilitador, o menor de todos
(o `contains`).

Onde os 60 se concentram: **model/view** (9), **gráficos** (6) e **overlays**
(8). Vinte e três linhas em três blocos — e o mapeamento da §6.2 mostra que os
três não estão atrás de três itens diferentes: model/view e boa parte dos
layouts saem de **uma** medição de coluna (Onda 6), os overlays saem de **uma**
generalização do que o `menu.rs` já tem (Onda 5), e os gráficos de **um**
`canvas` exposto (a Onda 7 esboçada). As quatro ondas escritas cobrem 30 das 60
linhas ⬜ com três itens de motor no total. Do resto, o troco decorativo — oito
linhas — está isolado na §6.3.

A §2.5 saiu dessa lista de um jeito que vale registrar: ela era 0 de 6 e passou
a 3 de 6 **sem** nenhum habilitador novo — só reclassificando os widgets de
builtin para primitiva (ver a correção na §3). E as outras 3 saem pelo mesmo
motivo, revisão seguinte: a Onda 3 da §6.2 leva a §2.5 a 6 de 6 sem tocar no
motor. Nem toda linha ⬜ está esperando o motor; algumas estão esperando alguém
perguntar em que nível elas deveriam estar — e este documento errou essa
pergunta três vezes seguidas (`TimePicker`, `Calendar`, `Accordion`), sempre
para o mesmo lado, o de superestimar o bloqueio.

As duas ondas mostram os dois regimes de custo. A onda 2 dependia de **um**
habilitador de motor (`<slot/>`) e, uma vez feito, valeu seis widgets — a §2.9,
o esqueleto da `QMainWindow`, passou de 1/8 para 7/8. A onda 1 não dependia de
nada: eram quatro widgets que o `iced` já sustentava e que só faltava expor.
Nenhuma das duas foi limitada por esforço de markup.

---

## 6. A fila de execução

O §5 ordena por fase; esta seção ordena por **execução**: o que construir a
seguir, na ordem, com o motor como ele está.

A primeira fila (as "dez", §6.1) foi executada inteira entre a 0.65 e a 0.67 e
fica registrada abaixo porque o *porquê* de cada item continua valendo como
documentação. A fila viva — o que vem agora — está na §6.2.

### 6.1 A primeira fila (as dez) — ✅ **concluída**

Dois critérios a ordenaram: primeiro o que não tinha bloqueio nenhum e mapeava a
widget nativo do `iced` (barato e imediato), depois o portão do `<slot/>`, depois
a família que ele destravava — cada item consumindo o anterior. Na execução as
duas ondas trocaram de lugar, a pedido: a 2 saiu primeiro.

#### Onda 1 — o que o `iced` já entrega (sem bloqueio) — ✅ **feita (0.66)**

Construída **depois** da onda 2, fora da ordem proposta, a pedido. Exemplo
executável em `examples/onda1` (`cargo run --example onda1`), que mostra os
quatro e — de propósito — a diferença entre primitiva e builtin com o mesmo
dado: `<slider>`/`<radio>` disparam a ação e o app grava a chave; `<radiogroup>`
grava sozinho.

Uma correção de rota no item 3: além do builtin `RadioGroup` planejado, nasceu
também a **primitiva `Radio`**. Desenhar a bolinha com glifos num builtin seria
pior do que usar o `radio` que o `iced` já tem — e com os dois, a divisão fica a
do Qt (`QRadioButton` + `QButtonGroup`).

| # | Widget | Nível | Por que aqui |
|---|---|---|---|
| 1 ✅ | **`Slider`** (`QSlider`) | Prim | A metade que falta do par que o `SpinBox` abriu — no Qt os dois quase sempre editam o mesmo valor. `◐`: valor numa chave nomeada + `on_change`, a forma do `<TextInput>`. Atributos saem iguais aos do `ProgressBar`; o evento é o `UiInputChanged` que o `<Checkbox>` já usa. Inclui `vertical_slider`, e as props baratas que o iced 0.14 dá de graça: `default` (duplo-clique reseta), `on_release` e `shift_step` |
| 2 ✅ | **`Space`** (`QSpacerItem`) | Prim | O espaçador flexível — custo quase zero (o `iced::widget::Space` já é usado internamente em `menu.rs`) e pré-requisito de layout das barras do item 9 |
| 3 ✅ | **`Radio`** + **`RadioGroup`** (`QRadioButton`/`QButtonGroup`) | Prim + Built | A última entrada de formulário básica que falta ao lado de `Checkbox`/`Toggle`. O grupo é o padrão do `SpinBox`: a chave nomeada guarda o **valor selecionado**, e cada opção escreve nela — nunca esteve bloqueado |
| 4 ✅ | **`Avatar`** | Built | O builtin apresentacional mais barato que resta (imagem ou iniciais num círculo), 100% prop-driven. Fecha a §2.6 ao lado de `Badge` |

#### Portão — `<slot/>`: filhos em componente (Motor, P1) — ✅ **feito (0.65)**

Não é widget, mas os seis itens seguintes são todos "envolver conteúdo", e
nenhum deles existiria sem isto. Foi o item mais barato do §3 e o de maior
alavancagem depois do estado por instância — sozinho, converteu seis linhas ⬜
da tabela em construídas.

Como ficou: `NodeType::Slot` no parser, e a expansão de componente em
`eval_owned` avalia os filhos do uso **antes** de entrar no template, no
contexto e com o dono de quem escreveu — é o que faz `on_click="salvar"` dentro
de um `<GroupBox>` chegar na tela em vez de virar `GroupBox::salvar`. Os filhos
do próprio `<slot>` são o conteúdo de reserva. Um uso com conteúdo fica fora do
cache de componente (as dependências dele são do quadro de quem chamou).
Detalhes e armadilhas em `BUILTINS.md`.

#### Onda 2 — a família dos agrupadores (depois do portão) — ✅ **feita (0.65)**

Todos os seis nasceram na mesma leva, com exemplo executável em
`examples/onda2` (`cargo run --example onda2`), que os mostra juntos montando o
esqueleto de uma janela: barra de ferramentas, abas, conteúdo e rodapé.

| # | Widget | Nível | Por que aqui |
|---|---|---|---|
| 5 ✅ | **`GroupBox`** (`QGroupBox`) | Built | O primeiro consumidor do `<slot/>` — moldura com título, o caso mais simples possível. Serve de validação do mecanismo antes dos outros cinco |
| 6 ✅ | **`Frame`** (`QFrame`) | Built | Irmão do 5 (borda/relevo configurável, sem título); sai quase de graça depois dele |
| 7 ✅ | **`Card`** | Built | Promove a componente do `examples/perfil` a builtin — é o que a tabela §2.6 afirmava existir e não existia. Com `<slot/>`, é `Frame` + convenções de padding/sombra |
| 8 ✅ | **`ToolButton`** (`QToolButton`) | Built | Botão-ícone com variantes flat/menu. Útil sozinho, e é a peça que o item 9 monta em série |
| 9 ✅ | **`ToolBar` + `StatusBar`** | Built | As duas faixas da janela principal. A `ToolBar` consome o `ToolButton` do 8 e o `Space` do 2; a `StatusBar` é o mesmo padrão invertido. Com elas, `MenuBar` (já ✅) + `ToolBar` + `StatusBar` fecham o esqueleto de uma `QMainWindow` |
| 10 ✅ | **`TabBar`** (`QTabBar`) | Built | A barra de abas com a aba ativa numa chave nomeada (padrão `SpinBox`), enquanto o conteúdo continua trocando por `se`/`senao`. É o `QTabWidget` entregue em duas etapas: a barra agora, o container de páginas quando houver slot **nomeado** — sem esperar o estado por instância |

#### O que a primeira fila deixou para trás

Três itens foram adiados **com motivo registrado**: `Calendar`/`DatePicker`
(esperavam estado por instância), `Grid` (caro — o `iced` não tem grade) e o
`Tabs` completo (espera nome dinâmico de slot).

Dos três, **um motivo não sobreviveu à revisão**: os campos de data saíram na
0.68 sem habilitador nenhum, e o `Calendar` sai na Onda 3 pela mesma razão — ver
§2.5. Os outros dois continuam de pé, e o `Grid` deixou de ser pré-requisito de
qualquer coisa da fila.

### 6.2 A fila viva — o que vem agora

Refeita em **2026-09-01**, depois da 0.73. Nada aqui está comprometido; é a
ordem que o documento recomenda a quem for retomar.

**O critério mudou.** A primeira fila (§6.1) ordenava por *custo*: o barato e
desbloqueado primeiro, o habilitador caro depois, para que cada rodada
entregasse algo visível. Funcionou, e por isso a versão anterior desta seção
abria com uma "bandeja de troco" — `QrCode`, `Chip`, `Skeleton`,
`NotificationDot` —, oito widgets pequenos e sem comportamento nenhum.

A leva seguinte mostrou por que isso estava invertido. Os
`<dateedit>`/`<timeedit>`/`<datetimeedit>` (0.68, teclado na 0.70) foram os
últimos widgets construídos e são os que mais mudaram o que dá para escrever
com o motor — porque **fazem** alguma coisa: aritmética de calendário, seções
com foco, avanço automático de dígito. Um `Skeleton` é um `<container>` cinza
com um `border_radius`; qualquer app escreve o dele em quatro linhas de markup e
nunca sentiu falta de uma tag. Um `Calendar`, não.

Então esta fila ordena por **função**: widgets que carregam lógica no `update`
(ou no `render_node`, quando são primitiva). O troco decorativo desce para a
§6.3, onde continua registrado e continua barato — é para quando faltar assunto,
não para abrir uma rodada.

Quatro ondas, em dois regimes — os mesmos dois que a §6.1 já tinha mostrado.

As duas primeiras **não consomem motor nenhum** (fora um habilitador de meia
tarde), e existem porque o documento superestimou o bloqueio:

- **Onda 3** — o calendário, e com ele o foco declarado do projeto.
- **Onda 4** — os widgets que carregam lógica, pelo padrão do `SpinBox`.

As duas seguintes são o outro regime: **um item de motor que vira meia dúzia de
widgets**, como o `<slot/>` foi na Onda 2.

- **Onda 5** — o conteúdo que sai da tela e entra no widget (abas com página,
  overlays ancorados).
- **Onda 6** — a grade: uma medição de colunas, e os seis widgets que saem dela.

As quatro respondem a mesma pergunta, que é a lição repetida deste documento:
**em que nível este widget deveria estar, e ele está mesmo bloqueado?** Nove das
linhas abaixo estão marcadas `Comp ●` — "bloqueado por estado por instância" — e
nenhuma delas está.

---

#### Onda 3 — o calendário: uma primitiva, três tags (o foco declarado)

A §2.5 está 3 de 6 e as três que faltam saem **juntas**, do mesmo arquivo, pelo
mesmo caminho que os campos de edição já abriram: uma primitiva em
`src/widget.rs` + um `NodeType` em `src/parser.rs`, com as tags decidindo só
quais partes aparecem — exatamente como `<dateedit>` e `<timeedit>` são o mesmo
`NodeType::DateTimeEdit`.

Nenhum habilitador de motor. A demonstração de por que (e a autópsia do erro que
mantinha essas três linhas marcadas como bloqueadas) está no blockquote da §2.5.

| # | Widget | Nível | Tag | O que grava | Por que aqui |
|---|---|---|---|---|---|
| 1 | **`Calendar`** (`QCalendarWidget`) | Prim | `<calendar>` | `YYYY-MM-DD` | O coração do foco declarado, e a peça de que as outras duas saem por variação. Grade 7×6, navegação de mês, dia selecionado |
| 2 | **`MonthYearPicker`** | Prim | `<monthyearpicker>` | `YYYY-MM` | A **mesma primitiva** em `mode="month"`: a tela de drill-up que o `QCalendarWidget` abre ao clicar no título, promovida a tag. Custo marginal quase zero depois do 1 — daí ter subido de P3 para P2 |
| 3 | **`DateRangePicker`** | Prim | `<daterangepicker>` | duas chaves | A **mesma primitiva** com `range`: `start`/`end` em chaves separadas (não `"a/b"` numa só, para o `date.diff` do Luau ler as duas direto) e `months="2"` desenhando dois meses lado a lado, como todo seletor de reserva de hotel |

**A forma proposta**, para não ficar em aberto na hora de escrever:

```xml
<!-- o caso simples: o widget grava a chave sozinho -->
<calendar value="entrada" today="{hoje}" />

<!-- com validação: quem grava é o handler -->
<calendar value="entrada" onChange="validar_entrada" min="{hoje}" />

<!-- só mês e ano -->
<monthyearpicker value="competencia" />

<!-- intervalo, dois meses visíveis -->
<daterangepicker start="entrada" end="saida" months="2" today="{hoje}" />
```

| Prop | Papel |
|---|---|
| `value` | nome da chave (o padrão do `SpinBox`); `start`/`end` no modo intervalo |
| `onChange` | vazio = **o widget grava a chave sozinho**; preenchido = delega. O contrato idêntico ao do `<datetimeedit>` e ao do `<TextInput>` |
| `today` | a data de hoje, para o realce. Prop, não relógio — ver a §4: é `date.today()` numa linha de Luau, e sem ela nenhum dia fica destacado |
| `month` | opcional: chave que dirige o mês visível. Sem ela, o mês visível mora numa chave do motor derivada da chave editada (`__cal_<chave>`), e o app não configura nada |
| `min` / `max` | limites; dias fora da faixa saem inertes |
| `mode` | `day` (padrão) · `month` · `year` — a mesma escada de drill-up do Qt |
| `first_day` | `sunday` (padrão, a base do `date.weekday` do prelúdio) ou `monday` |
| `months` | quantas grades desenhar lado a lado. Default 1 |

**As três coisas que precisam ser escritas em Rust**, e é só isso:

1. `days_from_civil` — seis linhas ao lado do `dias_no_mes` que o `Instante` já
   tem, para saber em que dia da semana o mês começa. **Não existe no lado
   Rust** hoje, ao contrário do que a §4 afirmava (correção registrada lá): só
   no `prelude.luau`.
2. O laço da grade — `Column` de `Row`s de `button`, com as células do mês
   anterior/seguinte esmaecidas. Aqui é que o `Grid` (`QGridLayout`) deixa de
   ser pré-requisito: grade em Rust é um `for`.
3. O hover do intervalo — a faixa que se pinta entre `start` e o dia sob o
   cursor. Estado global de verdade, e legitimamente: só uma célula da tela
   inteira está sob o cursor por vez. Mesma família do `__timeedit`.

**O que fica de fora, com motivo:** a variante `calendarPopup` do `QDateEdit`
(o `<dateedit>` abrindo este calendário num popup ancorado ao campo) continua
esperando o overlay ancorado genérico — §3, item 5. É o único item da §2.5 que
segue bloqueado, e ele é uma *composição* dos dois widgets, não um terceiro.

Fecha a §2.5 em **6 de 6** e a Fase D do §5.

---

#### Habilitador — `contains` no condicional (Motor, P1, o menor da lista)

Vem entre as ondas porque a Onda 4 consome, e porque é pequeno:

```xml
<template if="{abertas}" contains="rede"> … </template>
```

O `one_of="a b c"` já existe (§ `parser.rs`, `if_one_of`) e é o caso
**simétrico**: lá o valor da chave é um item e a lista está no markup; aqui a
lista está na chave e o item está no markup. Mesmo ponto do parser, mesmo ponto
do eval, uma comparação invertida.

O que ele destrava é o **conjunto nomeado** descrito na §3: um `Accordion` com
várias seções abertas, um `ListView` de seleção múltipla, um campo de filtros
por tags. Sem ele, esses três ficam presos ao estado por instância sem
precisar — e é a terceira vez que este documento descobre que um `●` era outra
coisa.

---

#### Onda 4 — os widgets que têm função

Todos **builtins** (exceto o 5), todos pelo padrão do `SpinBox`: a chave é
nomeada pelo app, a ação carrega o nome, o `update` faz a conta. Nenhum precisa
de estado por instância; o único habilitador é o `contains` acima, e só para
dois deles.

Ordenados por quanto cada um abre de tela real:

| # | Widget | Nível | Prio | Por que aqui |
|---|---|---|---|---|
| 1 | **`Pagination`** | Built | P1 | A função mais pura da lista: primeira/anterior/`n`/próxima/última, com o clamp e o cálculo da janela de números no `update`. Sem ele, toda lista longa de todo app é escrita à mão em Luau. Estava citado na §3 como "nunca esteve bloqueado" e **nem sequer tinha linha na tabela** — entrou na §2.4 nesta revisão |
| 2 | **`ListView` com seleção** | Built | P1 | Fecha o 🟡 mais antigo da §2.4. A coleção numa chave, o item escolhido noutra — é o `TabBar` com scroll. Seleção múltipla usa o conjunto nomeado (e o `contains`); a simples não precisa de nada. Casa com o 1 |
| 3 | **`Accordion`** + **`ToolBox`** | Built | P1 / P2 | Os dois modos do mesmo widget: várias seções abertas (`Accordion`, conjunto nomeado) e uma só (`ToolBox`, que é o `TabBar` na vertical e não precisa nem do `contains`). O `Accordion` está marcado **precisa estado** na §2.7 desde o começo do documento e não precisa |
| 4 | **`ButtonBox`** (`QDialogButtonBox`) | Built | P1 | Fecha o outro 🟡 da §2.1. A função é a que o Qt tem: **papéis** (`accept`/`reject`/`destructive`) e a ordem por plataforma decidida no widget, não na tela. Já existe dentro de `dialogs.rs`; com `<slot/>` (0.65) vira widget de tela |
| 5 | **`MaskedInput`** | **Prim** | P2 | A quarta aplicação da lição do `DateEdit`: máscara é função pura da string no `on_input`, e o que impedia o builtin era a indireção `{{value}}`. Alto valor num projeto que escreve em pt-BR — CPF, CNPJ, telefone, CEP, placa. Guarda cru na chave, exibe mascarado (a mesma separação valor/`displayFormat` do `<dateedit>`) |
| 6 | **`Rating`** | Built | P2 | Estrelas numa chave nomeada, com pré-visualização no hover (chave global, uma por tela — como o hover do `DateRangePicker`). Pequeno, mas é função, não desenho. Também estava citado na §3 e faltava na tabela |
| 7 | **`decimals` no `SpinBox`** | Built | P1 | Fecha o 🟡 do `QDoubleSpinBox`: hoje as casas saem do `step` (`step="0.25"` → 2 casas), o que acerta por acidente e erra em `step="1"` sobre um preço. Meia hora de trabalho no `spin_box.rs` |

Um exemplo executável por onda, como nas anteriores (`examples/onda3` …
`examples/onda6`), é o que fecha cada uma.

**Saldo das ondas 3 e 4:** a §2.5 vai a 6/6, a §2.4 ganha três, a §2.7 dois, os
dois 🟡 mais velhos (`ButtonBox`, `QDoubleSpinBox`) fecham. E o mais
interessante não é a contagem: são **sete linhas** que este documento declarava
bloqueadas por estado por instância e que não estão. Vale terminar a leva
passando o resto dos `●` pela mesma pergunta.

---

#### Onda 5 — o conteúdo que sai da tela e entra no widget

Duas coisas que hoje o app monta à mão, na tela, e deveriam morar **dentro** do
widget: a página de uma aba e o painel que flutua. São dois habilitadores
diferentes, mas a pergunta é a mesma — *de quem é este conteúdo?* — e por isso
saem juntos.

**Habilitador A — nome dinâmico de slot** (Motor, P1, pequeno).
`<slot name="{aba}"/>`, resolvido contra o contexto. A partição por nome já
existe desde a 0.67; falta interpolar o nome antes da busca, no mesmo ponto em
que o `eval` já roda `process_tpl`. É o menor habilitador que restou do §3.

**Habilitador B — overlay ancorado genérico** (Motor, P1, médio).
`src/menu.rs` já construiu um overlay ancorado com cascata de submenus, e o
`DIALOGS.md` já mapeou as armadilhas (`Interaction::Idle` + `on_press` sempre
presente). O que existe está **fechado sobre `MenuNode`**: generalizar isso — ou
trocá-lo por um `iced::advanced::{Widget, Overlay}` custom, o caminho que o
próprio `menu.rs` documenta como o certo — é o trabalho. Não é pesquisa; é
refatoração com um consumidor já escrito para validar.

| # | Widget | Nível | Prio | Hab. | Por que aqui |
|---|---|---|---|---|---|
| 1 | **`Tabs` completo** (`QTabWidget`) | Built | P1 | A | A barra saiu na 0.65; a página continua sendo `se`/`senao` na tela. Com o nome dinâmico, vira `addTab(widget, "Geral")` — o conteúdo passa a ser filho do widget, e a tela deixa de repetir a lista de abas duas vezes. Fecha o 🟡 mais visível da §2.8 |
| 2 | **`calendarPopup` no `<dateedit>`** | Prim | P1 | B | O único item que a Onda 3 deixou para trás, e o último buraco do **foco declarado**: o campo por seções abre a grade de mês ancorada nele. Os dois lados já existirão — é a solda |
| 3 | **`Popover`** | Prim | P2 | B | O mecanismo cru virado tag: conteúdo por `<slot/>`, ancorado a um gatilho, aberto/fechado numa chave nomeada. É o `Popup` do QML e o que todo menu de usuário/seletor de emoji/painel de filtro pede |
| 4 | **`Popup`** | Prim | P2 | B | A **mesma primitiva** sem âncora — centrado na janela, sem a modalidade de um `<dialog>`. Custo marginal, pelo padrão `<dateedit>`/`<timeedit>` |
| 5 | **`Completer`** / **`Autocomplete`** | Prim | P2 | B | A função de verdade desta onda: filtrar enquanto se digita, navegar a lista com ↑↓, aceitar com Enter, desistir com Esc — e devolver o foco ao campo. Fecha a linha duplicada nas §2.2/§2.4/§2.12, e é o widget que mais aparece em app real dos que faltam |
| 6 | **`Drawer`** | Built | P2 | — | Painel lateral deslizante. **Não precisa de nenhum dos dois habilitadores** — é `<slot/>` (0.65) + uma chave nomeada + a animação que o motor já tem (`ANIMACOES.md`); entra aqui por parentesco, não por bloqueio. Se a onda atrasar, é o item que dá para adiantar |

**O que fecha:** a §2.12 sai de 2/11 para 5/11 e a §2.8 fecha o `QTabWidget`. E
some a última ressalva da §2.5 — a linha do `QDateEdit` está ✅ desde a 0.68 mas
carrega um "falta a variante `calendarPopup`" desde então. O foco declarado do
projeto termina aqui, não na Onda 3.

---

#### Onda 6 — a grade: uma medição, seis widgets

Esta é a onda cara, e ela é cara **uma vez só**. O documento catalogava dois
itens separados como caros — o `Grid` ("o `iced` não tem grade") e o `TableView`
("**grande**: cabeçalho, seleção, sort, edição") — sem notar que a parte difícil
dos dois é **a mesma**: descobrir a largura de uma coluna a partir de todas as
células que passam por ela, antes de desenhar qualquer uma.

Feito uma vez, como um `iced::advanced::Widget` que mede filhos no `layout()`,
esse cálculo paga seis linhas da tabela. É a maior alavancagem que sobrou no
catálogo, e é o mesmo tipo de aposta que o `<slot/>` foi na Onda 2: um item de
motor que vira meia dúzia de widgets.

**Correção de rota, a quinta da mesma família.** O §3 lista "binding a coleção
(model/view)" como habilitador P2 "caro, e o maior investimento restante". Ele
**já existe**: a convenção `items="chave"` — um array JSON numa chave de
contexto — é o que o `<Menu items="…">` e o `<TabBar items="…">` usam hoje, e é
o que todo `for-each` do motor sempre leu. O que falta não é ligar a coleção; é
medir a grade e convencionar seleção e ordenação — e as duas últimas são o
padrão do `SpinBox`, que já está escrito.

E o `TreeView` sai junto pelo motivo que a Onda 4 já usou duas vezes: o conjunto
de nós abertos é um **conjunto nomeado** (`abertos="raiz,raiz/src"`), não um bit
de estado por nó. Com o `contains` da Onda 4, ele deixa de esperar o estado por
instância — que é onde o §3 ainda o coloca.

| # | Widget | Nível | Prio | Por que aqui |
|---|---|---|---|---|
| 1 | **`Grid`** (`QGridLayout`) | Prim | P1 | A medição, e o widget mais simples que a exercita. Sai primeiro porque é o teste do mecanismo antes de haver cabeçalho, ordenação e seleção por cima — o mesmo papel que o `GroupBox` teve para o `<slot/>` |
| 2 | **`Flow`** / **`Wrap`** | Prim | P2 | A mesma medição num eixo só: quebra automática de linha. Fecha a §2.11 (layouts) e é o que um campo de tags/chips pede |
| 3 | **`TableHeader`** (`QHeaderView`) | Prim | P2 | Cabeçalho clicável (ordenar) e arrastável (redimensionar). O arrasto mora numa chave global — um por vez, a família do `__drag_key` que o motor já tem |
| 4 | **`TableView`** | Prim | P2 | Cabeçalho + corpo, com **ordenação** (coluna e direção em chaves nomeadas, a comparação no `update`) e **seleção** (chave nomeada; múltipla pelo conjunto nomeado). Edição de célula fica para depois — reusa o `<TextInput>`, não a medição |
| 5 | **`TreeView`** | Prim | P2 | Recursão sobre a coleção + conjunto nomeado de nós abertos. Sai do estado por instância pela mesma porta que o `Accordion` |
| 6 | **`ColumnView`** | Prim | P3 | Navegação Miller (o Finder): uma `ListView` por nível, o nível escolhido numa chave. Quase de graça depois do 5 |

**Habilitador desta onda — virtualizar a lista (Motor, P1). A FAZER.**

Hoje o motor constrói **todas** as linhas de uma lista, mesmo que só vinte
caibam na tela: numa lista de 2000, as outras 1980 são avaliadas, ocupam
memória e são desenhadas para fora da área visível. Virtualizar é construir só
as que aparecem, descartando as que saem e montando as que entram conforme se
rola — o que toda lista grande faz.

As duas releases de custo (0.74 e 0.75) chegaram até onde dava sem isto:
encolheram o nó, tiraram a cópia do cache e adiaram a montagem das variáveis de
item. Uma lista de 2000 linhas saiu de 26,6 ms para ~10 ms por mudança de
estado que nem toca nela. **O que falta para tirar esse custo de vez é a
virtualização** — e as duas releases deixaram registrado, no CHANGELOG, que o
lugar dela é aqui.

Por que ela não é ajuste e sim obra:

| O que falta | Por quê |
|---|---|
| saber **quanto** já se rolou | o `scrollable` do iced sabe (`on_scroll`), mas o motor não plumba esse evento hoje: não há mensagem, não há chave de contexto, ninguém escuta |
| saber a **altura** de uma linha | para calcular quais índices caem na janela visível. Ou se assume altura fixa (simples, e cobre a tabela) ou se mede (geral, e caro) |
| conviver com o que já existe | o arrasto de reordenação guarda a ordem inteira da lista, e o cache de avaliação é indexado por `mix(node_id, índice)` — os dois precisam continuar corretos com só uma fatia dos itens avaliada |

Sai junto do `TableView` (item 4) porque é ele quem precisa disso de verdade —
mas serve `ListView`, `TreeView` e qualquer `for-each` dentro de um
`<scrollable>`, então vale construir como capacidade do motor, não como
detalhe de um widget.

**O que fecha:** a §2.4 (seleção/listas/árvores) sai de 1/11 — a categoria mais
atrasada do catálogo desde o começo — para 8/11, e a §2.11 (layouts) fecha. É a
onda que muda mais o resumo numérico de todas — e, com a virtualização, a que
tira o último teto de tamanho de lista do motor.

**Onda 7, se alguém perguntar:** o `canvas` como primitiva, e a família que ele
destrava de uma vez — `Dial`, `Gauge`, `LcdNumber`, `ColorDialog` e a §2.13
inteira (gráficos, 6 linhas em 0). Mesmo formato das ondas 5 e 6: um item de
motor, meia dúzia de widgets. Fica sem detalhar porque as decisões dele (`canvas`
na mão vs. `plotters`, §4) ainda estão abertas, e escrevê-las agora seria
inventar.

---

#### Onde os habilitadores do §3 foram parar

O §3 lista nove itens de motor. Depois de distribuí-los pelas quatro ondas,
sobram **dois** — e nenhum dos dois bloqueia coisa alguma da fila:

| Habilitador (§3) | Onde ficou |
|---|---|
| `contains` no condicional | **Onda 4**, como pré-requisito de dois itens |
| Nome dinâmico de slot | **Onda 5**, habilitador A |
| Overlay ancorado genérico | **Onda 5**, habilitador B |
| `Grid` | **Onda 6**, item 1 — deixou de ser pré-requisito do `Calendar` (§2.5) e virou a ponta do mecanismo de medição |
| Binding a coleção (model/view) | **Onda 6** — e menor do que estava catalogado: a ligação já existe (`items="chave"`), falta a medição e as convenções de seleção/ordenação |
| `ctx.dispatch(acao)` | Continua P2 e continua sem consumidor urgente: o caso declarativo já se resolveu com o prefixo `app:` (0.63) |
| Contexto tipado / valor de data | **Fechado pela negativa** (0.72/0.73): o global `date` do prelúdio Luau cobre o lado do script, o `Instante` cobre o do widget. Sai da lista |
| **Canvas como primitiva** | **Onda 7** (esboçada acima): `Dial`, `Gauge`, `LcdNumber`, `ColorDialog` e a §2.13 inteira |
| **Estado por instância** | O último de pé, e o que sobrou dele é pequeno: `MdiArea`, `Dock`, `RangeSlider` e a edição de célula em árvore profunda. Rebaixado de P0 para P1 nesta revisão — não por ter encolhido, mas porque parou de ser o caminho crítico de qualquer coisa que se queira construir |
| Subscriptions de teclado | Continua P2, independente das quatro ondas (`Shortcut`/`Action` globais) |

A leitura que isso permite: o item que este documento chamou por três revisões
de "o desbloqueio de maior alavancagem" **não é o gargalo de nada** que se queira
construir nas próximas quatro levas. O gargalo real é a medição da Onda 6 — e
esse nunca esteve na lista do §3.

### 6.3 A bandeja de troco (fica registrada, não abre rodada)

Os widgets pequenos que a versão anterior desta seção usava como Onda 3.
Continuam corretos e continuam baratos — o que mudou é que nenhum deles é motivo
para abrir uma leva. Entram de carona quando um da fila acima passar perto:

| Widget | Nível | Prio | Nota |
|---|---|---|---|
| `QrCode` | Prim | P3 | O `iced` tem `qr_code` nativo; é o último item barato da Fase A, quase só encanamento |
| `Chip` | Built | P2 | `Badge` com um "×" e uma ação de remover. Sai junto de um campo de tags — que por sua vez quer o `contains` da Onda 4 |
| `Skeleton` | Built | P2 | Placeholder de carregamento. Quatro linhas de markup em qualquer app; vira tag só por conveniência |
| `CommandLink` | Built | P2 | Título + descrição + seta, sem estado |
| `PageIndicator` | Built | P2 | Os pontinhos — o rosto visual do `Pagination` (Onda 4, item 1), com quem compartilha a chave. Sai de graça depois dele |
| `NotificationDot` | Built | P2 | Pontinho sobre um ícone; pede `Stack` dentro do builtin, ou um `padding` negativo bem escolhido |
| `RoundButton` | Built | P3 | `border-radius` total |
| `TextBrowser` | Built | P2 | Render read-only de markdown com links, sobre o `markdown` do iced |
