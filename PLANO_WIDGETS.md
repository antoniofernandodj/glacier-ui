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
§6.1 guarda a fila já cumprida, porque o *porquê* de cada item continua valendo.

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
| QCommandLinkButton | `CommandLink` | Built | button+col | — | P2 | ⬜ | título + descrição + seta |
| QDialogButtonBox | `ButtonBox` | Built | row+button | — | P1 | 🟡 | existe nos diálogos; expor como builtin de tela |
| (switch/QML Switch) | `Toggle`/`Toggler` | Prim | toggler | ◐ | P0 | ✅ | já existe |
| QML RoundButton | `RoundButton` | Built | button | — | P3 | ⬜ | border-radius total |
| QML DelayButton | `DelayButton` | Comp | button+canvas | ● | P3 | ⬜ | anel de progresso ao segurar |

### 2.2 Entradas de texto

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QLineEdit | `TextInput` | Prim | text_input | ◐ | P0 | ✅ | já existe |
| QLineEdit (password) | `TextInput password` | Prim | text_input | ◐ | P1 | ✅ | flag `secure`/`password`/`seguro`/`senha` no `<TextInput>`, sobre o `.secure()` do iced |
| QLineEdit (mask/validator) | `MaskedInput` | Comp | text_input | ● | P2 | ⬜ | máscara + validação (CPF, telefone…) |
| QTextEdit (rich) | `TextEditor` | Prim | text_editor | ● | P1 | ✅ | multi-linha; rich text é limitado |
| QPlainTextEdit | `PlainTextEditor` | Prim | text_editor | ● | P1 | 🟡 | variante sem formatação |
| QTextBrowser | `TextBrowser` | Built | markdown/scrollable | — | P2 | ⬜ | render read-only + links |
| QKeySequenceEdit | `ShortcutInput` | Comp | text_input | ● | P3 | ⬜ | captura combinação de teclas |
| QComboBox (editable) | `ComboEdit` | Prim | combo_box | ◐ | P1 | ✅ | `options`/`value`/`onChange`/`onSelect`/`placeholder` + `labelField`/`valueField` para listas de objetos (ver `examples/combo_edit`) |
| — (autocomplete) | `Autocomplete` | Comp | text_input+overlay | ● | P2 | ⬜ | ver QCompleter (§2.12) |

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

### 2.4 Seleção, listas e árvores (model/view)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QComboBox | `Select` / `Combo` | Prim | pick_list / combo_box | ◐ | P0 | ✅ | ambos existem |
| QFontComboBox | `FontSelect` | Comp | combo_box | ● | P3 | ⬜ | lista fontes do sistema |
| QListWidget | `ListView` | Comp | scrollable+ForEach | ● | P1 | 🟡 | dá para fazer com `for`; falta seleção/estado |
| QListView (model) | `ListView bind` | Motor+Comp | scrollable | ● | P2 | ⬜ | ligado a coleção do contexto |
| QTreeWidget/QTreeView | `TreeView` | Comp | column+recursão | ● | P2 | ⬜ | expandir/recolher = estado por nó |
| QTableWidget/QTableView | `TableView` | Comp | column+row / canvas | ● | P2 | ⬜ | **grande**: cabeçalho, seleção, sort, edição |
| QHeaderView | `TableHeader` | Comp | row+button | ● | P2 | ⬜ | parte da TableView; sort/resize |
| QColumnView | `ColumnView` | Comp | row+ListView | ● | P3 | ⬜ | navegação Miller (finder) |
| QListWidgetItem etc. | (dados, não widget) | — | — | — | — | — | modelados como valores de contexto |
| QCompleter | `Completer` | Comp | overlay+ListView | ● | P2 | ⬜ | popup de sugestões (ver §2.12) |
| QML PageIndicator | `PageIndicator` | Built | row | — | P2 | ⬜ | pontinhos de página |

