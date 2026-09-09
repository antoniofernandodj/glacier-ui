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

Última revisão da fila: **2026-09-09**, sobre a 0.98 (ondas 3 a 9 fechadas; **a
10 retomada e feita PARCIALMENTE**: `FontSelect`, `FontDialog`, `PlainTextEditor`
🟡→✅; o `TextBrowser` fica, pede uma primitiva `<markdown>`; e **a 11
completada pelo habilitador C** — série múltipla nos gráficos,
`AreaChart`/`Scatter` 🟡→✅. Sobra o `Dock`, que passa a ser a Onda 12 sozinho.
88,8% → **92,0%** (115/125). **Ondas 12 a 14 desenhadas em 2026-09-09**, ainda
não executadas. A ordenação por **função** — widgets que carregam lógica — que a
revisão anterior adotou levou a fila até o fim, e a **Onda 8** (o diálogo que
carrega markup) parecia ter fechado o último item do catálogo com o formato "um
habilitador, meia dúzia de widgets".

Parecia. A revisão de 2026-09-08 achou mais um, pelo erro de sempre — sete linhas
espalhadas por seis seções da tabela que são **um** mecanismo: o ponteiro
apertado ao longo do tempo (`Splitter`, `RangeSlider`, `Tumbler`, `SwipeView`,
`DelayButton`, `RubberBand`, `SizeGrip`). O motor já o escreve três vezes, em
três lugares que não conversam. Era a **Onda 9** (§6.2), com a subscription de
teclado de carona, e ela **saiu na 0.95**: levou o catálogo de 74% para
**82,4%**, fechou a navegação e a janela/barras inteiras, e gastou dois itens de
motor — `src/grip.rs` e `src/keys.rs`.

~~Depois dela o que sobra é de outro tipo, e é pouco: o registro de famílias de
fonte (duas linhas), a gerência de janela interna (`MdiArea`/`Dock` — um debate,
não um bloqueio de motor), o troco decorativo da §6.3 (sete) e seis linhas fora
de escopo por escrito. **O regime "um habilitador, meia dúzia de widgets"
acabou**, e desta vez a afirmação é verificável: o único `●` que sobra por
estado de verdade é `MdiArea`/`Dock`.~~

**A frase durou uma revisão** (2026-09-08, à tarde, ainda sobre a 0.95). A
primeira metade estava certa — o registro de famílias é mesmo um habilitador
pequeno, e é a **Onda 10**. A segunda estava errada pelo motivo de sempre: as
quatro linhas que sobravam fora do troco (`MdiArea`, `Dock`, `SplashScreen`,
`NotificationDot`) estão em três seções diferentes da tabela e **elas mesmas já
escrevem o nome do que falta**: duas dizem `stack` na coluna "Base iced", uma
terceira diz "pede `Stack` dentro do builtin" na nota, e a quarta é composição
do `<splitter>` que a Onda 9 acabou de entregar. É uma capacidade que o `iced`
tem, que o `widget.rs` usa **uma vez, hardcoded**, e que o markup nunca expôs. É
a **Onda 11**, e é a quarta vez que o gargalo real não estava na lista do §3.

~~As duas levam o catálogo de **82,4% para 85,6% e depois para 93,6%** — e a 11
esgota o catálogo tal como escrito: o que sobra depois dela são cinco linhas
justificadas como fora de escopo e três `🟡` que são `🟡` de propósito.~~ **A
Onda 10 foi pulada** (a pedido, direto para a 11) **e a 11 saiu com dois
cortes** — `Dock` e a série múltipla dos gráficos — que a própria seção da
onda já tinha escrito como risco antes de qualquer código. O catálogo foi de
**82,4% para 88,8%** (103 → 111/125): sete widgets e dois habilitadores de
motor (`<stack>`/`pin`, `grip::Alvo::Ponto`), a 17ª correção de nível
(`MdiArea`). O detalhe está em "O resultado real (0.96)", ao fim da Onda 11
na §6.2.

**A fila continua** (revisão de 2026-09-09): a **Onda 10** foi retomada e saiu
**PARCIALMENTE (0.98)** — `FontSelect`, `FontDialog` e o `PlainTextEditor` 🟡→✅,
sobre o registro de famílias `src/fonts.rs`; o `TextBrowser` ficou (pede uma
primitiva `<markdown>`). E a **Onda 11 foi completada** pelo **habilitador C** —
série múltipla nos gráficos (`series="[{name,points}]"`, legenda, ciclo de cores
do tema), que fecha o `AreaChart`/`Scatter` 🟡→✅. 88,8% → **92,0%** (115/125).
Sobra da 11 só o `Dock`, que vira a **Onda 12** sozinho. Depois dela, a **Onda
13** executa a frase condicional da Onda 7 sobre o `canvas` declarativo e emenda
o `QWhatsThis`, e a **Onda 14** — só se a promessa do documento mudar — entra em
wgpu e impressão (`Shader`/`Q3D`/`PrintDialog`). Levam o catálogo a **92,8%,
94,4% e 96,8%**; com o `TextBrowser`, a **97,6%** — o catálogo tal como escrito,
esgotado. Detalhe nas seções das ondas na §6.2, previsão na §5.

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
| QCommandLinkButton | `CommandLink` | Built | button+col | — | P2 | ✅ | título + descrição + seta, sobre o `<Button>` com filhos (a convergência de templates já aceitava mais de um). **Onda 11** |
| QDialogButtonBox | `ButtonBox` | Built | row+button | — | P1 | ✅ | `<buttonbox accept="Salvar" on_accept="salvar" reject="Cancelar" …/>`: os três **papéis** e a ordem por plataforma decididos no widget — em Rust, no `template()`, por `cfg!(target_os)`, com uma prop `order` para forçar. O destrutivo fica na ponta oposta em qualquer ordem, e o `<slot/>` põe o "Ajuda" à esquerda. Onda 4 (0.85) |
| (switch/QML Switch) | `Toggle`/`Toggler` | Prim | toggler | ◐ | P0 | ✅ | já existe |
| QML RoundButton | `RoundButton` | Built | button | — | P3 | ✅ | `<Button>` com `border_radius` total — a mesma conta do círculo de iniciais do `<Avatar>` (passar o LADO inteiro, não a metade). `color` ganhou um default inline: sem fundo, o `<Button>` do motor não aplica `border_radius` nenhum (só entra no `.style()` junto do fundo). **Onda 11** |
| QML DelayButton | `DelayButton` | **Prim** | button+canvas | ◐ | P3 | ✅ | anel de progresso ao segurar. **Onda 9** (0.95): a metade do arrasto em que o que anda é o tempo, não o pixel — a fração numa chave global (`__hold`), recalculada do relógio a cada tique, e o ticker ligado **só** enquanto o botão está apertado. Soltar antes do fim desiste, que é o ponto todo |

### 2.2 Entradas de texto

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QLineEdit | `TextInput` | Prim | text_input | ◐ | P0 | ✅ | já existe |
| QLineEdit (password) | `TextInput password` | Prim | text_input | ◐ | P1 | ✅ | flag `secure`/`password`/`seguro`/`senha` no `<TextInput>`, sobre o `.secure()` do iced |
| QLineEdit (mask/validator) | `MaskedInput` | **Prim** | text_input | ◐ | P2 | ✅ | `<maskedinput value="cpf" mask="cpf" />`. Guarda **cru** na chave, exibe mascarado — a mesma separação valor/`displayFormat` do `<dateedit>`. Gramática `#`/`A`/`*` + literais, com presets `cpf`/`cnpj`/`telefone`/`cep`/`placa`/`date`/`hora`/`cartao`; a dica default é a máscara com `_`. A reclassificação (de `Comp ●` para `Prim ◐`) se confirmou. Onda 4 (0.85) |
| QTextEdit (rich) | `TextEditor` | Prim | text_editor | ● | P1 | ✅ | multi-linha; rich text é limitado |
| QPlainTextEdit | `PlainTextEditor` | Prim | text_editor | ● | P1 | 🟡 | variante sem formatação. O 🟡 é a fonte: `font_for` conhece **duas** famílias (`widget.rs`), então "texto simples em mono declarável" não dá para escrever. Fecha na **Onda 10** |
| QTextBrowser | `TextBrowser` | Built | markdown/scrollable | — | P2 | ⬜ | render read-only + links. §6.3, de carona na **Onda 10** — o bloco de código dele é o primeiro consumidor do registro de famílias |
| QKeySequenceEdit | `ShortcutInput` | **Prim** | text_input | ◐ | P3 | ✅ | captura combinação de teclas. **Onda 9** (0.95): é o listener de teclado do habilitador B em modo de captura — a combinação vai numa chave nomeada, e qual campo captura é global (`__shortcut_cap`), como o `__timeedit`. É um **botão**, não um `<textinput>`: um campo de texto consumiria a tecla antes de o listener global a ver |
| QComboBox (editable) | `ComboEdit` | Prim | combo_box | ◐ | P1 | ✅ | `options`/`value`/`onChange`/`onSelect`/`placeholder` + `labelField`/`valueField` para listas de objetos (ver `examples/combo_edit`) |
| — (autocomplete) | `Autocomplete` | **Prim** | text_input+overlay | ◐ | P2 | ✅ | a mesma tag do `Completer` (§2.12), vista do lado do campo. Recorta a lista sem acento e sem caixa ("sao paulo" acha "São Paulo"), ▲▼ navegam, Enter aceita, Esc desiste — e as três teclas ganham do campo focado porque quem as recebe é o **overlay** (0.92) |

### 2.3 Entradas numéricas e de valor

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QSpinBox | `SpinBox` | Built | text_input+button | ◐ | P1 | ✅ | campo + degraus, `min`/`max`/`step`, `layout="stacked"` (as setinhas ▴▾ coladas no campo, o QSpinBox clássico) ou `"inline"` (`− campo +`, o SpinBox do Qt Quick); a aritmética roda no `update` em Rust — **reclassificado de `●`**: o número mora numa chave que o app nomeia (prop `value`) e a ação carrega essa chave, então N instâncias não colidem (ver `src/builtins/spin_box.rs`) |
| QDoubleSpinBox | `SpinBox decimals` | Built | text_input+button | ◐ | P1 | ✅ | prop `decimals` no `<spinbox>`. Sem ela as casas continuam saindo do `step`, como sempre — o que acertava por acidente e errava justamente em `step="1"` sobre um preço (`10`, não `10.00`). Onda 4 (0.85) |
| QSlider | `Slider` | Prim | slider / vertical_slider | ◐ | P1 | ✅ | `min`/`max`/`step`, `vertical`, mais `default` (duplo clique), `on_release` e `shift_step`. Casas decimais da saída vêm do `step` como escrito. `disabled` deixa inerte, sem esmaecer: o `slider::Status` do iced 0.14 não tem `Disabled` |
| QML RangeSlider | `RangeSlider` | **Prim** | canvas | ◐ | P2 | ✅ | dois cursores. **Onda 9** (0.95): duas chaves nomeadas, exatamente o que o `<daterangepicker range>` já faz com `start`/`end`; qual ponta está presa é o único estado, e mora no `Program::State`. Com as duas juntas o clique pega o **fim** — senão a faixa de largura zero, que é o estado inicial de um filtro, travaria |
| QDial | `Dial` | **Prim** | canvas | ◐ | P2 | ✅ | knob rotativo: arrasta, clica no arco ou rola a roda. **Reclassificado de `●` para primitiva** — o valor mora na chave que o markup nomeia (`value="volume"`), como no `<slider>`; o único estado interno é o arrasto, e ele vive no `Program::State` do `iced`. É a terceira vez que essa marca cai (Spinner 0.66, Rating 0.85). Onda 7 |
| QScrollBar | `ScrollBar` | Motor | scrollable | — | P2 | 🟡 | embutido no `scrollable`; expor avulso é raro |
| QProgressBar | `ProgressBar` | Prim | progress_bar | — | P1 | ✅ | `value`/`min`/`max`/`vertical`/`showValue`; `color` = preenchimento |
| QProgressBar (busy) | `Spinner`/`BusyIndicator` | Prim | fill_quad (sem canvas) | — | P1 | ✅ | indeterminado; fase de rotação no `tree::State` do widget — **não** exige estado por instância no contexto (reclassificado de `●`; ver `src/spinner.rs`) |
| QLCDNumber | `LcdNumber` | **Prim** | canvas | — | P3 | ✅ | dígitos de sete segmentos. O valor é lido como TEXTO e só vira número formatado quando parseia como tal — é o que deixa `12:34` de relógio passar inteiro; `.`/`,`/`:` ocupam menos que um dígito. Onda 7 |
| QML Tumbler | `Tumbler` | **Prim** | canvas | ◐ | P3 | ✅ | roleta de valores. **Onda 9** (0.95): o `<dial>` desenrolado numa linha — o **texto** escolhido na chave (não o índice: reordenar a coleção não move a escolha), o arrasto no `Program::State`. Vira dentro de si, como as seções do `<timeedit>` |
| QML Gauge / medidor | `Gauge` | **Prim** | canvas | — | P2 | ✅ | medidor de arco com faixas coloridas (`bands`, JSON no atributo ou nome de chave), agulha, unidade e legenda. Apresentacional: não escreve nada. `start`/`sweep` em graus fazem meio-arco. Onda 7 |
| — (nota por estrelas) | `Rating` | **Prim** | row+button | ◐ | P2 | ✅ | N estrelas numa chave nomeada, com pré-visualização no hover (chave global `__rating`) e `readonly` para listas. **Reclassificado de Built para Prim na construção**, por dois motivos independentes: repetição dirigida por um número (não por coleção) e o hover, que o markup não expõe. Onda 4 (0.85) |

### 2.4 Seleção, listas e árvores (model/view)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QComboBox | `Select` / `Combo` | Prim | pick_list / combo_box | ◐ | P0 | ✅ | ambos existem |
| QFontComboBox | `FontSelect` | Comp | combo_box | ● | P3 | ⬜ | lista fontes do sistema — o mesmo bloqueio do `FontDialog` (§2.10), e não é o combo: falta o **registro de famílias de fonte** no motor. Os dois saem juntos quando ele existir, e é a **Onda 10** (§6.2) |
| QListWidget | `ListView` | **Built** | scrollable+ForEach | ◐ | P1 | ✅ | `<listview items="servicos" value="servico" selected="{servico}" />` — o `TabBar` na vertical, com scroll. `mode="multi"` guarda um **conjunto** numa chave só e é o primeiro consumidor do `contains` (0.84). `virtualize` repassado para listas longas. Onda 4 (0.85) |
| QListView (model) | `ListView bind` | Motor+Comp | scrollable | ◐ | P2 | ✅ | ligado a coleção do contexto — e a ligação **já existe** (`items="chave"`, a convenção do `<Menu>`/`<TabBar>`). Onda 6. **Contabilidade atrasada**, fechada na revisão da Onda 9: a Onda 6 escreveu que "nem existia como trabalho" e o ⬜ ficou por esquecimento |
| QTreeWidget/QTreeView | `TreeView` | **Prim** | column+recursão | ◐ | P2 | ✅ | ~~expandir/recolher = estado por nó~~ — é um **conjunto nomeado** (`abertos="raiz,raiz/src"`) + o `contains` da Onda 4. A identidade de um nó é o **caminho**, então um `id` repetido em ramos diferentes não colide (0.92) |
| QTableWidget/QTableView | `TableView` | **Prim** | column+row | ◐ | P2 | ✅ | cabeçalho, ordenação (numérica quando os dois lados são número), seleção simples e múltipla, colunas arrastáveis. Cabeçalho e corpo são a **mesma grade**, que é o que os mantém alinhados. Edição de célula fica para depois (0.92) |
| QHeaderView | `TableHeader` | **Prim** | row+button | ◐ | P2 | ✅ | a mesma primitiva do `TableView` **sem o corpo**, para quem monta as linhas à mão. O arrasto mora em `__colgrip`, na família do `__drag_key` (0.92) |
| QColumnView | `ColumnView` | **Prim** | row+ListView | ◐ | P3 | ✅ | navegação Miller (o Finder); quase de graça depois do `TreeView` — mesma coleção, mesma identidade por caminho (0.92) |
| QListWidgetItem etc. | (dados, não widget) | — | — | — | — | — | modelados como valores de contexto |
| QCompleter | `Completer` | **Prim** | overlay+ListView | ◐ | P2 | ✅ | popup de sugestões — a mesma tag do `Autocomplete` (§2.2), vista do lado da lista (0.92) |
| QML PageIndicator | `PageIndicator` | **Prim** | row+button | ◐ | P2 | ✅ | pontinhos de página — o irmão visual do `Pagination`, mesma chave. Saiu de carona no `SwipeView` da **Onda 9** (0.95), e é **a mesma primitiva**: `<pagination dots>`, uma tag a menos no motor, como `<sparkline>` é `<linechart>` sem moldura. A única diferença de comportamento é a base — ele conta do **zero**, porque marca o `currentIndex` de um `<swipeview>` |
| — (paginação) | `Pagination` | **Prim** | row+button | ◐ | P1 | ✅ | `« ‹ 1 … 4 [5] 6 … 20 › »`, com a janela andando e grudando nas pontas e as setas **inertes** no limite. **Reclassificado de Built para Prim na construção**: a janela de números é repetição dirigida por um número, e o `for-each` lê coleção. Onda 4 (0.85) |

### 2.5 Data e hora — **foco declarado do projeto**

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QCalendarWidget | `Calendar` | **Prim** | compõe | ◐ | P1 | ✅ | `<calendar value="dia" today="{hoje}" />` — grade 7×6, navegação ‹ ›, escada de drill-up (dia → mês → ano) no clique do título, `min`/`max` deixando os dias de fora inertes, `first_day` girando o cabeçalho da semana e rótulos de mês/dia por prop (pt-BR embutido). **A previsão se confirmou**: nenhum habilitador de motor, nenhum `Grid`, nenhuma crate de data — `days_from_civil` são oito linhas ao lado do `dias_no_mes`. Onda 3 (0.84) |
| QDateEdit | `DateEdit` / `DatePicker` | Prim | compõe | ◐ | P1 | ✅ | edição por **seções** (ano/mês/dia), com o realce da paleta na seção ativa e ▴▾ agindo sobre ela. Calendário respeitado: 31/01 + 1 mês satura em 28 ou 29. `format="br"` troca só a exibição — a chave é sempre ISO. Teclado completo na 0.70 (▲▼ na seção, ←→ para trocar, dígitos com avanço automático). A variante `calendarPopup` — a grade de mês ancorada ao campo — fechou na 0.92, com o overlay ancorado da Onda 5 |
| QTimeEdit | `TimeEdit` / `TimePicker` | Prim | compõe | ◐ | P1 | ✅ | as mesmas seções, para hora/minuto\[/segundo\]. Cada seção vira **dentro de si** (o `wrapping` do `QAbstractSpinBox`): mexer no minuto não empurra a hora |
| QDateTimeEdit | `DateTimeEdit` | Prim | compõe | ◐ | P1 | ✅ | as duas famílias de seção no mesmo campo. É **a mesma primitiva** dos dois acima — a tag só decide quais seções aparecem |
| — (range) | `DateRangePicker` | **Prim** | compõe | ◐ | P2 | ✅ | intervalo início→fim: **a mesma primitiva** com `range`, duas chaves (`start`/`end`) e `months="2"` desenhando as duas grades lado a lado. A faixa provisória entre o início e o cursor sai de uma chave global (`__cal_hover`), rastreada só enquanto há uma ponta aberta. Onda 3 (0.84) |
| — (mês/ano) | `MonthYearPicker` | **Prim** | compõe | ◐ | **P2** | ✅ | seleção só de mês/ano: **a mesma primitiva** em `mode="month"` — a tela de drill-up que o `QCalendarWidget` abre ao clicar no título, promovida a tag. Grava `YYYY-MM`; `mode="year"` grava `YYYY`. Custo marginal medido: **zero linhas de render próprias** — os dois níveis já existiam para a escada. Onda 3 (0.84) |

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
> | a decisão `chrono` vs. `time` | fechada na 0.72 (§4). Dia da semana é `days_from_civil`, oito linhas ao lado do `dias_no_mes` que o `Instante` já tem |
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
>
> **Fechado na 0.84**, e as três previsões acima se confirmaram uma a uma: a
> primitiva é um `NodeType::Calendar` só, o mês visível mora em `__cal_<chave>`
> sem o app configurar nada, e o `days_from_civil` que faltava tem oito linhas.
> A §2.5 fecha em **6 de 6**. O que sobra não é uma linha da tabela: é a
> variante `calendarPopup` do `<dateedit>` — uma *composição* das duas
> primitivas que já existem, esperando o overlay ancorado da Onda 5.
>
> **E ela fechou na 0.92**, com esse overlay: `calendarPopup="true"` põe um
> botão 📅 ao lado das setas, e ele abre a MESMA grade do `<calendar>` ancorada
> ao campo. Escolher um dia grava e fecha no mesmo passo — fosse uma segunda
> mensagem, o painel ficaria um quadro no ar depois do clique.

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
| — (chip removível) | `Chip` | Built | row+button | — | P2 | ✅ | `<Badge>` com um "×" que dispara `on_remove` — sem `on_remove`, o "×" nem desenha. **Onda 11** |
| — (separador) | `Divider` / `Rule` | Prim | rule | — | P0 | ✅ | `Rule` existe |
| QFrame | `Frame` | Built | container | — | P2 | ✅ | três formas: `box` (contorno), `filled` (contraste, o `QFrame::Panel`) e `none`. Sem `Raised`/`Sunken`: o `UiNode` não tem campo de sombra |
| — (skeleton) | `Skeleton` | Built | container | — | P2 | ✅ | um `<Container>` cinza do tamanho declarado — de propósito sem a pulsação animada que alguns kits desenham (pediria um tique de relógio por instância para uma economia que não se paga). **Onda 11** |
| — (QR) | `QrCode` | Prim | qr_code | — | P3 | ✅ | `qr_code` do iced atrás da feature `qr_code`; o `qr_code::Data` (não é `Clone` — guarda um `canvas::Cache`) é cacheado por conteúdo com um `Box::leak` por string distinta, o mesmo troco que o registro de fontes da Onda 10 preveria fazer. **Onda 11** |
| QGraphicsView | `Canvas` | Prim | canvas | ● | P3 | ⬜ | superfície de desenho livre. A Onda 7 usou o `canvas` do `iced` como **capacidade** (`src/canvas.rs`) e decidiu NÃO expor tag: um callback imperativo o `.gv` não lê, o `.gss` não estiliza e o Luau não alcança. Se sair, sai como vocabulário declarativo (`<path>`, `<arc>`, `<circle>`) |
| QOpenGLWidget | `Shader` | Prim | shader | ● | P3 | ⬜ | iced `shader` (wgpu) |
| — (toast) | `Toast` | Motor | stack | — | P1 | ✅ | já existe (`toasts.rs`) |

### 2.7 Containers e agrupadores

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QWidget/QFrame | `Container` | Prim | container | — | P0 | ✅ | já existe |
| QGroupBox | `GroupBox` | Built | container+text | — | P1 | ✅ | moldura com título + `flat="true"` (o `QGroupBox::flat`), e ações no cabeçalho por `<slot name="actions"/>` — onde vai o `<checkbox>` que faz o papel do `setCheckable` |
| QScrollArea | `Scrollable` / `rolagem` | Prim | scrollable | — | P0 | ✅ | já existe |
| QSplitter | `Splitter` / `PaneGrid` | **Prim** | row/column | ◐ | P2 | ✅ | painéis redimensionáveis. **Onda 9** (0.95): é a alça de coluna do `<tableheader>` aplicada a um container — mesmo formato de trilhas do `columns` do `<grid>`, mesmo `grip::Alvo::Trilha`, mesmo código. Aninha nos dois eixos, e duas instâncias na mesma tela nomeiam duas chaves |
| QToolBox | `ToolBox` | **Built** | column+button | ◐ | P2 | ✅ | `<toolbox>` + `<toolboxitem title="…" value="secao" open="{secao}" id="…">`: **uma** aberta por vez, e clicar na aberta a fecha. Nunca esteve bloqueado, e nem precisou do `contains`. Onda 4 (0.85); abre/fecha **animado** pelo `<Reveal>` (0.90) |
| — (accordion) | `Accordion` | **Built** | column+button | ◐ | P1 | ✅ | `<accordion>` + `<accordionitem …>`: **várias** abertas, num conjunto numa chave só (`abertas="rede,disco"`). ~~precisa estado por instância~~ — precisava do `contains` (0.84), e é o consumidor que o justificou. Uma tag por seção porque o **conteúdo** de cada uma é diferente, e conteúdo é de quem escreve a tela (`<slot/>`, 0.65) — a mesma forma do `QToolBox::addItem`. Onda 4 (0.85); abre/fecha **animado** pelo `<Reveal>` (0.90) |
| QMdiArea/QMdiSubWindow | `MdiArea` | **Prim** | stack | ◐ | P3 | ✅ | janelas internas: `<mdiarea>` + `<mdisubwindow title="…" x="…" y="…" w="…" h="…">`, uma tag por janela — a mesma razão do `<accordion>` (o CONTEÚDO de cada uma é diferente). **A previsão da §6.2 errou um detalhe**: não é uma coleção (`items=`) — os filhos são markup ESTÁTICO, como o `<accordionitem>` — e por isso não há reordenação Z por clique; a pilha desenha na ordem do markup. Mover (barra de título) e redimensionar (canto) são o MESMO `grip::Alvo::Ponto`, só com limites diferentes — a 17ª correção de nível: o `●` nunca valeu, quatro chaves por janela é a mesma forma que o `<rangeslider>` usa para duas. **Onda 11** (0.96) |
| QDockWidget | `Dock` | Comp | pane_grid → splitter+stack | ● | P3 | ⬜ | painéis acopláveis. **A Onda 11 tentou e cortou** (0.96): o risco que a própria onda tinha escrito por antecipação se confirmou — acoplado→flutuante é uma MUDANÇA DE PAI no meio de um arrasto, e o `grip.rs` foi escrito para reescrever VALORES (a chave que uma coisa já é), não para trocar a ESTRUTURA da árvore enquanto o dedo está apertado. Não é um bloqueio de motor novo: é decisão de design que esta rodada não tomou. Fica para uma leva futura, isolado do resto — que saiu inteiro |
| QSpacerItem | `Space` | Prim | space | — | P1 | ✅ | sem `width`/`height` é `Fill` nos dois eixos (o espaçador flexível); com eles, vão fixo. Duplicado na §2.11 por ser layout **e** container |

### 2.8 Navegação (abas, wizard, stacks)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QTabWidget/QTabBar | `TabBar` / `Tabs` | Built | row+button | ◐ | P1 | ✅ | duas tags: a **barra** sozinha (`<tabbar>`, 0.65) e a barra **mais a página** (`<tabs>`, 0.92). O que faltava era o **nome dinâmico de slot** (`<slot name="{aba}"/>`), uma linha no `eval` — não o estado por instância. Com ele, `addTab(widget, "Geral")` do Qt vira `<template slot="geral">` e a tela deixa de repetir a lista de abas duas vezes |
| QStackedWidget | `Stack`/`StackView` | **Built** | slot nomeado | ◐ | P1 | ✅ | `<stackview active="{passo}">` — é `<tabs>` **sem a barra**, o mesmo nome dinâmico de slot da 0.92. A página é escolhida por **nome**, não por posição numa escada de `se`. Onda 8 (0.94) |
| QWizard/QWizardPage | `Wizard` | **Built + Prim** | slot + WizardNav | ◐ | P2 | ✅ | `<wizard steps="…" titles="…" valid="{…}" on_finish="…">`. Saiu **em dois**, e é a descoberta da onda: slots pedem builtin, contas pedem primitiva. O composto é builtin; a aritmética (voltar inerte, finalizar no fim, travar sem validar, saturar nas pontas) é a primitiva `<wizardnav>` (`src/wizard.rs`). Onda 8 (0.94) |
| QML SwipeView | `SwipeView` | **Prim** | stack | ◐ | P3 | ✅ | páginas deslizáveis. **Onda 9** (0.95): `<stackview>` (0.94) escolhe a página por nome de slot; este escolhe por **posição**, porque é a posição que um arrasto move (o `currentIndex` do QML). O conteúdo segue o dedo, e satura nas pontas |
| QML Drawer | `Drawer` | **Built** | reveal+slot | ◐ | P2 | ✅ | painel lateral deslizante — `<slot/>` + chave nomeada + `axis="x"` no `<reveal>` (o motor já animava altura desde a 0.90). Ele **empurra**, não cobre: quem cobre é um `<popover>` colado na borda (0.92) |
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
| QSizeGrip | `SizeGrip` | **Built** | container+rule | — | P3 | ✅ | canto de redimensionamento. ~~**Onda 9**: o mesmo gesto do arrasto, com a janela no lugar da chave~~ — **não consumiu o arrasto**: `window:resize:se` já era ação da titlebar custom e `cursor="se"` já era atributo universal, então ele é um **builtin** de vinte linhas (`src/builtins/size_grip.rs`). A 15ª correção de nível deste catálogo (0.95) |

### 2.10 Diálogos (módulo `dialogs.rs`)

| Qt | Tag / API glacier-ui | Nível | Base | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QDialog (próprio) | `Dialog` / `<dialog>` | **Motor + tag** | stack+tela | ◐ | P1 | ✅ | a classe-base que o catálogo nunca listou: `<dialog name="…">` no `<resources>`, corpo em markup, aberto por `dialog:nome` e fechado por `dialog:close`. O corpo vira um template comum sob o mesmo nome, e é por isso que `render(nome)` o monta sem saber que é um diálogo. Onda 8 (0.94) |
| QMessageBox (info) | `DialogSpec::information` | Diál | stack | — | P0 | ✅ | existe |
| QMessageBox (warning) | `DialogSpec::warning` | Diál | stack | — | P0 | ✅ | existe |
| QMessageBox (critical) | `DialogSpec::error` | Diál | stack | — | P0 | ✅ | existe |
| QMessageBox (question) | `DialogSpec::question` | Diál | stack | — | P0 | ✅ | existe |
| — (confirm) | `DialogSpec::confirm` | Diál | stack | — | P0 | ✅ | existe |
| QInputDialog | `InputDialog` | Diál | stack+TextInput | ◐ | P1 | ✅ | `prompt{ kind = "text"|"int"|"double"|"item" }` — as quatro variantes estáticas do Qt são **um** diálogo com corpos diferentes. **Reclassificado de `●`**: o diálogo é singleton no motor (`dialog: Option<DialogSpec>`), então nunca há segunda instância com que colidir. Onda 8 (0.94) |
| QProgressDialog | `ProgressDialog` | Diál | stack+ProgressBar | ◐ | P1 | ✅ | `progress{}`/`progress_set()`/`progress_close()` — o único da família que **não suspende** (ele acompanha um trabalho em curso) e o único atualizado enquanto aberto, e por isso o progresso mora numa chave e não no `DialogSpec`. Sem `value` nasce indeterminado, com o `<spinner>` da 0.66. Onda 8 (0.94) |
| QFileDialog (abrir arquivo) | `FileDialog::open` | Diál | **`rfd`** (nativo do SO) | — | P1 | ✅ | `src/file_dialog.rs`; Luau `open_file()`/`open_files()`, suspensivo como `confirm()`/`fetch()` (ver `examples/file_dialog`) |
| QFileDialog (salvar) | `FileDialog::save` | Diál | `rfd` | — | P1 | ✅ | Luau `save_file()` |
| QFileDialog (diretório) | `FileDialog::directory` | Diál | `rfd` | — | P1 | ✅ | Luau `pick_folder()` |
| QColorDialog | `ColorDialog` | Diál | stack+canvas | ◐ | P2 | ✅ | `pick_color{}` — anel de matiz (180 setores) mais o quadrado saturação×valor, sobre o `src/canvas.rs` da Onda 7. É um `prompt{}` cujo campo é uma roda: mesma porta, mesmo retorno. A primitiva `<colorwheel>` também vale avulsa. Onda 8 (0.94) |
| QFontDialog | `FontDialog` | Diál | stack+lista | ● | P3 | ⬜ | escolher fonte/tamanho. **Onda 10** (§6.2), sobre o corpo em markup da Onda 8. **Não é um item de diálogo**: o motor conhece duas fontes (`font_for`, `widget.rs`), não tem registro de famílias e o `iced` não enumera as do SO. Espera um item de **Motor** (famílias de fonte), com quem sai junto do `FontSelect` da §2.4 — ver Onda 8, "fica de fora" |
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
| QGridLayout | `Grid` | **Prim** | — | P1 | ✅ | grade linhas×colunas, com a largura de uma coluna medida a partir de **todas** as células dela. `columns="3"` (medidas) ou `columns="140 fill 80"` (trilhas). Sem `colspan` — ver `src/grid.rs` (0.92) |
| QFormLayout | `Form` / `formulario` | Prim | — | P1 | ✅ | `Form` existe |
| QStackedLayout | `se`/`senao` + Stack | Motor | ◐ | P1 | 🟡 | condicional existe |
| QSpacerItem | `Space` | Prim | — | P1 | ✅ | ver §2.7 |
| (flow layout) | `Flow`/`Wrap` | **Prim** | row+quebra | — | P2 | ✅ | quebra automática de linha. **Não** saiu da medição da Onda 6: o `Row::wrap()` do próprio `iced` já fazia isso, e `<flow>` são três linhas no `widget.rs` (0.92) |

### 2.12 Overlays, dicas e utilitários

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QToolTip | `tooltip=` | Prim | tooltip | — | P1 | ✅ | **atributo universal**, não tag: `tooltip`/`title`/`dica` em *qualquer* nó, com `tooltip_position` |
| QML ToolTip | (idem) | Prim | tooltip | — | P1 | ✅ | mesmo atributo |
| QWhatsThis | — | — | — | — | P3 | ⬜ | ajuda contextual (raro) |
| QCompleter | `Completer` | **Prim** | text_input+overlay | ◐ | P2 | ✅ | sugestões enquanto digita, com ↑↓/Enter/Esc (0.92) |
| QML Popup | `Popup` | **Prim** | overlay | ◐ | P2 | ✅ | genérico, centrado na janela — a mesma primitiva do `Popover` sem âncora (0.92) |
| — (menu popover) | `Popover` | **Prim** | overlay | ◐ | P2 | ✅ | conteúdo flutuante **ancorado ao layout do gatilho** (não ao cursor), medido antes de posicionado — vira para o outro lado quando não cabe. Abre e fecha sozinho (0.92) |
| QSplashScreen | `SplashScreen` | **Built** | stack | — | P3 | ✅ | cobre o `<slot/>` principal com um `<slot name="splash">` enquanto `show` for verdadeiro. **A previsão da §6.2 errou o mecanismo**: não anima com `<reveal>` — um `<reveal>` interpola a ALTURA NATURAL do filho, e o painel precisa ser `height="fill"` para cobrir a tela, as duas coisas juntas são a armadilha do `Length::Fill` do `PRIMITIVAS.md`. Aparece/some sem transição, como o `QSplashScreen` do próprio Qt. **Onda 11** (0.96) |
| QRubberBand | `RubberBand` | **Prim** | canvas | ◐ | P3 | ✅ | retângulo de seleção. **Onda 9** (0.95): o retângulo é uma chave global (`__band`, um por tela, como o `__cal_hover`) e o que ele seleciona é o conjunto nomeado do `<listview mode="multi">`. Ele **desenha os alvos** que conhece, e não só a faixa — o motor não tem `<stack>` no markup, e a geometria ele já tinha |
| QShortcut/QAction | `Shortcut`/`Action` | Motor+tag | subscription | — | P2 | ✅ | atalhos globais de teclado. **Onda 9, habilitador B** (0.95): um sexto `listen_with` na lista de cinco que o daemon já registrava, lendo `<shortcut key="ctrl+s" on_press="salvar"/>` da árvore **avaliada** — e é por isso que ele vai no layout, não no `<resources>`. Sem modificador não rouba a tecla de um campo focado; com, atravessa |
| QScroller | (no scrollable) | Motor | scrollable | — | P3 | 🟡 | rolagem por gesto |
| — (badge de notificação) | `NotificationDot` | Built | container | — | P2 | ✅ | `<stack>` (a primeira camada é o `<slot/>`, o ícone) + `anchor="top-right"` (a segunda, o pontinho) — sem `pin`, porque um canto não precisa saber o tamanho do ícone. Primeiro consumidor do habilitador A. **Onda 11** (0.96) |

### 2.13 Gráficos e visualização (Qt Charts / DataVisualization)

| Qt | Tag glacier-ui | Nível | Base iced | Estado? | Prio | Status | Notas |
|---|---|---|---|---|---|---|---|
| QChartView (linha) | `LineChart` | **Prim** | canvas | — | P2 | ✅ | eixos com escala 1·2·5, grade, área e pontos. `min`/`max` vazios = automático; escritos, fixam a escala. Onda 7 |
| QChartView (barra) | `BarChart` | **Prim** | canvas | — | P2 | ✅ | base sempre no ZERO quando `min` não é declarado; `colorful` dá uma cor por categoria. Onda 7 |
| QChartView (pizza) | `PieChart` | **Prim** | canvas | — | P2 | ✅ | setores com percentual; `<donut>` é a mesma tag com o buraco aberto. Onda 7 |
| QChartView (área/scatter) | `AreaChart`/`Scatter` | **Prim** | canvas | — | P3 | ✅ | `area="true"` e `points="true"` no `<linechart>`, e desde a 0.98 **também com série múltipla**: `series="chave"` guarda `[{name, points, color?}]`, a `Moldura`/`escala` não mudam, `limites_multi` concatena os pontos para a faixa, `cor_ciclica` (do `<piechart>`) dá cor a quem não declarou e a legenda sai no canto. Habilitador C da Onda 11, entregue ao completá-la — ver `src/charts.rs::SerieNomeada` |
| — (sparkline) | `Sparkline` | **Prim** | canvas | — | P2 | ✅ | é `<linechart axes="false">` — a mesma primitiva, outros defaults. Onda 7 |
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
10. ~~**Virtualizar a lista**~~ ✅ **feito na 0.77** — `virtualize="<altura>"`
    numa coluna dentro de um `<scrollable>` monta só os filhos visíveis, com
    vãos do tamanho exato nas pontas. O motor passou a plumbar o `on_scroll`
    (só quando há o que virtualizar) e a altura da linha é **declarada**, não
    medida — a troca do `uniformItemSizes` do `QListView`. O render de uma lista
    de 300 cartões caiu de 1,81 ms para 47 µs por quadro, e o custo virou
    constante. Ver `PRIMITIVAS.md`.
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
  oito linhas —, mas este documento dizia que ele já estava "em uso nos dois
  lados", e **não estava**. Existia só no Luau (`date.weekday`, em
  `src/luau/prelude.luau`); o lado Rust tinha o `dias_no_mes` e mais nada. As
  oito linhas entraram junto do `Calendar` na 0.84, ao lado dele em
  `src/widget.rs`, com teste contra 1970, 1900 e 2000 — errar a regra do século
  desloca a grade inteira em silêncio, que é o pior modo de falha possível para
  um calendário.