### 2.5 Data e hora — **foco declarado do projeto**

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QCalendarWidget | `Calendar` | Comp | column+row+button | ● | P1 | ⬜ | grade de mês, navegação, dia selecionado |
| QDateEdit | `DateEdit` / `DatePicker` | Prim | compõe | ◐ | P1 | ✅ | edição por **seções** (ano/mês/dia), com o realce da paleta na seção ativa e ▴▾ agindo sobre ela. Calendário respeitado: 31/01 + 1 mês satura em 28 ou 29. `format="br"` troca só a exibição — a chave é sempre ISO. Falta a variante `calendarPopup`, que espera o overlay ancorado (§3) |
| QTimeEdit | `TimeEdit` / `TimePicker` | Prim | compõe | ◐ | P1 | ✅ | as mesmas seções, para hora/minuto\[/segundo\]. Cada seção vira **dentro de si** (o `wrapping` do `QAbstractSpinBox`): mexer no minuto não empurra a hora |
| QDateTimeEdit | `DateTimeEdit` | Prim | compõe | ◐ | P1 | ✅ | as duas famílias de seção no mesmo campo. É **a mesma primitiva** dos dois acima — a tag só decide quais seções aparecem |
| — (range) | `DateRangePicker` | Comp | 2×Calendar | ● | P2 | ⬜ | intervalo início→fim |
| — (mês/ano) | `MonthYearPicker` | Comp | pick_list×2 | ● | P3 | ⬜ | seleção só de mês/ano |

> **A dependência de datas não foi necessária** para os três campos de edição: a
> aritmética que eles pedem é somar 1 numa seção e saber quantos dias tem o mês
> — bissexto incluído, regra do século incluída —, o que cabe em vinte linhas
> (`Instante`, em `src/widget.rs`). A decisão `chrono` vs. `time` (§4) segue em
> aberto e passa a valer para o que realmente precisa dela: o `Calendar`, que
> tem de saber em que **dia da semana** um mês começa, e qualquer aritmética de
> intervalo.
>
> Os três campos **também não precisaram de estado por instância** — ver a
> correção na §3.

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
| — (chip removível) | `Chip` | Built | row+button | — | P2 | ⬜ | badge com "×" |
| — (separador) | `Divider` / `Rule` | Prim | rule | — | P0 | ✅ | `Rule` existe |
| QFrame | `Frame` | Built | container | — | P2 | ✅ | três formas: `box` (contorno), `filled` (contraste, o `QFrame::Panel`) e `none`. Sem `Raised`/`Sunken`: o `UiNode` não tem campo de sombra |
| — (skeleton) | `Skeleton` | Built | container | — | P2 | ⬜ | placeholder de carregamento |
| — (QR) | `QrCode` | Prim | qr_code | — | P3 | ⬜ | iced tem nativo |
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
| QToolBox | `ToolBox` | Comp | column+button | ● | P2 | ⬜ | seções empilhadas expansíveis |
| — (accordion) | `Accordion` | Comp | column+button | ● | P1 | ⬜ | itens abre/fecha — **precisa estado** |
| QMdiArea/QMdiSubWindow | `MdiArea` | Comp | canvas/stack | ● | P3 | ⬜ | janelas MDI internas |
| QDockWidget | `Dock` | Comp | pane_grid | ● | P3 | ⬜ | painéis acopláveis |
| QSpacerItem | `Space` | Prim | space | — | P1 | ✅ | sem `width`/`height` é `Fill` nos dois eixos (o espaçador flexível); com eles, vão fixo. Duplicado na §2.11 por ser layout **e** container |

### 2.8 Navegação (abas, wizard, stacks)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QTabWidget/QTabBar | `TabBar` | Built | row+button | ◐ | P1 | 🟡 | a **barra** existe (0.65): abas de uma coleção do contexto, ativa numa chave que o app nomeia (padrão `SpinBox`). O empilhado de páginas continua sendo `se`/`senao`: o `QTabWidget` inteiro precisa de uma página por aba, e o slot nomeado da 0.67 tem **nome fixo** — falta o nome **dinâmico** (`<slot name="{aba}"/>`) |
| QStackedWidget | `Stack`/`StackView` | Comp | condicional (`se`) | ◐ | P1 | 🟡 | já dá com `se`; formalizar |
| QWizard/QWizardPage | `Wizard` | Comp | Stack+ButtonBox | ● | P2 | ⬜ | passos com voltar/avançar/finalizar |
| QML SwipeView | `SwipeView` | Comp | stack | ● | P3 | ⬜ | páginas deslizáveis |
| QML Drawer | `Drawer` | Comp | stack+animação | ● | P2 | ⬜ | painel lateral deslizante |
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
| QGridLayout | `Grid` | Prim/Motor | — | P1 | ⬜ | grade linhas×colunas — o `iced` não tem, então é composição de `Row`/`Column` com medição. O item mais caro que sobrou da Fase A |
| QFormLayout | `Form` / `formulario` | Prim | — | P1 | ✅ | `Form` existe |
| QStackedLayout | `se`/`senao` + Stack | Motor | ◐ | P1 | 🟡 | condicional existe |
| QSpacerItem | `Space` | Prim | — | P1 | ✅ | ver §2.7 |
| (flow layout) | `Flow`/`Wrap` | Comp | row+quebra | — | P2 | ⬜ | quebra automática de linha |