- ~~**Gráficos**: `canvas` na mão vs. integrar `plotters`.~~ — **decidida na
  Onda 7 (0.93): na mão**, sobre o `canvas` do `iced`, com a caixa de
  ferramentas em `src/canvas.rs` (arcos, séries, escala 1·2·5, sete segmentos).
  Três razões, em ordem de peso: a **cor** (o `plotters` traz o sistema de
  estilo dele, e um gráfico que ignora o `theme.json` é um retângulo
  estrangeiro no meio do app), a **manutenção** (o `plotters-iced` oficial
  parou no `iced 0.13`; para o 0.14 só existe um fork de comunidade, e a §2.13
  inteira ficaria atrás dele a cada bump) e o **tamanho** (este crate já
  compila `wgpu`, `naga`, Luau e os codecs estaticamente). O que o `plotters`
  traria de graça — eixos, escala e rótulos — coube em duzentas linhas que
  servem os quatro gráficos.
- **Convenção de nomes**: manter aliases PT-BR (`botao`, `seletor`, `rolagem`…)
  para todo widget novo, ou só para o núcleo? (hoje o núcleo tem os dois.)
- **Rich text no `TextBrowser`/`QLabel`**: quanto do HTML/markdown do Qt vale
  reproduzir sobre o `markdown`/`rich` do iced.

---

## 5. Fases de construção (síntese priorizada)

Corte transversal da tabela por prioridade, na ordem que maximiza valor:

**Fase A — fechar o núcleo primitivo (P0/P1 sobre iced direto)**
~~`Radio`~~ ✅ · ~~`Slider`~~ ✅ · ~~`ProgressBar` (formalizar)~~ ✅ ·
~~`Tooltip`~~ ✅ · ~~`Space`~~ ✅ · ~~`Grid`~~ ✅ (0.92) · ~~`password`/`secure`
no `TextInput`~~ ✅ · `QrCode`. Sobra **um**: o `QrCode` (nativo do iced,
barato), de carona na **Onda 11**, que esvazia a §6.3. O `Grid` — "o caro, porque o iced não tem grade" — saiu na Onda 6, e o
caro nele não era a grade: era a **medição de coluna** (`src/grid.rs`).

**Fase B — destravar estado por instância (Motor P0)**
Sem markup novo; habilita a fase C inteira.

**Fase C — widgets compostos comuns (P1, ~~dependem de estado~~ dependem do
padrão da chave nomeada) — ✅ fechada (0.92)**
~~`Tabs`~~ ✅ (a barra na 0.65, o empilhado de páginas na 0.92, com o nome
dinâmico de slot) ·
~~`Accordion`~~ ✅ · ~~`SpinBox`~~ ✅ · ~~`GroupBox`~~ ✅ ·
~~`ListView` (com seleção)~~ ✅ · ~~`ToolBox`~~ ✅ · ~~`Pagination`~~ ✅ ·
~~`Rating`~~ ✅ · ~~`ToolBar`/`StatusBar`~~ ✅ · ~~`Avatar`~~ ✅ ·
~~`Spinner`/`BusyIndicator`~~ ✅. Era a **Onda 4** da §6.2, e ela consumiu **um**
habilitador (o `contains`); o `Tabs` completo fechou na Onda 5, com outro.

**Fase D — data/hora (P1, foco declarado) — ✅ fechada (0.84)**
~~`Calendar` → `DatePicker` → `TimePicker` → `DateTimePicker`. Depende de estado
+ overlay ancorado + valor de data.~~ Não dependia de nenhum dos três. Os campos
de edição saíram na 0.68 (teclado na 0.70) e o `Calendar`/`MonthYearPicker`/
`DateRangePicker` na 0.84 — **uma primitiva com três tags** e zero
habilitadores, como a Onda 3 da §6.2 previu. A variante `calendarPopup` do
`QDateEdit` — uma composição das duas primitivas, não uma sétima linha — saiu na
0.92, com o overlay ancorado da Onda 5. **O foco declarado do projeto termina
ali.**

**Fase E — diálogos ricos (P1) — ✅ fechada (0.94), foi a Onda 8**
~~`FileDialog` (open/save/directory via `rfd`)~~ ✅ · ~~`Dialog` (o `<dialog>`
próprio)~~ ✅ · ~~`InputDialog`~~ ✅ · ~~`ProgressDialog`~~ ✅ ·
~~`ColorDialog`~~ ✅ (que a Fase H listava e era o mesmo trabalho).

A releitura se confirmou: os "diálogos ricos" não eram N trabalhos, eram **um**
— o `DialogSpec` ganhar um **corpo em markup** e um **retorno tipado**, e aí
cada diálogo da lista virou composição de widgets que já existiam. Sobram o
`FontDialog` (que espera um item de motor, não um diálogo) e o `PrintDialog`
(fora de escopo), os dois justificados na Onda 8.

**Fase F — overlays e menus (P2) — ✅ fechada (0.92)**
~~`Menu` · `ContextMenu` · `MenuBar`~~ ✅ (overlay próprio em `src/menu.rs`) →
~~`Popover` · `Popup` · `Completer` · o `calendarPopup` do `<dateedit>`~~ ✅. Era
a **Onda 5** da §6.2, junto do `Tabs` completo — que não é overlay, mas responde
a mesma pergunta (de quem é este conteúdo?).

O overlay **não** saiu de generalizar o `src/menu.rs`, como esta fase previa:
saiu como um `iced::advanced::{Widget, Overlay}` próprio (`src/anchored.rs`),
que é o caminho que o cabeçalho do `menu.rs` sempre documentou como o certo. O
`menu.rs` continua como está — reescrevê-lo agora seria trabalho sem
consumidor.

**Fase G — model/view pesado (P2) — ✅ fechada (0.92)**
~~`TableView` · `TableHeader` · `TreeView` · `ColumnView`, mais o `Grid` e o
`Flow`~~ ✅. Era a **Onda 6**, e a releitura que ela trouxe se confirmou: o item
caro não era o "binding a coleção" (que já existia, via `items="chave"`) e sim a
**medição de coluna** — que o `Grid` e o `TableView` compartilham, e que ninguém
tinha catalogado como item de motor.

Duas das seis não passaram por ela: o `Flow` é o `Row::wrap()` do próprio
`iced`, e o `TreeView` é recursão mais um conjunto nomeado. Fica anotado que a
edição de célula continua de fora — ela reusa o `<textinput>`, não a medição.

**Fase H — canvas e visualização (P2/P3)**
~~`Dial`~~ ✅ · ~~`Gauge`~~ ✅ · ~~`LineChart`/`BarChart`/`PieChart`~~ ✅ ·
~~`Sparkline`~~ ✅ (todos na Onda 7, 0.93). Sobra a **série múltipla** dos
gráficos; o `ColorDialog` saiu desta fase e foi para a E, porque o que faltava
nele nunca foi o canvas — era o diálogo saber carregar conteúdo.

**Fase I — nicho/avançado (P3)**
~~`MdiArea`~~ ✅ (**Onda 11**, 0.96 — e não pelo motivo que esta fase supunha,
ver §6.2: é uma coleção de janelas ESTÁTICAS, não estado por instância) ·
`Dock` (tentado na Onda 11, cortado — mudança de pai no meio de um arrasto) ·
~~`SwipeView`~~ ✅ (Onda 9) · ~~`Drawer`~~ ✅ (0.92, adiantado pela Onda 5 —
ele não dependia de nada desta fase) · ~~`SystemTray`~~ ✅ · `Shader`/3D ·
impressão. O `Wizard` saiu daqui para a **Onda 8**: no Qt ele é um `QDialog`, e
com o corpo em markup ele deixa de ser nicho e vira soma de peças prontas.

---

### Resumo numérico

Contagem sobre as linhas que têm status, atualizada em 2026-09-07 (0.94, com a
Onda 8 fechada). Duas ressalvas: a §2.4 tem
uma linha (`QListWidgetItem`) que é dado, não widget, e fica de fora; e o
`Space` aparece duas vezes (§2.7 como container, §2.11 como layout), então o
total tem uma duplicata — 124 widgets distintos, não 125.

| Categoria | Widgets catalogados | ✅ prontos | 🟡 parciais | ⬜ a fazer |
|---|---|---|---|---|
| Botões e ações | 10 | **8** | 0 | 2 |
| Entradas de texto | 9 | **7** | 1 | 1 |
| Numéricas/valor | 12 | **11** | 1 | 0 |
| Seleção/listas/árvores | 11 | **10** | 0 | 1 |
| Data e hora | 6 | **6** | 0 | 0 |
| Displays/indicadores | 15 | 10 | 0 | 5 |
| Containers | 9 | **7** | 0 | 2 |
| Navegação | 6 | **6** | 0 | 0 |
| Janela/barras | 8 | **8** | 0 | 0 |
| Diálogos | 15 | **13** | 0 | 2 |
| Layouts | 7 | **6** | 1 | 0 |
| Overlays/utilitários | 11 | **7** | 1 | 3 |
| Gráficos | 6 | **4** | 1 | 1 |
| **Total** | **125** | **103** | **5** | **17** |

O motor entrega **~82%** do catálogo Qt de superfície (34% antes da onda 2, 39%
antes da onda 1, 43% antes dos campos de data/hora, 44% antes da onda 3, 47%
antes da onda 4, 53% antes das ondas 5 e 6, 64% antes da onda 7, 69% antes da
onda 8, 74% antes da onda 9). As ondas 5 a 9 somaram **trinta e seis** widgets e
consumiram **sete** itens de motor — o nome dinâmico de slot (uma linha no
`eval`), o overlay ancorado (`src/anchored.rs`), a medição de colunas
(`src/grid.rs`), o canvas como capacidade (`src/canvas.rs`), o corpo do diálogo
(um parâmetro e uma chamada), o arrasto (`src/grip.rs`) e o teclado
(`src/keys.rs`, um sexto `listen_with` numa lista de cinco).

Dos onze que a Onda 9 fechou, **um não custou código nenhum**: o `ListView bind`
já existia desde a Onda 6 e o ⬜ era contabilidade atrasada.

Onde os 17 se concentram, depois da Onda 9: **displays** (5) e **overlays** (3)
— oito dos 17, e sete deles estão na bandeja de troco da §6.3. ~~O que sobra fora do troco é
pequeno e disperso: `Splitter`, `MdiArea`, `Dock`, `RangeSlider`, `Tumbler`,
`SwipeView`, `Shader`/3D, os dois de fonte e os dois diálogos justificados.~~

**"Disperso" era a leitura errada da mesma lista** (revisão de 2026-09-08).
`Splitter`, `RangeSlider`, `Tumbler` e `SwipeView` estão em quatro seções
diferentes da tabela e são o **mesmo mecanismo** — com `DelayButton`,
`RubberBand` e o `SizeGrip`, sete linhas de um habilitador só. A tabela agrupa
por *onde o widget aparece na tela*, que é o eixo em que um mecanismo
compartilhado fica invisível; é a terceira vez que isso acontece (a medição da
Onda 6 e o corpo do diálogo da Onda 8 estavam escondidos do mesmo jeito). Ver a
**Onda 9** na §6.2.

**A Onda 9, prevista e medida** (a previsão está preservada porque ela acertou):

| | ✅ | % do catálogo | previsto |
|---|---|---|---|
| antes (0.94, Onda 8 fechada) | 92 | 74% | — |
| depois da Onda 9 (9 widgets + `PageIndicator` de carona) | 102 | 81,6% | 102 / ~82% |
| mais a linha de contabilidade do `ListView bind` | **103** | **82,4%** | 103 / ~82% |

Sobram **17** ⬜: o troco da §6.3 (7 — o `PageIndicator` saiu de lá), as duas de
fonte, as duas de janela interna (`MdiArea`/`Dock`) e seis justificadas como fora
de escopo (`Canvas` como tag, `Shader`, `Q3D`, `PrintDialog`, `QWhatsThis`,
`SplashScreen`).

**As Ondas 10 e 11, previstas** (desenhadas em 2026-09-08; a previsão fica
escrita para ser conferida depois, como a da Onda 9):

| | ✅ | 🟡 | ⬜ | % do catálogo |
|---|---|---|---|---|
| hoje (0.95, Onda 9 fechada) | 103 | 5 | 17 | 82,4% |
| depois da **Onda 10** (fonte: `FontSelect`, `FontDialog`, `TextBrowser`, `PlainTextEditor` 🟡→✅) | 107 | 4 | 14 | **85,6%** |
| depois da **Onda 11** (`stack`/`pin`: 4 · o troco da §6.3: 5 · série múltipla 🟡→✅: 1) | **117** | 3 | 5 | **93,6%** |

Os **5 ⬜** que sobram são os cinco justificados por escrito como fora de escopo
(`Canvas` como tag, `Shader`, `Q3D`, `PrintDialog`, `QWhatsThis`) e os **3 🟡**
são os três que estão 🟡 de propósito (`ScrollBar` embutido no `scrollable`, o
`se`/`senao` no lugar do `QStackedLayout`, o `QScroller`). Ou seja: **93,6% é o
catálogo tal como escrito, esgotado** — depois da Onda 11 não sobra linha que
alguém tenha decidido fazer e não tenha feito.

E o teto, se um dia se quiser: dos cinco, dois não são "outro projeto" — o
`Canvas` como vocabulário declarativo (`<path>`/`<arc>`/`<circle>` sobre o
`src/canvas.rs` da Onda 7) e o `QWhatsThis` (o `tooltip=` num modo pegajoso).
Levariam a **119/125, 95,2%**. `Shader`, `Q3D` e `PrintDialog` são wgpu e
impressão, que este documento nunca prometeu.

**O que de fato aconteceu (0.96)**: a Onda 10 foi pulada por pedido explícito
(direto para a 11), e a Onda 11 saiu com dois cortes que a própria seção da
onda já cravava como risco antes de escrever uma linha — `Dock` (mudança de
pai no meio de um arrasto, fora do que `grip.rs` sabe fazer) e a série
múltipla dos gráficos (não por dificuldade, por ordem de prioridade dentro do
tempo da rodada).

| | ✅ | 🟡 | ⬜ | % do catálogo |
|---|---|---|---|---|
| hoje (0.95, Onda 9 fechada, **Onda 10 pulada**) | 103 | 5 | 17 | 82,4% |
| depois da **Onda 11 de fato entregue** (`MdiArea`, `NotificationDot`, `SplashScreen`, `QrCode`, `Chip`, `Skeleton`, `CommandLink`, `RoundButton`) | **111** | 5 | 9 | **88,8%** |

Os **9 ⬜** que sobram: `Dock` (cortado nesta onda), `FontSelect`/`FontDialog`/
`TextBrowser` (Onda 10, não executada) e os cinco fora de escopo por escrito
(`Canvas` como tag, `Shader`, `Q3D`, `PrintDialog`, `QWhatsThis`). Os **5 🟡**
não mudaram — a série múltipla continua 🟡 (`AreaChart`/`Scatter`), e os outros
quatro são 🟡 de propósito, como sempre foram.

**As Ondas 12 a 14, previstas** (desenhadas em 2026-09-09, sobre a 0.96, com a
Onda 10 ainda pulada; a previsão fica escrita para ser conferida, como as das
ondas 9 a 11):

| | ✅ | 🟡 | ⬜ | % do catálogo |
|---|---|---|---|---|
| Onda 11 parcial, Onda 10 pulada (0.96) | 111 | 5 | 9 | 88,8% |
| **Onda 10 retomada (0.98)** — `FontSelect`, `FontDialog`, `PlainTextEditor` 🟡→✅ | 114 | 4 | 7 | 91,2% |
| **Onda 11 completa (0.98)** — habilitador C: `AreaChart`/`Scatter` 🟡→✅ | **115** | 3 | 7 | **92,0%** |
| depois da **Onda 12** (`Dock` — só ele; C já entrou) | 116 | 3 | 6 | **92,8%** |
| depois da **Onda 13** (`Canvas` declarativo + `QWhatsThis`) | 118 | 3 | 4 | **94,4%** |
| depois da **Onda 14** (`Shader`, `Q3D`, `PrintDialog` — pende decisão de escopo) | 121 | 3 | 1 | **96,8%** |
| — e com o `TextBrowser` (pede uma primitiva `<markdown>`) | 122 | 3 | 0 | **97,6%** |

Uma correção de nível nas ondas 10 e 12 (`FontSelect`, a 16ª; `Dock`, a 18ª;
nenhuma nas outras), **nenhum item novo do §3** — o registro de famílias já
estava listado, o habilitador C foi `charts.rs` autocontido, o D é decisão de
design sobre peças prontas, o vocabulário de formas é o `canvas` da Onda 7
aberto em nós, e a Onda 14 é `iced`/`rfd`. O único ⬜ que sobra depois da 14 é o
`TextBrowser` (que virou trabalho próprio — a primitiva `<markdown>`); os **3
🟡** finais (`ScrollBar`, `QStackedLayout`, `QScroller`) são `🟡` de propósito.
**97,6% é o catálogo esgotado.**

**Duas categorias fecharam na Onda 8**, e foi a primeira vez que duas fecharam
juntas: os diálogos (13/15, com os dois restantes justificados por escrito) e a
navegação (5/6, sobrando o `SwipeView`). O bloco de model/view, que era o maior atraso do catálogo
desde o começo, deixou de ser um: a §2.4 saiu de 3/11 para **8/11** e a §2.11
(layouts) fechou tudo o que tinha ⬜.

**A Onda 9 fechou duas sem nenhuma linha aberta** — a navegação (6/6, com o
`SwipeView`) e a janela/barras (8/8, com o `SizeGrip`) —, o que não tinha
acontecido com nenhuma categoria até aqui: as duas da Onda 8 fecharam com
ressalvas escritas. A §2.3 chegou a 11/12, e o que sobra nela é o `ScrollBar`
🟡, que está 🟡 de propósito ("embutido no `scrollable`; expor avulso é raro"). A §2.4 chegou a **10/11** e a
§2.12 (overlays), que era a segunda maior concentração de ⬜, saiu de 5/11 para
**7/11**.

A §2.5 saiu dessa lista de um jeito que vale registrar: ela era 0 de 6, passou a
3 de 6 na 0.68 e fechou em **6 de 6** na 0.84 — mas com uma ressalva escrita na
própria linha do `QDateEdit` ("falta a variante `calendarPopup`") desde a 0.68.
A 0.92 apagou essa frase, e é por isso que **o foco declarado do projeto termina
na Onda 5, não na Onda 3**.

Nem toda linha ⬜ está esperando o motor; algumas estão esperando alguém
perguntar em que nível elas deveriam estar — e este documento errou essa
pergunta **treze** vezes, sempre para o mesmo lado, o de superestimar o
bloqueio:

| # | linha | o que o documento dizia | o que era |
|---|---|---|---|
| 1–3 | `TimePicker`, `DateEdit`, `Calendar` | builtin bloqueado por estado | primitiva, sem bloqueio |
| 4 | `Accordion` | precisa de estado por instância | conjunto nomeado + `contains` |
| 5–6 | `Pagination`, `Rating` | builtin | primitiva — *repetição dirigida por número* |
| 7 | `QTabWidget` | precisa de estado por instância | precisava de **uma linha** no `eval` (nome de slot dinâmico) |
| 8 | `TreeView` | estado por nó | conjunto nomeado, o mesmo do `Accordion` |
| 9 | binding a coleção (§3) | "o maior investimento restante" | **já existia** (`items="chave"`) desde o `<menu>` |
| 10 | `Flow`/`Wrap` | sai da medição da Onda 6 | **o `iced` já tinha** (`Row::wrap()`) |
| 11–13 | `InputDialog`, `ProgressDialog`, `ColorDialog` | exigem estado por instância | o diálogo é **singleton** no motor (um campo, não um mapa), então nunca há segunda instância — o valor cabe numa chave nomeada |

As de número 9 e 10 são as mais instrutivas, porque são de tipos novos: a 9 é
uma capacidade que o próprio motor já tinha e ninguém releu; a 10 é uma que a
**biblioteca de baixo** já tinha. As duas só apareceram quando alguém foi
escrever o código.

As três últimas são de um quarto tipo, e é o mais barato de checar: o widget
não pode ter estado por instância porque **não pode existir duas vezes**. As dez
anteriores exigiram escrever o widget para descobrir que o estado era o valor;
esta se lê no tipo — `dialog: Option<DialogSpec>` é um campo, não um mapa.

Depois delas, todo `●` que sobra no catálogo é de widget que existe N vezes na
mesma tela (`Splitter`, `MdiArea`, `Dock`, `RangeSlider`, `Tumbler`), que é o
caso em que a marca sempre foi honesta.

E os três regimes de custo, agora todos exemplificados:

- **Nenhum habilitador** (ondas 1, 3, 4): widgets que o `iced` já sustentava, ou
  que só estavam classificados no nível errado.
- **Um habilitador que vira meia dúzia de widgets** (ondas 2, 5, 6): o
  `<slot/>`, o overlay ancorado, a medição de colunas. É o regime de maior
  alavancagem, e o único em que vale abrir uma rodada por um item de motor.
- **Um habilitador de meia tarde** (o `contains`, entre as ondas 3 e 4): cabe de
  carona na onda que o consome.

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

- **Onda 3** ✅ (0.84) — o calendário, e com ele o foco declarado do projeto.
- **Onda 4** ✅ (0.85) — os widgets que carregam lógica, pelo padrão do `SpinBox`.

As duas seguintes são o outro regime: **um item de motor que vira meia dúzia de
widgets**, como o `<slot/>` foi na Onda 2.

- **Onda 5** ✅ (0.92) — o conteúdo que sai da tela e entra no widget (abas com
  página, overlays ancorados).
- **Onda 6** ✅ (0.92) — a grade: uma medição de colunas, e os seis widgets que
  saem dela.

As quatro respondem a mesma pergunta, que é a lição repetida deste documento:
**em que nível este widget deveria estar, e ele está mesmo bloqueado?** Nove das
linhas abaixo estão marcadas `Comp ●` — "bloqueado por estado por instância" — e
nenhuma delas está.

**A fila está cumprida**, e mais duas ondas saíram no mesmo regime da 5 e da 6:

- **Onda 7** ✅ (0.93) — o canvas como capacidade, e os sete que saem dele.
- **Onda 8** ✅ (0.94) — o diálogo que carrega markup, e os seis que saem dele.

E uma nona, proposta e feita em 2026-09-08, no mesmo regime e pelo mesmo erro:

- **Onda 9** ✅ (0.95) — o arrasto como capacidade (o motor já o escrevia três
  vezes) e a subscription de teclado; sete widgets no primeiro, dois no segundo,
  e o catálogo passou de 74% para **82,4%**.

A Onda 8 fez a mesma pergunta na última categoria em que ela ainda cabia: a
§2.10 tinha cinco ⬜ que pareciam cinco trabalhos e eram **um** — o modal deixar
de ser uma tela paralela escrita em Rust. Três das cinco estavam marcadas `●`, e
nenhuma podia estar, porque o diálogo é singleton.

~~**Não há Onda 9 esboçada, e é de propósito.** O que sobra no catálogo não tem
mais o formato que estas seis ondas exploraram: são widgets isolados
(`Splitter`, `MdiArea`, `SwipeView`), um habilitador de motor sem consumidor
urgente (famílias de fonte) e o troco da §6.3.~~ **Errado, e pelo motivo de
sempre** (revisão de 2026-09-08): a frase lista três widgets como três assuntos,
e eles são **um** — o ponteiro preso ao longo do tempo. Com mais quatro que a
mesma capacidade sustenta, é a **Onda 9**, e ela é a última no regime "um
habilitador, meia dúzia de widgets". Ver abaixo.

O que continua verdadeiro é o resto do parágrafo: as famílias de fonte são um
habilitador de duas linhas e o troco da §6.3 não abre rodada — mas **"não abre
rodada" não é "não entra em nenhuma"**, e é assim que ele sai (revisão da tarde
de 2026-09-08, que desenhou mais duas):

- **Onda 10** 🟡 **EM ANDAMENTO (0.98)** — a fonte que o motor não sabe nomear:
  o **último** habilitador de motor que o catálogo tem escrito
  (`src/fonts.rs`), e quatro linhas em cima dele. Pulada a pedido na época da
  11; retomada depois. Saíram três — `FontSelect` (16ª correção de nível),
  `FontDialog` (`pick_font{}`), `PlainTextEditor` 🟡→✅; o `TextBrowser` fica
  (pede uma primitiva `<markdown>`). 88,8% → **91,2%**.
- **Onda 11** 🟡→ quase completa — o que fica por cima (`<stack>`/`pin`) saiu na
  0.96 com dois cortes; o menor, a **série múltipla** (habilitador C), voltou na
  **0.98** — `series="[{name,points,color?}]"`, legenda e ciclo de cores do
  tema, `AreaChart`/`Scatter` 🟡→✅, §2.13 a 5/6. Sobra só o `Dock`. 82,4% →
  88,8% → **92,0%**. Detalhe no fim da seção da onda, abaixo.
- **Onda 12** ⬜ — **só o `Dock`** agora (o habilitador C já entrou ao completar
  a 11). Uma decisão de design: `grip::Alvo::Zona` que comete um valor de modo
  na soltura, sem reparent no meio do gesto. 92,0% → **92,8%**.
- **Onda 13** ⬜ — o `canvas` declarativo (`<path>`/`<arc>`/`<circle>`), que é a
  frase condicional da Onda 7 executada, mais o `QWhatsThis` de carona. 92,8% →
  **94,4%**.
- **Onda 14** ⬜ **pendente de decisão de escopo** — `Shader`, `Q3D` e
  `PrintDialog`, as três linhas que o documento sempre marcou como "outro
  projeto". Só vira fila com um "sim". 94,4% → **96,8%** (**97,6%** com o
  `TextBrowser`).

---

#### Onda 3 — o calendário: uma primitiva, três tags (o foco declarado) — ✅ **FEITA (0.84)**

> **Como saiu.** Um `NodeType::Calendar` em `src/parser.rs`, um braço em
> `src/eval.rs`, um `render_calendar` em `src/widget.rs` e as oito linhas do
> `days_from_civil`. **Zero habilitadores de motor**, como previsto — e zero
> `Grid`. A forma proposta abaixo saiu quase intacta; as três diferenças estão
> anotadas no fim da seção. Exemplo: `cargo run --example onda3`.

A §2.5 estava 3 de 6 e as três que faltavam saíram **juntas**, do mesmo arquivo, pelo
mesmo caminho que os campos de edição já abriram: uma primitiva em
`src/widget.rs` + um `NodeType` em `src/parser.rs`, com as tags decidindo só
quais partes aparecem — exatamente como `<dateedit>` e `<timeedit>` são o mesmo
`NodeType::DateTimeEdit`.

Nenhum habilitador de motor. A demonstração de por que (e a autópsia do erro que
mantinha essas três linhas marcadas como bloqueadas) está no blockquote da §2.5.

| # | Widget | Nível | Tag | O que grava | Por que aqui |
|---|---|---|---|---|---|
| 1 ✅ | **`Calendar`** (`QCalendarWidget`) | Prim | `<calendar>` | `YYYY-MM-DD` | O coração do foco declarado, e a peça de que as outras duas saem por variação. Grade 7×6, navegação de mês, dia selecionado |
| 2 ✅ | **`MonthYearPicker`** | Prim | `<monthyearpicker>` | `YYYY-MM` | A **mesma primitiva** em `mode="month"`: a tela de drill-up que o `QCalendarWidget` abre ao clicar no título, promovida a tag. Custo marginal quase zero depois do 1 — daí ter subido de P3 para P2 |
| 3 ✅ | **`DateRangePicker`** | Prim | `<daterangepicker>` | duas chaves | A **mesma primitiva** com `range`: `start`/`end` em chaves separadas (não `"a/b"` numa só, para o `date.diff` do Luau ler as duas direto) e `months="2"` desenhando dois meses lado a lado, como todo seletor de reserva de hotel |

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