### 2.12 Overlays, dicas e utilitários

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QToolTip | `tooltip=` | Prim | tooltip | — | P1 | ✅ | **atributo universal**, não tag: `tooltip`/`title`/`dica` em *qualquer* nó, com `tooltip_position` |
| QML ToolTip | (idem) | Prim | tooltip | — | P1 | ✅ | mesmo atributo |
| QWhatsThis | — | — | — | — | P3 | ⬜ | ajuda contextual (raro) |
| QCompleter | `Completer` | Comp | text_input+overlay | ● | P2 | ⬜ | sugestões enquanto digita |
| QML Popup | `Popup` | Comp | stack | ● | P2 | ⬜ | genérico ancorado |
| — (menu popover) | `Popover` | Comp | stack | ● | P2 | ⬜ | conteúdo flutuante ancorado |
| QSplashScreen | `SplashScreen` | Comp | stack | — | P3 | ⬜ | tela de abertura |
| QRubberBand | — | Comp | canvas | ● | P3 | ⬜ | retângulo de seleção |
| QShortcut/QAction | `Shortcut`/`Action` | Motor | subscription | ● | P2 | ⬜ | atalhos globais de teclado |
| QScroller | (no scrollable) | Motor | scrollable | — | P3 | 🟡 | rolagem por gesto |
| — (badge de notificação) | `NotificationDot` | Built | container | — | P2 | ⬜ | pontinho sobre ícone |

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
`QProgressBar (busy)` já tinha recebido.

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
> O que **de fato** espera o estado por instância na §2.5 é o `Calendar`: uma
> grade de mês tem estado de navegação (que mês estou vendo) que é dele, não do
> app. Regra prática que fica: **antes de declarar um widget bloqueado, pergunte
> se ele não é uma primitiva.**

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

1. **Estado por instância** (`●` do segundo tipo, acima). **P0.**
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
   falta para telas densas. **P1.**
5. **Sistema de overlay ancorado reutilizável** — `Stack` + posição relativa a
   um widget âncora. **Meio resolvido:** `src/menu.rs` já construiu um overlay
   ancorado com cascata de submenus, mas fechado sobre `MenuNode` — não é um
   mecanismo genérico. Generalizá-lo (ou trocá-lo por um
   `iced::advanced::{Widget, Overlay}` custom, o caminho que o próprio
   `menu.rs` documenta como o "certo") destrava `Popup`, `Popover`,
   `Completer` e o popup do `DatePicker`. Lição de `DIALOGS.md` já mapeou os
   cuidados (`Interaction::Idle` + `on_press` sempre presente). **P1.**
6. **Contexto tipado / valor de data** — reduz `to_string()`/parse manual;
   necessário para pickers de data robustos. **P1.**
7. **Binding a coleção (model/view)** — `ListView`/`TableView`/`TreeView`
   ligados a uma coleção do contexto, com seleção. **P2.**
8. **Canvas exposto como primitiva** — destrava `Dial`, `Gauge`,
   `ColorDialog`, `LcdNumber`, todos os gráficos. **P2.** (`Spinner` saiu
   desta lista: acabou não precisando de `canvas` nem de estado por
   instância — ver a nota da linha `QProgressBar (busy)` na tabela §2.3.)
9. **Subscriptions de teclado** — `Shortcut`/`Action` globais. **P2.**

---

## 4. Decisões em aberto

- ~~**Diálogos nativos vs. próprios** (arquivo)~~ — **decidido e feito**: `rfd`
  nativo do SO, em `src/file_dialog.rs`, exposto ao Luau como
  `open_file`/`open_files`/`save_file`/`pick_folder`. A versão própria
  estilizável segue em P3. Cor e fonte continuam em aberto pela mesma pergunta.