1. `days_from_civil` — oito linhas ao lado do `dias_no_mes` que o `Instante` já
   tem, para saber em que dia da semana o mês começa. **Não existia no lado
   Rust**, ao contrário do que a §4 afirmava (correção registrada lá): só no
   `prelude.luau`.
2. O laço da grade — `Column` de `Row`s de `button`, com as células do mês
   anterior/seguinte esmaecidas. Aqui é que o `Grid` (`QGridLayout`) deixa de
   ser pré-requisito: grade em Rust é um `for`. (Saíram **vazias e inertes**, não
   esmaecidas — ver as três diferenças no fim desta seção.)
3. O hover do intervalo — a faixa que se pinta entre `start` e o dia sob o
   cursor. Estado global de verdade, e legitimamente: só uma célula da tela
   inteira está sob o cursor por vez. Mesma família do `__timeedit`.

**O que fica de fora, com motivo:** a variante `calendarPopup` do `QDateEdit`
(o `<dateedit>` abrindo este calendário num popup ancorado ao campo) continua
esperando o overlay ancorado genérico — §3, item 5. É o único item da §2.5 que
segue bloqueado, e ele é uma *composição* dos dois widgets, não um terceiro.

Fecha a §2.5 em **6 de 6** e a Fase D do §5.

**As três diferenças entre o proposto e o construído** (0.84), porque o proposto
está escrito acima e vale saber onde ele foi corrigido pela realidade:

1. **O nível de drill-up viaja junto com o mês visível**, não numa chave
   separada: `__cal_<chave>` guarda `YYYY-MM|<nível>`. Subir a escada é
   navegação, não escolha, e separar as duas obrigaria o app a semear duas
   chaves quando ele dirige o mês por `month=`.
2. **O `onChange` do modo intervalo entrega as duas pontas numa string**
   (`"<início> <fim>"`, com o fim vazio no primeiro clique) — não porque a
   forma `"a/b"` tenha voltado, mas porque uma mensagem do motor carrega **um**
   valor. Quem grava as duas chaves separadas é o handler; o widget continua
   gravando as duas quando não há `onChange`, que é o caso comum.
3. **As células dos meses vizinhos são inertes**, não esmaecidas-e-clicáveis.
   Escolher um dia do mês seguinte teria de mover o mês visível *junto com* a
   escolha, e no modo que **delega** a escrita o widget não pode fazer as duas
   coisas numa mensagem só. As setas ‹ › cobrem o caso sem ambiguidade.

E dois achados de percurso, ambos do mesmo tipo — algo que o documento dizia
existir e não existia:

- `days_from_civil` **não estava** do lado Rust (só no `prelude.luau`), como a
  §4 afirmava. São oito linhas, e agora estão em `src/widget.rs`, com teste
  contra 1970, 1900 e 2000 — errar isso desloca a grade inteira em silêncio.
- O realce de **hoje** confirmou a §4 pela negativa: é prop, não relógio. O
  exemplo `onda3` é todo em Luau por causa dela — `date.today()` é a linha que
  fecha o buraco sem uma crate de data no motor.

---

#### Habilitador — `contains` no condicional (Motor, P1, o menor da lista) — ✅ **FEITO (0.84)**

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

Saiu na 0.84, junto da Onda 3, e a previsão de tamanho estava certa: mesmo ponto
do parser, mesmo ponto do eval, uma comparação invertida. Duas decisões que a
proposta não tinha: o **item** também interpola (`contains="{item.id}"`, que é a
forma que o `Accordion` usa dentro de um `for-each`), e os três separadores —
vírgula, ponto-e-vírgula e espaço — valem ao mesmo tempo, porque quem monta o
conjunto é código de app e um `concat` com vírgula e um `table.concat` com
espaço são igualmente naturais.

---

#### Onda 4 — os widgets que têm função — ✅ **FEITA (0.85)**

> **Como saiu.** Sete linhas do catálogo, e o achado da leva não está na
> contagem: **dois dos sete não eram builtins.** `Pagination` e `Rating`
> viraram primitivas, pela mesma causa — *repetição dirigida por um número, não
> por uma coleção*. O `for-each` do motor lê uma chave com um array; a janela
> `4 5 6` e as cinco estrelas não existem em array nenhum, são derivadas. E
> derivar é justamente o que um template não faz.
>
> É a quinta e a sexta aplicação da lição do `DateEdit`, e a primeira vez que a
> mesma causa aparece duas vezes seguidas — o que sugere que ela merece um
> nome. Fica registrada abaixo, depois da tabela. Exemplo:
> `cargo run --example onda4` — e `cargo run --example onda4_luau`, a mesma tela
> sem `impl Component`, que é a prova de que nenhum dos sete pede script.

Todos **builtins** (exceto o 5), todos pelo padrão do `SpinBox`: a chave é
nomeada pelo app, a ação carrega o nome, o `update` faz a conta. Nenhum precisa
de estado por instância; o único habilitador é o `contains` acima, e só para
dois deles.

Ordenados por quanto cada um abre de tela real:

| # | Widget | Nível | Prio | Por que aqui |
|---|---|---|---|---|
| 1 ✅ | **`Pagination`** | ~~Built~~ **Prim** | P1 | A função mais pura da lista: primeira/anterior/`n`/próxima/última, com o clamp e o cálculo da janela de números no `update`. Sem ele, toda lista longa de todo app é escrita à mão em Luau. Estava citado na §3 como "nunca esteve bloqueado" e **nem sequer tinha linha na tabela** — entrou na §2.4 nesta revisão |
| 2 ✅ | **`ListView` com seleção** | Built | P1 | Fecha o 🟡 mais antigo da §2.4. A coleção numa chave, o item escolhido noutra — é o `TabBar` com scroll. Seleção múltipla usa o conjunto nomeado (e o `contains`); a simples não precisa de nada. Casa com o 1 |
| 3 ✅ | **`Accordion`** + **`ToolBox`** | Built | P1 / P2 | Os dois modos do mesmo widget: várias seções abertas (`Accordion`, conjunto nomeado) e uma só (`ToolBox`, que é o `TabBar` na vertical e não precisa nem do `contains`). O `Accordion` está marcado **precisa estado** na §2.7 desde o começo do documento e não precisa |
| 4 ✅ | **`ButtonBox`** (`QDialogButtonBox`) | Built | P1 | Fecha o outro 🟡 da §2.1. A função é a que o Qt tem: **papéis** (`accept`/`reject`/`destructive`) e a ordem por plataforma decidida no widget, não na tela. Já existe dentro de `dialogs.rs`; com `<slot/>` (0.65) vira widget de tela |
| 5 ✅ | **`MaskedInput`** | **Prim** | P2 | A quarta aplicação da lição do `DateEdit`: máscara é função pura da string no `on_input`, e o que impedia o builtin era a indireção `{{value}}`. Alto valor num projeto que escreve em pt-BR — CPF, CNPJ, telefone, CEP, placa. Guarda cru na chave, exibe mascarado (a mesma separação valor/`displayFormat` do `<dateedit>`) |
| 6 ✅ | **`Rating`** | ~~Built~~ **Prim** | P2 | Estrelas numa chave nomeada, com pré-visualização no hover (chave global, uma por tela — como o hover do `DateRangePicker`). Pequeno, mas é função, não desenho. Também estava citado na §3 e faltava na tabela |
| 7 ✅ | **`decimals` no `SpinBox`** | Built | P1 | Fecha o 🟡 do `QDoubleSpinBox`: hoje as casas saem do `step` (`step="0.25"` → 2 casas), o que acerta por acidente e erra em `step="1"` sobre um preço. Meia hora de trabalho no `spin_box.rs` |

Um exemplo executável por onda, como nas anteriores (`examples/onda3` …
`examples/onda6`), é o que fecha cada uma.

**Saldo das ondas 3 e 4** (medido, 0.85): a §2.5 fechou em 6/6, a §2.4 ganhou
dois (`ListView`, `Pagination`), a §2.7 dois (`Accordion`, `ToolBox`), a §2.3
dois (`Rating`, `decimals`), a §2.2 um (`MaskedInput`) e a §2.1 um
(`ButtonBox`) — e os três 🟡 mais velhos do catálogo (`ButtonBox`,
`QDoubleSpinBox`, `ListView`) fecharam. **Dezesseis widgets, um habilitador de
motor** (o `contains`), que era o menor da lista do §3.

E o mais interessante continua não sendo a contagem: são **nove linhas** que
este documento declarava bloqueadas — sete por estado por instância e duas por
nível errado — e que não estavam.

---

##### O padrão que apareceu duas vezes: repetição dirigida por número

Vale nomear, porque é a primeira causa de reclassificação que se repete e
porque ela **prevê** os próximos casos.

O motor tem uma forma de repetir: `for-each` sobre uma chave que guarda um
array JSON. Ela cobre tudo que é **coleção** — os itens de um menu, as abas de
uma barra, as linhas de uma lista. Não cobre o que é **contagem**:

| widget | o que repete | de onde sairia o array |
|---|---|---|
| `Pagination` | `4 5 6 7 8` | de lugar nenhum — é derivado de `pagina` e `total` |
| `Rating` | 5 estrelas | de lugar nenhum — é derivado de `max` |

A saída de builtin seria o app calcular o array e passá-lo por `items=` — que é
exatamente o trabalho que estes widgets existem para poupar. Em Rust, é um `for`
de uma linha.

**A regra, para a próxima vez:** se a repetição é dirigida por um *número* e não
por uma *coleção*, o widget é primitiva — mesmo que tudo o mais nele pareça
markup. Ela prevê pelo menos mais três linhas do catálogo: `PageIndicator`
(§2.4, os pontinhos), `Grid` (§2.11, `columns="3"`) e `Flow`/`Wrap` (idem).

O `Rating` teve ainda um **segundo** motivo, independente e igualmente
definitivo: o **hover**. O motor expõe `on_press`, `on_double_click`, `cursor` e
`tooltip` em qualquer nó, mas não um `on_enter` — e a pré-visualização ao passar
o mouse é metade do que um `Rating` faz. Fica anotado como um habilitador
possível (`on_enter`/`on_exit` no markup); ele não bloqueia nada da fila, mas é
o que faria um `Rating` builtin ser viável, e vale saber que existe.

---

#### Onda 5 — o conteúdo que sai da tela e entra no widget — ✅ **FEITA (0.92)**

> **Como saiu.** Os seis itens, na ordem, e os dois habilitadores. O achado da
> leva foi o **tamanho relativo dos dois**: o habilitador A — que este documento
> chamava de "o menor que restou do §3" — é literalmente **uma interpolação**
> em `eval.rs`, e é ele que destrava o `QTabWidget`, o 🟡 mais visível do
> catálogo desde a 0.65. O habilitador B, previsto como "médio", saiu como um
> `iced::advanced::{Widget, Overlay}` de verdade (`src/anchored.rs`), e não como
> a generalização do `menu.rs` que a proposta descrevia — ver a nota no fim
> desta seção.
>
> Exemplos: `cargo run --example onda5` e `cargo run --example onda5_luau`, a
> mesma tela sem `impl Component`.

Duas coisas que hoje o app monta à mão, na tela, e deveriam morar **dentro** do
widget: a página de uma aba e o painel que flutua. São dois habilitadores
diferentes, mas a pergunta é a mesma — *de quem é este conteúdo?* — e por isso
saem juntos.

**Habilitador A — nome dinâmico de slot** (Motor, P1, pequeno). ✅ **FEITO (0.92).**
`<slot name="{aba}"/>`, resolvido contra o contexto. A partição por nome já
existia desde a 0.67; faltava interpolar o nome antes da busca, no mesmo ponto
em que o `eval` já roda `process_tpl`. Era o menor habilitador que restou do §3,
e a previsão de tamanho estava certa: **uma interpolação**, mais a simétrica do
lado de quem usa (`slot="{item.id}"` dentro de um `for-each`).

Duas decisões que a proposta não tinha: um nome que interpola para **vazio**
volta a ser o slot anônimo (senão a página sumiria enquanto a chave não fosse
semeada), e um nome sem balde correspondente cai no **conteúdo de reserva** do
`<slot>` — que num `<tabs>` é vazio, e é melhor mostrar nada do que a página da
aba anterior.

**Habilitador B — overlay ancorado genérico** (Motor, P1, médio). ✅ **FEITO (0.92).**
`src/menu.rs` já construiu um overlay ancorado com cascata de submenus, e o
`DIALOGS.md` já mapeou as armadilhas. O que existe está **fechado sobre
`MenuNode`**, e a proposta dava duas saídas: generalizar aquilo, ou escrever um
`iced::advanced::{Widget, Overlay}` custom — "o caminho que o próprio `menu.rs`
documenta como o certo".

**Saiu o segundo**, em `src/anchored.rs`, e vale registrar por que a primeira
opção foi descartada: generalizar o `menu.rs` teria mantido as duas limitações
que o cabeçalho dele já documentava — a âncora é o **cursor**, não o widget, e
não há medição antes de posicionar. Para um menu de linhas de altura fixa aberto
no ponto do clique isso basta; para um popover, não. Um `Widget::overlay()` de
verdade dá as duas coisas de graça: a âncora é `layout.bounds() + translation`
(então o painel acompanha a rolagem) e o painel é medido contra o tamanho da
janela antes de ser movido (então virar para cima quando o rodapé corta é uma
conta, não um chute).

O que faltava para isso não era conhecimento, era **precedente**: `reveal.rs` e
`animated_toggler.rs` já eram `iced::advanced::Widget`; este é o primeiro a ir
até o `Overlay`. O `menu.rs` continua como está — ele funciona, e reescrevê-lo
agora seria trabalho sem consumidor.

| # | Widget | Nível | Prio | Hab. | Por que aqui |
|---|---|---|---|---|---|
| 1 ✅ | **`Tabs` completo** (`QTabWidget`) | Built | P1 | A | A barra saiu na 0.65; a página continua sendo `se`/`senao` na tela. Com o nome dinâmico, vira `addTab(widget, "Geral")` — o conteúdo passa a ser filho do widget, e a tela deixa de repetir a lista de abas duas vezes. Fecha o 🟡 mais visível da §2.8 |
| 2 ✅ | **`calendarPopup` no `<dateedit>`** | Prim | P1 | B | O único item que a Onda 3 deixou para trás, e o último buraco do **foco declarado**: o campo por seções abre a grade de mês ancorada nele. Os dois lados já existirão — é a solda |
| 3 ✅ | **`Popover`** | Prim | P2 | B | O mecanismo cru virado tag: conteúdo por `<slot/>`, ancorado a um gatilho, aberto/fechado numa chave nomeada. É o `Popup` do QML e o que todo menu de usuário/seletor de emoji/painel de filtro pede |
| 4 ✅ | **`Popup`** | Prim | P2 | B | A **mesma primitiva** sem âncora — centrado na janela, sem a modalidade de um `<dialog>`. Custo marginal, pelo padrão `<dateedit>`/`<timeedit>` |
| 5 ✅ | **`Completer`** / **`Autocomplete`** | Prim | P2 | B | A função de verdade desta onda: filtrar enquanto se digita, navegar a lista com ↑↓, aceitar com Enter, desistir com Esc — e devolver o foco ao campo. Fecha a linha duplicada nas §2.2/§2.4/§2.12, e é o widget que mais aparece em app real dos que faltam |
| 6 ✅ | **`Drawer`** | Built | P2 | — | Painel lateral deslizante. **Não precisa de nenhum dos dois habilitadores** — é `<slot/>` (0.65) + uma chave nomeada + a animação que o motor já tem (`ANIMACOES.md`); entra aqui por parentesco, não por bloqueio. Se a onda atrasar, é o item que dá para adiantar |

**O que fecha:** a §2.12 saiu de 2/11 para 5/11 e a §2.8 fechou o `QTabWidget`.
E sumiu a última ressalva da §2.5 — a linha do `QDateEdit` estava ✅ desde a 0.68
mas carregava um "falta a variante `calendarPopup`" desde então. **O foco
declarado do projeto termina aqui, não na Onda 3.**

**As quatro diferenças entre o proposto e o construído**, porque o proposto está
escrito acima e vale saber onde a realidade o corrigiu:

1. **Quem abre e fecha o painel é o widget, não o app.** A proposta descrevia o
   `<popover>` como "aberto/fechado numa chave nomeada", o que dava a entender
   um handler por painel do lado do app. Não é preciso: pressionar o gatilho
   abre, clicar fora fecha, Esc fecha — e o `Anchored` faz as três. Abrir vem
   **antes** de o gatilho ver o evento (um `<button>` consome o pressionar
   dentro dos limites dele, então um `mouse_area` por fora nunca dispararia) e,
   ao contrário do fechar, **não consome**: o botão continua disparando o
   `on_click` que o markup lhe deu.
2. **O clique que fecha não chega a mais nada.** É o preço de o overlay receber
   o evento primeiro, é o comportamento de um menu de SO, e é o que faz um
   gatilho que alterna a chave não reabrir o painel no mesmo quadro.
3. **O `<popover>` reparte os filhos por `slot`, não por posição.** Isso pediu
   uma mudança de uma linha no `eval`: a etiqueta `slot` passou a **atravessar a
   avaliação** quando o pai é uma primitiva (numa fronteira de componente ela
   continua sendo consumida pela partição, como sempre).
4. **O `<drawer>` precisou de um eixo no `<reveal>`.** A proposta dizia "a
   animação que o motor já tem"; o motor animava **altura**. `axis="x"` é o
   mesmo mecanismo na largura, e é literalmente a mesma função com um `if`.

---

#### Onda 6 — a grade: uma medição, seis widgets — ✅ **FEITA (0.92)**

> **Como saiu.** A aposta pagou: a medição é `src/grid.rs`, um
> `iced::advanced::Widget` que mede os filhos em dois passos, e dela saem
> `<grid>`, `<tableheader>`, `<tableview>` e `<columnview>`. Mas **duas** das
> seis linhas não passaram por ela, e as duas são correções de rota do mesmo
> tipo — algo catalogado como caro que já existia:
>
> - o **`Flow`/`Wrap`** é o `Row::wrap()` do próprio `iced`, três linhas em
>   `widget.rs`. A primeira vez neste documento em que quem já tinha a
>   capacidade era a biblioteca de baixo, não o motor;
> - o **`TreeView`** é recursão sobre a coleção mais um conjunto nomeado — nem
>   grade, nem estado por instância.
>
> Exemplos: `cargo run --example onda6` e `cargo run --example onda6_luau`.

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
| 1 ✅ | **`Grid`** (`QGridLayout`) | Prim | P1 | A medição, e o widget mais simples que a exercita. Sai primeiro porque é o teste do mecanismo antes de haver cabeçalho, ordenação e seleção por cima — o mesmo papel que o `GroupBox` teve para o `<slot/>` |
| 2 ✅ | **`Flow`** / **`Wrap`** | Prim | P2 | A mesma medição num eixo só: quebra automática de linha. Fecha a §2.11 (layouts) e é o que um campo de tags/chips pede |
| 3 ✅ | **`TableHeader`** (`QHeaderView`) | Prim | P2 | Cabeçalho clicável (ordenar) e arrastável (redimensionar). O arrasto mora numa chave global — um por vez, a família do `__drag_key` que o motor já tem |
| 4 ✅ | **`TableView`** | Prim | P2 | Cabeçalho + corpo, com **ordenação** (coluna e direção em chaves nomeadas, a comparação no `update`) e **seleção** (chave nomeada; múltipla pelo conjunto nomeado). Edição de célula fica para depois — reusa o `<TextInput>`, não a medição |
| 5 ✅ | **`TreeView`** | Prim | P2 | Recursão sobre a coleção + conjunto nomeado de nós abertos. Sai do estado por instância pela mesma porta que o `Accordion` |
| 6 ✅ | **`ColumnView`** | Prim | P3 | Navegação Miller (o Finder): uma `ListView` por nível, o nível escolhido numa chave. Quase de graça depois do 5 |

**Habilitador desta onda — virtualizar a lista (Motor, P1). ✅ FEITO (0.77).**

Saiu antes da onda, e fora de ordem, porque apareceu em produção: uma tela do
`rustploy` com algumas dezenas de cartões de serviço rolava a poucos quadros por
segundo. O perfil mostrou que o motor não era o gargalo (montar os `Element` de
1.682 nós custa 431 µs, 2,6% do orçamento de 60 fps) — o custo estava no `iced`
medindo e desenhando **todos** os filhos, inclusive os fora da tela.

`virtualize="<altura>"` numa coluna dentro de um `<scrollable>` monta só os
filhos visíveis, com vãos do tamanho exato nas pontas para a barra de rolagem
continuar honesta. O motor passou a plumbar o `on_scroll` do `iced` (só quando
há o que virtualizar logo abaixo, para não cobrar do caso comum) e guarda o
deslocamento numa chave interna — que **não** dispara reavaliação, senão rolar
custaria uma árvore nova por pixel.

A altura da linha é **declarada**, não medida: descobri-la exige o layout, que é
o trabalho a evitar. É a troca do `uniformItemSizes` do `QListView`. Ver
`PRIMITIVAS.md` para a forma e as três armadilhas.

Render de uma lista de 300 cartões: **1,81 ms → 47 µs por quadro**, e o custo
virou constante — 300 itens custam o mesmo que 10. Serve `ListView`, `TreeView`
e qualquer `for-each` dentro de um `<scrollable>`, que é por que valia
construir como capacidade do motor e não como detalhe do `TableView`.

Fica de fora, com motivo: a virtualização é de **render**, não de avaliação — a
árvore inteira continua avaliada e na memória. Fazer a avaliação também seguir a
janela economizaria memória, mas exigiria reavaliar a cada rolagem, que é
exatamente o que esta versão evita.

**O que fecha:** a §2.4 (seleção/listas/árvores) saiu de 3/11 — a categoria mais
atrasada do catálogo desde o começo — para **8/11**, e a §2.11 (layouts) fechou
tudo o que tinha ⬜. É a onda que mais mudou o resumo numérico de todas.

**As três diferenças entre o proposto e o construído:**

1. **A ligação a coleção já existia**, como a proposta suspeitava — e é a nona
   linha que este documento marcou como bloqueada sem estar. `items="chave"` é
   a mesma convenção do `<menu items>` desde sempre; o que faltava era a
   medição, mais as convenções de seleção e ordenação (que são o padrão do
   `SpinBox`, já escrito).
2. **A ordenação é numérica quando os dois lados parseiam como número.** Não
   estava na proposta e é o que separa uma tabela usável de uma que coloca
   `"10"` antes de `"9"` numa coluna de contagem.
3. **O arrasto de coluna adia o zero para o primeiro movimento do mouse.** A
   proposta dizia "o arrasto mora numa chave global — um por vez, a família do
   `__drag_key`", e isso saiu como previsto (`__colgrip`). O que ela não previu
   é a consequência da própria economia do motor: enquanto não há alça presa, o
   motor **não escuta o mouse** (`precisa_do_cursor`), então no instante do
   clique a última posição conhecida é lixo. O zero do arrasto fica em aberto e
   o primeiro `CursorMoved` o ancora — um quadro sem redimensionar, que ninguém
   vê, e exato daí em diante.

**Uma armadilha nova, para quem for escrever a próxima primitiva de layout:**
dentro de um `<scrollable>` o teto vertical é **infinito**, e qualquer filho que
se declare `height="fill"` mede infinito. Um `Length::Fill` de 1px na alça do
cabeçalho fez a linha inteira medir infinito e empurrou o corpo da tabela para
fora da tela — a tabela aparecia **vazia, sem erro nenhum**. É a gêmea da
armadilha do `Length::Fill` no wrap de background que o `PRIMITIVAS.md` já
registra, do outro lado do eixo. O `grid.rs` agora ignora altura não-finita ao
medir uma linha, e a alça tem altura declarada.

**E uma do `iced`, que custa a última coluna de toda tabela:** a barra de
rolagem de um `scrollable` **flutua sobre** o conteúdo, a menos que um
`spacing` seja declarado. Sem `.spacing(0)`, o que ela cobre numa tabela é
exatamente a coluna da direita.

---

#### Onda 7 — o canvas, e os sete que saem dele — ✅ **FEITA (0.93)**

```text
Habilitador — o canvas como CAPACIDADE, não como tag   (motor, `src/canvas.rs`)

1. Dial       — o knob rotativo                        (primitiva)
2. Gauge      — o medidor de arco, com faixas          (primitiva)
3. LcdNumber  — sete segmentos                         (primitiva)
4. Sparkline  — a linha sem moldura                    (primitiva)
5. LineChart  — a linha com eixos                      (primitiva)
6. BarChart   — barras                                 (primitiva)
7. PieChart   — setores, e a rosquinha                 (primitiva)
```

**A decisão que sustenta a onda: o `canvas` não virou tag.** A tradução literal
do `QGraphicsView` seria um `<canvas>` com callback de desenho em Rust. Isso
devolveria ao app um bloco imperativo que o `.gv` não lê, o `.gss` não estiliza
e o lado Luau não alcança — três regressões para ganhar uma. O `canvas` ficou
como **capacidade** (`src/canvas.rs`: arcos, séries, escala 1·2·5, sete
segmentos) e o que o app vê são sete tags que se comportam como todas as outras.
O `<canvas>` avulso continua catalogado em P3; se um dia sair, sai como
vocabulário declarativo (`<path>`, `<arc>`, `<circle>`), não como escape hatch.

**A segunda decisão fecha a §4:** gráficos **na mão**, e não `plotters` — a
cor, a manutenção e o tamanho, nessa ordem (o raciocínio inteiro está lá).

**O que fecha:** a §2.13 saiu de **0/6** — a única categoria zerada do catálogo
— para 5/6 (sobra o 3D, que é `shader`/wgpu), e a §2.3 perdeu três ⬜.

**As três diferenças entre o proposto e o construído:**

1. **Nenhum dos três medidores era `●`.** O catálogo marcava o `Dial` como
   "exige estado por instância" e classificava os três como componente. O valor
   sempre coube numa chave que o app nomeia — `value="volume"`, como
   `value="nota"` —, e o único estado interno de verdade (o arrasto em curso)
   vive no `canvas::Program::State`, que o `iced` dá por widget. É a **terceira**
   reclassificação desse tipo, depois do `Spinner` (0.66) e do `Rating` (0.85), e
   já é regra: *o que parece estado por instância quase sempre é o valor*.
2. **`<sparkline>` não é uma tag a mais.** É `<linechart axes="false">` — a
   mesma `NodeType`, outros defaults. Um lugar a menos onde um bug de escala
   pode morar, pelo mesmo raciocínio que fez `<popup>` ser `<popover>`.
3. **O `ColorDialog` ficou de fora.** Estava na proposta da onda; sozinho é do
   tamanho de dois ou três dos outros, e nada depende dele.

**Uma armadilha nova, para quem for desenhar:** o `arc` do `iced` chama
`Builder::ellipse`, e essa função faz `move_to` **sempre**. Dois arcos no mesmo
`Path` viram dois sub-caminhos soltos e o `close()` não fecha nada — um anel
montado como "arco externo de ida, arco interno de volta" preenche errado, com
as fatias da pizza faltando pedaço perto do centro e as faixas do `<gauge>`
enchendo até o miolo. `crate::canvas::{arco, anel}` desenham por polilinha, um
segmento por grau.

**Uma lição de nome, que um teste pegou:** um substantivo comum não pode virar
tag. `<linha>` chegou a ser apelido do `<linechart>` e roubou o nome de todo
componente chamado `Linha` — o parser mapeia a tag **antes** de procurar
componentes, então um `<component name="Linha">` do app passava a ser ignorado
em silêncio. Valia igual para `<display>`, `<barras>` e `<pizza>`. Os apelidos
em pt-BR ficaram nos nomes que ninguém usaria para um componente próprio
(`grafico_linha`, `medidor`, `minigrafico`), e há teste para isso.

**E uma que não é do motor, mas custou a onda inteira:** o `iced_tiny_skia`
0.14.0 — o renderizador de **software**, que é onde se cai quando o `wgpu` não
sobe — aplica a transformação **duas vezes** ao recorte de um grupo de
primitivas de `canvas`. O sintoma engana: o primeiro desenho da tela sai cortado,
todos os seguintes somem, e o texto de todos continua aparecendo. A correção é de
uma linha, e o repositório a carregou em `vendor/iced_tiny_skia` até a 0.14.1
sair — o que aconteceu, e o vendor foi apagado na 0.94.3. Ver
`TROUBLESHOOTING.md`.

**O que sobra desta família:** **série múltipla**. Um `items` é uma série;
comparar duas no mesmo eixo pede uma segunda convenção de dados, uma legenda e
uma paleta por série. É trabalho de verdade e não é o gargalo de nada — o caso
comum de um painel é um número por gráfico.

---

#### Onda 8 — o diálogo que carrega markup — ✅ **FEITA (0.94)**

> **Como saiu.** O habilitador pagou como previsto e por um motivo que a
> proposta acertou: `GlacierUI::render(nome)` já montava qualquer template, e o
> corpo do diálogo virou **um parâmetro e uma chamada**. Os seis saíram, o
> `FontDialog` ficou de fora pelo motivo escrito, e **duas** das seis linhas
> mudaram de forma no caminho — as duas pelo mesmo motivo, que é a descoberta
> desta onda: *um template não calcula, e um slot não atravessa componente*.
>
> Exemplos: `cargo run --example onda8` e `cargo run --example onda8_luau`.

```text
Habilitador — o modal ganha CORPO e RETORNO    (motor: `dialogs.rs`, `lib.rs`, `luau/`)

1. Dialog          — o `<dialog>` próprio, que é o QDialog     (motor + tag)
2. InputDialog     — pede texto, número ou item de lista       (diálogo, P1)
3. ProgressDialog  — progresso cancelável                      (diálogo, P1)
4. ColorDialog     — roda/HSV/hex, sobre o canvas da Onda 7    (diálogo, P2)
5. StackView       — o QStackedWidget, formalizado             (primitiva, 🟡)
6. Wizard          — passos com voltar/avançar/finalizar       (primitiva, P2)
```

**A frase que esta onda derruba** está escrita no `DIALOGS.md`, e estava certa
quando foi escrita: *"Um diálogo não segue esse caminho: ele é transiente
(aberto e fechado por código, não faz parte de nenhuma tela) e construído
inteiramente em Rust (`src/dialogs.rs`), **sem markup**."* Isso é
verdade enquanto todo diálogo é uma caixa de mensagem: ícone, texto, botões, e
os cinco construtores de conveniência dão conta. Deixa de ser verdade no
instante em que um diálogo precisa de um **campo** — o `QInputDialog::getText`
é um `QLineEdit` dentro de um cartão, e o motor já sabe desenhar `<textinput>`,
já sabe estilizá-lo pelo `.gss` e já sabe ligá-lo a uma chave. Escrever um
segundo caminho de render, em Rust, para cada diálogo que tenha conteúdo é
escrever o motor duas vezes — e é o mesmo erro que a Onda 7 evitou ao recusar o
`<canvas>` com callback imperativo.

Por isso os cinco ⬜ da §2.10 não são cinco trabalhos. São **um**: o diálogo
deixar de ser uma tela paralela e virar uma **moldura em volta de uma tela**.
É o regime das ondas 2, 5 e 6 — um item de motor que vira meia dúzia de
widgets — e é o último desse tipo que o catálogo ainda oferece.

**A reclassificação, a 11ª à 13ª da mesma família.** O catálogo marca
`InputDialog`, `ProgressDialog` e `ColorDialog` com `●` — "exige estado por
instância". Nenhum dos três exige, e aqui a marca é *estruturalmente*
impossível: o diálogo é **singleton** no motor (`dialog: Option<DialogSpec>`,
`lib.rs:176`), então nunca existe uma segunda instância com que colidir. O que
o usuário digita mora numa **chave nomeada**, como no `SpinBox` (0.85) e no
`Dial` (0.93) — `<textinput value="__dialog.nome"/>`, e o botão OK lê essa
chave. É a mesma pergunta que este documento errou dez vezes, e é a última vez
que ela cabe: depois desta onda, todo `●` que sobra é de widget que existe **N
vezes na mesma tela** (`Splitter`, `MdiArea`, `Dock`, `RangeSlider`, `Tumbler`),
que é o caso em que a marca sempre foi honesta.

##### O habilitador, em três partes — e a primeira já está escrita

**A. O corpo é o nome de um template já avaliado.** `GlacierUI::render(name)`
(`lib.rs:2529`) devolve um `Element<EngineMessage>` para qualquer template
avaliado — componente ou tela, tanto faz. Então `DialogSpec` ganha
`body: Option<String>`, `dialogs::overlay` recebe o `Element` pronto e o encaixa
entre a mensagem e os botões, e `render_current` (`lib.rs:711`) passa a montá-lo
antes de empilhar. O diálogo para de ser um segundo caminho de render e vira uma
moldura. **Custo real: um parâmetro e uma chamada** — o resto o motor já faz.

Duas consequências que valem o preço sozinhas: o corpo é estilizável pelo `.gss`
como qualquer tela — hoje o cartão do diálogo se pinta pelo
`theme.extended_palette()` e mais nada, sem seletor nenhum por cima —, e o lado
Luau alcança o conteúdo do modal, que hoje não alcança.

**B. O retorno deixa de ser um `bool`.** `resume_dialog_inner`
(`src/luau/mod.rs:539`) retoma a corrotina com `Value::Boolean(confirmed)`. A
forma geral já existe **ao lado**, em `resume_file_dialog_inner` (`:556`), que
retoma com uma string ou `nil` — é o `open_file()` que o motor já entrega. Unificar
os dois em um retorno `Option<Value>` dá `prompt{} → string|nil` e `pick_color{}
→ "#rrggbb"|nil` sem inventar mecanismo nenhum, e `confirm{}` continua devolvendo
booleano porque booleano é o valor dele. O `nil` do cancelamento já é a
convenção escrita: *"cancelado vira `nil` — o mesmo silêncio que `confirm()` dá"*
(`luau/mod.rs:953`).

**C. Abrir um diálogo a partir do markup.** Hoje só dá por Rust
(`ctx.show_dialog`) ou por Luau (`confirm()`); uma tela declarativa pura não
consegue abrir um modal. O lugar disso é a família de prefixos de ação que a
0.63 abriu com o `app:` (`eval.rs:1552`): `on_click="dialog:confirmar_exclusao"`
abre o `<dialog name="confirmar_exclusao">` da própria tela. É a menor das três
partes e é a que faz o `<dialog>` ser um widget do catálogo em vez de uma API de
Rust.

##### Os seis

| # | Widget | Nível | Prio | Por que aqui |
|---|---|---|---|---|
| 1 | **`Dialog`** (`QDialog`) | Motor + tag | P1 | O que o catálogo nunca listou, e é a **classe-base** de todo o resto da §2.10: um bloco `<dialog name="…">` no `.gv`, registrado como uma tela, aberto por `dialog:nome` e fechado por ação. Sai primeiro porque é o teste do habilitador antes de haver campo, progresso ou roda de cor por cima — o mesmo papel que o `GroupBox` teve para o `<slot/>` e o `<grid>` para a medição. Os botões continuam vindo do `DialogSpec`, não do markup: é lá que mora a ordem por plataforma que o `ButtonBox` (0.85) já resolveu |
| 2 | **`InputDialog`** (`QInputDialog`) | Diál | **P1** | As quatro variantes do Qt (`getText`/`getInt`/`getDouble`/`getItem`) são **um** diálogo com corpos diferentes — `<textinput>`, `<spinbox decimals>` e `<select>`, três tags que já existem. O trabalho é a conveniência: `prompt{ kind = "text"|"int"|"double"|"item", … }` no Luau, com validação (`min`/`max`, obrigatório) travando o botão OK antes de retomar a corrotina |
| 3 | **`ProgressDialog`** (`QProgressDialog`) | Diál | **P1** | O único da família que é **atualizado enquanto está aberto**, e o único que não suspende: quem suspende é o `fetch`/stream que ele acompanha. O progresso vai numa chave que o corpo lê (`<progressbar value="__dialog.progresso"/>`), o cancelamento escreve outra que o laço do app consulta. Fecha o par com o `Spinner` (0.66): indeterminado avulso lá, determinado e cancelável aqui |
| 4 | **`ColorDialog`** (`QColorDialog`) | Diál | P2 | Ficou de fora da Onda 7 por tamanho; agora custa menos, porque a roda é `crate::canvas::{arco, anel}` (0.93) e o campo hex é um `<maskedinput>` (0.85). Três painéis — roda HS + barra V, os canais em `<slider>`, e o hex —, todos escrevendo a **mesma** chave `#rrggbb`. É o primeiro consumidor do corpo em markup que não caberia em Rust sem duplicar meia dúzia de widgets |
| 5 | **`StackView`** (`QStackedWidget`) | Prim | P1 | O 🟡 mais antigo da §2.8, que a Onda 5 devia ter fechado e não fechou: é `<tabs>` **sem a barra**, o mesmo nome dinâmico de slot (`<slot name="{passo}"/>`, 0.92). Sai aqui porque o item 6 precisa dele, e sozinho é a formalização que o documento promete desde a primeira revisão |
| 6 | **`Wizard`** (`QWizard`) | Prim | P2 | No Qt o `QWizard` **é** um `QDialog` — então ele é a soma exata desta onda: o `<dialog>` do item 1, as páginas do item 5, o `<buttonbox>` (0.85) e o número do passo numa chave nomeada. A lógica que ele carrega é aritmética de passo com portão (`Voltar` inerte no primeiro, `Avançar` travado enquanto a página não valida, `Finalizar` no lugar de `Avançar` no último) — repetição dirigida por número, o padrão que a Onda 4 nomeou |

##### Fica de fora, com motivo

**`FontDialog` (e o `FontSelect` da §2.4), porque o problema deles não é
diálogo.** O motor conhece **duas** fontes: `font_for` (`widget.rs:413`) mapeia
`mono`/`monospace`/`code` para a monoespaçada e devolve `None` para o resto —
`font="bold"` nem é família, é peso. Não existe registro de famílias, não existe
`Font::with_name` no caminho do render, e o `iced` não enumera as fontes do
sistema (isso é `fontdb`). Um `<fontdialog>` construído hoje mostraria uma lista
de duas linhas.

As duas linhas estão catalogadas no lugar errado: são um **item de Motor**
("famílias de fonte: registro, `font-family` no `.gss` e enumeração do SO"), não
um diálogo e não um combo. A onda que as construir é a que fizer o subsistema —
e aí as duas saem juntas, de graça, como o `TableHeader` saiu da medição. O
`PrintDialog` continua fora de escopo, como já estava declarado.

**Série múltipla nos gráficos** continua onde a Onda 7 a deixou: é trabalho de
verdade, não é o gargalo de nada, e não tem relação com esta onda.

##### As três armadilhas previstas

1. **O bloqueio do modal, agora pelo outro lado.** O `DIALOGS.md` registra um bug
   caro: o fundo escurecido precisava capturar hover **e** clique para o input
   não vazar para a tela de baixo — `.interaction(Interaction::Idle)` mais um
   `on_press` sempre presente. Um corpo em markup fica **acima** desse fundo e
   não pode ser bloqueado por ele. O sintoma de errar isso é discreto e feio: o
   campo do diálogo simplesmente não pega foco, sem erro nenhum. Vale um teste
   por cima do item 1, antes de haver o que digitar.
2. **A colisão é por nome, não por instância.** O corpo é avaliado no contexto do
   app, então as chaves que ele escreve são chaves do app — e dois diálogos que
   usem `nome` colidem, mesmo sendo singletons. A convenção proposta é o prefixo
   `__dialog.`, **limpo no fechamento**; sem a limpeza, a segunda abertura do
   mesmo diálogo já vem preenchida com a resposta anterior, que é o bug que
   ninguém reporta e todo mundo estranha.
3. **`show_dialog` troca o `DialogSpec` inteiro** (`lib.rs:656`) — e o item 3
   atualiza o diálogo dezenas de vezes por segundo. Se o progresso morar no
   `spec`, cada tique reconstrói a especificação e o cartão; se morar numa chave
   que o corpo lê, o `spec` nunca muda e o progresso anda pelo mesmo caminho de
   qualquer outro valor do motor. É o desenho que impede o `ProgressDialog` de
   virar caso especial — e é o argumento A desta onda pagando de novo.

##### A ordem, e por que ela

`A → 1 → 2 → B → 3 → C → 4 → 5 → 6`. A parte **A** primeiro porque é o
habilitador e é pequena; o item **1** logo atrás para exercitá-la vazia, antes de
haver conteúdo que esconda um erro de camada. O **2** valida o corpo com o widget
mais simples que existe (um campo) e só então vem a parte **B**, que é o que ele
precisa para devolver a string. O **3** entra antes do **C** porque não depende de
abrir do markup — quem o abre é o código que já está rodando. O **C** fecha o lado
declarativo, e os **4/5/6** são os consumidores grandes, na ordem de custo.