- **Dependência de datas**: `chrono` vs. `time` para o módulo de data/hora.
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

**Fase C — widgets compostos comuns (P1, dependem de estado)**
`Tabs` (só a barra ✅; o empilhado espera **nome dinâmico de slot**) ·
`Accordion` · ~~`SpinBox`~~ ✅ · ~~`GroupBox`~~ ✅ · `ListView` (com seleção) ·
~~`ToolBar`/`StatusBar`~~ ✅ · ~~`Avatar`~~ ✅ ·
~~`Spinner`/`BusyIndicator`~~ ✅. Restam dois, e cada um espera um
habilitador diferente: o `Tabs` completo, o nome dinâmico; o `Accordion`, o
estado por instância.

**Fase D — data/hora (P1, foco declarado)**
`Calendar` → `DatePicker` → `TimePicker` → `DateTimePicker`. Depende de estado
+ overlay ancorado + valor de data.

**Fase E — diálogos ricos (P1)**
~~`FileDialog` (open/save/directory via `rfd`)~~ ✅ · `InputDialog` ·
`ProgressDialog`.

**Fase F — overlays e menus (P2)**
~~`Menu` · `ContextMenu` · `MenuBar`~~ ✅ (overlay próprio em `src/menu.rs`) →
generalizar esse overlay → `Popover` · `Popup` · `Completer`.

**Fase G — model/view pesado (P2)**
`TableView` · `TreeView` · `ColumnView`. O maior investimento; requer binding a
coleção.

**Fase H — canvas e visualização (P2/P3)**
`Dial` · `Gauge` · `ColorDialog` · `LineChart`/`BarChart`/`PieChart` ·
`Sparkline`.

**Fase I — nicho/avançado (P3)**
`MdiArea` · `Dock` · `Wizard` · `SwipeView` · `Drawer` · `SystemTray` ·
`Shader`/3D · impressão.

---

### Resumo numérico

Contagem sobre as linhas que têm status. Duas ressalvas: a §2.4 tem uma linha
(`QListWidgetItem`) que é dado, não widget, e fica de fora; e o `Space` aparece
duas vezes (§2.7 como container, §2.11 como layout), então o total tem uma
duplicata — 121 widgets distintos, não 122.

| Categoria | Widgets catalogados | ✅ prontos | 🟡 parciais | ⬜ a fazer |
|---|---|---|---|---|
| Botões e ações | 10 | 6 | 1 | 3 |
| Entradas de texto | 9 | 4 | 1 | 4 |
| Numéricas/valor | 11 | 4 | 2 | 5 |
| Seleção/listas/árvores | 10 | 1 | 1 | 8 |
| Data e hora | 6 | 3 | 0 | 3 |
| Displays/indicadores | 15 | 10 | 0 | 5 |
| Containers | 9 | 4 | 0 | 5 |
| Navegação | 6 | 1 | 2 | 3 |
| Janela/barras | 8 | 7 | 0 | 1 |
| Diálogos | 14 | 9 | 0 | 5 |
| Layouts | 7 | 4 | 1 | 2 |
| Overlays/utilitários | 11 | 2 | 1 | 8 |
| Gráficos | 6 | 0 | 0 | 6 |
| **Total** | **122** | **55** | **9** | **58** |

O motor já entrega ~45% do catálogo Qt de superfície (34% antes da onda 2, 39%
antes da onda 1, 43% antes dos campos de data/hora). O gargalo não é volume de
markup — é o punhado de habilitadores de Motor do §3 (estado por instância +
**nome dinâmico de slot** + `Grid` + overlay ancorado genérico + canvas), que
sozinhos destravam a maioria dos 58 pendentes.

Onde os 58 se concentram: **model/view** (8, atrás do binding a coleção),
**gráficos** (6, atrás do canvas) e **overlays** (8, atrás do overlay ancorado
genérico). São 22 presos a três itens de motor; o resto é, em boa medida, a
bandeja de troco da §6.2.

A §2.5 saiu dessa lista de um jeito que vale registrar: ela era 0 de 6 e passou
a 3 de 6 **sem** nenhum habilitador novo — só reclassificando os widgets de
builtin para primitiva (ver a correção na §3). Nem toda linha ⬜ está esperando
o motor; algumas estão esperando alguém perguntar em que nível elas deveriam
estar.

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