Entregáveis, no formato das ondas anteriores: `examples/onda8` e
`examples/onda8_luau`, `DIALOGS.md` reescrito (a seção "Por que é diferente do
resto do glacier-ui" deixa de valer), `PRIMITIVAS.md` para `<stackview>` e
`<wizard>`, e o `vscode-gv` com as tags e atributos novos.

##### As cinco diferenças entre o proposto e o construído

1. **O `Wizard` não é primitiva, e não é builtin: é os dois.** A proposta o
   classificava `Prim`, pela regra da Onda 4 (repetição dirigida por número). Na
   escrita apareceu a contradição: um wizard **hospeda páginas**, e página é
   `<slot>`, que é mecanismo de *componente* — uma primitiva não tem slots. Mas
   um builtin é um template, e template **não calcula**: dizer "este é o
   primeiro passo, então `Voltar` fica inerte" exige achar `active` dentro de
   `steps`, e não há como escrever isso em markup.

   A saída é a que a Onda 5 já tinha usado sem nomear: **partir em dois**. O
   `<wizard>` é o builtin que compõe; a `<wizardnav>` (`src/wizard.rs`) é a
   primitiva que faz a conta e desenha os botões. É a mesma divisão de
   `<tabs>`/`<tabbar>`, e agora ela tem um critério em vez de um acidente:
   *slots pedem builtin, contas pedem primitiva, e um widget que precise dos
   dois é dois widgets*.

2. **Um `<slot>` não atravessa a fronteira de um componente aninhado** — e esta
   é a armadilha nova da onda, porque falha **em silêncio**. O `<wizard>`
   deveria delegar as páginas ao `<stackview>` do item 5, assim:

   ```xml
   <StackView active="{active}"><slot name="{active}"/></StackView>
   ```

   Não funciona. A partição do `<slot/>` acontece **uma vez**, sobre os filhos
   crus de quem escreveu a tag; o que chega ao `StackView` é um filho já
   resolvido e **sem etiqueta**, e o `<slot name="{active}"/>` do template dele
   não acha etiqueta com que casar. O resultado é uma página em branco, sem erro
   nenhum. O `<wizard>` monta a coluna da página ele mesmo — as mesmas quatro
   linhas —, e o `<stackview>` continua existindo para quem troca de página sem
   ser um wizard.

3. **O `ProgressDialog` precisou de duas requisições novas no motor.** A
   proposta dizia que ele cabia na máquina existente, e não cabia: toda a
   maquinaria de diálogo da camada Luau **suspende** a corrotina, e este é o
   único que não pode — ele acompanha um trabalho que continua rodando.
   Nasceram `__glacier_dialog_open` e `__glacier_dialog_close`, as primeiras
   requisições de diálogo que retomam no mesmo turno. O resto da previsão se
   confirmou: o progresso mora numa chave, o `DialogSpec` nunca muda, e o
   cancelamento é uma **ação comum**, não um mecanismo novo.

4. **O `pick_color{}` entrou pela porta do `prompt{}`.** A proposta os tratava
   como itens separados (2 e 4). São o mesmo: um seletor de cor é um `prompt`
   cujo campo é uma roda, e o que os separa é o **nome do corpo**. Ficaram na
   mesma função de construção, com o mesmo retorno e a mesma convenção de
   desistência — um lugar a menos onde um bug de cancelamento pode morar, pelo
   mesmo raciocínio que fez `<sparkline>` ser `<linechart axes="false">` na
   Onda 7.

5. **A 11ª à 13ª reclassificação se confirmaram, e a checagem foi mais barata
   do que as dez anteriores.** As outras exigiram escrever o widget para
   descobrir que o estado era o valor. Esta cabe numa linha do `lib.rs`:
   `dialog: Option<DialogSpec>` — um campo, não um mapa. Um widget que não pode
   existir duas vezes não pode ter estado *por instância*, e isso se lê no tipo.

##### O que ficou de fora, e o que ficou torto

**`FontDialog` e `FontSelect` continuam fora, e a proposta acertou o motivo.**
O `font_for` (`widget.rs`) conhece duas fontes — a padrão e a monoespaçada — e
`font="bold"` nem é família, é peso. Os dois esperam um item de **Motor**
(registro de famílias, `font-family` no `.gss`, enumeração do SO) que o §3 nunca
listou.

**Um torto que apareceu e foi consertado, e o conserto pegou um bug maior.** O
campo hexadecimal do `ColorDialog` escrevia direto na chave que a roda lê, então
ela piscava branco enquanto alguém digitava `#ff8800` (a chave passava por `#f`,
`#ff`, `#ff8`). A correção é o par rascunho/cometido — `__dialog.value__hex`
guarda o texto, `__dialog.value` guarda a cor, e só um hexadecimal **inteiro**
comete.

Ao escrever isso apareceu o bug de verdade, e ele era invisível nos testes:
**um `<TextInput>` nunca grava a chave sozinho**. Ele despacha `onChange` com o
texto novo, e sem `onChange` a ação é vazia — que é roteada para a tela ativa e
não trata nada. Os campos do `__InputDialog` e do exemplo eram **decorativos**:
mostravam o valor inicial, aceitavam digitação e não guardavam nada. Os testes
não pegaram porque escreviam a chave com `define_data`, que é justamente o que o
campo deveria fazer.

Duas lições ficam:

1. **Testar o widget pela mensagem que ele despacha**, não pelo estado que se
   escreve à mão. O teste novo (`digitar_no_prompt_escreve_a_chave`) procura a
   ação de `onChange` na árvore avaliada e falha se ela for vazia.
2. **O corpo de um diálogo não é um uso de tag**, e por isso a ação dele não
   ganha namespace: ele é montado por `render(nome)` como template de topo, sem
   dono. O `onChange` traz `__InputDialog::` escrito à mão, com o porquê ao lado.

E uma decisão de desenho: o campo aceita **seis** dígitos, não a forma curta.
`#f0a` é cor válida em CSS, mas também é o meio do caminho de quem digita
`#ff8800` — aceitá-la trocaria "a roda pisca branco" por "a roda pisca amarelo",
que é mais discreto e mais confuso.

**Uma pegadinha do contexto, sem consequência mas vale saber:** escrever `""`
numa chave a faz **sumir** em vez de ficar vazia. Não muda nada — ausente e
vazia leem igual em todo o motor —, mas um teste que espere `Some("")` falha.
É o que faz `progress_set(nil)` voltar corretamente ao indeterminado.

##### O que fecha

A §2.10 saiu de **9/14** para **13/15** (a linha do `QDialog` avulso é nova), e
os dois que sobram estão justificados por escrito. A §2.8 saiu de 3/6 mais um 🟡
para **5/6**, sobrando o `SwipeView`. E o `DIALOGS.md` perdeu a frase de
abertura dele: um diálogo **não** é mais "construído inteiramente em Rust, sem
markup".

E fica registrado, porque é a segunda vez: **o habilitador desta onda nunca
esteve no §3**. O da Onda 6 (a medição de colunas) também não. A lista de
pré-requisitos do motor acertou o que era caro e errou o que era o gargalo — o
gargalo real, das duas vezes, foi uma capacidade que ninguém tinha catalogado
como capacidade.

---
#### Onda 9 — o ponteiro preso, e o teclado que ninguém escuta — ✅ **FEITA (0.95)**

> **Como saiu.** O habilitador A pagou como previsto e pelo motivo que a
> proposta acertou: o motor já arrastava três vezes, e generalizar o `__colgrip`
> foi tirar a conta de dentro dele (`src/grip.rs`, um `enum Alvo` e um
> `aplica`). Os nove saíram; as ASSERÇÕES dos dois testes de arrasto da Onda 6
> não mudaram uma linha, e é isso que elas passaram a guardar além do widget.
>
> **Três coisas mudaram de forma no caminho**, e as três pelo mesmo motivo — o
> de sempre neste documento, que é descobrir escrevendo que a peça já existia:
>
> - o **`SizeGrip` encolheu de Motor para builtin de vinte linhas**. Ele entrou
>   como o sétimo consumidor do arrasto e não consumiu nada: `window:resize:se`
>   já era ação da titlebar custom (`daemon.rs`) e `cursor="se"` já era atributo
>   universal (`widget.rs`). Sobrou desenhar o cantinho. É a **15ª** correção de
>   nível deste catálogo, e a primeira para o lado da infraestrutura em vez do
>   estado;
> - o **`<rubberband>` passou a desenhar os alvos**, e não só a faixa. O
>   `QRubberBand` não faz isso — lá a faixa é um widget solto sobre uma view.
>   Aqui não há "por baixo": o motor não tem `<stack>` no markup, e um laço que
>   só desenhasse a faixa pediria para arrastar sobre o vazio. Como ele **já
>   tem** a geometria (é com ela que decide o que tocou), desenhá-la foi de
>   graça;
> - o **`<shortcut>` mora no layout, não no `<resources>`**. Parece declaração e
>   é — mas quem o encontra é `collect_tree_bindings`, que varre a **árvore
>   avaliada**, e o `<resources>` não é árvore. O parser recusou a primeira
>   tentativa com a mensagem certa ("dentro do `<resources>` só entram…").
>
> E o habilitador B **não custou nada ao Luau**, que é o achado do exemplo
> `onda9_luau`: a Onda 8 precisou de quatro globais novos porque um diálogo
> *suspende*; um arrasto escreve numa chave, e o Luau já escrevia chaves. Nove
> widgets, zero linhas em `src/luau/`.
>
> Exemplos: `cargo run --example onda9` e `cargo run --example onda9_luau`.


```text
Habilitador A — o ARRASTO como capacidade      (motor: `src/grip.rs`, `lib.rs`, `daemon.rs`)

1. Splitter        — painéis redimensionáveis (QSplitter)      (primitiva, P2)
2. RangeSlider     — dois cursores numa faixa                  (primitiva, P2)
3. Tumbler         — a roleta de valores                       (primitiva, P3)
4. SwipeView       — páginas deslizáveis                       (primitiva, P3)
5. DelayButton     — o botão que se segura                     (primitiva, P3)
6. RubberBand      — retângulo de seleção                      (primitiva, P3)
7. SizeGrip        — o canto que redimensiona a janela         (motor,     P3)
   · PageIndicator — os pontinhos, de carona no 4              (builtin,   P2)

Habilitador B — a SUBSCRIPTION de teclado      (motor: `lib.rs`, `daemon.rs`)

8. Shortcut/Action — atalhos globais declarados no markup      (motor+tag, P2)
9. ShortcutInput   — o campo que captura uma combinação        (primitiva, P3)
```

**Por que existe uma Onda 9, depois de esta seção ter escrito que não haveria.**
A frase acima — *"o que sobra no catálogo não tem mais o formato que estas seis
ondas exploraram: são widgets isolados (`Splitter`, `MdiArea`, `SwipeView`)"* —
lista três widgets como se fossem três assuntos. Não são. `Splitter`,
`RangeSlider`, `Tumbler`, `SwipeView`, `DelayButton`, `RubberBand` e o `SizeGrip`
são **um** assunto: o ponteiro apertado sobre um widget ao longo do tempo,
escrevendo num valor enquanto anda. Sete linhas do catálogo, espalhadas por seis
seções diferentes, e é isso que escondeu que eram a mesma.

O erro tem a forma exata dos dois anteriores. A Onda 6 catalogava `Grid` e
`TableView` como dois trabalhos caros sem notar que a parte difícil dos dois era
a mesma medição; a Onda 8 catalogava cinco diálogos sem notar que a parte difícil
dos cinco era o modal carregar markup. Aqui são sete widgets separados pela
**seção da tabela** — botões, entradas numéricas, containers, navegação, janela,
overlays —, que é o eixo errado para enxergar mecanismo.

##### O habilitador A já está escrito três vezes

E é essa a evidência de que ele é uma capacidade, não um widget: o motor **já
arrasta**, em três lugares independentes, sem que nada disso apareça no §3.

| Onde | O que guarda o arrasto | O que faltou generalizar |
|---|---|---|
| reordenar lista | `__drag_key` + `{var}.__dragging` por item (`lib.rs:339`, `eval.rs:847`) | o alvo é um **índice**, e a conta é "sobre qual item estou" |
| alça de coluna do `<tableheader>` | `__colgrip` = `chave\|indice\|x0\|w0` (`widget.rs:2526`), aplicado em `arrasta_coluna` (`lib.rs:2534`) | o eixo é fixo em `x`, e o mapeamento pixel→valor está embutido na função |
| giro do `<dial>` | `EstadoDial { arrastando }` no `canvas::Program::State` (`gauges.rs:82`) | não sai do canvas: só serve a quem desenha o próprio widget |

Os três já resolveram, cada um por si, os dois problemas difíceis do arrasto — e
os dois estão **documentados no código**, que é o que torna a generalização
barata em vez de arriscada:

- **o zero do arrasto não é o clique.** Enquanto não há alça presa o motor não
  escuta o mouse, então a última posição conhecida do cursor é de um menu aberto
  meia hora atrás. O `__colgrip` nasce com `x0 = "?"` e o **primeiro**
  `CursorMoved` é que vira o zero (`lib.rs:1470`). Custa um quadro que ninguém
  vê e é exato daí em diante;
- **o listener só existe entre o pressionar e o soltar.** `precisa_do_cursor`
  (`lib.rs:2515`) liga o `listen_with` do movimento só quando há menu em jogo ou
  alça presa, e a nota ao lado tem o número medido: 70 movimentos por segundo
  davam 65 quadros, 110 davam 10. Um arrasto genérico que ligasse o listener
  para sempre estrangularia a rolagem do app inteiro.

O que **falta** é pequeno e é exatamente o que `arrasta_coluna` tem hardcoded:
o eixo (`x`, `y` ou os dois), o mapeamento de delta de pixel para valor (linear
numa faixa, fração de um pai, índice numa lista, fração de um tempo) e o
`clamp` de cada consumidor. Um `src/grip.rs` com um `enum Alvo` e uma função
`aplica(delta) -> String` é `arrasta_coluna` com a conta fatorada para fora — e
o `DragEnd`, que já é a mensagem de soltar de **dois** dos três, continua sendo
a única.

##### A 14ª reclassificação, e é a última em que a pergunta cabe

Seis dos sete estão marcados `Comp ●` — "exige estado por instância". Nenhum
exige, e desta vez o motivo é o mesmo para os seis, o que é a própria evidência:
**o que o arrasto move é sempre um valor que o app nomeia**, e o que sobra dele
(estou arrastando? desde onde?) é global por natureza, porque só se arrasta uma
coisa por vez numa tela.

| Widget | O que o catálogo diz | O que é |
|---|---|---|
| `Splitter` | painéis redimensionáveis, `●` | é a alça de coluna aplicada a um container: as frações moram numa chave (`sizes="0.3 0.7"`), no **mesmo formato de trilhas** que o `columns` do `<grid>` já parseia. `<tableheader>` já é um splitter horizontal com cabeçalho |
| `RangeSlider` | dois cursores, `●` | duas chaves nomeadas — é o que o `<daterangepicker range>` já faz com `start`/`end` (Onda 3). Qual das duas pontas está presa é a identidade dentro do `__grip`, como o índice da coluna |
| `Tumbler` | roleta de valores, `●` | o `<dial>` desenrolado numa linha: o valor na chave, a inércia no `Program::State`. Terceira vez que esta reclassificação sai do mesmo widget |
| `SwipeView` | páginas deslizáveis, `●` | `<stackview active="{passo}">` (0.94) + arrasto no eixo `x` + a animação do `<reveal>` (0.90). As duas metades existem; o que falta é a do meio |
| `DelayButton` | anel de progresso ao segurar, `●` | a metade do arrasto em que **o que anda é o tempo, não o pixel**. A fração vai numa chave global (`__hold`) e o anel é o `<gauge>` da Onda 7 |
| `RubberBand` | retângulo de seleção, `●` | o retângulo é uma chave global (um por tela, como o `__cal_hover`); o que ele seleciona é o **conjunto nomeado** que o `<listview mode="multi">` já guarda (0.85) |

O `SizeGrip` (§2.9) é o sétimo e o único sem `●`, porque o alvo dele não é uma
chave: é a janela. O gesto é o mesmo; quem recebe é o daemon, que já lê e escreve
geometria de janela (é o que o gancho `on_close` consulta).

**Depois desta onda o `●` para de significar alguma coisa.** Sobram `MdiArea` e
`Dock` — os dois únicos em que a marca continua honesta, e os dois estão fora
desta onda por um motivo que não é o estado (ver abaixo) — mais `Canvas`,
`Shader`, `Q3D` e `FontSelect`, que estão fora de escopo ou bloqueados por outra
coisa. É a última vez que a pergunta que este documento fez treze vezes tem onde
ser feita.

##### O habilitador B: a sexta linha de uma lista de cinco

`Shortcut`/`Action` é o item 9 do §3, P2, e é o único da lista que ainda tem
consumidor no catálogo. Ele parece um item de motor e é uma **entrada numa lista
que já existe**: o daemon registra hoje cinco `iced::event::listen_with`
(`daemon.rs:1106`), cada um uma `fn(Event, Status, window::Id) -> Option<EngineMessage>`
— `drag_end_from_event` (`lib.rs:3142`), `tab_focus_from_event` (`:3158`),
`timeedit_key_from_event` (`:3189`), `cursor_from_event` (`:3245`),
`menu_escape_from_event` (`:3261`).

Um sexto na mesma forma, lendo uma tabela de atalhos declarados na árvore
(`<shortcut key="ctrl+s" on_press="salvar"/>`), é `Shortcut`/`Action` inteiro. O
`timeedit_key_from_event` já é o modelo do cuidado que ele precisa: `if status ==
Captured { return None }` — um atalho não rouba a tecla de um campo focado.

E `ShortcutInput` (§2.2, `●` P3) é **esse mesmo listener em modo de captura**: a
combinação que ele grava vai numa chave nomeada, e "qual campo está capturando" é
global por natureza — uma chave do motor com a identidade da instância no valor,
que é literalmente o que o `__timeedit` faz (`"inicio:h"`, `widget.rs:16`).

##### O que fica de fora, e por escrito

- **`MdiArea` e `Dock`** (§2.7, `●` P3). Não são arrasto: são **gerência de
  janela**, e este projeto já tem janelas de verdade — o daemon (`src/daemon.rs`,
  um `GlacierUI` por janela) e o `open_window` do Rust e do Luau. Uma área MDI é
  uma reimplementação, dentro de um retângulo, de uma capacidade que o motor já
  entrega melhor fora dele. Se saírem, saem de um debate sobre janelas, não desta
  onda — e é aí que o estado por instância finalmente vira o assunto.
- **`FontSelect` e `FontDialog`** (§2.4 e §2.10). O bloqueio deles é real e é
  outro: o registro de famílias de fonte, que o §3 nunca listou. São dois widgets
  para um habilitador, e o habilitador tem uma decisão de §4 dentro dele — o
  `iced` não enumera as fontes do SO, então ou entra uma crate (`fontdb`,
  `font-kit`) ou o app declara as famílias que usa. É uma onda pequena e própria,
  não um apêndice desta.
- **Séries múltiplas** nos gráficos (§2.13, o único 🟡 da seção). Continua sendo
  o próximo passo natural daquela família, e continua não sendo ponteiro: é uma
  convenção de dados (`series="[{nome, pontos}]"`), uma legenda e uma paleta por
  série, tudo anotado em `charts.rs:24`. Cabe numa onda com o resto do troco
  visual da §6.3.
- **`Canvas` como tag, `Shader`, `Q3D`, `PrintDialog`, `QWhatsThis`,
  `SplashScreen`.** Já justificados onde estão; nenhum ganha nada com o arrasto.

##### O que fecha

| Seção | Hoje | Depois da Onda 9 |
|---|---|---|
| §2.1 Botões e ações | 7/10 | 8/10 (`DelayButton`) |
| §2.2 Entradas de texto | 6/9 | 7/9 (`ShortcutInput`) |
| §2.3 Numéricas/valor | 9/12 | **11/12** (`RangeSlider`, `Tumbler`) |
| §2.4 Seleção/listas | 8/11 | 9/11 (`PageIndicator`) |
| §2.7 Containers | 6/9 | 7/9 (`Splitter`) |
| §2.8 Navegação | 5/6 | **6/6** (`SwipeView`) |
| §2.9 Janela/barras | 7/8 | **8/8** (`SizeGrip`) |
| §2.12 Overlays/utilitários | 5/11 | 7/11 (`RubberBand`, `Shortcut`/`Action`) |
| **Total** | **92/125 (74%)** | **102/125 (~82%)** |

**Duas seções fecham inteiras** — a navegação (6/6) e a janela/barras (8/8) —, e
a §2.3 fica a um item, que é o `ScrollBar` 🟡 de propósito ("embutido no
`scrollable`; expor avulso é raro"). Nenhuma onda anterior fechou duas categorias
com todas as linhas em ✅; a Onda 8 chegou perto, com duas categorias fechadas
mas os restos justificados por escrito.

**Depois dela, o regime acaba.** Esta é a última onda do formato "um habilitador
que vira meia dúzia de widgets": o que sobra no catálogo são as fontes (duas
linhas, um habilitador), a gerência de janela (duas linhas, um debate), o troco
da §6.3 (sete linhas, nenhum habilitador), seis linhas justificadas como fora de
escopo e a de contabilidade abaixo. Vale escrever isso agora, para que a próxima
revisão não tenha de descobrir de novo que não há tema — e desta vez a frase é
verificável: **o único `●` que sobra por estado de verdade é a dupla
`MdiArea`/`Dock`**, e ela está fora por um debate sobre janelas, não por falta de
motor.

##### Uma linha de contabilidade, de graça

`ListView bind` (§2.4, `QListView (model)`) continua ⬜ e a própria nota da linha
diz que *"a ligação **já existe** (`items="chave"`)"*, o que a Onda 6 confirmou
por escrito ("nem existia como trabalho"). É a 14ª correção de nível deste
documento e a única que não custa código: a linha deveria estar ✅ desde a 0.92.
Fechá-la leva o total a **103/125 (~82%)** — e ela fica aqui, e não na tabela
acima, porque somar um widget que já existe ao placar de uma onda seria contar
duas vezes.

---
#### Onda 10 — a fonte que o motor não sabe nomear — 🟡 **EM ANDAMENTO (0.98)**

**Alvo (proposta de 2026-09-08): 103 → 107 de 125 (de 82,4% para 85,6%).** A
Onda 11 saiu na frente (a pedido), então a base real é **111/125 (88,8%)** e o
alvo desta onda vira **111 → 114** — `FontSelect`, `FontDialog` e o
`PlainTextEditor` 🟡→✅; o `TextBrowser` fica para depois (precisa de uma
primitiva `<markdown>` que não existe). O texto da proposta segue abaixo; a
nota de resultado está no fim da seção.

É o **último habilitador de motor que este catálogo tem escrito**. A tabela da
§6.2 ("Onde os habilitadores do §3 foram parar") já o dizia com todas as letras,
até esta revisão —
*"Famílias de fonte: sem onda, e agora é o único bloqueio de motor que sobrou"*
—, e ele é pequeno. O que faltava não era o trabalho: era um consumidor que
valesse a rodada. São quatro, e um deles é um `🟡` que ninguém tinha ligado a
isto.

**O estado de hoje, medido no código:**

| Onde | O que há |
|---|---|
| `font_for` (`widget.rs`) | conhece **duas** fontes: `mono`/`monospace`/`code` → `Font::MONOSPACE`, `bold` → peso. Qualquer outro nome retorna `None`, isto é, **a fonte padrão, em silêncio** |
| `font-family` no `.gss` (`stylesheet.rs`) | é aceita, guarda a string em `rule.font`… e cai no mesmo funil acima |
| `GlacierDaemon::font(bytes)` (`daemon.rs`) | **já carrega** `.ttf`/`.otf` e já sabe definir a padrão (`default_font`) |

Os três lados existem; o que não existe é o **nome** entre eles. A lista de
bytes registrada não guarda com que família cada um entrou, então um `.gss` não
tem como pedir "Inter" e um `.gv` não tem como pedir a mono do app. É a mesma
forma dos outros gargalos deste documento: nenhuma peça falta, falta ligá-las —
e, como sempre, isso é invisível na tabela, porque a falha aparece como *"a
propriedade foi ignorada"*, que é exatamente o que a convenção do `CLAUDE.md`
avisa ser a família de bug mais cara aqui.

##### O habilitador — o registro de famílias (`src/fonts.rs`)

1. **`GlacierDaemon::font_named("Inter", bytes)`**, ao lado do `font(bytes)` que
   fica como está. Guarda um `Vec<(String, Font)>` que sobrevive ao boot e vive
   no runtime, do lado do `default_font`.
2. **`font_for` consulta o registro** antes de desistir: apelidos primeiro
   (`mono`/`bold`, compatibilidade), nome registrado depois, sem caixa. Nada no
   markup muda de forma — `font="JetBrains Mono"` e
   `font_family: "JetBrains Mono"` passam a funcionar pelo caminho que **já
   existe**, o que faz desta onda duas linhas de motor e um módulo de registro.
3. **A lista no contexto**, numa chave do motor (`__fonts`, a família do
   `__cal_hover`/`__band`/`__colgrip`), para que um `<combo items="__fonts">` a
   consuma sem API nova. É o mesmo truque do `<listview items="chave">`, e é o
   que faz o `FontSelect` não precisar de nada além de markup.
4. **A decisão da §4 mora aqui, e a recomendação é `fontdb` atrás de uma
   feature** (`system-fonts`), desligada por padrão — o `iced` não enumera as
   fontes do SO, e a escolha é entre uma crate ou o app declarar o que usa. Com
   a feature desligada, `__fonts` traz as famílias registradas, que é o que um
   app empacotado quer mesmo; com ela ligada, traz também as do SO. É a forma do
   `tray-icon` da §2.9 (feature + custo só para quem pede), e evita pôr uma
   crate de índice de fontes no caminho de quem só queria uma mono.

**A armadilha desta onda**, para ficar escrita antes de custar meia tarde: o
`iced` resolve família por `Font::with_name(&'static str)` — o nome precisa
viver tanto quanto o app. Um `Box::leak` no registro (uma vez por família, no
boot, número fixo e pequeno) é a saída honesta, e é o que os próprios exemplos
do `iced` fazem.

##### Os quatro widgets

| Linha | Hoje | Depois |
|---|---|---|
| `FontSelect` (§2.4) | ⬜ `Comp ●` | ✅ **Built** — `<combo>` sobre `__fonts` com cada item desenhado **na própria fonte** (uma linha: `font=` no item), que é o que o separa de um `<select>` qualquer. O `●` cai pelo motivo de sempre: a família escolhida é um texto numa chave nomeada. Seria a **16ª correção de nível** |
| `FontDialog` (§2.10) | ⬜ `Diál ●` | ✅ — `prompt{ kind = "font" }`: o mesmo caminho do `pick_color{}` da Onda 8, com corpo em markup (`<FontSelect>` + `<spinbox>` de tamanho + negrito/itálico + uma amostra) e `DialogOutcome`. **Zero linhas de diálogo novas** — o corpo em markup da Onda 8 já é a porta. E o `●` não podia estar aí desde a 0.94: o diálogo é singleton (correções 11–13 da §5) |
| `PlainTextEditor` (§2.2) | 🟡 | ✅ — o 🟡 dele **é esta onda**, e ninguém tinha ligado os dois. A diferença entre `QPlainTextEdit` e `QTextEdit` é "texto simples, fonte declarável"; sem registro de famílias, a segunda metade não dá para escrever |
| `TextBrowser` (§2.2, hoje na §6.3) | ⬜ | ✅ de carona — o `markdown` do `iced` é read-only e já entrega links; o que ele pede do motor é uma família para o bloco de código (esta onda) e uma ação para o clique (`on_link`, o formato do `on_click`). Sai daqui porque é aqui que a família existe |

##### Custo previsto e o que fica de fora

Um módulo (`src/fonts.rs`, na casa de 150 linhas), duas linhas em `font_for`, um
builtin, um corpo de diálogo, uma feature de Cargo e um exemplo
(`examples/fontes`). **Fica de fora, por escrito:** `@font-face` no `.gss` (o
motor não carrega fonte em tempo de execução, e esta onda não inventa isso),
fallback por script (CJK) e qualquer coisa de rasterização — os três são do
`iced`/`cosmic-text`, não deste catálogo.

**O que esta onda fecha de categoria:** **duas** — a §2.2 (9/9, com o 🟡 do
`PlainTextEditor` junto) e a §2.4 (11/11) —, e a §2.10 vai a 14/15, com o
`PrintDialog` justificado por escrito. Só a Onda 8 tinha fechado duas de uma
vez, e com ressalva nas duas.

##### Como está saindo (0.98), contra a proposta acima

**O habilitador saiu como desenhado.** `src/fonts.rs` é o registro
(`RwLock<Vec<(String, Font)>>` global, `Box::leak` do nome uma vez por família),
`font_for` ganhou as **duas linhas** previstas (apelidos primeiro, registro
depois), `GlacierDaemon::font_named("Inter", bytes)` fica ao lado do `font(bytes)`
que não mudou, e a chave `__fonts` é semeada em `set_initial_screen` a partir do
registro — `<combo items="__fonts">` / `<fontselect>` a consomem sem API nova. A
feature de Cargo é **`system-fonts`** (desligada por padrão), sobre `fontdb`, e
acrescenta as famílias do SO à lista de `__fonts`; a decisão da §4 mora aí, como
a proposta dizia. A armadilha do `&'static str` foi tratada no `Box::leak`.

**Três dos quatro widgets:**

| Linha | Estado | Como |
|---|---|---|
| `PlainTextEditor` (§2.2) | 🟡 → ✅ | **zero código de widget** — o `<texteditor>` já chamava `font_for`, que agora resolve `font="JetBrains Mono"`. A previsão ("a diferença é só a fonte declarável") era literal |
| `FontSelect` (§2.4) | ⬜ `Comp ●` → ✅ **Built** | **16ª correção de nível**. `src/builtins/font_select.rs`: um `<listview>` sobre `__fonts` com `font="{fam}"` no rótulo de cada item (a linha que o separa de um `<select>`). A proposta dizia `<combo>`; um `pick_list` do `iced` não estiliza opção por opção, então o corpo é a lista de botões do `<listview>` — mesma ideia, tag viável |
| `FontDialog` (§2.10) | ⬜ `Diál ●` → ✅ | `prompt{ kind = "font" }` em `build_dialog`, corpo `__FontDialog` (`<FontSelect>` + `<spinbox>` de tamanho + amostra), `pick_font{}` no prelúdio. **Zero linhas de diálogo novas**, como previsto. **Ressalva:** `pick_font` devolve **só a família** (uma string, como `pick_color` devolve o hex) — tamanho/estilo são pré-visualização. O `DialogOutcome` tipado (`{family, size, bold}`) pede uma chave de retorno composta que o resume (uma chave, uma string) ainda não tem; fica anotado |
| `TextBrowser` (§2.2/§6.3) | ⬜ | **não saiu.** A proposta o punha "de carona", mas ele precisa de uma **primitiva `<markdown>`** (parser + render + estado dos itens + `on_link`) que não existe — é trabalho próprio, não carona. Continua na §6.3, agora esperando `<markdown>` e não só o registro de famílias |

**A conta, então:** 111 → **114/125 (91,2%)** — três widgets, um habilitador de
motor (`src/fonts.rs`), uma feature de Cargo, uma correção de nível
(`FontSelect`, a 16ª). Fecha **uma** categoria e meia: a §2.4 (**11/11**) e a
§2.2 vai a **8/9** (sobra o `TextBrowser`). A §2.10 vai a **14/15**. Exemplos:
`examples/fontes` (Rust) e `examples/fontes_luau` (o `pick_font{}`), o par de
sempre — rode com `WGPU_BACKEND=gl` nesta máquina.

---
#### Onda 11 — o que fica por cima: `<stack>` e `pin` — 🟡 **FEITA PARCIALMENTE (0.96)**

**Alvo original: 107 → 117 de 125 (de 85,6% para 93,6%), contando com a Onda
10 antes dela.** O usuário pediu para pular a 10 (o registro de fontes) e ir
direto para esta — o alvo real, então, partiu de **103** (82,4%, sem a Onda 10)
e teve DOIS cortes sobre o desenho original: o `Dock` e a série múltipla dos
gráficos (habilitador C) ficaram de fora, os dois por decisão tomada durante a
implementação, não por bloqueio de motor. **Resultado: 103 → 111 (de 82,4%
para 88,8%)** — sete widgets e dois habilitadores de motor, os dois previstos
e os dois confirmados do jeito que a proposta abaixo desenhou. O texto da
proposta original fica preservado abaixo; a nota de resultado está no fim da
seção, depois de "A previsão, para ser conferida depois".

##### A observação (é a quarta vez, e a segunda do mesmo tipo)

Quatro frases, escritas em quatro seções diferentes deste documento, em quatro
revisões diferentes:

- §2.6, `Avatar`: *"Sem indicador de presença (pediria `Stack` dentro do
  builtin)"*;
- §6.3, `NotificationDot`: *"pede `Stack` dentro do builtin, ou um `padding`
  negativo bem escolhido"*;
- §2.12, `RubberBand` (Onda 9): *"ele desenha os alvos que conhece, e não só a
  faixa — **o motor não tem `<stack>` no markup**"*;
- e a coluna "Base iced" de três linhas ⬜ — `MdiArea` (`canvas/stack`),
  `SplashScreen` (`stack`), `NotificationDot` (`container`, mas a nota diz
  `Stack`) — que **já nomeiam a capacidade que falta**.

É **uma** capacidade: sobrepor um filho a outro e ancorá-lo. E, como o
`Row::wrap()` da Onda 6 (correção nº 10 da §5), **a biblioteca de baixo já a
tem**: `stack` e `pin` estão na lista de widgets do `iced 0.14` reproduzida na
§1 deste documento, e o `widget.rs` até usa `stack!` **uma vez, hardcoded**,
para centralizar o percentual sobre o `<progressbar>`. Nunca virou tag.

Quarta vez que o gargalo real não estava na lista do §3 (depois da medição de
colunas, do corpo do diálogo e do arrasto), e segunda em que a capacidade já
existia embaixo, esperando alguém a chamar de capacidade.

##### Habilitador A — `<stack>` + posição livre

- **`<stack>`**: empilha os filhos no mesmo espaço, o primeiro embaixo. É o
  `iced::widget::stack`, e são três linhas no `widget.rs` — o mesmo tamanho que
  o `<flow>` teve na Onda 6.
- **`x=`/`y=` dentro de um `<stack>`**: posição livre sobre o
  `iced::widget::pin`. Valores dirigidos por dado, portanto **inline no `.gv`**
  pela regra do `CLAUDE.md` (é o mesmo caso do `background="{cor}"`), não no
  `.gss`.
- **`anchor="top-right"`** e os outros sete cantos, para o caso comum sem
  número: é o que o `NotificationDot`, o ponto de presença do `Avatar` e o
  `<sizegrip>` da Onda 9 querem, e nenhum dos três quer coordenada.

##### Habilitador B — `grip::Alvo::Ponto`, o arrasto em dois eixos

O `src/grip.rs` da Onda 9 tem hoje um `Arrasto` com **um** `eixo: Eixo` e
**uma** `origem: Option<f32>`, e dois alvos (`Trilha`, `Indice`) — tudo
unidimensional, porque os sete widgets da Onda 9 arrastam em uma direção só.
Uma janela interna arrasta em duas.

`Alvo::Ponto { min_x, max_x, min_y, max_y }` guarda o par, escreve `x,y` numa
chave (ou em dois campos do item da coleção) e **reusa o resto inteiro** — a
âncora no primeiro movimento e o listener condicional, que o cabeçalho do
`grip.rs` chama de "os dois problemas difíceis", já estão resolvidos e
comentados lá. É a mesma extensão que a Onda 9 fez ao tirar a conta de dentro do
`__colgrip`, um nível acima.

##### Os quatro widgets do empilhamento

| Linha | Hoje | Depois |
|---|---|---|
| `MdiArea` (§2.7) | ⬜ `Comp ●` | ✅ **Prim** — e o `●` é a **17ª correção de nível**. Sub-janelas são uma **coleção** (`items="janelas"`, a ligação que existe desde o `<menu>`), a posição de cada uma são dois campos do item, arrastar a barra de título é o `Alvo::Ponto`, redimensionar é o `<sizegrip>` que a Onda 9 já entregou, e a ordem Z é a **ordem da coleção** — trazer para frente é mover para o fim, que é o `__drag_key` de reordenar lista, também pronto. O que a marca `●` chamava de "estado por instância" é uma coleção com geometria |
| `Dock` (§2.7) | ⬜ `Comp ●` | ✅ **Built** — `<splitter>` (Onda 9) para o painel acoplado, `<stack>` para o flutuante, `Alvo::Ponto` para arrastar de um estado ao outro. Não é widget novo: é a composição que faltava de peças feitas. **Ressalva escrita:** acoplar **entre janelas** do daemon (arrastar um dock de uma janela para outra) fica de fora — atravessa o limite do `GlacierUI`, que é um por janela |
| `NotificationDot` (§2.12/§6.3) | ⬜ | ✅ — a frase da §6.3 vira duas linhas com `anchor="top-right"`, e o `padding` negativo "bem escolhido" que ela sugeria como alternativa deixa de ser necessário |
| `SplashScreen` (§2.12) | ⬜ `Comp` | ✅ **Built** — `<stack>` sobre a tela + o `<reveal>` da 0.90 para sumir + a janela sem decoração que o `open_window` do daemon já abre. As três peças existem desde a 0.90 |

De carona e sem linha no catálogo: o **indicador de presença do `Avatar`**, que a
§2.6 registra como ausente desde a 0.65, e um `<stack>` de verdade por baixo do
`RubberBand`, que hoje desenha os alvos que conhece justamente por não ter um.

##### O troco da §6.3, que esta onda esvazia

Cinco linhas de composição pura, que **não** justificam rodada própria e por
isso estão lá desde sempre — mas a onda passa no lugar delas (uma delas *é* o
habilitador A) e um troco ao lado de trabalho de verdade custa uma tarde:
`QrCode` (o `qr_code` nativo do `iced`, encanamento, e o último item da Fase A),
`Chip`, `Skeleton`, `CommandLink`, `RoundButton`. Depois delas a §6.3 fica
**vazia**.

##### Habilitador C — a série múltipla (de carona, sem relação com A e B)

Do mesmo jeito que a Onda 9 juntou o arrasto e o teclado, que nada têm a ver um
com o outro: nenhum dos dois abre rodada sozinho, e os dois são pequenos.

O cabeçalho do `src/charts.rs` já o descreve com a decisão tomada — *"séries
múltiplas … pediria uma segunda convenção de dados (`series="[{nome, pontos}]"`),
uma legenda e uma paleta por série … fica anotado como o próximo passo natural
desta família"*. É isso: uma convenção de dados, uma legenda e um ciclo de cores
sobre a `Moldura` e a `escala` que os quatro gráficos já compartilham. Fecha o
`AreaChart`/`Scatter` 🟡 → ✅ e leva a §2.13 a 5/6.

##### A conta da onda

| | ✅ | % |
|---|---|---|
| entra (depois da Onda 10) | 107 | 85,6% |
| habilitadores A+B (`MdiArea`, `Dock`, `NotificationDot`, `SplashScreen`) | 111 | 88,8% |
| o troco da §6.3 (`QrCode`, `Chip`, `Skeleton`, `CommandLink`, `RoundButton`) | 116 | 92,8% |
| habilitador C (`AreaChart`/`Scatter` 🟡 → ✅) | **117** | **93,6%** |

**Categorias que fecham:** a §2.1 (10/10, com o troco) e a §2.7 (9/9, com as
duas de janela interna). A §2.6 vai a 13/15 — o que sobra é `Canvas` como tag e
`Shader`, os dois fora de escopo por escrito —, a §2.12 a 9/11 (sobram
`QWhatsThis` ⬜ e `QScroller` 🟡) e a §2.13 a 5/6. A §2.2, a §2.4 e o resto já
terão fechado na Onda 10.

##### O que sobra depois — e por que isto é o fim da fila

Cinco `⬜` e três `🟡`, **todos justificados por escrito antes desta revisão**:

| Sobra | Por quê |
|---|---|
| `Canvas` como tag | decisão da Onda 7: um callback imperativo o `.gv` não lê. Se sair, sai como vocabulário declarativo (`<path>`, `<arc>`, `<circle>`) |
| `Shader`, `Q3D` | wgpu; escopo distante |
| `PrintDialog` | impressão; fora do escopo inicial |
| `QWhatsThis` | ajuda contextual; o `tooltip=` num modo pegajoso |
| `ScrollBar` 🟡 | embutido no `scrollable` de propósito; expor avulso é raro |
| `QStackedLayout` 🟡 | é o `se`/`senao`, e agora também o `<stackview>` da Onda 8 |
| `QScroller` 🟡 | rolagem por gesto, no `scrollable` |

Não é "acabou o que fazer" — é que **acabou o que este documento decidiu fazer e
não fez**. As duas coisas são diferentes, e a segunda é a única que uma fila de
execução consegue medir. Depois da Onda 11, avançar o catálogo passa a exigir
uma decisão nova (o vocabulário de desenho declarativo, o `shader`, a impressão)
e não mais uma releitura da tabela.

E se a decisão for tomada: `Canvas` declarativo + `QWhatsThis` são os dois únicos
que não são "outro projeto", e levam a **119/125 — 95,2%**.

##### A previsão, para ser conferida depois

Este documento tem o hábito de guardar a previsão junto do resultado (a da Onda
9 acertou 102/~82%). As destas duas, então, escritas antes de qualquer linha de
código: **Onda 10 = 107 / 85,6%**, **Onda 11 = 117 / 93,6%**, uma correção de
nível em cada (`FontSelect`, a 16ª; `MdiArea`, a 17ª), e **nenhum item novo do
§3** — o registro de famílias já estava listado, e o `<stack>`/`pin` é do
`iced`, não do glacier-ui.

O risco declarado, também para ser conferido: a Onda 11 aposta que `MdiArea` e
`Dock` são composição de peças prontas. Se essa aposta furar, ela fura no
`Dock` — arrastar um painel de acoplado para flutuante é uma **mudança de pai**
no meio de um arrasto, e o `grip.rs` foi escrito para mexer em *valores*, não em
estrutura. Se for esse o caso, o `Dock` sai da onda e ela entrega **116/125
(92,8%)**, o que não muda a leitura de nenhuma das duas.

##### O resultado real (0.96), contra a previsão acima

**A aposta furou exatamente onde a previsão dizia que furaria.** Escrito antes
de qualquer linha de código, o parágrafo acima apontou o `Dock` como o risco —
"mudança de pai no meio de um arrasto" — e foi lá que a implementação parou:
ao escrever o `render_mdi_subwindow` (`src/panes.rs`) ficou claro que
acoplar/soltar exigiria o `<splitter>` e o `<stack>` **trocando de pai** no
mesmo nó, sob o mesmo arrasto que hoje só reescreve uma chave — não uma
mudança de código pequena, uma decisão de design que esta rodada não tomou.
`Dock` fica ⬜, isolado, sem levar `MdiArea` junto.

**Um segundo corte que a previsão não tinha antecipado**: a série múltipla dos
gráficos (habilitador C) também ficou de fora — não por dificuldade, mas por
ordem de prioridade dentro do tempo da rodada; o `MdiArea` (o item novo mais
pesado) e o troco (cinco widgets) vieram primeiro, e a série múltipla é
autocontida em `charts.rs`, sem dependência de nada que saiu aqui. Fica
anotada como o próximo passo natural da família, exatamente como o cabeçalho
de `charts.rs` já dizia antes desta onda.

E a base mudou: como a Onda 10 não rodou (pulada a pedido), o "antes" real é
**103/125 (82,4%)**, não 107. A conta fechada:

| | ✅ | % |
|---|---|---|
| antes (0.95, Onda 9 fechada, Onda 10 pulada) | 103 | 82,4% |
| **depois da Onda 11** (`MdiArea`, `NotificationDot`, `SplashScreen` + o troco: `QrCode`, `Chip`, `Skeleton`, `CommandLink`, `RoundButton`) | **111** | **88,8%** |

Sete widgets, dois habilitadores de motor (`<stack>`/`pin` e
`grip::Alvo::Ponto`), uma correção de nível (`MdiArea`, a 17ª — `SplashScreen`
não contou como correção porque nunca teve `●`, e o catálogo já o tinha como
`Comp` sem marca de estado). **Os dois habilitadores previstos saíram como
previsto**; os dois cortes (`Dock`, série múltipla) ficam registrados para uma
leva futura, e nenhum dos dois bloqueia o resto do que a Onda 11 entregou —
o `MdiArea` funciona sozinho, sem o `Dock`.

Testes: as 6 propriedades novas de `Alvo::Ponto` em `src/grip.rs` (origem
ancorando os dois eixos juntos, escrita nas duas chaves, saturação
independente por eixo, chave `y` ausente), mais as 7 já existentes
reescritas para o novo formato de `Arrasto` — **347 testes passando** no
crate inteiro, nenhuma quebra. Exemplos: `examples/onda11` e
`examples/onda11_luau` (rode com `cargo run --example onda11` /
`onda11_luau`, `WGPU_BACKEND=gl` nesta máquina) — o par de sempre desde a Onda
4, e a versão Luau confirma a mesma observação da Onda 9: nem `<stack>`
(markup puro) nem `grip::Alvo::Ponto` (escreve as chaves do `<mdisubwindow>`
por baixo, como o `<slider>` já fazia) pediram uma linha de script. O que
sobrou para o Luau fazer são as quatro coisas de sempre — semear, alternar um
booleano, um clique com payload (`remover_tag:<tag>`, a convenção
`nome:sufixo` do dispatcher) e um clique sem payload —, e o campo do `QrCode`
nem isso: sem uma função `qr_texto`, o binding legado `ctx[ação] = valor`
resolve sozinho.

##### Completando a Onda 11: habilitador C entregue (0.98), `Dock` continua fora

Dos dois cortes, o menor voltou primeiro. A **série múltipla** (habilitador C)
saiu **como o cabeçalho do `charts.rs` já a descrevia**: `series="chave"` guarda
um array de `{name, points, color?}` ([`charts::SerieNomeada`]), a `Moldura` e a
`escala` 1·2·5 **não mudaram** (`limites_multi` só concatena os pontos para a
faixa do eixo), a legenda é um retângulo translúcido no canto e as cores sem
`color` vêm do `cor_ciclica` que o `<piechart>` já usava. Vale para o
`<linechart>` e portanto para `area=`/`points=` — o **`AreaChart`/`Scatter` da
§2.13 sai de 🟡 para ✅**, e a §2.13 vai a **5/6** (sobra o `Q3D`, wgpu).
`<barchart>` e `<piechart>` seguem série-única de propósito. 114 → **115/125
(92,0%)**. Exemplos: `examples/series_multiplas` e
`examples/series_multiplas_luau`.

O **`Dock`** continua ⬜, e a tentativa desta rodada confirmou o diagnóstico da
Onda 11 por um segundo caminho: além do "trocar de pai no meio do arrasto", ele
esbarra no modelo de **builtin**. O `<accordion>`/`<accordionitem>` empilha
itens num eixo; um dock põe painéis nas **quatro bordas** com o conteúdo no
meio, e um builtin com `<slot/>` recebe todos os filhos numa região só, sem como
ordená-los por lado. As saídas são duas, e as duas são decisão de design que
esta rodada não tomou: (a) `pane_grid` do `iced` como substrato — e aí o `●`
volta a ser honesto, é estado por instância de verdade —, ou (b) um
`grip::Alvo::Zona` que só **comete** um valor de modo na soltura (o desenho da
Onda 12), com um fantasma seguindo o cursor durante o gesto. Fica para a Onda
12, isolado.

---
#### Onda 12 — o `Dock` que a Onda 11 cortou — ⬜ **proposta (2026-09-09), reduzida**