Três itens foram adiados **com motivo registrado**, e os três reaparecem na fila
nova abaixo: `Calendar`/`DatePicker` (esperam estado por instância), `Grid` (caro
— o `iced` não tem grade) e o `Tabs` completo (espera nome dinâmico de slot).

### 6.2 A fila viva — o que vem agora

Nada aqui está comprometido; é a ordem que o documento recomenda a quem for
retomar. Como na primeira fila, o barato-e-desbloqueado vem antes do habilitador
de motor, para que cada rodada entregue algo visível.

#### Onda 3 — a bandeja de troco (sem bloqueio nenhum)

Widgets pequenos, todos construíveis hoje. Nenhum é individualmente importante;
juntos fecham buracos que aparecem em qualquer tela real, e servem de rodada
curta entre dois habilitadores caros.

| Widget | Nível | Prio | Por quê |
|---|---|---|---|
| **`QrCode`** | Prim | P3 | O `iced` tem `qr_code` nativo — é o último item barato da Fase A, quase só encanamento |
| **`Chip`** | Built | P2 | `Badge` com um "×" e uma ação de remover; o par de qualquer campo de tags/filtros |
| **`Skeleton`** | Built | P2 | Placeholder de carregamento — hoje uma tela em espera não tem o que mostrar além do `<spinner>` |
| **`ButtonBox`** | Built | P1 | Já existe **dentro** dos diálogos (§2.1 está 🟡 por isso); com `<slot/>`, expor como widget de tela é barato |
| **`CommandLink`** | Built | P2 | Botão com título + descrição + seta; props, sem estado |
| **`PageIndicator`** | Built | P2 | Pontinhos de página, pelo padrão do `SpinBox` (coleção + chave nomeada) — o irmão do `TabBar` |
| **`NotificationDot`** | Built | P2 | Pontinho sobre um ícone; pediria `Stack`, ou um `padding` negativo bem escolhido |
| **`decimals` no `SpinBox`** | Built | P1 | Fecha o 🟡 do `QDoubleSpinBox`: hoje as casas saem do `step`, falta a prop explícita |

#### Habilitador — nome **dinâmico** de slot (Motor, P1)

`<slot name="{aba}"/>`, resolvido contra o contexto. A partição por nome já
existe (0.67); o que falta é interpolar o nome antes da busca, no mesmo ponto em
que o `eval` já roda `process_tpl`.

Destrava o **`Tabs` completo** (`QTabWidget`) — a página visível passa a morar
dentro do widget, como o `addTab(widget, "Geral")` do Qt, em vez de um
`se`/`senao` na tela. É o menor dos habilitadores que restam.

#### Habilitador — `Grid` (`QGridLayout`, P1)

O `iced` não tem grade: é composição de `Row`/`Column` com medição. Caro, e sem
substituto — o `Form` cobre formulário, não painel. Vale antes da família de
data/hora porque um `Calendar` **é** uma grade 7×6.

#### Habilitador — estado por instância (Motor, **P0**)

O item de maior alavancagem do §3, e o único que separa o projeto do seu **foco
declarado**. Sem ele a §2.5 inteira (0 de 6) fica parada, e com ele vem também
`Accordion`, `ToolBox`, `ListView` com seleção e o `TreeView`.

Vale reler a §3 antes de começar: o `●` marca três coisas diferentes e só uma
delas é este item. Duas já se resolveram sem ele — o valor que o app nomeia (o
padrão do `SpinBox`) e o estado sem nome que cabe no `tree::State` do widget
nativo (o `Spinner`).

#### Onda 4 — data e hora (o foco declarado) — 🟡 **metade feita (0.68)**

Os três **campos de edição** saíram: `<dateedit>`, `<timeedit>` e
`<datetimeedit>`, como uma primitiva só com edição por seções (§2.5). Não
precisaram de habilitador nenhum — ver a correção na §3.

Sobra a metade **calendário**: `Calendar` (a grade de mês, que precisa de estado
de navegação por instância e de saber em que dia da semana o mês começa — é ela
que puxa a decisão `chrono` vs. `time` da §4) e, sobre ela, a variante
`calendarPopup` do `QDateEdit`, que espera o overlay ancorado genérico (§3, item
5). Mais o `DateRangePicker` e o `MonthYearPicker`, que vêm de graça depois do
`Calendar`.