**Alvo: 115 → 116 de 125 (de 92,0% para 92,8%).**

A Onda 11 saiu com dois cortes. O menor — a **série múltipla** (habilitador C) —
foi entregue ao completar a Onda 11 (0.98): `AreaChart`/`Scatter` 🟡→✅, §2.13 a
5/6. Ver "Completando a Onda 11", ao fim da seção da Onda 11. **Sobra o `Dock`**,
e é ele que esta onda cobre.

##### Habilitador D — a troca de estrutura entre quadros

A Onda 11 parou o `Dock` num ponto exato, e ele está transcrito no resultado
dela: acoplar/soltar exigiria o `<splitter>` e o `<stack>` **trocando de pai no
mesmo nó, sob o mesmo arrasto que hoje só reescreve uma chave**. O `grip.rs`
mexe em valor, não em estrutura — e essa é a decisão de design que a 11 não
tomou.

A saída não é ensinar o `grip.rs` a reparentar no meio do gesto: é **não
reparentar no meio do gesto**. O arrasto de um painel de dock escreve uma chave
de **modo** (`acoplado` / `flutuante` / `lado-esquerdo` / …), não geometria, e
escreve **na soltura**, quando o cursor cruza um limiar — não a cada movimento.
Enquanto o dedo está apertado, o que segue o cursor é um fantasma num `<stack>`
de overlay (o habilitador A da Onda 11, pronto) desenhado como o `<rubberband>`
já desenha (`__band`, um por tela, da Onda 9). No release, uma escrita de chave.
Entre um quadro e o outro, o template lê o modo num `se`/`senao` e escolhe o
pai — `<splitter>` para o acoplado, `<stack>` com `x=`/`y=` para o flutuante. A
árvore muda **depois** do gesto, dirigida por um valor nomeado, que é o terreno
em que o motor sempre soube andar.

Com isso o `Dock` é **Built**, não `Comp ●` — `<splitter>` (Onda 9) + `<stack>`
(Onda 11) + o fantasma + a chave de modo, tudo peça pronta. Seria a **18ª
correção de nível**, e pelo motivo de sempre: o `●` marcava "estado por
instância" onde havia uma chave nomeada e uma composição.

**Ressalva escrita, a mesma da Onda 11:** acoplar **entre janelas** do daemon
(arrastar um dock de uma janela para outra) fica de fora — atravessa o limite do
`GlacierUI`, que é um por janela. O que esta onda entrega é o dock dentro de uma
janela: acoplado nas quatro bordas, flutuante, e o trânsito entre os dois
estados.

##### A conta da onda

| | ✅ | % |
|---|---|---|
| entra (0.98, Onda 11 completa com o habilitador C) | 115 | 92,0% |
| habilitador D (`Dock` ⬜ → ✅) | **116** | **92,8%** |

**Categorias que fecham:** a §2.7 (**9/9**) — `MdiArea` entrou na 11, `Dock`
entra aqui, e os containers ficam inteiros.

##### A previsão, para ser conferida depois

**Onda 12 = 116 / 92,8%**, uma correção de nível (`Dock`, a 18ª), nenhum item
novo do §3 — o habilitador D é uma decisão de design sobre peças que as ondas 9
e 11 já entregaram. O risco declarado: o habilitador D aposta que "commit na
soltura" é suficiente e que ninguém vai sentir falta do reparent contínuo. Se
furar, fura aí — e o `Dock` volta a `Comp ●`, à espera do estado por instância
de verdade.

---
#### Onda 13 — o desenho que o `.gv` escreve — ⬜ **proposta (2026-09-09)**

**Alvo: 113 → 115 de 125 (de 90,4% para 92,0%).**

Aqui a fila deixa de ser releitura da tabela e passa a exigir **uma decisão
nova**, como o fim da Onda 11 antecipou. A decisão é a que a Onda 7 já tinha
escrito como condicional: *"se [o `Canvas`] sair, sai como vocabulário
declarativo (`<path>`, `<arc>`, `<circle>`)"*. Esta onda executa essa frase.

##### A observação

A Onda 7 pôs o `canvas` do `iced` no motor como **capacidade**
(`src/canvas.rs`) e decidiu, de propósito, **não** expor uma tag `<canvas>`: um
callback imperativo o `.gv` não lê, o `.gss` não estiliza e o Luau não alcança.
A decisão foi certa e continua certa — o que muda é que agora há um caminho que
não é o callback imperativo: um **vocabulário de formas**, cada uma um nó, cada
uma lida no render em Rust, do mesmo jeito que os quatro gráficos já são.

O `src/canvas.rs` da Onda 7 já constrói `canvas::Path` internamente para arco,
setor, linha e série — a caixa de ferramentas existe. O que falta é o parser
abrir os nós e o `.gss` alcançar `fill`/`stroke`/`stroke_width` num nó de forma.

##### O habilitador — o vocabulário de formas

- **`<canvas>`** como pai, e sob ele `<path d="…">`, `<arc>`, `<circle>`,
  `<rect>`, `<line>`, `<polyline>`, `<polygon>`, `<text>` — cada um um
  `NodeType`, cada um mapeado a um `canvas::Path` que o `src/canvas.rs` já sabe
  montar.
- **Coordenada e dado da forma são dado**, portanto **inline no `.gv`** pela
  regra do `CLAUDE.md` (o mesmo caso do `background="{cor}"`): `d="{traçado}"`,
  `points="{serie}"`, `cx="{x}"`.
- **Traço e preenchimento são estilo**, portanto **classe no `.gss`** com nome
  de papel: `.contorno-forte { stroke: var(--traço); stroke_width: 2; }`. É a
  divisão do `CLAUDE.md` aplicada a um alvo novo.
- **Sem `on_click` numa forma**, e a razão é a da Onda 7: um alvo de clique
  dentro de um `canvas` é geometria que o `.gv` não descreve. Se um dia sair,
  sai como outra decisão.

##### `QWhatsThis` de carona (sem relação com o vocabulário)

O mesmo padrão da Onda 9 (arrasto + teclado): dois itens pequenos, nenhum abre
rodada. O `QWhatsThis` é o `tooltip=` num modo pegajoso — `whats_this="…"` como
**atributo universal** (a forma do `tooltip=`), mais uma ação `whatsthis:on` que
liga uma chave do motor (`__whatsthis`, a família do `__band`). Com a chave
ligada, todo hover mostra o `whats_this` **preso até o clique**, em vez do
`tooltip` transiente. Reusa o overlay de tooltip inteiro. Leva a §2.12 a
**10/11** (sobra o `QScroller` 🟡, de propósito).

##### A conta da onda

| | ✅ | % |
|---|---|---|
| entra (depois da Onda 12) | 113 | 90,4% |
| o vocabulário de formas (`Canvas` ⬜ → ✅) | 114 | 91,2% |
| `QWhatsThis` de carona (⬜ → ✅) | **115** | **92,0%** |

**Categorias que fecham:** nenhuma inteira, mas a §2.6 vai a **14/15** (sobra o
`Shader`, wgpu) e a §2.12 a **10/11** (sobra o `QScroller` 🟡).

##### O que fica de fora, por escrito

Interação numa forma (`on_click`, hover), animação de forma (um tique de relógio
por `<canvas>`) e importação de SVG arbitrário como árvore de formas — o `<svg>`
da §2.6 já cobre o SVG como imagem, e virá-lo em nós editáveis é outro trabalho.
Os três podem sair depois; nenhum é pré-requisito do vocabulário básico.

##### A previsão, para ser conferida depois

**Onda 13 = 115 / 92,0%**, nenhuma correção de nível (o `Canvas` sempre esteve
catalogado como `Prim ⬜`, não como `●`), nenhum item novo do §3 — o `canvas` é
capacidade desde a Onda 7. O risco: o vocabulário de formas pode crescer além de
"sete nós e o `.gss` alcança o traço" se `<path d="…">` precisar de um parser de
comando SVG completo (curvas de Bézier, arcos elípticos, `A`/`Q`/`C`). Se for
esse o caso, a onda entrega o subconjunto reto (`M`/`L`/`Z` +
`<circle>`/`<rect>`/`<line>`) e as curvas ficam anotadas — o mesmo corte que a
série múltipla teve na Onda 11.

---
#### Onda 14 — o que o documento nunca prometeu — ⬜ **proposta (2026-09-09), pendente de decisão de escopo**

**Alvo: 115 → 118 de 125 (de 92,0% para 94,4%).**

As três linhas que sobram depois da Onda 13 são as que este documento marcou,
revisão após revisão, como **"outro projeto"**: `Shader` e `Q3D` (wgpu) e
`PrintDialog` (impressão). Esta onda só existe se a resposta à pergunta *"o
glacier-ui entra em GPU custom e impressão?"* for sim. Enquanto não for, ela
fica aqui como o teto declarado, não como fila.

##### Os três

| Linha | Hoje | Depois |
|---|---|---|
| `Shader` (§2.6) | ⬜ `Prim ●` | ✅ **Prim** — `<shader src="…">` sobre o `iced::widget::shader`, apontando um WGSL que o daemon carrega como já carrega `font(bytes)`. O `●` **fica honesto** aqui: um shader tem estado de GPU por instância, e este é o primeiro `●` do catálogo que não vira chave nomeada |
| `Q3D` (§2.13) | ⬜ `Comp ●` | ✅ **Comp** — barras/scatter 3D sobre `<shader>`, um irmão de `charts.rs` que renderiza para um primitivo de shader em vez do `canvas`. Depende do `<shader>` acima |
| `PrintDialog` (§2.10) | ⬜ `Diál ●` | ✅ **Diál** — `print{}` sobre `rfd`/SO, a forma da família `FileDialog` (suspensivo como `confirm()`), com `DialogOutcome`. **Zero linhas de diálogo novas** — o corpo em markup da Onda 8 é a porta, como seria para o `FontDialog`. É o mais barato dos três e o mais separável: não é wgpu, é integração de SO como os diálogos de arquivo que já existem |

##### A conta da onda

| | ✅ | % |
|---|---|---|
| entra (depois da Onda 13) | 115 | 92,0% |
| `PrintDialog` (⬜ → ✅) | 116 | 92,8% |
| `Shader` (⬜ → ✅) | 117 | 93,6% |
| `Q3D` (⬜ → ✅) | **118** | **94,4%** |

**Categorias que fecham:** a §2.6 (**15/15**, com o `Shader`) e a §2.13
(**6/6**, com o `Q3D`). A §2.10 vai a 14/15 (sobra o `FontDialog`, que é da Onda
10).

##### O que sobra depois da Onda 14 — e o teto real

Sete linhas, e nenhuma é "alguém decidiu fazer e não fez":

- **Onda 10, pulada a pedido** (3 ⬜ + 1 🟡): `FontSelect`, `FontDialog`,
  `TextBrowser` e o `PlainTextEditor` 🟡. Quando ela rodar, o catálogo vai a
  **122/125 — 97,6%**.
- **Três `🟡` de propósito**: `ScrollBar` (embutido no `scrollable`),
  `QStackedLayout` (é o `se`/`senao` e o `<stackview>`), `QScroller` (rolagem por
  gesto no `scrollable`). Nenhum vira ✅ sem deixar de ser o que é.

Ou seja: **97,6% é o catálogo tal como escrito, esgotado** — com a Onda 10
somada às três aqui propostas, o que sobra são os três `🟡` que são `🟡` de
propósito desde que foram escritos.

##### A previsão, para ser conferida depois

**Onda 14 = 118 / 94,4%** (isolada) ou **122 / 97,6%** com a Onda 10 junto.
Nenhuma correção de nível — o `●` do `Shader` e do `Q3D` é o primeiro do
catálogo que **continua `●` depois de construído**, porque GPU por instância não
é uma chave nomeada. É a onda que muda a promessa do documento, e por isso ela
espera um "sim" antes de virar fila.

---
#### Onde os habilitadores do §3 foram parar

O §3 lista nove itens de motor. Depois de distribuí-los pelas quatro ondas,
sobram **dois** — e nenhum dos dois bloqueia coisa alguma da fila:

| Habilitador (§3) | Onde ficou |
|---|---|
| `contains` no condicional | **Onda 4**, como pré-requisito de dois itens — ✅ feito na 0.84 |
| Nome dinâmico de slot | **Onda 5**, habilitador A — ✅ feito na 0.92, e era uma interpolação |
| Overlay ancorado genérico | **Onda 5**, habilitador B — ✅ feito na 0.92, como `iced::advanced::Overlay` (`src/anchored.rs`), não como generalização do `menu.rs` |
| `Grid` | **Onda 6**, item 1 — ✅ feito na 0.92 (`src/grid.rs`). Deixou de ser pré-requisito do `Calendar` (§2.5) e virou a ponta do mecanismo de medição |
| Binding a coleção (model/view) | **Onda 6** — ✅ e **nem existia como trabalho**: a ligação já era `items="chave"`, a mesma do `<menu>`. Do que estava catalogado sobrou a medição e as convenções de seleção/ordenação |
| `ctx.dispatch(acao)` | Continua P2 e continua sem consumidor urgente: o caso declarativo já se resolveu com o prefixo `app:` (0.63) |
| Contexto tipado / valor de data | **Fechado pela negativa** (0.72/0.73): o global `date` do prelúdio Luau cobre o lado do script, o `Instante` cobre o do widget. Sai da lista |
| **Canvas como primitiva** | **Onda 7** — ✅ feito na 0.93, e **não como primitiva**: virou capacidade do motor (`src/canvas.rs`), com sete tags declarativas por cima. Destravou `Dial`, `Gauge`, `LcdNumber` e a §2.13 inteira; o `ColorDialog` ficou de fora, e nada depende dele |
| **Estado por instância** | O último de pé, e o que sobrou dele é pequeno: `MdiArea`, `Dock`, `RangeSlider` e a edição de célula em árvore profunda. Rebaixado de P0 para P1 nesta revisão — não por ter encolhido, mas porque parou de ser o caminho crítico de qualquer coisa que se queira construir |
| Subscriptions de teclado | Continua P2, independente das quatro ondas (`Shortcut`/`Action` globais) |

A leitura que isso permitia, escrita antes das ondas 5 e 6: o item que este
documento chamou por três revisões de "o desbloqueio de maior alavancagem" **não
é o gargalo de nada** que se queira construir nas próximas quatro levas; o
gargalo real é a medição da Onda 6, e esse nunca esteve na lista do §3.

**As duas ondas confirmaram, e por uma margem maior do que a previsão.** O
estado por instância continua sem bloquear nada que se queira construir, e dos
cinco habilitadores que sobravam na lista, três foram feitos (dois deles muito
menores do que o catálogo dizia) e um — o binding a coleção, que a lista chamava
de "o maior investimento restante" — simplesmente **não existia**: a capacidade
já estava no motor desde o `<menu>`.

Do §3 sobra, depois da Onda 9, **um** item — e ele nunca bloqueou nada:

| Habilitador | Estado |
|---|---|
| ~~**Canvas como primitiva**~~ | ✅ Onda 7 (0.93). Era mesmo o de maior alavancagem: sete widgets, e a §2.13 saiu de 0/6 para 5/6 |
| `ctx.dispatch(acao)` | continua P2, continua sem consumidor urgente |
| ~~Subscriptions de teclado~~ | **Onda 9, habilitador B** — ✅ feito na 0.95 (`src/keys.rs`), e era mesmo uma sexta entrada numa lista de cinco `listen_with` que o daemon já registrava. Era o único item do §3 que ainda tinha consumidor no catálogo, e ele saiu com os dois (`Shortcut`/`Action` e `ShortcutInput`) |
| ~~**Estado por instância**~~ | **A Onda 9 desmontou o que sobrava dele**: `RangeSlider`, `Tumbler`, `Splitter`, `SwipeView`, `DelayButton` e `RubberBand` são arrasto sobre chave nomeada, como o `<dial>` e a alça de coluna. **A Onda 11 confirmou para `MdiArea`** ✅ (0.96): quatro chaves por janela, o mesmo `grip::Alvo::Ponto` generalizado para duas dimensões — a 17ª correção de nível. Sobra só `Dock`, e não por estado: por uma decisão de design que a Onda 11 não tomou (mudança de pai no meio de um arrasto, ver a onda na §6.2) — mais a edição de célula em árvore profunda. **Este item nunca foi o caminho crítico de nada**, e agora está escrito com a lista fechada |

E um **terceiro** fora do §3, achado na mesma revisão e pelo mesmo ponto cego:

| Habilitador (fora do §3) | Estado |
|---|---|
| **O arrasto como capacidade** | **Onda 9, habilitador A** — ✅ feito na 0.95 (`src/grip.rs`). O motor arrastava em três lugares que não conversavam — `__drag_key` (reordenar lista), `__colgrip` (alça de coluna) e o `Program::State` do `<dial>` — e nenhum dos três estava catalogado como capacidade. Foi a **terceira** vez que o gargalo real não estava na lista do §3, depois da medição de colunas (Onda 6) e do corpo do diálogo (Onda 8). Generalizá-lo foi tirar a conta de dentro do `__colgrip`: um `enum Alvo` com dois mapeamentos (trilha e índice), e os dois problemas difíceis — a âncora no primeiro movimento e o listener condicional — já estavam resolvidos e comentados no código |

E dois **novos**, que o §3 nunca listou porque não os enxergou como
capacidade — o mesmo ponto cego que a medição da Onda 6 revelou:

| Habilitador (fora do §3) | Estado |
|---|---|
| **Corpo e retorno do diálogo** | **Onda 8** — ✅ feito na 0.94: `DialogSpec.body` com o nome de um template, `DialogOutcome` no lugar do `bool`, e o prefixo `dialog:` para abrir do markup. Destravou `<dialog>`, `InputDialog`, `ProgressDialog`, `ColorDialog` e o `Wizard` — e o corpo custou **um parâmetro e uma chamada**, porque `render(nome)` já existia |
| **Famílias de fonte** | **Onda 10** (proposta em 2026-09-08, **pulada** a pedido) — era o **único bloqueio de motor que sobrou** no catálogo, e o que faltava nunca foi o trabalho: era um consumidor que valesse a rodada. São quatro, e um deles é um 🟡 que ninguém tinha ligado a isto. Registro de famílias, `font-family` no `.gss` e enumeração do SO. É o bloqueio real do `FontDialog` (§2.10) e do `FontSelect` (§2.4), que o catálogo atribui, os dois, ao widget errado — e tem uma decisão de §4 dentro (o `iced` não enumera as fontes do SO: ou entra uma crate, `fontdb`/`font-kit`, ou o app declara as famílias que usa) |
| **`<stack>`/`pin` como capacidade** | **Onda 11, habilitador A** — ✅ feito na 0.96 (`NodeType::Stack` em `parser.rs`/`widget.rs`). O `iced` expõe `stack`/`pin` desde sempre (§1 deste documento os lista) e o `widget.rs` usava `stack!` **uma vez, hardcoded**, no `<progressbar>` — nunca virou tag. `x=`/`y=` num filho usam `pin`; `anchor=` usa um `container` `Fill` alinhado, sem precisar de coordenada. Destravou `NotificationDot`, `SplashScreen` e o `MdiArea` |
| **`grip::Alvo::Ponto`, o arrasto em 2D** | **Onda 11, habilitador B** — ✅ feito na 0.96 (`src/grip.rs`), como extensão do habilitador A da Onda 9. O `Arrasto` ganhou uma segunda dimensão (`chave_y`/`origem_y`/`valor0_y`) e um terceiro `Alvo`, sem tocar nos dois existentes — os dois problemas difíceis (âncora no primeiro movimento, listener condicional) não mudaram, só passaram a ancorar/mover os DOIS eixos juntos. Único consumidor: `MdiArea` (mover a janela e redimensioná-la são o MESMO alvo, com limites diferentes) |

E o `on_enter`/`on_exit` no markup, anotado no fim da Onda 4 como "o que faria um
`Rating` builtin ser viável", continua sem consumidor: o hover do
`<autocomplete>` e o da tabela saíram pelo `Status::Hovered` do `button` do
`iced`, sem precisar de mensagem nenhuma. A Onda 9 não o pediu tampouco — quem
desenha o próprio arrasto num `canvas` recebe o cursor direto.

**O balanço final desta lista**, agora que ela fechou: o §3 catalogou nove itens
de motor ao longo de quatro revisões. Dois deles não existiam como trabalho (o
binding a coleção e o contexto tipado), um encolheu para uma linha (o nome
dinâmico de slot), um nunca bloqueou nada (o estado por instância — o item que o
documento chamou por três revisões de "o desbloqueio de maior alavancagem") e um
continua sem consumidor (`ctx.dispatch`). Os **três gargalos reais** — a medição
de colunas (Onda 6), o corpo do diálogo (Onda 8) e o arrasto (Onda 9) — não
estavam na lista, e os três eram capacidades que o motor já tinha em algum
canto sem ninguém as ter chamado de capacidade. É a lição mais cara e mais
repetida deste documento, e ela valeu **na mesma tarde em que foi escrita**: a
**Onda 11** é o quarto caso, e a segunda vez (depois do `Row::wrap()`) em que a
capacidade estava na biblioteca de baixo. O `iced` expõe `stack` e `pin` desde
sempre, a §1 deste documento os lista, o `widget.rs` usa `stack!` uma vez
hardcoded — e quatro linhas ⬜ do catálogo escrevem "Stack" na nota ou na coluna
"Base iced" sem que ninguém a tivesse lido como um item de motor.

### 6.3 A bandeja de troco (fica registrada, não abre rodada)

**A Onda 11 esvaziou esta bandeja quase inteira** (0.96): `QrCode`, `Chip`,
`Skeleton`, `CommandLink`, `NotificationDot` e `RoundButton` saíram, todos de
carona no habilitador A (`<stack>`) ou sozinhos — nenhum abriu rodada própria,
exatamente como esta seção sempre disse que devia ser.

Sobra **um**, e ele não podia sair aqui: o `TextBrowser` espera o registro de
famílias de fonte para o bloco de código, que é a **Onda 10** — proposta, não
executada (o usuário pediu para pular direto para a 11). Fica exatamente onde
estava, esperando a mesma coisa de sempre.

| Widget | Nível | Prio | Nota |
|---|---|---|---|
| `TextBrowser` | Built | P2 | Render read-only de markdown com links, sobre o `markdown` do iced — o bloco de código é o primeiro consumidor do registro de famílias da Onda 10 |
