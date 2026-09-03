# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

O crate está em **0.x**: pela convenção do Cargo, um bump de *minor* (`0.40` →
`0.41`) **pode quebrar API**, e é o que este projeto usa para mudanças
incompatíveis. Toda quebra vem listada em **Quebras** com o que fazer para migrar.

---

## [0.88.0] — 2026-09-03

### Adicionado
- **A CLI passa a gerar um `AGENTS.md`** em todo projeto novo (`glacier-cli`
  0.3.0), com o que a instrumentação da 0.87 mediu — em ordem de retorno, e com
  os números que justificam a ordem.

  O que ele diz, em resumo: **quase nunca é o motor.** Numa tela de 111 nós com
  20 caixas pintadas, pintar custou ~45 ms por quadro e o motor 0,07 ms — seis
  centenas de vezes menos. As três regras que pagam são não repintar a janela
  inteira por cima do tema, empilhar menos camadas na mesma área, e guardar o
  canto arredondado para as caixas pequenas.

  Traz também o procedimento de medida (`GLACIER_PERF` + `GLACIER_PERF_STRESS` +
  `GLACIER_NO_PAINT`) e as **duas armadilhas de leitura** que já custaram
  diagnósticos errados neste próprio repositório: julgar velocidade sem o modo
  estresse (um app ocioso apareceu como "quadro de 19 segundos") e comparar sem
  desligar a pintura.

  Vai no `_comum`, então todo preset o herda; um teste fixa isso.

### Notas
- Aplicado num app real (o `rustploy`), o item 1 sozinho removeu **nove** camadas
  de pintura redundante — a mesma cor do tema repintada por cima, em nove
  regras, com duas empilhadas no mesmo arquivo. Nenhuma mudança visual, e a
  rolagem melhorou de forma perceptível.

---

## [0.87.0] — 2026-09-03

Duas variáveis de diagnóstico — e, com elas, a resposta de uma investigação que
já tinha passado por seis hipóteses erradas.

### Adicionado
- **`GLACIER_PERF_STRESS=1`** — pede um quadro por vsync, para o relatório medir
  **capacidade** em vez de demanda. Sem isto, um app orientado a evento fica
  ocioso entre eventos e o intervalo medido é a espera, não o custo. É o buraco
  que fez a instrumentação apontar "travamento de 19 segundos" para um app
  parado.

- **`GLACIER_NO_PAINT=1`** — o render pula todo fundo, borda e canto
  arredondado. Separa "lento por número de nós" de "lento por área pintada",
  que nenhuma contagem distingue. A tela fica feia de propósito.

  Os dois juntos resolvem em dois minutos o que aqui levou muitas rodadas: rode
  com `STRESS`, anote o intervalo; rode com `NO_PAINT` junto e compare.

### O que eles mediram

Intel HD 2500 (Ivy Bridge, 2012), `examples/componentes_locais` — **111 nós, 20
caixas pintadas**, janela de 900×720:

| | intervalo por quadro |
|---|---|
| Com pintura | 84–111 ms |
| Sem pintura | 49–63 ms |
| Render do motor | **0,07 ms** |

As vinte caixas custam **~45 ms por quadro**: metade do total, e seiscentas
vezes o motor inteiro. Uma tela de 300 nós **sem** fundo roda mais rápido que uma
de 111 nós pintada — o custo é da **área** rasterizada, não do número de nós, e
as camadas se somam por sobreposição.

Isto não é limitação do motor: o mesmo custo aparece em `iced` puro com o mesmo
estilo. O que fazer está em `PRIMITIVAS.md` ("O que custa num quadro") — em
resumo: não pintar a janela inteira por cima do tema, menos camadas sobrepostas,
canto arredondado só nas caixas pequenas.

### Notas
- Vale registrar o erro que atrasou este diagnóstico: o app de controle em
  `iced` puro, usado para dizer "o motor não é o gargalo", **não pintava nada** —
  `container` com padding, sem fundo, sem borda. Comparava-se uma tela crua com
  uma pintada, e a conclusão ("é o hardware") saiu certa pelo motivo errado, o
  que fechou a porta para a causa real por várias rodadas.

---

## [0.86.0] — 2026-09-02

### Adicionado
- **`<component name="X">` dentro do `<resources>`: declarar um componente na
  própria tela.** A terceira forma de ter um componente — as outras duas trazem
  de um arquivo (`<import>`, `<link rel="component">`); esta não traz de lugar
  nenhum, o componente **é** o que está escrito entre as tags.

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

  **Por que.** A maior parte dos componentes de uma tela é pequena e só serve
  àquela tela — a linha de um item, o cabeçalho de um cartão, um rótulo com um
  `<badge>` do lado. Obrigar cada um a virar arquivo troca três linhas de markup
  por um arquivo, um caminho relativo e um `<import>`, e espalha por seis
  arquivos o que se lê melhor num. É a mesma razão de existir um `<style>`
  inline ao lado do `<link rel="stylesheet">`: a forma curta para o que é local,
  o arquivo para o que é compartilhado.

  **A casca é a mesma, de propósito.** O que vai entre `<component name="X">` e
  `</component>` é *byte a byte* o que iria num `.gv` próprio: `<props>` e
  depois o layout, montados pela **mesma** função
  (`parser::corpo_de_componente`, extraída do caminho de arquivo justamente para
  os dois partilharem). A única diferença é o `name` — no arquivo o nome vem do
  `<import>` que o traz; aqui ele precisa ser dito. Promover uma declaração a
  arquivo, ou o contrário, é recortar e colar.

  A mesma tag em dois papéis, e o `name` é o que os separa: na **raiz** do
  arquivo, `<component>` é o cabeçalho e não leva atributo nenhum; no
  `<resources>`, leva só o `name` — e sem ele é erro.

  **O que ele não tem:**
  - **`<script>` próprio** — não há arquivo contra o qual resolver um
    `src`/`require`, a mesma limitação do markup inline dos builtins. Na prática
    é o comportamento desejado: as ações escritas dentro de um componente local
    caem no `update` da **tela que o declarou**, que é de quem elas são. O id do
    item viaja dentro da ação, como no `<SpinBox>` (`on_click="detalhar:{s.id}"`).
  - **Escopo** — o nome entra no mesmo espaço de nomes de tudo o mais, com a
    mesma regra do `<import>`: declara se o nome está livre **ou** se hoje ele
    guarda um builtin da lib. Um componente registrado pelo app vence a
    declaração local; um builtin, não.

  Componentes locais se compõem entre si e convivem com `<import>` no mesmo
  `<resources>`. O hot-reload reescreve a declaração ao editar o arquivo — o
  motor guarda quais nomes vieram de uma declaração, e é isso que separa
  "reescrever a minha" de "atropelar o componente de verdade do app".

  Erros de parse, todos posicionados (o sintoma de deixá-los passar é
  silencioso — a tag simplesmente não existe): `<component>` sem `name` dentro
  do `<resources>`, `name` vazio, qualquer outro atributo, e declaração sem
  layout.

- **`examples/componentes_locais`** (`cargo run --example componentes_locais`) —
  três componentes declarados na tela (`Rotulo`, `LinhaLog`, `Metrica`, o
  segundo usando o primeiro) e um quarto vindo de arquivo por `<import>`, lado a
  lado no mesmo `<resources>`, para comparar as duas formas.

- **Extensão do VS Code (v0.12.0)**: completação e diagnóstico de props e
  **ir-para-definição** passam a enxergar um componente declarado no próprio
  documento — sem isso a tag ficaria sem metade do que o editor faz por um
  componente importado. O leitor de `<props>` virou um só, partilhado entre a
  forma de arquivo e a declarada, pelo mesmo motivo do lado do motor: ler cada
  uma com o seu parser é pedir para elas divergirem.

---

## [0.85.0] — 2026-09-02

**Onda 4 do `PLANO_WIDGETS.md`: os widgets que têm função.** Sete itens do
catálogo, e o que os une não é a aparência — é o `update`. Com a onda anterior,
o motor passa de metade do catálogo Qt de superfície pela primeira vez (53%),
tendo consumido **um** habilitador de motor no caminho (o `contains`).

### Adicionado
- **`<pagination>`** — `« ‹ 1 … 4 [5] 6 … 20 › »`. A janela de números anda com
  a página e **gruda nas pontas** (estar na página 1 mostra `1 2 3 4 5`, não
  `3 4 5 6 7`), as reticências só aparecem quando há mesmo algo escondido, e as
  setas ficam **inertes** no limite — o que o `<spinbox>`, por ser builtin, não
  consegue fazer.

  ```xml
  <pagination value="pagina" total="{total_paginas}" />
  <pagination value="pagina" total="{total}" window="3" ends="false" onChange="repaginar" />
  ```

  O widget conta **páginas**, não itens: recortar a lista é do app, porque só o
  app sabe o que é um item.

- **`<listview>`** — a lista rolável cujo item escolhido mora numa chave; o
  `<tabbar>` na vertical, com scroll. Fecha o 🟡 mais antigo da §2.4.

  ```xml
  <listview items="servicos" value="servico" selected="{servico}" />
  <listview items="servicos" value="marcados" selected="{marcados}" mode="multi" />
  ```

  `mode="multi"` guarda um **conjunto** numa chave só (`"api,cache"`) e é o
  primeiro consumidor do `contains` (0.84) — a demonstração de que a seleção
  múltipla nunca precisou de estado por instância, que é o que o
  `PLANO_WIDGETS.md` §3 dizia. Repassa `virtualize` para listas longas.

- **`<accordion>`/`<accordionitem>` e `<toolbox>`/`<toolboxitem>`** — os dois
  modos do mesmo widget: o accordion guarda um **conjunto** de seções abertas,
  o tool box guarda **uma** (e clicar na aberta a fecha).

  ```xml
  <accordion>
      <accordionitem title="Rede" value="abertas" open="{abertas}" id="rede">
          <input value="host" />
      </accordionitem>
  </accordion>
  ```

  **Uma tag por seção, e não uma coleção**, porque o *conteúdo* de cada seção é
  diferente — e nomes de slot são fixos no template do componente (0.67), então
  o widget não pode inventar um por item. É a mesma forma que o Qt usa:
  `QToolBox::addItem(widget, "Título")` também recebe uma seção por chamada.

- **`<buttonbox>`** — `QDialogButtonBox` como widget de tela. Fecha o outro 🟡
  da §2.1.

  ```xml
  <buttonbox accept="Salvar"  on_accept="salvar"
             reject="Cancelar" on_reject="cancelar"
             destructive="Excluir" on_destructive="excluir" />
  ```

  **A ordem é da plataforma, não da tela** — é a razão de o Qt ter um
  `QDialogButtonBox` em vez de um `QHBoxLayout` com dois botões. GNOME/macOS
  põem o afirmativo por último; Windows, primeiro. O widget escolhe por
  `cfg!(target_os)` **em Rust**, dentro do `template()` (que é uma função, não
  uma constante); `order="gnome"`/`"windows"` força. O destrutivo fica na ponta
  oposta em qualquer ordem, junto com o `<slot/>`.

- **`<maskedinput>`** — `QLineEdit` com `setInputMask`: guarda **cru** na chave,
  exibe mascarado. A mesma separação valor/`displayFormat` do `<dateedit>`, e
  pelo mesmo motivo: `"12345678901"` é o que um backend espera, `"123.456.789-01"`
  é o que uma pessoa lê.

  ```xml
  <maskedinput value="cpf" mask="cpf" />
  <maskedinput value="placa" mask="AAA#*##" />
  ```

  Gramática: `#` dígito, `A` letra, `*` alfanumérico; o resto é literal e entra
  **antes** do dado seguinte, nunca pendurado no fim. Presets: `cpf`, `cnpj`,
  `telefone`, `cep`, `placa`, `date`, `hora`, `cartao`. A dica default é a
  própria máscara com `_` no lugar dos símbolos.

- **`<rating>`** — a nota por estrelas, com pré-visualização ao passar o mouse
  (chave global `__rating`, como o hover do `<daterangepicker>`). Clicar na
  estrela já marcada **zera** a nota; `readonly="true"` desenha sem clique nem
  hover, que é o `Rating` de uma lista de avaliações.

- **`decimals` no `<spinbox>`** — fecha o `QDoubleSpinBox`. Sem a prop, as casas
  continuam saindo do `step` como escrito, que é o comportamento de sempre e
  acerta quase sempre; ele errava justamente onde importa: `step="1"` sobre um
  preço formatava `10`, não `10.00`.

- **`examples/onda4`** (`cargo run --example onda4`) — os sete em três abas.

### Notas de projeto

**Dois dos sete não eram builtins.** O plano marcava `Pagination` e `Rating`
como builtins; os dois viraram **primitivas**, pela mesma causa — e é a primeira
vez que uma causa de reclassificação se repete nesta base:

> **Repetição dirigida por um número, não por uma coleção.** O `for-each` do
> motor lê uma chave que guarda um array JSON. A janela `4 5 6` de uma paginação
> e as cinco estrelas de uma nota não existem em array nenhum — são *derivadas*
> de `pagina`/`total` e de `max`. A saída de builtin seria o app calcular o
> array e passá-lo por `items=`, que é exatamente o trabalho que estes widgets
> existem para poupar. Em Rust, é um `for` de uma linha.

A regra fica registrada no `PRIMITIVAS.md` e no `BUILTINS.md`, junto dos outros
dois sinais já conhecidos, porque ela **prevê**: `PageIndicator`, `Grid`
(`columns="3"`) e `Flow`/`Wrap` caem nela antes de alguém escrever uma linha
deles.

O `Rating` teve ainda um segundo motivo, independente e igualmente definitivo: o
**hover**. O motor expõe `on_press`, `on_double_click`, `cursor` e `tooltip` em
qualquer nó, mas não um `on_enter` — e a pré-visualização ao passar o mouse é
metade do que um `Rating` faz.

Com isso, são **seis** widgets que este documento classificou no nível errado
(`TimePicker`, `DateEdit`, `Calendar`, `Accordion`, `Pagination`, `Rating`),
sempre para o mesmo lado: o de superestimar o bloqueio.

### Compatibilidade
- O payload interno do `SpinBox` ganhou um quinto campo
  (`inc:qtd|1|99|1|<decimals>`). Ele pode vir **vazio**, e um payload de quatro
  campos continua sendo lido — um template que o app tenha copiado do widget
  não quebra.

---

## [0.84.0] — 2026-09-02

**Onda 3 do `PLANO_WIDGETS.md`: o calendário.** O foco declarado do projeto —
a §2.5 do catálogo — fecha em **6 de 6**, e fecha do jeito que o plano previu:
uma primitiva, três tags, **zero habilitadores de motor**.

### Adicionado
- **`<calendar>`, `<monthyearpicker>` e `<daterangepicker>`** — a grade do
  `QCalendarWidget`, o seletor de mês/ano e o de intervalo, todos o **mesmo**
  `NodeType::Calendar`, exatamente como `<dateedit>`/`<timeedit>`/
  `<datetimeedit>` são um `NodeType::DateTimeEdit` só:

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

  | tag | o que grava | chave |
  |---|---|---|
  | `<calendar>` | um dia | `YYYY-MM-DD` |
  | `<monthyearpicker>` | um mês | `YYYY-MM` |
  | `<daterangepicker>` | duas datas, em **duas chaves** | `start` e `end` |

  Props: `value`/`start`/`end` (nomes de chave), `onChange`, `today` (o realce
  de hoje), `min`/`max` (dias fora saem inertes), `mode` (`day`/`month`/`year`
  — onde o clique para de navegar e passa a gravar), `month` (chave que dirige
  o mês visível), `first_day`, `months`, `month_names`/`day_names`.

  **A escada de drill-up do Qt** está inteira: clicar no título sobe um degrau
  (dia → mês → ano), descer escolhe o mês/ano visível sem tocar na chave. O
  `<monthyearpicker>` é literalmente essa tela do meio promovida a tag — custo
  marginal zero, que é por que a linha dele subiu de P3 para P2 no plano.

  **O intervalo grava duas chaves separadas**, não `"a/b"` numa só: é o que
  deixa o `date.diff` do Luau ler as duas direto. Entre o primeiro clique e o
  segundo, a faixa sob o cursor é pintada — o hover mora numa chave global
  (`__cal_hover`) e é rastreado **só** enquanto há uma ponta aberta, para não
  cobrar uma mensagem por célula visitada no resto do tempo.

  **O mês visível não é global**, ao contrário do foco de seção do
  `<datetimeedit>`: ele mora em `__cal_<chave>`, derivado da chave editada, e
  duas grades na mesma tela navegam meses diferentes ao mesmo tempo. Sem
  `month=`, o app não configura nada.

- **`contains` no condicional** — o **simétrico** do `one_of` que já existia:
  lá a lista está no markup e o valor da chave é um item; aqui a lista está na
  **chave** e o item está no markup.

  ```xml
  <!-- ctx.abertas = "rede,disco" -->
  <template if="{abertas}" contains="rede"> … </template>
  ```

  É o que dá ao motor o **conjunto nomeado**: várias seções de um accordion
  abertas, uma seleção múltipla, um filtro por tags — coisas que o
  `PLANO_WIDGETS.md` §3 declarava presas ao estado por instância e que nunca
  estiveram. Vírgula, ponto-e-vírgula e espaço valem como separador ao mesmo
  tempo (quem monta o conjunto é código de app), e o item comparado também
  interpola — `contains="{s.id}"` dentro de um `for-each` é a forma que a Onda
  4 vai usar.

  Aliases: `contem`, `contém`, `has`, `inclui`.

- **`days_from_civil`/`dia_da_semana` no lado Rust** (`src/widget.rs`), oito
  linhas ao lado do `dias_no_mes` que o `Instante` já tinha. Com teste contra
  1970, 1900 e 2000: errar a regra do século desloca a grade inteira em
  silêncio, que é o pior modo de falha possível para um calendário.

- **`examples/onda3`** (`cargo run --example onda3`) — as três tags nas três
  abas, todo em Luau. E é em Luau por causa de **uma prop**: `today`. O realce
  de hoje é prop, não relógio — o motor não lê a hora do sistema em lugar
  nenhum, de propósito, e `date.today()` do prelúdio é a linha que fecha esse
  buraco sem uma crate de data no motor.

### Notas de projeto

Três coisas que o `PLANO_WIDGETS.md` afirmava e que a construção corrigiu:

1. O `Grid` (`QGridLayout`) **não era pré-requisito** do calendário. Ele só
   seria, para um *builtin*, cujo template é markup. Em Rust, grade é um `for`
   — é a quarta vez que este projeto descobre que um widget "bloqueado" era, na
   verdade, uma primitiva mal classificada (`TimePicker`, `DateEdit`,
   `Calendar`, `Accordion`).
2. `days_from_civil` **não existia** do lado Rust, ao contrário do que a §4
   dizia; existia só no `prelude.luau`.
3. As células dos meses vizinhos ficaram **inertes**, não
   esmaecidas-e-clicáveis: escolher um dia do mês seguinte teria de mover o mês
   visível *junto com* a escolha, e no modo que delega a escrita o widget não
   pode fazer as duas coisas numa mensagem só.

---

## [0.83.0] — 2026-09-02

### Alterado
- **O `GLACIER_PERF` diz quanto custa cada tipo de mensagem**, não só quantas
  chegaram:

  ```text
  msgs: LuauStream×8 152.3ms tot/19.1 pior  Scrolled×10 0.1ms tot/0.0 pior
  ```

  A contagem sozinha diz **quem** está pedindo quadro; o tempo diz **quanto
  custa**. Uma sem a outra engana nos dois sentidos — uma mensagem barata que
  chega cem vezes por segundo e uma cara que chega três vezes são problemas
  diferentes, com consertos diferentes, e no total agregado as duas somam igual.

  A lista sai ordenada pelo que mais **custa**, não pelo que mais aparece.

  O caso que motivou: um app alimentado por um stream SSE, em que `LuauStream`
  aparecia em toda janela lenta e o `dispatch` chegava a 19 ms — mas o relatório
  não permitia atribuir aquele tempo àquela mensagem, porque contagem e tempo
  eram números separados.

---

## [0.82.0] — 2026-09-02

Correção de **leitura**: o relatório do `GLACIER_PERF` media o intervalo entre
quadros e o apresentava como se fosse custo. Num app orientado a evento esse
intervalo é quase todo **espera** — sem mensagem não há quadro, e o relógio
corre.

### Corrigido
- **O relatório mostra o MIN do intervalo, e quantos quadros saíram colados.**

  ```text
  [glacier perf] 6 quadros 1.15s (5.2/s) | nós 301 | quadro MIN 15.8ms (5 colados)
                 | intervalo 192.5 méd 447.0 p95 447.0 máx [inclui ócio]
  ```

  O **MIN** é o menor intervalo da janela: o quadro que saiu colado no anterior,
  ou seja, o app trabalhando sem folga. Se ele for 16 ms, o app dá conta de
  sessenta por segundo e todo o resto é ócio; se for 200 ms, são 200 ms de
  trabalho. O `colados` diz de quantos quadros essa amostra é feita — com
  poucos, o MIN é frágil.

  A média e o p95 do intervalo continuam no relatório, agora rotulados
  `[inclui ócio]`, porque medem o quanto o app ficou **parado**, não o quanto
  ele demora.

  **O erro que isto causou**, e vale registrar: um app ocioso por dezenove
  segundos apareceu como "quadro de 19004 ms máx" e foi lido — por quem escreveu
  a ferramenta — como um travamento de dezenove segundos. Duas rodadas de
  diagnóstico saíram dessa leitura. A ressalva estava escrita na doc do módulo
  desde a 0.78; escrever a ressalva não impede de ignorá-la, um número que não
  se presta à confusão impede.

---

## [0.81.0] — 2026-09-02

A release em que a instrumentação achou um problema do **próprio motor** — e não
onde se procurava.

### Corrigido
- **O movimento do mouse deixa de forçar um quadro quando não há menu na tela.**

  O motor registrava um listener global de `CursorMoved`, sempre, só para ter a
  âncora de posição de um `<MenuBar>`/`<ContextMenu>`. No `iced`, **cada
  mensagem provoca um quadro**: com o listener ligado, atravessar a tela com o
  cursor fazia o app redesenhar cem, cento e cinquenta vezes por segundo, sem
  nada ter mudado.

  Medido num app real, com a mesma árvore de 301 nós:

  | `CursorMoved`/s | quadros/s |
  |---|---|
  | 63–70 | 60–66 |
  | 89–99 | 6,9–7,9 |
  | 111–147 | 9,3–12,4 |

  Agora o listener só é assinado quando alguma janela tem menu em jogo — um
  `<MenuBar>`, `<Menu>` ou `<ContextMenu>` na árvore avaliada, ou um menu já
  aberto. A varredura acontece junto da coleta que a 0.76 já fazia por árvore,
  sem passada nova. Nova consulta pública: `GlacierUI::precisa_do_cursor`.

  A âncora continua correta: para clicar num menu o cursor precisa chegar até
  ele, e o `subscription` é reavaliado a cada `update`, então o listener já está
  no ar quando o clique acontece.

### Adicionado
- **`GLACIER_PERF` ganha a parcela `app`**: o tempo dos ganchos do aplicativo
  (`GlacierDaemon::on_message`), que rodam na thread da UI logo depois do
  `dispatch`. Antes caíam no `resto` e eram lidos como custo do `iced` — o que
  esconde a pior categoria de travamento: um gancho que pega um lock disputado
  com outra thread para a UI por segundos sem gastar um microssegundo de render
  nem de dispatch.

  As quatro parcelas agora são `render`, `dispatch`, `app` e `resto`.

### Notas
- O que motivou: um app com **45 nós** na tela e o mouse parado registrou um
  quadro de **11,5 segundos** (média 356 ms, p95 30 ms — ou seja, um único
  travamento). Render 0,1 ms, dispatch 0,001 ms. Sem separar os ganchos do app
  do resto, não havia como saber se aquilo era `iced`, driver ou o próprio
  aplicativo.

---

## [0.80.0] — 2026-09-02

### Adicionado
- **`GLACIER_PERF` diz agora quais mensagens chegam**, e quantas de cada tipo:

  ```text
  msgs: CursorMoved×148  ToastTick×4  UiClick×2
  ```

  No `iced`, **toda mensagem provoca um quadro**. Uma tela parada que recebe
  cento e cinquenta mensagens por segundo está redesenhando cento e cinquenta
  vezes sem nada ter mudado — e numa GPU fraca é isso, e não o conteúdo, que
  satura o quadro. Sem saber *qual* mensagem é essa, o diagnóstico para no
  "alguma coisa está pedindo quadro demais".

  A motivação é um caso real: um app com **45 nós na tela** rendendo 3,6 quadros
  por segundo, com 67 a 155 mensagens por segundo e o `resto` (layout, texto,
  GPU) em 99,9% do quadro. Nem o `render` (0,1 ms) nem o `dispatch` (0,001 ms)
  explicavam; a contagem de mensagens é o próximo fio.

  `EngineMessage::nome()` (interno) dá o nome da variante para o relatório.

---

## [0.79.0] — 2026-09-02

Correção da ferramenta que a 0.78 acabou de entregar: ela media uma parcela e
chamava o resto de "fora do motor", atribuindo ao `iced` trabalho que era do
próprio motor e dos handlers do app.

### Alterado
- **`GLACIER_PERF` separa `dispatch` de `resto`.** O intervalo entre dois `view`
  não é só layout e desenho: ele inclui o `update` — o `dispatch` do motor, os
  handlers Luau, a reavaliação da árvore. Chamar tudo isso de "fora do motor"
  era apontar para o `iced` um custo que muitas vezes é do app.

  ```text
  [glacier perf] 58 quadros 1.00s 58.0fps | nós 1682 | quadro 17.2ms méd 21.0 p95 34.1 máx
                 render 0.43 méd 0.71 p95 | dispatch 1.20/quadro (14 msgs, 4.9 máx)
                 resto 15.6ms/quadro (90.7%)
  ```

  Agora são três parcelas, e cada uma aponta para um lugar diferente:

  | Parcela grande | Onde está o problema |
  |---|---|
  | `render` | o motor monta `Element` demais → menos nós (`virtualize`) |
  | `dispatch` | tratamento de mensagem: `update`, Luau, reavaliação |
  | `resto`, com árvore pequena | `iced`/`wgpu`: layout, texto, rasterização |

  Entraram também o **máximo** do quadro e a **mensagem mais cara** da janela:
  uma travada episódica não aparece na média nem no p95, e é justamente ela que
  faz o mesmo conteúdo render a 40 quadros por segundo num instante e a 5 no
  seguinte.

  As chamadas recursivas de `dispatch` (o repasse de ação, a reordenação) vão
  direto ao corpo interno, senão o tempo delas entraria duas vezes.

### Notas
- O que motivou: num app real, com **301 nós**, o relatório da 0.78 mostrava
  render de 0,2 ms e "fora do motor" de 100–190 ms, oscilando entre 40 e 5
  quadros por segundo com a mesma árvore. Custo de pixel não oscila assim — mas
  a ferramenta não sabia dizer se aquilo era `iced` ou `update`, porque somava
  os dois. Agora sabe.

---

## [0.78.0] — 2026-09-02

Release de **ferramenta**: o motor não mudou de comportamento, ganhou um jeito
de responder a pergunta que as três releases de custo anteriores só souberam
responder por inferência — *de um quadro lento, quanto é o motor e quanto é o
resto?*

### Adicionado
- **`GLACIER_PERF=1`**: um relatório por segundo no `stderr`, com o custo do
  render, quantos nós, e — por diferença — quanto do quadro **não** é do motor.

  ```text
  [glacier perf] 58 quadros em 1.00s = 58.0 fps  |  render 0.43ms méd, 0.71ms p95
                 nós 1682  |  motor 2.5% do quadro  |  fora do motor 16.8ms/quadro
  ```

  O motor cronometra a parte dele — percorrer a árvore avaliada e montar os
  `Element`. O que vem depois (medir o layout, moldar o texto, desenhar na GPU)
  acontece dentro do `iced` e do `wgpu`, fora do alcance daqui; mas o intervalo
  entre duas chamadas de `view` é o quadro inteiro, e o que sobra ao descontar o
  render é tudo que não é motor.

  **Por que isso faltava.** A 0.77 nasceu de um sintoma — uma tela rolando a
  poucos quadros por segundo — e o diagnóstico teve de ser montado de fora, com
  medidas sintéticas e eliminação: o render do motor era 2,6% do orçamento, o
  gancho do app 19 µs, a GPU era de 2012. Concluiu-se que o tempo estava no
  layout do `iced`, mas **por inferência**, não por medida. Esta variável mede.

  Ressalvas, que estão na doc do módulo: num app folgado boa parte do "fora do
  motor" é espera pelo vsync, não trabalho (compare rolando e parado); e com
  mais de uma janela as medidas se somam num relatório só.

  Desligada, custa a leitura de um `bool` já resolvido por quadro.

---

## [0.77.0] — 2026-09-02

**Virtualização de lista.** A quarta release de custo, e a primeira que sai de
um sintoma em produção em vez de um perfil: uma tela do `rustploy` com algumas
dezenas de cartões de serviço rolava a poucos quadros por segundo.

O perfil disse que o motor **não** era o culpado — montar os `Element` de 1.682
nós custa 431 µs, 2,6% do orçamento de 60 fps, e as micro-otimizações que
tentei antes moveram isso para 427 µs. O custo estava um andar abaixo: o `iced`
mede e desenha **todos** os filhos de um `<scrollable>`, inclusive os que estão
fora da tela. Nenhuma otimização do avaliador alcança isso; a única saída é não
entregar os invisíveis.

### Adicionado
- **`virtualize="<altura>"`** (`virtualizar`, `itemHeight`) numa coluna dentro
  de um `<scrollable>`: o motor monta só os filhos que caem na janela visível e
  põe, nas pontas, vãos do tamanho exato do que ficou de fora.

  ```xml
  <scrollable height="fill">
    <column spacing="12" virtualize="300">
      <ForEach items="servicos" var="s"> … </ForEach>
    </column>
  </scrollable>
  ```

  | Cartões | Render por quadro, sem | Com |
  |---|---|---|
  | 40 | 182 µs | 48 µs |
  | 80 | 364 µs | 47 µs |
  | 300 | 1,81 ms | **47 µs** |

  O custo do render vira **constante** — 300 itens custam o mesmo que 10 —, e o
  ganho maior é o que a tabela não mostra: o `iced` deixa de medir e desenhar
  290 cartões.

  **A altura é declarada, não medida.** Descobrir a altura real exige o layout,
  que é justamente o trabalho a evitar; é a troca que o `uniformItemSizes` do
  `QListView` faz. Errar o valor não quebra nada — só desalinha a barra de
  rolagem.

  **Três degradações seguras**, todas para "renderiza tudo, como antes": sem
  `<scrollable>` acima, com `virtualize="0"`, ou com a lista já cabendo inteira
  na tela (ali a virtualização só acrescentaria dois vãos).

- **`EngineMessage::Scrolled`**, e o deslocamento de cada `<scrollable>` guardado
  no motor. O `dispatch` grava e volta — **sem reavaliar**, que é a diferença
  entre rolar a 60 quadros e rolar a 6. O aviso de rolagem (`on_scroll` do
  `iced`) só é pendurado quando há uma coluna com `virtualize` logo abaixo:
  cada evento vira uma mensagem, que passa pelo `dispatch` e por qualquer gancho
  que o app tenha ali, e cobrar isso de quem não virtualiza seria piorar o caso
  comum.

### Alterado
- **`parse_length`, `font_for`, `parse_text_align` e `is_truthy` pararam de
  alocar.** Faziam `to_ascii_lowercase()` só para comparar sem diferenciar
  maiúsculas — uma `String` por atributo, por nó, por quadro; numa tela de 800
  nós a 60 fps, ~150 mil alocações por segundo. Trocado por
  `eq_ignore_ascii_case`. Ganho honesto: 431 → 427 µs. Real, grátis, e
  irrelevante perto da virtualização — fica registrado porque a tentação de
  medir o motor pelo que é fácil de otimizar foi exatamente o erro que o perfil
  desfez.

### Quebras
- **`widget::render_node` recebe um `RenderView`** — a sexta posição, com o
  deslocamento de cada `<scrollable>` e a altura da janela. Só alcança quem
  chama `render_node` direto; o caminho normal (`GlacierUI::render`) não muda.

### Notas
- A virtualização é de **render**, não de avaliação: a árvore inteira continua
  avaliada e na memória. Fazer a avaliação seguir a janela economizaria memória,
  mas exigiria reavaliar a cada rolagem — exatamente o que esta versão evita.
- A altura usada como "área visível" é a da **janela do app**, não a do
  `<scrollable>` (que só o layout do `iced` conhece, e perguntar seria
  circular). A conta erra por excesso: monta-se alguns itens a mais, nunca de
  menos.
- Só `Column` virtualiza. `Row` (lista horizontal) usa a mesma aritmética e
  entra quando houver caso de uso.

---

## [0.76.0] — 2026-09-02

A terceira release de custo, e a primeira guiada por **perfil** em vez de
suspeita. As duas anteriores atacaram o que se via lendo o código; esta rodou o
avaliador sob `callgrind` e foi atrás do que a contagem de instrução apontou —
que não foi o que se esperava.

O caminho medido é o mais comum de todos: uma mudança de estado que **não** toca
a lista da tela (mexer num contador, trocar uma aba, chegar um dado de outra
chave). Antes: 1.986M de instruções. Depois: **1.008M** — metade.

### Alterado
- **A lista deixou de ser reparseada a cada avaliação.** Uma coleção mora no
  contexto como texto (`"[{...},{...}]"`) e o `for-each` precisa dela como
  `Value`, então parseava a lista inteira — com um `BTreeMap` por objeto — a
  cada reavaliação, mesmo quando nada nela tinha mudado e **todos** os itens
  vinham do cache. Era **13,8%** de todo o trabalho, jogado fora em seguida.

  Agora o array parseado fica guardado no `EvalCache`, por chave de contexto. A
  validade é conferida **comparando o texto**, não um hash: um `memcmp` de 20 KB
  é ordens de grandeza mais barato que o parse, e não abre a porta que um hash
  de 64 bits abriria — servir a lista velha por colisão, em silêncio.

- **Os mapas internos do avaliador trocaram o SipHash pelo FxHash.** O hash da
  `std` é escolhido para resistir a colisão hostil em mapa alimentado por
  entrada de rede; aqui as chaves são nomes de variável do próprio app e
  caminhos de nó gerados pelo motor. Ele respondia por **18%** do trabalho — com
  o caso mais absurdo sendo hashar um `u64` (o caminho de uma entrada de cache)
  com SipHash.

  São ~30 linhas em `eval.rs`, sem dependência nova. Trocado no cache de
  avaliação, no rastreador de leituras, no cache de JSON acima e **no contexto**
  (ver Quebras).

- **`Reads::record` parou de alocar a cada leitura.** A mesma chave é lida
  muitas vezes por subárvore, e o `entry` da `std` exige a chave *possuída* —
  alocava uma `String` a cada leitura só para descobrir que já havia uma igual.
  Agora consulta antes de inserir: um segundo hash (barato, com o Fx) no lugar
  de uma alocação.

- **As duas varreduras completas da árvore viraram uma, e só quando a árvore
  muda.** A cada reavaliação, o motor percorria a árvore inteira duas vezes —
  uma atrás de `<TextArea>`, outra atrás de `<ComboEdit>` — para sincronizar os
  buffers desses widgets. Quase sempre para não achar nada: a tela típica não
  tem nenhum dos dois. Eram **5,3%**.

  A coleta passou a acontecer **uma vez, junto da avaliação que produziu a
  árvore** (`GlacierUI::tree_bindings`), numa passada só para os dois tipos. Uma
  árvore reaproveitada não é varrida de novo.

### Números

`tests/perf_arvore.rs`, `--release`, mesma máquina. A coluna da 0.73 é a linha de
base de antes das três releases de custo:

| N linhas | acerto de cache | reavaliação completa | memória do processo |
|---|---|---|---|
| 100 | 97 µs (0.75: 242 · 0.73: 834) | 1,09 ms (1,22 · 2,03) | 0,7 MB |
| 500 | **545 µs** (0.75: 1.300 · 0.73: 5.590) | 6,15 ms (6,74 · 12,2) | 4,0 MB (3,5 · 9,8) |
| 2000 | **4,46 ms** (0.75: 10,06 · 0.73: 26,6) | 31,8 ms (33,7 · 53,8) | 15,2 MB (13,9 · 33,4) |

O acerto de cache ficou **2,3× mais rápido que a 0.75** e **6,0× mais rápido que
a 0.73** numa lista de 2000 linhas; numa de 500, **10,3×** contra a 0.73.

**O preço, e ele é real:** guardar a lista parseada custa memória. Numa lista de
2000 linhas o processo subiu de 13,9 para 15,2 MB (+9%) — o array de `Value`
retido, mais uma cópia do texto para a comparação exata. Continua **2,2× abaixo**
da 0.73 (33,4 MB), e a troca é 1,3 MB por 2,3× no caminho mais percorrido do
motor. Se um dia incomodar, o lugar de mexer é a política de descarte do
`EvalCache::json`, que hoje só limpa quando a época avança.

### Quebras

Uma só, e é de **tipo**, não de uso.

- **O contexto virou `glacier_ui::ContextMap`** — o mesmo `HashMap<String,
  String>` de sempre, com o hasher rápido no terceiro parâmetro. Isso alcança
  `GlacierUI::context()`, `eval::process_template` e `EvalCtx::new`.

  `get`, `contains_key`, `insert`, `iter` e companhia **não mudam** — todos os
  52 usos de `.context()` nos testes deste repositório passaram sem edição. Só
  quebra quem **anota o tipo à mão**:

  ```rust
  // antes                                     // agora
  let ctx: &HashMap<String, String> = ...      let ctx: &ContextMap = ...
  fn f(c: &HashMap<String, String>) { }        fn f(c: &ContextMap) { }
  ```

  E, num literal, `HashMap::new()` vira `HashMap::default()` (o `new` só existe
  para o hasher da `std`).

### Notas
- **O que o perfil ainda mostra**, para quem continuar: `EvalCtx::lookup` é
  agora o item isolado mais caro (17%), e é o preço de a resolução de uma chave
  percorrer as camadas antes de cair no mapa. Depois dele vêm as comparações de
  string (6,7%) e a cópia de nós no acerto de cache (5%).
- Continua fora a **virtualização** da lista, pelo mesmo motivo das duas
  anteriores — é funcionalidade, não otimização. Ver `PLANO_WIDGETS.md`, Onda 6.

---

## [0.75.0] — 2026-09-01

A continuação direta da 0.74, e o item que ela mesma deixou anotado como "a
conta que sobra": numa lista, a camada de variáveis de **cada** item era
montada **antes** da consulta ao cache — ou seja, também para o item que ia ser
reaproveitado inteiro e jogada fora em seguida.

### Alterado
- **A camada de um item de `for-each` ficou preguiçosa.** O item permanece como
  JSON e cada `{item.campo}` se resolve na leitura, não na montagem.

  A versão ansiosa fazia, por item e por reavaliação: um `format!("{var}.{k}")`
  e um clone do valor para **cada** campo, mais uma serialização do item inteiro
  para o `{item}` que um `spread=` repassa. Numa lista de 2000 linhas de 4
  campos, 8000 `format!` + 8000 clones + 2000 serializações — em boa parte
  descartados no `continue` do acerto de cache, três linhas abaixo.

  Agora um campo de **texto** — o caso esmagadoramente comum — sai emprestado do
  próprio JSON, sem alocar nada. O que não é texto (número, booleano, o item
  inteiro) materializa uma vez numa `OnceCell` e fica.

  **Por que não bastava consultar o cache antes de montar a camada**, que era o
  conserto óbvio: as dependências de um item incluem os campos dele
  (`("l.nome", "Ana")`) — é exatamente isso que faz o cache perceber que a linha
  3 mudou —, e validá-las exige a camada montada. A preguiça resolve os dois
  lados: a validação lê o que precisa, e só isso.

### Corrigido
- **Menções a KDL na documentação viva.** O motor não parseia KDL — não há
  dependência nem parser —, mas `src/component.rs` e `src/asset_source.rs` ainda
  anunciavam `.kdl` ao lado de `.gv`. As entradas antigas do changelog ficam
  como estão: são registro do que era verdade quando foram escritas.
- **A coluna `RSS` do `tests/perf_arvore.rs` virou `processo`.** Era *Resident
  Set Size*, a memória residente do processo, mas a sigla se lê como nome de
  formato de arquivo ao lado de `.gv` e `.gss` numa tabela. Mesma correção na
  tabela de números da 0.74.

### Adicionado
- **Sete testes que fixam a semântica da camada preguiçosa** (`engine_tests`):
  campo de texto, escalares que não são texto (número/booleano/nulo/decimal),
  o item inteiro de um objeto, o item escalar, campo inexistente, e duas
  armadilhas que só existem porque a resolução virou sob demanda:

  - **prefixo não é campo**: com `var="l"` e um item que tem um campo `ista`, a
    chave de contexto `{lista}` não pode ser lida como `l` + `ista`. O corte de
    prefixo exige o ponto, e o teste vigia isso com o campo homônimo montado de
    propósito.
  - **o cache por item continua correto**: mudar o nome do item do meio de uma
    lista de três muda aquele item e só ele.

  Os dois foram **verificados por mutação**: quebrando o corte de prefixo, e
  fazendo a entrada de cache deixar de listar os campos do item como
  dependência (o erro clássico de quem resolve sob demanda e esquece de
  validar), cada um derruba o seu teste. Um teste que passa dos dois jeitos não
  estava protegendo nada — foi o que aconteceu na primeira versão deles.

### Números

Mesmo aparelho (`tests/perf_arvore.rs`, `--release`), 0.74 → 0.75, e a coluna
de referência da 0.73 para o acumulado das duas releases:

| N linhas | acerto de cache | reavaliação completa |
|---|---|---|
| 100 | 358 → **242 µs** (0.73: 834 µs) | 1,33 → **1,22 ms** (0.73: 2,03 ms) |
| 500 | 1,95 → **1,30 ms** (0.73: 5,59 ms) | 7,60 → **6,74 ms** (0.73: 12,2 ms) |
| 2000 | 12,8 → **10,1 ms** (0.73: 26,6 ms) | 37,7 → **33,7 ms** (0.73: 53,8 ms) |

O acerto de cache — o caminho de uma mudança de estado que não mexe na lista —
ficou **2,6× mais rápido que a 0.73** numa lista de 2000 linhas, e **4,3× numa
de 500**. Memória não muda nesta release: a 0.74 já tinha feito essa parte.

### Notas
- Continua fora a **virtualização** da lista, pelo mesmo motivo da 0.74: é
  funcionalidade, não otimização (pede realimentação de rolagem, que o motor não
  plumba), e o lugar dela é junto do `TableView` — Onda 6 do `PLANO_WIDGETS.md`.
  Depois destas duas releases, uma lista de 2000 linhas custa ~10 ms por
  mudança de estado que não a toca; é ela quem tira isso de vez.

---

## [0.74.0] — 2026-09-01

Release de **custo**: o motor não ganhou nenhum widget nem nenhuma sintaxe — a
árvore avaliada passou a ocupar menos da metade da memória e a reagir a uma
mudança de estado no dobro da velocidade. Saiu de uma pergunta direta: *"essa
abstração sobre o iced faz o app consumir muito mais memória? compromete o
desempenho?"*.

A resposta honesta era **em parte sim**, e a medição apontou dois culpados
estruturais — nenhum deles no lugar onde se suspeitava. Montar os `Element` do
iced a partir da árvore (o `view()`, que roda por frame) sempre foi barato:
~2× o mesmo layout escrito à mão, dentro do orçamento de 60 fps com folga. O
caro era o `dispatch`, que reavalia a árvore a cada mensagem.

### Adicionado
- **`tests/perf_arvore.rs`**: o aparelho de medida, `#[ignore]` para não medir
  ruído no `cargo test` comum. Uma tela realista (lista de pedidos, 7 nós por
  linha) em quatro tamanhos, comparada com o mesmo layout montado à mão em iced.

  ```sh
  cargo test --release --test perf_arvore -- --ignored --nocapture
  ```

### Alterado
- **O `UiNode` encolheu de 1264 para 512 bytes.** Ele tinha 37 campos
  `Option<String>` no corpo, e um `Option<String>` custa 24 bytes **esteja ou
  não preenchido**: 888 bytes por nó reservados para atributos que um `<Text>`
  usa três ou quatro. Os raros foram para **grupos em caixa**, alocados só
  quando algum campo do grupo é usado:

  | Grupo | Campos | Quando existe |
  |---|---|---|
  | `Look` | `align_x` `align_y` `border_color` `font` `gradient` `text_align` `text_color` `slot_name` | aparência de segunda ordem |
  | `Interact` | `on_press` `on_double_click` `cursor` `tooltip` `tooltip_position` | interação de ponteiro |
  | `Cond` | `if_cond` `if_equals` `if_not_equals` `if_one_of` `if_platform` `else_if_cond` `for_each` `for_each_var` | nós de diretiva |
  | `Drag` | os seis `drag_*`/`*reorder*` | itens de lista reordenável |
  | `FormBits` | `form_control` `form_scope` `form_submit_action` `form_next_focus` | entradas dentro de um `<Form>` |
  | `Pseudo` | `hover_style` `focus_style` `active_style` `disabled_style` | nós com `:hover`/`:focus`/… resolvido |

  Os campos **quentes** (`width`, `height`, `padding`, `background`, `class`,
  `id`, os numéricos, os booleanos) ficaram onde estavam: eles são preenchidos
  o tempo todo, e uma indireção só os deixaria mais lentos sem economizar nada.

- **Os filhos de um nó passaram a ser compartilhados por contagem de
  referência** (`children: Children`, um `Arc<Vec<UiNode>>` com `Deref` e
  `IntoIterator`). O motivo é o cache de avaliação: um **acerto** de cache
  devolvia a subárvore guardada por **cópia profunda** — a árvore inteira
  memcpy'ada nó a nó, a cada mudança de estado, mesmo quando nada nela tinha
  mudado. Era o pior tipo de custo: pago justamente no caminho que existe para
  não pagar nada.

  Agora clonar um nó é um incremento de contador para tudo abaixo dele, e a
  árvore avaliada **divide** memória com a entrada de cache em vez de duplicá-la
  — que é de onde vem a maior parte da queda de memória residente. A escrita
  continua correta
  e continua possível: `Children::to_mut()` é copy-on-write (`Arc::make_mut`).

- **As dependências de uma entrada de cache também são compartilhadas**
  (`Arc<Deps>`). O acerto clonava o vetor inteiro de pares de `String` só para
  contornar o empréstimo do `&mut` no cache — trabalho puro de borrow checker,
  invisível, pago uma vez por item de lista.

### Números

Medidos com `tests/perf_arvore.rs` em `--release`, mesma máquina, 0.73 → 0.74:

| N linhas | nós | árvore | processo | render/frame | reavaliação | idem, c/ cache |
|---|---|---|---|---|---|---|
| 25 | 179 | 324 → **166 KB** | 2,9 → **2,6 MB** | 55 → 56 µs | 598 → **411 µs** | 200 → **108 µs** |
| 100 | 704 | 1281 → **653 KB** | 1,5 → **0,6 MB** | 199 → 202 µs | 2,03 → **1,33 ms** | 834 → **358 µs** |
| 500 | 3.504 | 6229 → **3188 KB** | 9,8 → **3,6 MB** | 1,33 → **1,14 ms** | 12,2 → **7,6 ms** | 5,59 → **1,95 ms** |
| 2000 | 14.004 | 24,9 → **12,7 MB** | 33,4 → **13,9 MB** | 7,27 → **5,98 ms** | 53,8 → **37,7 ms** | 26,6 → **12,8 ms** |

Em uma linha: **memória pela metade, acerto de cache 2,1× a 2,9× mais rápido,
reavaliação completa 1,4× mais rápida.** O `render` por frame mudou pouco
porque nunca foi o problema — ele continua em ~1,8× o iced escrito à mão, o que
para uma tela de 500 nós é 1,1 ms de um orçamento de 16,67 ms.

### Quebras

Só para quem lê ou escreve campos do `UiNode` **em Rust** (um componente que
inspeciona a árvore). Markup `.gv`, `.gss` e Luau não mudaram em nada — os
atributos são exatamente os mesmos.

- **Os 36 campos dos grupos acima viraram acessor.** A leitura devolve
  `Option<&str>` (antes `&Option<String>`), e a escrita ganhou um `set_`:

  ```rust
  // antes                          // agora
  node.on_press.as_deref()          node.on_press()
  node.tooltip.clone()              node.tooltip().map(str::to_string)
  node.font.is_some()               node.font().is_some()
  node.align_x = Some(v)            node.set_align_x(Some(v))
  ```

  Os grupos são públicos (`parser::{Look, Interact, Cond, Drag, FormBits,
  Pseudo}`) para quem preferir mexer neles direto.

- **`UiNode::children` é `Children`, não `Vec<UiNode>`.** Leitura não muda
  (`len()`, indexação, `iter()`, `for c in &node.children` continuam iguais, por
  `Deref` e `IntoIterator`). Muda quem escreve ou consome:

  ```rust
  // antes                          // agora
  node.children.push(x)             node.children.to_mut().push(x)
  node.children.iter_mut()          node.children.to_mut().iter_mut()
  for c in node.children { }        for c in node.children.into_vec() { }
  UiNode { children: v, .. }        UiNode { children: v.into(), .. }
  ```

- **`eval_condition` e os ajudantes `font_for`/`parse_text_align`/
  `parse_alignment` recebem `Option<&str>`** no lugar de `&Option<String>`.
  Internos, listados por completude.

### Notas
- **O que não foi feito, e por quê.** A terceira ideia da análise era
  **virtualizar** a lista — avaliar só as linhas visíveis, o que é o único
  conserto de verdade para 2000 linhas. Ela não entra aqui porque não é uma
  otimização, é uma funcionalidade: precisa de realimentação de rolagem
  (`scrollable::on_scroll`, que o motor não plumba hoje), de uma decisão sobre
  altura de item e de conviver com o arrasto de reordenação. O lugar dela é
  junto do `TableView` (Onda 6 do `PLANO_WIDGETS.md`), não antes de um release.
- **A conta que sobra**, para quem for continuar: numa lista, o `item_layer`
  monta a camada de variáveis de **cada** item (um `format!` e um
  `json_scalar` por campo, mais uma serialização do item inteiro) **antes** de
  consultar o cache — ou seja, mesmo quando o item vai ser reaproveitado.
  Inverter as duas coisas é o próximo ganho grande, e não foi feito agora porque
  mexe na ordem em que as dependências de uma entrada são validadas — o tipo de
  mudança que erra silencioso, servindo tela velha, se for feita com pressa.

---

## [0.73.0] — 2026-09-01

### Adicionado
- **O `date` passa a falar RFC 3339, e ganha a camada de fuso.** A 0.72 entregou
  um módulo *naive*, que resolve data e hora de tela mas não a fronteira com um
  backend — e um backend fala `2026-07-06T12:34:56Z`. Agora toda entrada aceita
  o separador `T`, sufixo de fuso (`Z`, `±HH:MM`, `±HHMM`, `±HH`) e fração de
  segundo (aceita e descartada, porque o motor guarda com resolução de segundo).

  Quatro funções novas, e a conversão é **explícita**: nenhuma outra desloca um
  instante sozinha.

  | | |
  |---|---|
  | `date.epoch(iso)` | segundos desde 1970 |
  | `date.from_epoch(secs, utc?)` | `YYYY-MM-DD HH:MM:SS` |
  | `date.to_local(iso)` / `date.to_utc(iso)` | o mesmo instante na outra hora de parede |

  ```lua
  ctx.inicio = date.format(date.to_local(dep.started_at), "DD/MM HH:mm")
  ```

  A convenção que amarra tudo: **sem fuso é hora local** — é o que `today`/`now`
  devolvem e o que os campos de edição guardam —, e por isso
  `date.epoch(date.now())` bate com `os.time()`. O offset viaja na forma do
  valor, então `add` o preserva: `date.add("…T12:34:56Z", { days = 1 })` devolve
  `"…T12:34:56Z"`, com o `T` e o `Z` no lugar.

### Alterado
- **`compare`, `diff` e `diff_seconds` normalizam para a hora local** antes de
  comparar, para que um `...Z` vindo do backend e um `date.today()` da tela
  sejam comparáveis sem conversão na chamada. Entre dois valores naive a
  resposta não muda — é o caso que a 0.72 já cobria.

### Corrigido
- Nada no motor, mas vale como aviso a quem escreve Luau: **o `os.time` do Luau
  usa `timegm`, não `mktime`**. Uma tabela de componentes é lida como UTC, e por
  isso o truque clássico de descobrir o offset local —
  `os.difftime(t, os.time(os.date("!*t", t)))` — devolve **zero** no dialeto,
  silenciosamente. O que funciona é `os.time(os.date("*t", t)) - t`, e é o que o
  `date` usa. Encontrado ao avaliar a adoção da 0.72 num app real, que carregava
  a versão do Lua desse cálculo há tempos sem sintoma visível (o zero era
  inofensivo lá, porque o `os.date` seguinte já fazia a conversão sozinho).

---

## [0.72.0] — 2026-09-01

### Adicionado
- **`date`: aritmética de data e hora no Luau, sem dependência nova.** Novo
  global do prelúdio (`src/luau/prelude.luau`) que opera sobre as **strings
  ISO** que os `<dateedit>`/`<timeedit>`/`<datetimeedit>` já gravam na chave de
  contexto — recebe string, devolve string, e por isso nada de "objeto data"
  vaza para uma chave (que é sempre texto).

  ```lua
  ctx.entrada = date.today()                          -- "2026-09-01"
  ctx.saida   = date.add(ctx.entrada, { days = 2 })   -- "2026-09-03"
  ctx.noites  = date.diff(ctx.saida, ctx.entrada)     -- 2
  ctx.rotulo  = date.format(ctx.entrada, "DD/MM/YYYY")
  ```

  Relógio: `today`, `now(segundos?)`, `time(segundos?)`. Leitura: `parse`,
  `valid`, `weekday` (1 = domingo, a base do `os.date("*t").wday`), `date_of`,
  `time_of`, `days_in_month`. Comparação: `compare`, `is_before`, `is_after`.
  Aritmética: `add`, `diff` (dias de calendário), `diff_seconds`. Exibição:
  `format` (`YYYY` `YY` `MM` `DD` `HH` `mm` `SS`).

  Três decisões que valem saber:

  - **Comparar é pelo instante, não pelo texto.** ISO ordena como string, mas só
    entre valores da mesma forma: `"2026-09-10 08:00" > "2026-09-10"` é
    verdadeiro só porque a string é mais longa, ainda que os dois sejam o mesmo
    dia. `is_after`/`compare` tiram isso da frente — uma data pura vale a
    meia-noite dela.
  - **`add` preserva a forma da entrada.** Somar um dia a um `YYYY-MM-DD` não
    inventa `00:00` no fim; somar uma hora a um `HH:MM` vira dentro do dia,
    porque não há data para onde transbordar. `months`/`years` andam pelo
    calendário e grudam no fim do mês (31/01 + 1 mês = 28/02, como o `QDateEdit`
    ao trocar a seção do mês); `days`/`hours`/`minutes`/`seconds` são duração.
  - **O parse é estrito**, ao contrário do `Instante` do widget: entrada
    inválida — inclusive uma data que não existe, como `2026-02-31` — devolve
    `nil`. O widget é tolerante porque não pode renderizar quebrado enquanto a
    pessoa digita; um script pode escolher o que fazer com o `nil`.

  Tipos em `views/scripts/glacier.d.luau` (templates da CLI) e `date` na lista
  de globais do `.luaurc`.

### Decidido
- **`chrono` vs. `time` (PLANO_WIDGETS.md §4): nenhuma das duas.** A pergunta
  estava aberta desde a 0.68 e a resposta é que o motor não precisa de crate de
  data. O lado Luau é o `date` acima — `today`/`now` saem do `os.date` que o
  dialeto Luau já tem (o sandbox tira `io` e `os.execute`, não o relógio), e o
  resto é `days_from_civil`. O lado Rust já tinha o `Instante`
  (`src/widget.rs`), cuja semântica é *anti*-calendário de propósito (cada seção
  vira dentro de si, sem carry) — exatamente o oposto de um `NaiveDateTime`.

  Isso destrava o `Calendar` da Onda 4, que se acreditava bloqueado por esta
  decisão: dia da semana é `days_from_civil`, não crate. Sobra um único caso
  para reabrir a pergunta, se um dia aparecer: o *offset local* em Rust (a `std`
  só dá epoch UTC). Se precisar, é **`chrono`** — o `time` devolve `Err` em
  `now_local()` dentro de processo multithread no Unix, e o motor é tokio.

### Alterado
- **`examples/data_hora_luau` reescrito sobre o `date`.** O script carregava um
  `days_from_civil` escrito à mão (e um recorte manual de `YYYY-MM-DD` para
  comparar formas diferentes) só para o exemplo não ser o que forçava a escolha
  da crate; nada disso é mais necessário. Os valores iniciais passam a sair do
  relógio em vez de datas fixas, entraram duas regras que só existem porque o
  script sabe que dia é hoje, e dois botões novos (`+1 noite`, `Adiar 1 mês`)
  mostram `date.add` andando pelo calendário.

---

## [0.71.0] — 2026-09-01

Release de ferramenta: **o motor não mudou** — `src/` é byte a byte o da 0.70.0.
A versão sobe para que a extensão de VS Code e a CLI que a embute tenham um
número de motor correspondente para apontar.

### Adicionado
- **A extensão Glacier View fecha a tag sozinha** (v0.9.0). Terminar uma
  abertura com `>` escreve o par e deixa o cursor no meio: `<Column>` vira
  `<Column></Column>`. Vale para componente do app também.

  Fica de fora o que o motor lê como folha — `<Image>`, `<Badge>`, `<Radio>`,
  `<Slider>`, `<TextInput>`, `<MenuItem>`, … —, que o markup do projeto inteiro
  escreve com `/>`: fechar `<Image src="a.png">` num par vazio seria devolver
  lixo para apagar. Também não dispara em `</x>`, `<x/>`, `>` dentro de valor de
  atributo (`title="a > b"`), `>` solto em texto, tag em comentário, nem dentro
  do corpo de um `<script>`/`<style>`.

  Desliga em `glacierView.autoClosingTags`.

### Quebras
- **O par `<`/`>` saiu de `autoClosingPairs`** da extensão. Com o `>`
  auto-inserido, digitá-lo apenas sobrescrevia o caractere e o editor não
  emitia mudança nenhuma — era ele ou o fechamento de tag. Na prática não se
  perde nada: o `>` agora vem junto do `</tag>`.

---

## [0.70.0] — 2026-09-01

### Adicionado
- **O `<dateedit>`/`<timeedit>`/`<datetimeedit>` ganhou teclado.** A 0.68
  entregou a edição por seções mas só no mouse — "não dá para digitar no campo"
  era uma limitação declarada. Com uma seção selecionada:

  | tecla | o que faz |
  |---|---|
  | ▲ / ▼ | mesmo passo dos botões de seta, na seção selecionada |
  | ← / → | troca de seção, sem alterar valor |
  | `0`–`9` | digita na seção, com o avanço automático do Qt |

  Digitar `0930` numa hora atravessa hora e minuto sozinho: cada seção **avança
  quando enche** (`09`) ou quando nenhum próximo algarismo caberia (`5` numa
  hora — não existe `5X` válido). Um engano recomeça a seção em vez de ser
  recusado, que é o que deixa corrigir sem apagar.

  O algarismo sai do **texto que a tecla produz**, não do código dela, então o
  teclado numérico e um layout não-ABNT entram igual.

  **Como funciona, e por que não é um widget focável.** A seleção já morava numa
  chave global (`__timeedit`); ela passou a carregar também a *configuração da
  instância* (quais seções existem, a ordem, e o `onChange`), porque quem trata
  a tecla é o `update` do motor — que recebe a tecla solta, sem o nó em mãos.
  O contrato de gravação não muda: sem `onChange` o widget grava a chave
  sozinho, com ele delega.

  **O guarda contra roubar teclas:** o listener é global e não sabe o que está
  focado, então ele só age quando **nenhum widget consumiu o evento**
  (`event::Status::Ignored`). Um `<TextInput>` focado captura os algarismos e o
  ← →, e eles não chegam aqui. Além disso, clicar em qualquer outro widget
  larga a seção selecionada.

  **Limite conhecido, e é honesto declarar:** com um `<TextInput>` de linha
  única focado, **▲▼ ainda alcançam** uma seção que tenha sido selecionada
  antes e não largada por um clique. O `text_input` do iced não usa ▲▼, então o
  evento chega como `Ignored` e é indistinguível de "ninguém quis". Fechar isso
  de vez exige o widget virar um nó focável de verdade, o que é outra obra.

### Notas
- `TimeEditKey` é exportado na raiz do crate, junto de `EngineMessage`: é o que
  permite um teste dirigir o teclado sem display.

---

## [0.69.0] — 2026-09-01

Uma release de **duas correções de contrato**: uma coisa que o motor deixava
escrever sem fazer nada, e um widget que engolia o que recebia.

### Corrigido
- **`class` numa tag de componente não fazia nada — e não avisava.** Escrever
  `<spinbox class="campo_num"/>` (ou o mesmo em qualquer componente, builtin da
  lib ou do app) era um **no-op silencioso**: a classe era lida pelo parser (é
  atributo genérico de nó), viajava no mapa de props do `NodeType::Component` e
  depois ninguém a usava. Nenhum erro, nenhum aviso, nenhum log — o
  `background` da raiz expandida saía `None`.

  É a pior forma de falhar, e a mesma família do seletor por vírgula no GSS e
  da auto-referência que dava `SIGABRT` sem mensagem antes da 0.68: quem
  escreve tem toda a razão de esperar que funcione, porque é o que funciona em
  qualquer outra tag do motor.

  Agora a classe (e o `id`) escritos **no uso** aplicam na **raiz expandida** do
  template. A escada de especificidade, do mais fraco ao mais forte:

  ```
  seletor de tag do componente  <  tag builtin  <  classe do template  <
  classe do USO  <  id do template  <  inline do template
  ```

  Em uma frase: **a classe escrita no uso vence as classes do template, e perde
  para os atributos inline do template.** É a intuição do CSS — a classe do
  autor do componente é um *default*, o atributo inline dele é uma *decisão*. E
  é o que faz `<card class="destaque"/>` conseguir repintar um cartão sem que o
  `.card-surface` do template precise sair da frente.

  A infraestrutura já existia inteira: o seletor de tag de componente
  (`spinbox { }`, item 12 do `PLANO_GSS_LIMITACOES.md`) já resolvia estilo **no
  escopo do uso** e o entregava à raiz do template como `underlay`. A classe do
  uso é o gêmeo dele no outro extremo da escada — um `overlay`, resolvido no
  mesmo lugar, mesclado depois do `resolve_classes` da raiz.

  **A classe do uso entra na chave do cache.** O cache de componente é indexado
  pelo caminho (derivado do `node_id`) e guarda as dependências lidas **dentro**
  da expansão; a interpolação de um `class="{estado}"` acontece no quadro de
  fora, então não estaria entre elas. Sem misturar o valor resolvido na chave,
  um `class` dinâmico que mudasse serviria a árvore antiga para sempre. Mesma
  armadilha que tirou o uso *com conteúdo de slot* do cache na 0.65 — só que
  aqui dá para manter o cache: basta valores diferentes ocuparem entradas
  diferentes.

  Ela aplica **só na raiz**. Estilizar um nó específico lá dentro continua sendo
  decisão do componente, que expõe uma prop com nome próprio — ver o item
  seguinte. Um seletor que fura a fronteira do componente (algo como o
  `::part()` do CSS) é uma porta muito maior, e nada hoje pede.

- **O `<SpinBox>` não repassava nada ao campo que ele monta.** Ele entregava ao
  `<TextInput>` interno só `value`, `onChange`, `placeholder` e `width`. Duas
  consequências, e a segunda é a que dói:

  - não dava para estilizar o campo — a classe do app não chegava nele;
  - **o campo ficava fora da `<Form>`**: sem `form_control` ele não tinha id de
    foco estável e **engolia o Enter** — não submetia o formulário nem avançava
    para o campo seguinte, ao contrário de todos os `<TextInput>` ao lado dele.
    Num formulário de seis campos numéricos, seis buracos no fluxo de teclado.

    Para não plantar folclore: **Tab não era o problema.** A travessia por Tab é
    um listener global do motor (`focus_next`/`focus_previous`, `lib.rs`), que
    percorre todo widget focável independentemente de `formControl` — o campo do
    `SpinBox` já era alcançado por ela antes desta release. O que `form_control`
    liga é o **Enter**.

  Props novas: **`field_class`** e **`form_control`**.

  ```gv
  <spinbox value="qtd" min="1" max="9"
           class="moldura"          <!-- a Row inteira: campo + degraus -->
           field_class="campo_num"  <!-- só o <TextInput> de dentro -->
           form_control="qtd" />    <!-- só o <TextInput> de dentro -->
  ```

  `field_class` **não se chama `class`** de propósito: com o item acima, `class`
  num `<spinbox>` passou a significar "estilize o widget inteiro" — a `Row`, que
  é o que estilizar um `QSpinBox` significa no Qt. As duas coisas são legítimas
  e diferentes; colapsá-las num nome só criaria a ambiguidade que esta release
  existe para matar. `form_control` não precisa do prefixo `field_` porque não
  há ambiguidade a desfazer: só existe um nó focável ali dentro.

  O que fez isto ser barato — e não era óbvio: **a hidratação da `<Form>` roda
  depois da expansão de componente**, sobre a árvore já avaliada. Um
  `formControl` que só passa a existir na expansão é encontrado normalmente, e o
  campo recebe `form_submit_action` e `form_next_focus` como qualquer outro.

### Notas
- O repasse **não** foi estendido aos outros builtins. Só o `SpinBox` tem um
  caso concreto (um campo focável, dentro de um formulário, com estilo do app);
  quando o segundo aparecer, o padrão já está estabelecido por ele.
- `PLANO_CLASS_EM_COMPONENTE.md` registra as duas decisões de projeto e o que
  ficou de fora.
- Extensão de VS Code **0.7.0**: a nota de `class` em componente abre a seção de
  builtins da referência, e o `<SpinBox>` ganha as duas props novas na tabela.

---

## [0.68.0] — 2026-09-01

### Adicionado
- **`<dateedit>`, `<timeedit>` e `<datetimeedit>`** — o `QDateEdit`, o
  `QTimeEdit` e o `QDateTimeEdit`, como **uma primitiva só**; a tag decide quais
  seções aparecem.

  A edição é por **seções**, que é o ponto todo: clicar numa (ano, mês, dia,
  hora, minuto, segundo) a seleciona — ela ganha o realce da paleta, como o
  `2001` destacado num `QDateEdit` — e as setas ▴▾ passam a mexer **naquela**
  seção. Um controle cobre o valor inteiro, sem prop de passo e sem um widget
  por campo.

  ```gv
  <dateedit value="nascimento" format="br" />
  <timeedit value="alarme" seconds="true" />
  <datetimeedit value="agendado" onChange="validar" />
  ```

  - **A chave é sempre ISO** (`YYYY-MM-DD`, `HH:MM[:SS]`), mesmo com
    `format="br"` — que só troca a ordem das seções na tela. É a separação que o
    Qt faz entre o valor e o `displayFormat`, é o que um backend espera, e é o
    que faz `a < b` entre duas chaves ser a comparação cronológica, sem parse.
  - **Sem `onChange` o widget grava a chave sozinho**; **com `onChange` ele só
    avisa** e quem grava é o handler. É o mesmo contrato do `<TextInput>`, com
    um default conveniente — e é o que permite validar ou recusar um valor.
  - **Cada seção vira dentro de si**: mexer no minuto não empurra a hora (o
    `wrapping` do `QAbstractSpinBox`). O ano satura em vez de virar.
  - **O calendário é respeitado**: 31/01 subindo o mês vira 28/02, ou 29 em ano
    bissexto (regra do século inclusive). Sem dependência de datas — a
    aritmética que seções pedem cabe em vinte linhas; a decisão `chrono` vs.
    `time` segue em aberto para o `Calendar`, que precisa de dia da semana.
  - A seção em foco vive numa chave global do motor (`__timeedit`, da família do
    `__drag_key`) com a identidade da instância no valor. Global não é atalho:
    só uma seção da tela pode estar selecionada por vez.
  - **Não dá para digitar** no campo: a interação é clicar na seção + setas.

- **`examples/data_hora_luau`** — as três tags **inteiramente controladas por
  Luau**: o `main.rs` só registra a tela (nenhum `impl Component`, nenhum
  `define_data`), e o script recusa uma saída anterior à entrada, avisa com um
  `toast` e recalcula o resumo. É a outra ponta do `examples/timepicker`, que
  não tem uma linha de código de app.

  A regra do `<datetimeedit>` do exemplo documenta uma armadilha real: comparar
  um `YYYY-MM-DD HH:MM` com um `YYYY-MM-DD` como texto puro dá errado
  (`"2026-09-10 08:00" > "2026-09-10"` é verdadeiro, ainda que sejam o mesmo
  dia). ISO só ordena entre formatos **iguais** — recortar o dia antes de
  comparar é o que restabelece isso.

### Corrigido
- **Todo `<button>` sem `padding` explícito nascia colado no texto.** O
  `parse_padding(None)` devolve `Padding::ZERO` e o braço do botão chamava
  `.padding()` incondicionalmente, sobrescrevendo o `DEFAULT_PADDING` (5px) do
  iced — o fundo grudava nos glifos e o botão lia como texto selecionado. O
  `<TextInput>` e o `<ComboEdit>` já tinham o guarda contra isso, com o
  raciocínio no comentário; o `<Button>` e o `<Select>` não. Afeta toda tela que
  não declarava padding.
- **Um componente que se referencia estourava a pilha** (`SIGABRT`, sem
  mensagem, sem nome, sem linha) em vez de errar. Agora a auto-referência direta
  é detectada por nome no primeiro nível, e um teto de profundidade segura os
  ciclos indiretos.

  O caso real: uma tela registrada com o mesmo nome de um widget embutido
  (`timepicker`) e usando a tag dele por dentro. O registro do app vence o
  builtin — como manda a regra de override —, então a tag passou a apontar para
  a própria tela. É o risco que a grafia minúscula da 0.66 introduziu.
- **`examples/lista_reordenavel` e `examples/toasts` não parseavam**: os dois
  `.gv` estavam sem o cabeçalho `<component>`, e desde a 0.61 um arquivo sem
  cabeçalho não parseia. Não era só o teste — os exemplos não rodavam.

### Quebras
- **`<TimePicker>` deixou de ser um builtin delegante e virou primitiva.** As
  props `on_change` e `on_pick` **não existem mais**, e o widget não é mais um
  campo de texto com um botão ao lado.

  **Migração:** quem usava `<TimePicker value="hora" on_change="…"
  on_pick="…"/>` com um seletor próprio pode apagar o seletor inteiro e usar
  `<timeedit value="hora"/>` — o widget faz o trabalho. Quem precisa reagir à
  alteração troca `on_change` por `onChange`, com a diferença de que agora o
  handler é **quem grava a chave** (o widget delega, em vez de gravar sozinho).

  O motivo da quebra: o widget antigo não selecionava hora nenhuma. Ele
  entregava um `<TextInput>` e um `<Button>` com um emoji, e o app tinha de
  escrever o seletor — o próprio `examples/timepicker` gastava ~40 linhas de
  Luau nisso. E a forma correta (seções, como o Qt) é impossível num builtin:
  o template precisaria ler partes de uma chave cujo *nome* vem de uma prop.

---

## [0.67.0] — 2026-09-01

### Adicionado
- **Slot nomeado, com nomes fixos.** O `<slot/>` da 0.65 era um buraco anônimo
  por componente, o que bastava para um widget de uma região só (`<groupbox>`,
  `<toolbar>`) e travava qualquer um com duas. Agora um componente declara
  `<slot name="footer"/>` e quem usa etiqueta o conteúdo:

  ```xml
  <card title="Servidor">
      <text content="uptime 31 dias" />
      <template slot="footer">
          <button text="Reiniciar" on_click="reiniciar" />
      </template>
  </card>
  ```

  `<template slot="…">` agrupa vários nós; para um nó só, o atributo direto
  (`<button slot="footer" …/>`) evita o embrulho. Vários blocos com o mesmo nome
  se concatenam, e o conteúdo anônimo preserva a ordem de documento mesmo com um
  bloco nomeado escrito no meio dele. A regra de posse não muda: a ação de dentro
  continua sendo de quem a escreveu.

  A partição roda sobre os filhos **crus** — é neles que o atributo `slot` ainda
  existe — e cada balde é expandido por conta própria, então um `<if>` ou um
  `for-each` dentro de um slot nomeado funciona como em qualquer outro lugar.

- **`{slot_<nome>}`: o marcador que permite decorar um slot opcional.** Um
  rodapé quer uma linha divisória acima dele, e só quando existe rodapé — mas o
  template não tinha como perguntar isso (o nome do slot não é uma prop, e o
  conteúdo não chega ao interpolador). O motor agora semeia, na fronteira do
  componente, um booleano por slot nomeado preenchido. Entra na camada **depois**
  das props, então uma prop de mesmo nome vence.

- **`<card>` ganhou rodapé** (`slot="footer"`) e **`<groupbox>` ganhou ações no
  cabeçalho** (`slot="actions"`, à direita da linha do título — onde vai o
  `<checkbox>` que faz o papel do `QGroupBox::setCheckable`). Nos dois, a região
  só se paga quando alguém a preenche.

### Notas
- O nome do slot é **fixo, resolvido no template**. `<slot name="{aba}"/>` —
  nome vindo do contexto — continua não existindo, e é só isso que separa o
  `<tabbar>` de hoje de um `QTabWidget` inteiro.

---

## [0.66.0] — 2026-08-31

### Adicionado
- **Quatro widgets da "onda 1"** (`PLANO_WIDGETS.md` §6) — os que o `iced` já
  sustentava e que só faltava expor. Ao contrário da onda 2, nenhum dependia de
  habilitador de motor.

  - **`<slider>`** (`QSlider`) — primitiva sobre `slider`/`vertical_slider`.
    `min`/`max`/`step`/`vertical`, mais o que o iced 0.14 dá barato: `default`
    (duplo clique devolve o cursor), `on_release` (a ação só ao soltar, para
    quem não quer efeito colateral por pixel arrastado) e `shift_step`. As casas
    decimais da saída vêm do `step` **como escrito no markup** (`step="0.05"` →
    2 casas), então a chave nunca recebe `0.30000001192092896` — o `step` é
    guardado como `f32` e como texto para isso.
  - **`<space>`** (`QSpacerItem`) — sem `width`/`height` é `Length::Fill` nos
    dois eixos (o espaçador flexível que empurra o resto para a borda); com
    eles, um vão fixo.
  - **`<radio>`** (`QRadioButton`) — primitiva sobre o `radio` do iced. O grupo
    **é a chave**, sem nó pai: `group="plano"` é o *nome* da chave (a mesma
    convenção do `checked=` do `<checkbox>` e do `value=` do `<textinput>`), e
    todo `<radio>` que aponta para ela é do mesmo grupo — dois grupos são duas
    chaves. Escrever `group="{plano}"` passaria o valor no lugar do nome e
    deixaria o grupo inteiro desmarcado. Como o `<checkbox>`, não grava sozinho:
    dispara a ação com o valor da opção e quem grava é o app.
  - **`<radiogroup>`** (`QButtonGroup`) — builtin sobre a primitiva, para o caso
    comum: as opções vêm de uma coleção do contexto e o `update` dele grava a
    chave sozinho (padrão do `SpinBox`), então o app não escreve handler nenhum.
    Uma prop só (`value`, o nome da chave) — e **não** o par `value`/`active`
    que o `<tabbar>` precisa, porque aqui quem resolve a marcação é a primitiva,
    em Rust, onde ler a chave cujo nome está numa prop é uma linha. No `tabbar`
    quem resolve é o template, que não consegue fazer essa indireção.
  - **`<avatar>`** — foto circular com as iniciais como reserva. Ocupa o mesmo
    espaço com foto ou sem, porque numa lista de usuários a foto que falta é o
    caso comum e um buraco vazio quebra o alinhamento da linha.

- **Toda tag de widget aceita minúsculas.** `<GroupBox/>` e `<groupbox/>`,
  `<ToolButton/>` e `<toolbutton/>` — a mesma convenção que as primitivas do
  motor já tinham (`<textinput/>`, `<progressbar/>`), agora também para os
  builtins, e a forma que os exemplos passam a usar.

  Uma primitiva casa num `match` de tags que lista as grafias à mão; um builtin
  resolve por igualdade exata de nome, então cada um passou a ser publicado sob
  dois registros (ver `builtins::builtin_aliases`). O alias é uma instância
  própria, o que importa para o roteamento: `<tabbar/>` produz `tabbar::pick:…`
  e é essa instância que recebe o `update`. Como todo builtin da lib é sem
  estado, ter duas instâncias não muda nada.

- **`examples/onda1`** (`cargo run --example onda1`) — os quatro juntos, e de
  propósito a diferença entre primitiva e builtin com o mesmo dado: o grupo
  "Plano" tem um `<radio>` escrito à mão (com handler no app) ao lado de um
  `<radiogroup>` (sem handler nenhum), os dois gravando na mesma chave. O grupo
  "Zoom" põe um `<slider>` e um `<spinbox>` na mesma chave — o par que o Qt usa
  o tempo todo.

### Corrigido
- **`<link rel="import" as="Card">` do app era engolido pelo builtin.** A regra
  da lib é "registro explícito do app vence o builtin", e ela valia para o
  `<import>` mas não para o `<link rel="import">`, que só checava se o nome
  estava livre. O furo era antigo e invisível: enquanto os builtins se chamavam
  `Badge`, `SpinBox` e `TimePicker`, nenhum app disputava esses nomes. A onda 2
  trouxe `Card`, `Frame`, `Avatar` e `ToolBar` — nomes comuns —, e aí um app que
  importasse o próprio `Card` por `<link>` via o builtin renderizar no lugar
  dele, sem erro nenhum. Os dois caminhos agora abrem a mesma exceção.
- **`<frame shape="filled">` saía sem fundo**, indistinguível do `shape="none"`.
  A causa vale a regra geral: o eval resolve um campo com
  `inline.map(process_tpl).or_else(classe)`, então um atributo escrito no
  template vence a classe **mesmo quando resolve para vazio** — um
  `background="{background|}"` gravava `""` em vez de cair para `.frame-filled`.
  Agora o atributo só é emitido quando a prop existe. Mesma armadilha evitada no
  `<avatar>`, que por isso usa defaults literais em vez de folha.

---

## [0.65.0] — 2026-08-31

### Adicionado
- **`<slot/>`: componente agora aceita filhos.** Até aqui,
  `NodeType::Component` carregava só props e o conteúdo escrito entre as tags de
  um componente era **descartado** na expansão. O efeito colateral era grande:
  todo widget cuja razão de existir é *envolver* conteúdo estava fora do nível
  Builtin — `GroupBox`, `Frame`, `Card`, `ToolBar`, `StatusBar` — e por isso
  apareciam na tabela do `PLANO_WIDGETS.md` como construíveis quando não eram.

  Agora `<slot/>` no template de um componente recebe esse conteúdo:

  ```xml
  <GroupBox title="Rede">
      <Checkbox label="Usar proxy" checked="proxy" />
      <Button text="Salvar" on_click="salvar" />
  </GroupBox>
  ```

  O ponto fino é a **posse**: o conteúdo é avaliado no contexto e com o dono de
  *quem escreveu*, antes de qualquer camada de props entrar em cena. Por isso o
  `on_click="salvar"` acima chega no `update` da tela e **não** vira
  `GroupBox::salvar` — não se escreve `app:` no conteúdo de um slot (esse
  prefixo continua sendo para a ação recebida por prop, como no `TimePicker`).
  Os filhos do próprio `<slot>` são o **conteúdo de reserva**, usado quando quem
  chama não escreve nada; esses sim são do componente e enxergam as props dele.

  Um uso **com** conteúdo fica fora do cache de componente: as dependências do
  conteúdo pertencem ao quadro de quem chamou, e uma entrada de cache não teria
  como perceber que ele mudou (mesma exceção que uma lista reordenável já
  tinha). São os containers da tela — o custo é desprezível.

  Ainda **não** existe slot **nomeado** (`<slot name="footer"/>`): um buraco
  anônimo por componente. É o que separa o `TabBar` novo de um `QTabWidget`
  inteiro, e o `Card` de um cartão com rodapé.

- **Seis widgets embutidos novos** — a "onda 2" do `PLANO_WIDGETS.md` §6, toda
  destravada pelo item acima. Nenhum precisa de registro: a lib os registra em
  `GlacierUI::new()`, como o `<Badge/>` e o `<SpinBox/>`.

  - **`<GroupBox/>`** (`QGroupBox`) — moldura com título, mais a forma
    `flat="true"` (título + linha, sem caixa) do `QGroupBox::flat`. Sem título,
    sobra a moldura pura.
  - **`<Frame/>`** (`QFrame`) — a moldura sozinha, em três formas: `box`
    (contorno), `filled` (contraste, o `QFrame::Panel`) e `none`. Sem
    `Raised`/`Sunken`: o `UiNode` não tem campo de sombra.
  - **`<Card/>`** — superfície de item com cabeçalho opcional (título e
    subtítulo, independentes) e corpo por slot. Substitui a linha que a tabela
    do plano dava como pronta desde a 0.35 e que na verdade era um componente
    específico do `examples/perfil`.
  - **`<ToolButton/>`** (`QToolButton`) — botão-ícone com `autoRaise` (fundo só
    no hover), glifo ou `.svg`, e as três formas do `Qt::ToolButtonStyle`
    (`icon`, `beside`, `under`). Delega o clique ao app pelo prefixo `app:`.
  - **`<ToolBar/>`** e **`<StatusBar/>`** (`QToolBar`/`QStatusBar`) — as duas
    faixas da janela. A `StatusBar` separa a mensagem da esquerda (a prop
    `message`, o `showMessage`) dos permanentes da direita (o slot, o
    `addPermanentWidget`). Com o `<MenuBar>`, que já era nativo, fecham o
    esqueleto de uma `QMainWindow`.
  - **`<TabBar/>`** (`QTabBar`) — a barra de abas, com as abas vindo de uma
    coleção do contexto e a ativa gravada numa chave que o app nomeia (o padrão
    do `SpinBox`: a chave vem por prop e viaja dentro da ação). `value` e
    `active` andam em par porque o template não consegue ler o valor da chave
    cujo *nome* está numa prop. O empilhado de páginas continua sendo
    `se`/`senao` — o `QTabWidget` inteiro espera slot nomeado.

- **`examples/onda2`** (`cargo run --example onda2`) — os seis juntos montando
  uma janela: barra de ferramentas, abas, conteúdo rolável e rodapé. O botão
  "Salvar rede", escrito dentro de um `<GroupBox>`, é a demonstração da regra de
  posse do slot.

### Corrigido
- **`PLANO_WIDGETS.md` batia com o código em onze linhas.** Dez estavam
  marcadas como pendentes ou parciais quando já existiam — `Checkbox tristate`,
  `TextInput secure`, `ComboEdit`, `MenuBar`/`Menu`/`ContextMenu`, `SystemTray`,
  os três `FileDialog` e o `tooltip` — e uma (`Card`) estava marcada como pronta
  sem existir. O resumo numérico também tinha erros de contagem próprios em
  quatro seções. Recontado: **47 ✅ / 10 🟡 / 65 ⬜** em 122 linhas.

---

## [0.64.0] — 2026-08-31

### Mudado
- **O `<SpinBox/>` ganhou a forma que o Qt tem.** Ele nascia como três blocos
  soltos — dois botões primários grandes com `▼`/`▲` em corpo de texto normal e
  um campo no meio, separados por 4px —, o que lia como "dois botões ao lado de
  um campo" em vez de um widget só. Agora:

  - **Os degraus encostam no campo** (`spacing="0"`): a borda do próprio
    `<TextInput>` vira a moldura do conjunto.
  - **Duas formas, via a prop nova `layout`.** `stacked` (default) é o
    `QSpinBox` clássico: uma coluna com `▴` em cima e `▾` embaixo, colada à
    direita do campo, ocupando a altura dele e nada mais. `inline` é a forma do
    `SpinBox` do Qt Quick Controls: `−  campo  +`, alvo de clique grande, para
    toque e para valores que se ajusta muito.
  - **Os degraus deixam de ser botões primários.** Não é o que o Qt desenha —
    lá eles são cromo discreto ao lado do campo, não a ação principal da tela.
    O visual sai de um `<style>` global declarado no próprio template do widget
    (classe `.spinbox-step`, com `:hover`/`:active`), instalado em
    `GlacierUI::new` e portanto **antes** de qualquer `.gss` do app — que por
    isso o vence por ordem: redefinir `.spinbox-step` numa folha do app é o
    caminho suportado para repintá-los.
  - **As cores dessa folha são neutras e translúcidas** (`#8080801f` de fundo):
    um cinza com alfa clareia sobre um tema escuro e escurece sobre um claro,
    então o mesmo default atravessa os quatro estilos embutidos sem que o widget
    precise saber qual está ativo — nenhum hex de paleta viaja no template.

  Props novas: `layout` (`stacked`/`inline`) e `glyph_size` (`11` no `stacked`,
  `15` no `inline`). Os defaults de `dec_text`/`inc_text` passam a depender da
  forma: `▾`/`▴` no `stacked`, `−`/`+` no `inline`.

  **Quem inspeciona a árvore avaliada precisa saber:** a raiz continua sendo uma
  `<Row>`, mas no `stacked` os filhos agora são `[campo, coluna dos degraus]` (2,
  não 3), e o glifo é um `<Text>` **filho** do `<Button>` (é ele que carrega o
  `size`), não mais o atributo `text=`.

- **A extensão Glacier View (`editors/vscode-gv`, v0.4.1) documenta o
  `<SpinBox/>`.** Ele não estava na tabela de tags nativas/builtin do
  `extension.js`, então era tratado como componente do app: o F12 não achava
  nada. Agora ele resolve para uma seção nova do doc de referência embutido
  (`references/glacier-view.md`), com a tabela de props, as duas formas de
  `layout` e a classe `.spinbox-step`. De quebra, a entrada do `<Badge/>` ganhou
  as props dele e perdeu a referência a `src/builtins.rs`, que virou diretório.

- **O exemplo `spinbox` semeia os valores iniciais** (`define_data`) e mostra a
  forma `inline` na linha do zoom. Um `<SpinBox/>` cuja chave nunca foi escrita
  nasce em branco — correto (o primeiro clique inicializa no `min`), mas um
  campo numérico vazio parece um campo quebrado numa captura de tela.

- **Os exemplos deixam de ser compilados por padrão** (`autoexamples = false`).
  Os 31 arquivos continuam em `examples/`; só a descoberta automática saiu.

  `cargo test` compilava todos eles, cada um linkando o motor inteiro
  estaticamente (iced + wgpu + naga + Luau + codecs de imagem + resvg). Medido:
  **31 × 506 MB = 15,2 GiB** em `target/debug/examples`, ~70% de um target de
  22 GiB. De cada binário, 86% era debuginfo (506 MB → 73 MB depois de `strip`),
  e como os 31 compartilham as mesmas dependências, boa parte disso era o mesmo
  debuginfo repetido 31 vezes.

  Depois da mudança, o target a frio vai de **22 GiB para 6,3 GiB**, com os
  mesmos 366 testes passando — os `.gv` dos exemplos seguem cobertos por
  `tests/exemplos_gv.rs`, que lê arquivo e não precisa compilar nada.

  Um exemplo novo entra explicitamente:

  ```toml
  [[example]]
  name = "contador"
  path = "examples/contador/main.rs"
  ```

  Para rodar um dos antigos, comente o `autoexamples = false`.

  **O que se perde:** o `main.rs` dos 31 exemplos não é mais compilado por
  ninguém, então um deles pode apodrecer sem que a suíte perceba.

---

## glacier-cli 0.2.1 — 2026-08-31

Republicação: **o código da CLI não mudou** (`git diff b944ac7 -- crates/glacier-cli`
é só a linha da versão). O que muda é o que ela carrega dentro:

- **A versão do motor que `glacier new` grava** no `Cargo.toml` do projeto novo.
  Ela sai do `engine-version.txt` que o `make sync-extensions` gera a partir da
  raiz — na 0.2.0 isso congelou `glacier-ui = "0.62"`. Agora sai `"0.64"`, então
  um projeto criado hoje já nasce com o `<SpinBox/>` na forma nova.
- **A extensão Glacier View embutida** (a CLI a instala sem Node/vsce) passa a
  ser a v0.4.1, que documenta o `<SpinBox/>`.

---

## [0.63.0] — 2026-08-30

Uma release de **widgets embutidos**: nasce o primeiro builtin que *faz* algo em
vez de só desenhar, e com ele os dois mecanismos que faltavam para a biblioteca
crescer — um para o widget que age, outro para o widget que delega. O
`<TimePicker/>`, que estava quebrado de quatro maneiras diferentes, é o primeiro
beneficiário.

### Adicionado
- **`<SpinBox/>` — o `QSpinBox` do Qt, embutido.** Campo numérico com as setas
  ▼▲: clicar soma ou subtrai `step`, saturando em `min`/`max`. Nenhuma linha de
  código do lado do app.

  ```xml
  <SpinBox value="quantidade" min="1" max="99" />
  <SpinBox value="preco" min="0" max="10" step="0.25" width="90" />
  ```

  Props: `value` (obrigatória), `min`/`max` (`0`/`100`, a faixa padrão do Qt),
  `step` (`1`), `width` (`72`), `placeholder`, `dec_text`/`inc_text` (`▼`/`▲`).

  - **As casas decimais saem do `step`.** `step="0.25"` formata com 2 casas — o
    `QDoubleSpinBox` sem um segundo widget, e de quebra sem o
    `0.30000000000000004` que somar `f64` produz.
  - **Chave vazia**: o primeiro clique inicializa no `min`, não em `min + step`.
  - **Digitação** entra filtrada (só dígitos, um `-` à frente e um `.`) e sem
    saturar, como o `QSpinBox`, que só valida ao terminar a edição; o clique
    seguinte satura.

  Ele é o primeiro builtin com **comportamento próprio** — a aritmética roda no
  `update`, em Rust. Os anteriores (`Badge`, `TimePicker`) só montam markup.

- **O padrão que deixa um builtin ter comportamento sem ter estado.** O `ctx` de
  um builtin é o contexto global: não há slot por instância. O `SpinBox` contorna
  isso não guardando valor nenhum — o número mora numa chave que **o app nomeia**
  (`value="quantidade"`), então duas instâncias com chaves diferentes são
  independentes.

  O elo que faltava era o `update` saber *qual* chave a instância clicada usa —
  ele recebe a ação, não as props. A ação passa a carregar os parâmetros:

  ```xml
  <Button text="▲" on_click="inc:{value}|{min|0}|{max|100}|{step|1}" />
  ```

  O eval interpola e prefixa o dono, o `dispatch` quebra no `::` e entrega ao
  `update` do próprio widget, que fatia a string. Documentado em `BUILTINS.md`,
  com a ressalva do `|` (dentro de `{…}` separa o default inline; fora é
  literal).

- **`app:` — o escape de namespace, para o widget que delega.** Toda ação escrita
  no template de um componente é prefixada com o dono. Isso quebra o widget que
  **repassa** uma ação recebida por prop: `on_click="{on_pick}"` dentro do
  `<TimePicker/>` virava `TimePicker::abrir_modal`, o motor achava o `TimePicker`
  no mapa de componentes e chamava o `update` **dele** — que não conhece ação
  nenhuma do app. E sem erro nenhum: o botão simplesmente não fazia nada.

  ```xml
  <Button text="{pick_icon|⏰}" on_click="app:{on_pick}" />
  ```

  O prefixo sai *no lugar* do prefixo de dono, e a ação chega em quem a definiu.
  `app:` quer dizer **a tela atual** — é onde o `dispatch` cai quando não há
  dono —, não o componente intermediário que porventura tenha usado o widget:
  delegar de componente para componente ainda depende de um `ctx.dispatch` que o
  motor não tem.

- **`examples/spinbox/`** (`cargo run --example spinbox`): cinco `<SpinBox/>` —
  inteiro, decimal, passo grande e duas instâncias lado a lado — com a chave de
  cada um ecoada em texto ao lado, para a independência entre instâncias ficar
  visível. Ele e o `timepicker` são os primeiros exemplos declarados com bloco
  `[[example]]`, como o comentário do `Cargo.toml` pede desde a 0.62.1 — ou
  seja, os dois voltam a ser compilados pelo `cargo test`.

### Corrigido
- **`<TimePicker/>` não funcionava** — quatro defeitos empilhados, três deles
  fora do widget:

  1. `value_var="{value}"` no template do builtin: o atributo que o parser lê é
     `value` (`value_var` é só o nome do campo interno do `NodeType`). O campo
     ficava ligado a chave nenhuma — não exibia o valor nem o que se digitava.
  2. `on_change`/`on_pick` eram engolidas pelo namespacing (ver `app:` acima).
  3. No exemplo, faltava o `<script src="app.luau">` em `<resources>`: o motor
     liga o comportamento Luau pelo `<script>` do template, não por convenção de
     nome de arquivo. O `init()` nunca rodava — nem o `09:00` inicial, nem as
     listas dos `<select>` do modal. Os dois `<select>` repetiam o erro do
     item 1 (`value_var=`).
  4. Ainda no exemplo, os handlers Luau liam a chave em vez do argumento
     (`formatar_tempo()` fazia `ctx.inicio:gsub(…)`). O motor **não** escreve
     sozinho na chave de um `<TextInput>`: o texto digitado chega como argumento
     da função (e no global `value`), e é o handler que grava.

  Um teste novo roda o exemplo ponta a ponta — `init()` semeando, `1445` virando
  `14:45`, o ⏰ abrindo o modal com `h_sel`/`m_sel` preenchidos, o confirmar
  escrevendo de volta.

### Mudado
- `BUILTINS.md` ganhou as duas seções novas — o widget que age (chave por prop,
  parâmetros na ação) e o widget que delega (`app:`) — e a restrição de contexto
  global deixou de dizer "mantenha os builtins apresentacionais": um builtin pode
  ter comportamento, desde que todo valor que ele guarda more numa chave nomeada
  por quem o usa.
- `PLANO_WIDGETS.md`: `SpinBox` vira ✅ e sai de `Comp`/`●` para `Built`/`◐`;
  `QDoubleSpinBox` vira 🟡 (coberto pelo `step`, falta a prop `decimals`). O §3
  ganhou duas correções de rumo:
  - o `●` estava marcando **duas** coisas — "valor que o app nomeia" (nunca
    bloqueou nada) e "estado sem nome natural" (esse sim). A mesma correção que
    a linha do `Spinner` já tinha recebido na 0.53;
  - o que de fato trava `Tabs`, `Accordion`, `GroupBox`, `Frame`, `ToolBar` e
    `StatusBar` é outro item, que não estava na lista: **componente não aceita
    filhos** (não há `<slot/>`; o conteúdo escrito dentro da tag é descartado na
    expansão). Entrou como habilitador P1.

### Quebras
- `app:` passa a ser **prefixo reservado** de ação, ao lado de `clipboard:`,
  `open:`, `window:` e `style:`. Uma ação que já se chamasse `app:algo` dentro de
  um componente era entregue ao `update` desse componente com o nome inteiro;
  agora ela perde o prefixo e vai para a tela atual. Renomeie a ação se for o
  caso — nenhum exemplo ou template do repositório usava esse nome.

---

## [0.62.2] — 2026-08-30

Publicação de manutenção: **o motor não mudou**. Nenhum arquivo de `src/` é
diferente da 0.62.1 — a comparação está no commit.

A 0.62.1 saiu junto com a primeira versão da CLI, e desde então o trabalho todo
foi do lado dela (`glacier-cli` 0.2.0: Makefile, `fazer.bat` e empacotamento nos
projetos criados). Esta versão existe para que o `engine-version.txt` que a CLI
grava — e portanto o `glacier-ui = "…"` do `Cargo.toml` de todo projeto novo —
aponte para uma release publicada no mesmo dia que ela, em vez de para uma
anterior.

Quem já está em `^0.62` não ganha nem perde nada atualizando.

---

## glacier-cli 0.2.0 — 2026-08-30

Só a CLI (`crates/glacier-cli`). O motor não mudou e segue em 0.62.1.

### Adicionado
- **Todo projeto criado por `glacier new` já sai empacotável e instalável**, nos
  dois sistemas, pelos dois lados: um `Makefile` (Linux) e um `fazer.bat`
  (Windows, onde não há make), herdados por todos os presets via `_comum`.

  | | Makefile | fazer.bat |
  |---|---|---|
  | para Windows | `make windows` (cross-compile, cargo-xwin) | `fazer build` (MSVC nativo) |
  | empacotar Windows | `make windows-dist` → `.zip` | `fazer dist` |
  | empacotar Linux | `make linux-dist` → `.tar.gz`, `make deb` | — |
  | instalar | `make install` / `install-sistema` | `fazer instalar` |

  O motivo: até aqui o `new` entregava um projeto que roda com `cargo run` e
  para por aí. Transformá-lo em algo que outra pessoa instala era um problema
  não resolvido — e um com armadilha, porque a parte que quebra não dá erro.

  O `.exe` sai com `+crt-static`: sem isso ele exige o Visual C++
  Redistributable na máquina de destino e falha com uma caixa de erro que não
  diz qual DLL faltou.

- **Instaladores em `packaging/`**, dentro do pacote. No Windows, um
  `instalar.bat` que copia para `%LOCALAPPDATA%\Programs` e cria o atalho no
  menu Iniciar **sem pedir administrador** — um app de usuário não precisa de
  elevação para ser instalado. No Linux, um `instalar.sh` que instala em
  `~/.local` (ou `--sistema` para `/usr/local`) e gera o `.desktop`.

- **`conferir-pacote`**, em que todo alvo de pacote termina. O app lê `views/`
  em runtime — é o que dá o hot-reload —, então um pacote sem essa pasta
  compila, empacota, instala e abre: numa janela vazia, na máquina de quem
  baixou, sem nenhuma mensagem que aponte a causa. A conferência compara a
  contagem de arquivos e falha antes de o `.zip` existir. A ideia (e o custo de
  não a ter) vem do Makefile do rustploy.

- **Um wrapper de três linhas** é o que vai para `/usr/bin` (no `.deb`) e para
  `~/.local/bin` (no `instalar.sh`); o binário de verdade fica ao lado do
  `views/`, e o wrapper faz `cd` antes de executar. Sem isso, rodar o app de
  qualquer outro diretório o faria procurar os templates onde eles não estão —
  o mesmo bug que a conferência acima previne no empacotamento, só que na
  instalação. O `.desktop` leva `Path=` pelo mesmo motivo, e o atalho do Windows
  leva `WorkingDirectory`.

### Corrigido
- `scaffold` agora substitui os marcadores `{{…}}` em arquivos **sem extensão**
  (`Makefile`) e em `.bat`/`.sh` — sem isso o Makefile gerado sairia com
  `{{nome_projeto}}` literal no lugar do nome do app —, e dá `+x` aos `.sh`
  escritos, que `fs::write` cria em `644`. Um `instalar.sh` sem o bit de
  execução propagaria a permissão errada para dentro do `.tar.gz`.

## [0.62.1] — 2026-08-30

### Adicionado
- **`glacier`, a CLI de bootstrap** (`crates/glacier-cli`, publicada como
  `glacier-cli`). O repositório vira um workspace: o motor continua sendo o
  pacote da raiz, e a CLI é um crate à parte.

  ```bash
  cargo install glacier-cli
  glacier new
  ```

  O motivo: um projeto glacier tem `Cargo.toml`, `src/main.rs`, um `.gv` com
  cabeçalho, um `.gss`, um `.luaurc` e uma árvore de scripts Luau — e montar
  isso à mão, lendo o README arquivo por arquivo, é a parte mais chata de
  começar. O `new` faz um questionário, mostra um resumo e **só então** escreve:
  até a confirmação final, nada foi criado.

  Quatro presets, todos herdando `.gitignore`, `.luaurc` e
  `views/scripts/glacier.d.luau`:

  | id | O que é |
  |---|---|
  | `completo` | janela sem decoração com titlebar própria, tema + `.gss`, componentes com `<props>`, navegação, `fetch`, toasts, `@media` |
  | `minimo` | uma tela, um `.gss` e um bloco de script Luau |
  | `janelas` | `open_window`/`broadcast`/`close_window`, bandeja, instância única, geometria lembrada |
  | `rust` | o trait `Component` com estado tipado |

  O código dos presets não leva comentário: `src/main.rs`, os `.gv` e os `.luau`
  saem enxutos, e toda a explicação (como os caminhos resolvem, o que o `ctx`
  guarda, por que `main_window` traz o que traz) vive no README de cada preset.
  Os `.gv` usam recuo de 2 espaços.

  `glacier install-extensions` instala as extensões de VS Code (`.gv` e `.gss`).
  Elas vêm embutidas no binário e são empacotadas em `.vsix` na hora — sem Node,
  sem `vsce`. Editores procurados no `PATH`: `code`, `code-insiders`, `cursor`,
  `codium`, `windsurf`.

  A CLI **não tem dependências** (nem `clap`, nem o próprio `glacier-ui`): ela
  existe para tirar alguém do zero, e um `cargo install` que leva minutos
  derrotaria o propósito.

- **Pacote Debian da CLI** (`make deb-cli` / `make install-cli`), para exercitar
  o `glacier` como o usuário final o vê: no `PATH`, longe do `target/`, sem
  passar pelo crates.io. São ~270 KB, só o binário.

  `make check-deb` (que o `deb-cli` já chama) confere o **DT_NEEDED do ELF
  empacotado**, e não a linha `Depends`: o `dpkg-shlibdeps` declara o mínimo e
  omite o que vem por transitividade — `libgcc_s.so.1` é exigido pelo binário e
  mesmo assim não aparece no `Depends`, porque `libgcc-s1` já vem por `libc6`.
  Conferir só o `Depends` deixaria passar uma biblioteca nova de verdade. O alvo
  falha se aparecer qualquer coisa fora da glibc.

- **`tests/presets_cli.rs`** — cada preset é materializado num diretório
  temporário e carregado num `GlacierUI` de verdade. É o que pega o que a
  compilação não pega: um `<link rel="import">` apontando para a pasta errada,
  uma stylesheet com caminho relativo errado, um erro de sintaxe no Luau —
  falhas que só apareceriam na primeira vez que alguém rodasse o projeto novo.

### Mudado
- **`sse`/`websocket`: o callback é a função, não o nome dela.** O motor sempre
  aceitou as duas formas (`handler_key` casa `Value::Function` antes de tentar
  resolver uma string como global), mas toda a documentação, o prelúdio e os
  exemplos ensinavam só a forma por nome — então na prática a API *era* por
  nome. Agora a função é a forma canônica em todo lugar:

  ```lua
  sse_conn = sse("https://sse.dev/test", {
      on_open    = function() ctx.sse_status = "aberto" end,
      on_message = function(data) ctx.sse_msg = data end,
  })
  ```

  Com isso vêm closure, upvalue e método de tabela — o handler não precisa mais
  ser global nem ter nome. O `glacier.d.luau` dos presets ganhou o tipo
  `StreamOptions`, que declara cada callback como função, para o luau-lsp guiar
  para a forma certa.

  O nome de função global **continua aceito**, como atalho: apps escritos antes
  disto não quebram. Mas ele obriga o handler a ser global, não fecha sobre
  nada, e falha em silêncio quando o nome está errado — o evento chega e não
  chama ninguém.

### Corrigido
- **`examples/stream_lua` não abria.** Os dois `.gv` apontavam para
  `stream_lua.luau`/`stream_local.luau` e os arquivos no disco eram `.lua`, então
  o exemplo morria com "Falha ao ler script Luau externo" na primeira execução.
  Nenhum teste pegava: o parse de um `.gv` não resolve o `src` (o bloco é
  recortado por texto antes), e o caminho só é lido quando o motor REGISTRA o
  componente. Arquivos renomeados, e `tests/exemplos_gv.rs` ganhou
  `todo_script_src_aponta_para_um_arquivo_existente`, que confere todo
  `<script src>` de `examples/`, `templates/` e dos presets da CLI.

- **Um `<script>` citado em comentário XML quebrava o template.** Escrever
  `<!-- mova para um arquivo com <script src="x.luau"> -->` num `.gv` derrubava
  o carregamento com um `syntax error: [string "<script:…>"]:1: Incomplete
  statement` — uma mensagem sem nenhuma relação visível com o comentário que a
  causava.

  As duas varreduras que procuram o bloco discordavam: `eval::strip_script`
  pulava comentários (via `find_script_open`), mas `luau::extract_script` e
  `luau::extract_script_src` faziam um `find("<script")` cru. O parser de markup
  tirava o bloco certo enquanto o Luau compilava o texto errado — o corpo
  "extraído" começava no `>` da tag CITADA e ia até o `</script>` de verdade,
  arrastando o resto do comentário e a tag de abertura real como se fossem
  código.

  `find_script_open` virou `pub(crate)` e passou a ser a única definição de onde
  o bloco começa; as três funções agora concordam sempre. Quatro testes de
  regressão em `src/luau/mod.rs`, e os templates do `glacier new` citam a tag
  nos comentários de propósito — `tests/presets_cli.rs` os carrega num motor, o
  que os torna fixture viva do caso.

- **`tests/exemplos_gv.rs` era mais estrito que o motor.** Ele parseava cada
  `.gv` cru, enquanto o motor faz duas passadas antes (`strip_script` e
  `normalize_bare_directives`, ver `parse_markup`). Markup que abre sem problema
  no app — um `else` pelado como atributo, um `<` dentro de um bloco de script —
  era recusado pelo teste. Agora ele espelha o pré-processamento do motor.
## [0.62.0] — 2026-08-28

### Adicionado
- **`spread`: passar um objeto inteiro no lugar de um atributo por campo.** O
  call-site de um card em lista era uma parede de mapeamentos **identidade** —
  o nome à esquerda igual ao campo à direita, ruído de digitação sem informação
  nenhuma:

  ```xml
  <ServiceCard for-each="linhas" var="c"
      id="{c.id}" nome="{c.nome}" porta="{c.porta}" cpu="{c.cpu}" mem="{c.mem}" />
  ```

  ```xml
  <ServiceCard for-each="linhas" var="c" spread="{c}" />
  ```

  Cada campo do objeto cai na prop declarada de mesmo nome. Apelido em
  português: `espalhar`.

  O que ele deliberadamente **não** é: uma prop-objeto (`card="{c}"` e
  `{card.id}` dentro do componente). Ali o `<props>` passaria a declarar `card`
  e mais nada, e `{card.nmae}` voltaria a renderizar vazio em silêncio — o typo
  invisível que a 0.61 existe para fechar. Semeando as props **declaradas**, o
  dentro do componente não muda (`{id}`, `{nome}`), todas seguem verificadas, e
  a checagem ainda **ganha alcance**: uma prop obrigatória que o *dado* não
  trouxe passa a errar, não só a que o markup esqueceu.

  - só as props que o `<props>` declara entram; campo sobrando é ignorado (o
    objeto que vem do Luau quase sempre carrega mais do que o componente usa, e
    recusá-lo tornaria o spread inútil no caso para o qual ele existe);
  - atributo escrito à mão **ganha** do spread — é como se sobrepõe um campo;
  - campo ausente cai no `default` do `<prop>`;
  - sem `<props>`, não há o que filtrar: todo campo entra na camada (segue
    valendo a regra da 0.61 — quem não declara não é checado);
  - valor **vazio** (a chave ainda não carregou) vale como "nenhum campo", e o
    que faltar erra como `MissingProp`, que aponta *qual* prop com mais
    precisão; um escalar ou uma lista, aí sim, é `InvalidSpread`.

  Uma **lista** aninhada já atravessava a fronteira de componente sem isto (o
  valor vai como JSON e o `for-each` de dentro reparseia a chave), e continua:
  `spread="{c}"` com um `c.tags` dá um `<text for-each="tags" var="t">` dentro
  do componente.

- **`{item}` de um `for-each` resolve para o item inteiro.** Um objeto expunha
  só os campos (`{c.nome}`); o item em si não estava na camada, e `{c}`
  renderizava vazio. Agora ele é o JSON do item — é o que o `spread` repassa.

### Mudado
- O `README` ganhou a seção `<props>` que o índice já prometia desde a 0.61
  (o link apontava para uma âncora que não existia), agora com o `spread`.

### Quebras
- `{item}`, para um item de **objeto** num `for-each`, deixa de cair para o
  contexto de baixo: antes um `{c}` solto pegava uma chave global `c`, se
  houvesse; agora resolve para o JSON do item. Só afeta um template que use o
  nome da variável de laço *também* como chave global — renomeie um dos dois.
- `GlacierError` ganhou `InvalidSpread`; um `match` exaustivo sobre ele precisa
  do braço novo.

---

## [0.61.0] — 2026-08-28

### Adicionado
- **`<props>`: o contrato de um componente.** Um `<component>` pode declarar as
  props que aceita, e a declaração passa a ser verificada no ponto de **uso**.

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
  - **sem `<props>`, nada muda** — declarar é opcional, e o componente que lê o
    contexto global em vez de receber props continua funcionando.

  O motivo de isto ser uma feature e não um comentário no topo do arquivo: as
  props entram como uma **camada** sobre o contexto de quem usa, e um lookup que
  falha na camada cai para o contexto de baixo. Sem contrato, `<Cartao
  nomee="Alice" />` não renderiza vazio — renderiza o `nome` que existir no
  contexto global. O typo era invisível até alguém reparar no valor errado na
  tela.

### Mudado
- **O cabeçalho passou a ser obrigatório em arquivo.** Todo `.gv` começa com
  `<screen>` (uma janela) ou `<component>` (o resto). Markup **inline**
  (`Template::Inline`, o que os builtins da lib usam) segue sendo fragmento: não
  há arquivo nem janela a que um cabeçalho se aplique, e a regra distingue pela
  **origem**, não pelo conteúdo.
- **O cabeçalho tem de envolver o arquivo inteiro.** Três meios-termos que
  passavam calados viraram erro de parse:
  - `<screen>` escrito como **irmão** do layout: os metadados até eram
    recolhidos (a janela abria com o título certo), mas o layout virava um
    `Fragment` e ganhava um `column!` implícito em `shrink` por volta — um
    `height: fill` na raiz mudava de comportamento sem aviso;
  - `<screen>`/`<component>`/`<resources>` **aninhados no meio do layout**: eram
    descartados na avaliação, então a subárvore sumia da tela em silêncio (a
    validação antiga só olhava o nível de topo);
  - `<props>` num `<screen>`: uma janela é aberta, não usada por outro template
    — não há quem lhe passe props.

### Quebras
- Um `.gv` sem cabeçalho não carrega mais. Migrar é envolver o arquivo em
  `<screen title="…" size="…">` (se for a tela de uma janela) ou `<component>`
  (se for importado por outro template), e mover `<style>`/`<link>`/`<import>`/
  `<script>` para um `<resources>` dentro dele. Os 35 `.gv` deste repositório
  foram migrados nesta versão e servem de referência.
- `GlacierError` ganhou `UnknownProp` e `MissingProp`; um `match` exaustivo sobre
  ele precisa dos dois braços novos.

---

## [0.60.0] — 2026-08-28

### Adicionado
- **`<component>`: o cabeçalho de quem não é janela.** A 0.59 deu ao template um
  cabeçalho (`<screen>` + `<resources>`), mas nem todo `.gv` é uma tela: um
  arquivo importado por outro (`<import>`) é um pedaço de tela — um card, um
  item de menu, um badge —, e `title`/`size` ali não têm a quem se aplicar. Usar
  `<screen>` nesses arquivos funcionaria, e era justamente o problema: a tag
  prometeria uma janela que não existe.

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

  Mesmo agrupamento do `<screen>` (o `<resources>` vale nas duas raízes, e é
  igualmente opcional), com uma diferença deliberada: **o `<component>` não leva
  atributo nenhum**. `title=` ali é erro de parse, com a explicação junto
  (`title/size descrevem uma JANELA, e um <component> não é uma — quem é janela
  usa <screen>`); qualquer outro atributo também erra, lembrando que as props de
  um componente vêm de quem o usa. Apelido em português: `<componente>`.

  Nada é obrigatório: um `.gv` sem cabeçalho continua válido, e um componente
  pode seguir com as declarações soltas na raiz.

---

## [0.59.0] — 2026-08-28

### Adicionado
- **Cabeçalho da tela: `<screen>` + `<resources>`** — um template pode declarar,
  no próprio arquivo, o que a **janela** é, e separar o que não desenha do que
  desenha. Até aqui um `.gv` era uma lista de coisas soltas no mesmo nível: o
  `<style>`, que não aparece na tela, ficava lado a lado com o `<container>`, que
  aparece — sem nenhuma fronteira entre "o que a tela precisa para existir" e "o
  que a tela mostra". E o arquivo que descreve a tela não sabia como ela se chama
  nem de que tamanho nasce: título e tamanho só existiam no builder Rust, o que
  obrigava a sair do `.gv`, editar o `main.rs` e recompilar (perdendo o
  hot-reload) para mudar um título — e dava um título só para todas as telas da
  mesma janela, porque ele era decidido uma vez no boot.

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

  Atributos: `title`/`titulo`, `size`/`tamanho` (`"960 700"`, `"960x700"` ou
  `"960, 700"` — um par de números, como o `padding`, porque `width`/`height` já
  querem dizer outra coisa no layout), `min-size`/`minSize` e
  `resizable`/`redimensionavel`. As tags aceitam apelidos em português
  (`<tela>`, `<recursos>`), como o resto do vocabulário.

  As regras de convivência:
  - **O cabeçalho é opcional** — nenhum `.gv` existente precisa mudar; a forma
    solta continua válida e as duas terminam na mesma árvore.
  - **O template ganha do builder** — `GlacierDaemon::title()`/`main_size()`
    viram o valor de quando o arquivo não diz nada; um campo não declarado não
    opina.
  - **O título acompanha a navegação** — entrar numa tela que declara `title`
    troca o da janela; sair para uma que não declara devolve o título base.
  - **O tamanho é de quem abre a janela** — navegar não redimensiona, e o
    hot-reload só redimensiona quando o número mudou no arquivo (senão cada
    `Ctrl+S` desfaria o arrasto que você deu no canto da janela).
  - **A geometria lembrada ganha do `size`** — num app com
    `remember_window_geometry`, o `size` do template é o de primeira abertura; o
    tamanho em que o usuário deixou a janela vence (com o `min-size` declarado
    valendo de piso).
  - **Janela-filha herda do arquivo** — `open_window({ file = "detalhe.gv" })`
    usa o título/tamanho declarados lá dentro, e a chamada ainda pode sobrepô-los.
  - **Componente importado ignora os metadados** — um `.gv` trazido por
    `<import>` é pedaço de tela, não janela.

  O `<resources>` é opcional: com uma ou duas declarações, elas podem ficar
  soltas dentro do próprio `<screen>`.

  **O cabeçalho erra alto.** Ele é a parte do template que não desenha nada, e
  por isso um engano ali não teria sintoma: a tela abriria igual, só que sem o
  que o autor escreveu. Viram erro de parse, com linha, coluna e trecho: atributo
  desconhecido no `<screen>` ou no `<resources>`, `size`/`min-size` que não seja
  um par de números (`size="960px"`, `size="960"`), `resizable` que não seja
  booleano, um widget dentro do `<resources>` (com a dica de movê-lo para depois
  do `</resources>`) e um `<resources>` fora de um `<screen>`.

  API nova: `GlacierUI::screen_meta`, `GlacierUI::current_screen_meta`,
  `GlacierUI::current_screen_name` e o tipo `ScreenMeta`, re-exportado na raiz.
  `examples/controle_externo` foi migrado para a forma nova.

---

## [0.58.6] — 2026-08-28

### Adicionado
- **`external::sender()`** — canal para **injetar ações no motor a partir de
  qualquer thread**. Até aqui tudo o que acontecia na UI nascia de um evento do
  loop do iced (clique, tecla, tick), o que deixava de fora um caso legítimo: o
  app tem uma thread própria — um servidor HTTP local, um watcher de arquivos,
  uma integração com o SO — que precisa **acionar** a UI, não só ler o estado
  dela. As saídas eram ruins: espelhar o estado num `Arc<Mutex<…>>` paralelo
  (que diverge do contexto do motor na primeira ação que esquecerem de
  replicar) ou simular eventos de entrada no servidor gráfico.

  O remetente ([`ExternalSender`]) é `Clone + Send`, criado **antes** de
  `run()`, e expõe o mesmo vocabulário dos templates: `click(acao)` (como um
  `<Button on_click>`), `action(acao, valor)` (como um `onChange`) e
  `patch(pares)` (escreve no contexto). Por isso **toda** ação que a UI declara
  já é alcançável de fora — inclusive as que forem adicionadas depois, sem
  lista para manter em dia.

  As mensagens vão sempre para o motor da janela **principal**, inclusive
  quando ela está recolhida na bandeja (nesse estado o motor segue vivo e só a
  janela sumiu, ver 0.48) — então um app de bandeja continua inteiramente
  dirigível de fora. A subscription que drena o canal só é registrada se alguém
  chamou `sender()`: quem não usa não paga o poll. Exemplo em
  `examples/controle_externo/`.

## [0.58.5] — 2026-08-24

### Adicionado
- **`zip_dir(origem, destino)`** (Luau) — compacta um diretório num `.zip`
  escrito direto no caminho de destino, criando os diretórios pais que
  faltarem. Mesmo molde síncrono de `write_file`/`append_file` (não
  suspende como `confirm()`/`fetch()`/`open_file()` — I/O local, não rede
  nem diálogo). Recebe o caminho **final** já resolvido em vez de zipar
  "em algum lugar" pra depois mover — mover um `.zip` pronto exigiria um
  primitivo binary-safe que não existe (`write_file` só aceita `String`
  Lua, que precisa ser UTF-8 válido). Combinado com `pick_folder()`
  (0.58.4), dá o roteiro "escolher pasta → zipar direto lá" sem arquivo
  intermediário. Nova dependência `zip` (`deflate-flate2`, reaproveitando
  o `flate2` já existente).

## [0.58.4] — 2026-08-24

### Adicionado
- **`open_file`/`open_files`/`save_file`/`pick_folder`** (Luau) — diálogo de
  arquivo/pasta **nativo do SO** (via `rfd`), cobrindo os quatro modos:
  arquivo único, múltiplos arquivos, salvar como e diretório. Segue
  exatamente o mesmo padrão suspensivo de `confirm()`/`fetch()`: a
  corrotina Lua cede um pedido (`__glacier_file_dialog`), o motor mostra o
  diálogo fora da thread de UI (`iced::Task::perform`, sem travar o app) e
  retoma a corrotina com o resultado — `local caminho = open_file{...}`
  tem a aparência de `async/await` síncrono. Cancelado vira `nil`;
  `open_files` devolve um array Lua de caminhos. `opts` aceita `title`,
  `filters` (`{{name=, extensions={...}}, ...}`), `starting_dir` e (só em
  `save_file`) `default_name`. Só Lua por enquanto — mesma limitação que
  `confirm()` já tem pro lado Rust hoje. Exemplo em `examples/file_dialog/`.

## [0.58.3] — 2026-08-24

### Corrigido
- **`hidden`/`disabled` com placeholder** (`hidden="{oculto}"`) agora
  resolvem de verdade contra o contexto. Eram comparados no PARSE contra a
  string crua do placeholder (nunca `"true"`/`"false"` ainda), então o data
  binding que a documentação promete nunca ligava — um `hidden="{parado}"`
  deixava o elemento visível (ou o spinner girando) pra sempre. O valor com
  `{...}` agora vai para `UiNode::bool_templates` e é interpolado em
  `eval.rs` com o mesmo teste de verdade do `if` (`true`/`1`/`yes`/`on`/
  `sim`), mesma solução que os atributos numéricos (`NumAttr`) já tinham.

### Adicionado
- **`append_file(path, texto)`** (Luau) — irmão do `write_file` que
  acrescenta em vez de sobrescrever, criando o arquivo e os diretórios que
  faltarem na primeira chamada. Sustenta um log que sobrevive a um crash no
  meio de uma produção longa, sem manter o arquivo inteiro em memória nem
  suspender a corrotina num read-modify-write via `fetch("file://…")`.

## [0.58.2] — 2026-08-24

### Adicionado
- **`<MenuBar>`/`<Menu>`/`<MenuItem>`/`<MenuSeparator>`/`<ContextMenu>`** —
  menu bar ancorada (File/Edit-style) e menu de contexto (botão direito),
  com submenus recursivos a profundidade arbitrária (`crate::menu`, novo
  módulo). `<Menu>` aninha normalmente dentro de outro `<Menu>`/
  `<ContextMenu>` (mesma mecânica genérica de filhos que `<Column>`/`<Row>`
  já tinham — nenhum campo recursivo novo no AST); `items="chave"` alterna/
  complementa a markup estática com um array JSON dinâmico vindo do
  contexto, mesma convenção de `<Select options="…">`, permitindo montar o
  menu inteiro por Luau (`ctx.meu_menu = {...}`). Cliques em `<MenuItem>`
  reaproveitam 100% do roteamento de ação existente
  (`route_to_owner`/`LuauComponent::run_inner`) — nenhuma API nova no lado
  Lua. Estado do menu aberto é um único `Option` global em `GlacierUI`
  (como `dialog`), não uma instância por widget: só um menu/cascata pode
  estar aberto por vez no app inteiro, então nenhum dos pré-requisitos de
  "estado por instância" do `PLANO_WIDGETS.md` §3 se aplica. Overlay
  próprio (`stack![]` + posicionamento por `padding`, ancorado na última
  posição de cursor conhecida — não no `iced::advanced::Overlay` nativo,
  documentado como upgrade de v2 no cabeçalho de `menu.rs`), com flip de
  borda quando o painel não cabe na tela e destaque de hover por linha
  (via `button`, não `container` — só widgets com `Status` recebem o hover
  do iced sozinhos). Exemplo em `examples/menus/`.

## [0.58.1] — 2026-08-21

### Adicionado
- **`<template>`** — uma tag unificando `<ForEach>`/`<If>`/`<ElseIf>`/`<Else>`
  sob um nome só (o nome que Vue/Alpine já usam para a mesma ideia:
  `<template v-if>`/`<template x-if>`/`<template x-for>`), mapeando pros
  MESMOS `NodeType` (zero mudança em `eval.rs`/`widget.rs`) — a flavour
  depende de qual atributo está presente: `for-each="…" var="…"` (aceita
  também os aliases de `<ForEach>` — `items`/`source`/`itens`/`origem` — e
  os da forma-atributo — `forEach`/`foreach`/`each`/`repeat`) vira
  `NodeType::ForEach`; `else` (bare) vira `NodeType::Else`; `else-if="…"`
  vira `NodeType::ElseIf`; `if="…"`/`cond="…"` vira `NodeType::If`; sem
  nenhum desses, `<template>` agrupa os filhos incondicionalmente (útil
  para um componente devolver mais de uma raiz sem `<Row>`/`<Column>`
  artificial). Como `<If>`/`<ForEach>`, hoista os filhos como irmãos do
  pai — SEM nó wrapper — o que a forma-atributo `if=`/`for-each=` num
  elemento comum não faz (ela sempre produz um único nó: o próprio
  elemento). Combinar `for-each` e `if`/`cond` no MESMO `<template>` não é
  suportado (a filtragem por item vai num `<template if=…>` ANINHADO no
  corpo do `for-each`, não em atributos irmãos na mesma tag) — mesma
  limitação que já existia entre `<ForEach>` e `<If>` como tags separadas.
  **Nota de implementação**: as quatro tags legadas já tinham exatamente
  essa semântica de "hoist sem wrapper" (`eval.rs::expand_children`, passo
  4) — o que faltava era só o NOME `<template>`; por isso `<template>` lê
  `if`/`else`/`else-if`/`for-each` pelo vocabulário das TAGS (`cond`/
  `items`), não pelo da forma-atributo (que usa os mesmos nomes de
  atributo em QUALQUER elemento e teria interceptado o nó antes do
  despacho por `NodeType` chegar a rodar). Estudo e proposta em
  `docs/plano-convergencia-templates-gui-webui.md` (rustploy) — é
  justamente o par sintático que a Fase 4 desse plano (transpilação para
  Alpine no browser) já ia gerar como SAÍDA; agora o `.gv` fonte pode
  falar a mesma língua.

## [0.58.0] — 2026-08-21

Sem quebra de API — o bump de *minor* aqui é só a marca da Fase 1 do plano
de convergência de templates do rustploy fechando (o próprio plano já
nomeava a versão-alvo da fase como "glacier-ui 0.58"), não o sinal de
incompatibilidade que a convenção deste changelog normalmente usa.

### Adicionado
- **`platform="desktop"`/`"web"`** em qualquer elemento — filtro
  independente de `if`/`else-if`/`else` (não participa da cadeia, não
  mexe em `last_if`, não precisa de `cond`/`if=` nenhum pra existir):
  `<if platform="desktop">`/`<column platform="web">`/combinado com outro
  directive no mesmo nó. Um valor que não bate com o novo
  `eval::current_platform()` — `"desktop"` para todo alvo de hoje,
  `"web"` só quando/se um alvo `wasm32` existir — some o nó inteiro, mesmo
  tratamento do skip de `<import>`/`<link>`/`<style>`. Deixa cromo
  só-desktop (titlebar borderless, resize handles) e só-web (PWA/service
  worker) convivendo no MESMO arquivo em vez de forçar dois. Item 7
  (opcional) da Fase 1 do plano de convergência de templates do rustploy —
  **fecha a Fase 1** (7/7 itens).

## [0.57.11] — 2026-08-21

### Adicionado
- **`href` de `<link rel="import" href="…">` resolvido relativo ao arquivo
  importador**, como o `require` do Luau já faz desde a 0.22 (mesmo
  algoritmo de normalização de caminho, `luau::normalize_key`, agora
  reaproveitado fora do universo Luau). Um `href` nu (`href="child.gv"`)
  passa a resolver contra o diretório do `.gv` que declarou o `<link>`, não
  contra o CWD do processo — permite dois templates vizinhos numa subpasta
  se referenciarem sem o caminho completo desde a raiz do workspace. O
  caminho absoluto-do-workspace continua funcionando sem mudança nenhuma:
  a resolução só troca de candidato quando o relativo ao importador não
  existe. Mesmo tratamento aplicado à forma tag `<Import from="…">`. Item 6
  da Fase 1 do plano de convergência de templates do rustploy.

## [0.57.10] — 2026-08-21

### Adicionado
- **`<hr>`/`<Hr>`/`<HR>` como alias de `<Rule>`.** O parser já tinha o
  mecanismo de alias (`if`/`se`, `on_change`/`on-change`…); só faltava
  aplicá-lo aqui. Sem mudança de comportamento — `<hr />` parseia
  exatamente como `<rule />` já parseava. Item 5 da Fase 1 do plano de
  convergência de templates do rustploy.

## [0.57.9] — 2026-08-21

### Adicionado
- **`empty`/`not_empty`** — condição bare (como `else`) que lê `cond` como o
  JSON cru de uma lista já presente no contexto (`ctx.proj_secrets =
  "[...]"`) e casa se ela tiver zero elementos (ou não for um array JSON
  válido — "sem lista ainda" também conta como vazio) ou o oposto. Funciona
  nas duas formas de `if`/`else-if` (atributo e tag). Aposenta os `*_count`
  que só existiam pra um template ter algo pra comparar com `equals="0"`.
  Item 4 da Fase 1 do plano de convergência de templates do rustploy.

### Alterado
- `normalize_bare_directives` (o preprocessor que reescreve `else`/`senao`
  desacompanhados) generalizado pra uma tabela de palavras em vez de dois
  blocos hardcoded — mesma proteção de fronteira de nome que a 0.57.7 deu
  pro `else-if`, agora reaproveitada pro `empty`/`not_empty` novos.

## [0.57.8] — 2026-08-21

### Adicionado
- **`one_of`** (alias `equals_any`) — compara `cond` contra uma lista
  separada por espaço, casando se bater com QUALQUER token dela:
  `if cond="{view}" one_of="projects project new_service service"`. Funciona
  nas duas formas de `if`/`else-if` (atributo e tag). Interpola a lista
  inteira de uma vez (não token a token), então tanto uma lista literal
  quanto uma vinda de variável (`one_of="{allowed}"`) funcionam — sem
  inventar gramática de expressão nova. Item 3 da Fase 1 do plano de
  convergência de templates do rustploy: o caso motivador é manter um item
  de navegação "aceso" em várias sub-telas ao mesmo tempo.

## [0.57.7] — 2026-08-21

### Adicionado
- **`else-if`**, nas duas formas que `if`/`else` já têm: atributo
  (`else-if="{x}" equals="y"` em qualquer elemento) e tag
  (`<ElseIf cond="{x}" equals="y">…</ElseIf>`). Encadeia com o `if`/`else-if`
  imediatamente anterior via o `last_if` que `expand_children` já mantinha —
  só avalia a própria condição quando o branch anterior deu falso, e
  short-circuita (nem avalia) quando algum já casou antes. Aplaina cadeias de
  tela que hoje precisam de um `<if>` aninhado dentro de cada `<else>`
  (um nível de indentação a mais por branch). Item 2 da Fase 1 do plano de
  convergência de templates do rustploy.

### Corrigido
- **`normalize_bare_directives` confundia `else-if`/`senao-if` com um
  `else`/`senao` desacompanhado.** O preprocessor que reescreve `<Text
  else>` em `<Text else="">` só checava se o que vinha depois das 4/5 letras
  era `=` (pulando espaços) — não se era parte de um nome de atributo mais
  longo. `else-if="{x}"` virava `else=""-if="{x}"`, XML inválido ("expected a
  whitespace not '-'"). Achado ao implementar o `else-if` acima; agora só
  reescreve quando o que segue é fronteira de nome (espaço, `=`, `>`, `/`).

## [0.57.6] — 2026-08-21

### Adicionado
- **`<Button>` aceita filhos.** Até aqui o conteúdo do botão só vinha do
  atalho `text="…"`; qualquer filho declarado era parseado (o campo
  `children` já é genérico pra todo `NodeType`) mas ignorado na renderização.
  Agora, quando o nó tem filhos visíveis, eles viram o conteúdo do botão —
  um filho único é usado direto (mesma ideia do `<Container>`), mais de um
  vira um `Row` implícito respeitando `spacing`/`align-y` do próprio nó
  (o caso comum é ícone + rótulo lado a lado). `text="…"` continua
  funcionando sem mudança quando não há filho nenhum. Item 1 da Fase 1 do
  plano de convergência de templates do rustploy (elimina o hack de usar
  `<row on_press>` no lugar de um `<Button>` de verdade, ex.: item de
  sidebar em `nav_item.gv`).

## [0.57.5] — 2026-08-20

### Corrigido
- **A bandeja não corrompe mais o parser Luau em locale de vírgula decimal.**
  `gtk::init()` (usado pela bandeja no Linux, via libappindicator) chama
  `setlocale(LC_ALL, "")`, e locale é estado **global do processo**. Em `pt_BR`,
  `de_DE`, `fr_FR` e afins o separador decimal vira vírgula, o `strtod` da libc
  passa a parar no ponto de `1024.0` — e o lexer do Luau, que usa `strtod` para
  converter literais numéricos, passa a rejeitar **todo número decimal** com
  `Malformed number`. Na prática, qualquer `require` de um módulo contendo um
  literal como `1024.0` falhava e o app quebrava.

  A thread da bandeja agora faz `setlocale(LC_NUMERIC, "C")` logo após o
  `gtk::init()`: datas, ordenação e textos seguem no locale do usuário, e só a
  conversão numérica volta ao neutro que o parser exige.

  Por que era difícil de enxergar: o bug **não reproduz** em máquina com locale
  de ponto decimal, e **não aparece nos testes** (que não sobem GTK, então o
  processo fica no locale `"C"`). Pior, num binário compilado com
  `panic = "abort"` o erro de Lua nem chega a ser reportado — ele viaja
  desenrolando a pilha através dos frames do `mlua`, que aí são `nounwind`, e o
  processo morre com `SIGABRT` sem mensagem nenhuma. Apps que embarcam a glacier
  devem manter `panic = "unwind"`, que é o que o `mlua` exige.

---

## [0.57.4] — 2026-08-19

### Corrigido
- **`<ComboEdit>` ainda sumia o texto ao desfocar depois de digitar um valor
  livre (uma URL nova, ainda não salva) do zero** — sobrou depois da 0.57.3,
  que só cobria o caso de "acabou de selecionar uma opção salva". Causa: um
  mismatch dentro do próprio `iced_widget::combo_box::ComboBox` (não é código
  do glacier-ui) entre `layout()` e `draw()`. Os dois recebem um override
  opcional pro texto a mostrar quando o campo está desfocado
  (`self.selection`, formatado a partir da opção combinada) — mas com
  condições diferentes: `draw()` só usa o override se `self.selection` não
  estiver vazia, senão cai no buffer digitado; `layout()` aplica o override
  sempre que desfocado, **mesmo vazio**. Como `layout()` roda antes de
  `draw()` e é quem atualiza o parágrafo em cache que `draw()` de fato
  desenha (`state.value.raw()`), um valor digitado que não bate com nenhuma
  opção salva (`self.selection` vazia) fazia `layout()` sobrescrever esse
  cache com string vazia no frame em que o campo perdia o foco — o texto
  digitado continuava certo no contexto e no buffer interno do combo, só não
  era o que ia pra tela. Sem acesso ao código do `iced_widget` (é
  dependência do crates.io, não faz sentido fork/patch pra isso), a correção
  ficou no ponto de chamada (`widget.rs`): quando o valor do contexto não
  bate com nenhuma opção salva, passa pro `combo_box()` uma `SelectOption`
  sintética (`label = value = current`) em vez de `None`, garantindo que
  `self.selection` nunca fique vazia enquanto o valor digitado não for vazio
  — os dois caminhos (`layout()`/`draw()`) passam a concordar.

## [0.57.3] — 2026-08-19

### Corrigido
- **`<ComboEdit>` "piscava" texto sumindo/reaparecendo ao trocar o foco** —
  dois efeitos compostos do `combo_box::State` do iced (0.14):
  1. Ao escolher uma opção do dropdown (clique ou Enter), o próprio
     `combo_box` do iced zera o buffer interno de texto digitado como parte
     da seleção (pra limpar o filtro de busca) — mas o motor marcava
     `combo_synced` como já sincronizado nesse mesmo evento (`UiComboSelected`),
     então `sync_combos()` nunca via motivo pra reconstruir o `State` depois.
     Resultado: focar o campo de novo mostrava vazio (o buffer zerado),
     desfocar mostrava certo de novo (fallback pro valor do contexto, que
     nunca tinha sido tocado) — te dava a piscada. Removida a sincronização
     prematura só em `UiComboSelected` (mantida em `UiComboInput`, onde é
     necessária pra não atropelar uma digitação em andamento): agora a
     seleção força `sync_combos()` a reconstruir o `State`, o que reseeda o
     buffer certo.
  2. `sync_combos()` reconstruía o `State` com `with_selection(opts, selected)`,
     onde `selected` vinha só de `opts.iter().find(|o| o.value == ctx_val)` —
     se `ctx_val` fosse texto livre digitado (uma URL nova, ainda não salva)
     que não batia com nenhuma opção salva, `selected` virava `None` e o
     buffer interno era seedado com string vazia em vez do texto digitado,
     mesmo com `context_data` guardando o valor certo. Agora, sem match nos
     `options`, cai numa `SelectOption` sintética (`label = value = ctx_val`)
     só pro seed do buffer — não entra na lista de opções do dropdown.

## [0.57.2] — 2026-08-19

### Corrigido
- **`ActivateRequested` (ping do [`single_instance`](GlacierDaemon::single_instance))
  nunca chegava em `update()`** — a causa real por trás do "nem o foco nem o
  loading do launcher melhoraram" observado testando a 0.57.1. `event_stream`
  bloqueava numa thread dedicada em `std::net::TcpListener::accept()`,
  ponteada pro lado async por um `std::sync::mpsc` (o mesmo padrão que
  `crate::tray::event_stream` usa pro `tray-icon`, que é síncrono por
  natureza). O problema: `iced::stream::channel` roda o corpo dentro de
  `futures::stream::select(receiver, stream::once(corpo))`, e uma chamada de
  `poll()` que nunca devolve `Poll::Pending` (por bloquear a thread de
  verdade em vez de ceder via `.await`) morre de fome pro lado `receiver`
  dentro do mesmo combinator — o item chegava a ser mandado pro canal
  interno (confirmado com `ss` vendo o accept+close no SO, e depois com
  `eprintln!` vendo o `output.send` retornar `Ok`), mas a metade que o
  entregaria pra fora nunca era repolada. Trocado por
  `tokio::net::TcpListener::accept().await` (feature `net` do tokio, já
  habilitada) — sem thread dedicada, sem canal síncrono, cede de verdade a
  cada iteração. Confirmado corrigido via `eprintln!` temporário no
  `update()`: antes, nada; depois, `ActivateRequested recebido` a cada ping.

## [0.57.1] — 2026-08-19

### Corrigido
- **"Open"/reabertura da principal não trazia a janela pra frente no Wayland
  nativo** — `open_main()` (bandeja e [`single_instance`](GlacierDaemon::single_instance))
  só chamava `window::gain_focus`, que por baixo é o `focus_window()` do
  winit: no X11 manda `_NET_ACTIVE_WINDOW` e funciona, mas no Wayland nativo é
  **no-op** (o protocolo não deixa um cliente ativar a janela de outro à
  força — mesma classe de restrição do `window:drag`, já documentada). Agora
  soma um `request_user_attention(Critical)`: no Wayland o winit implementa
  isso via `xdg_activation_v1` (o cliente pede um token pra própria
  superfície e se auto-ativa), que Mutter/KWin honram; no X11 vira
  `XUrgencyHint`, inofensivo por cima do `gain_focus` que já resolve ali.
  Efeito colateral esperado: também deve encurtar o "carregando" que o shell
  do Wayland mostra ao clicar no launcher com o app já rodando — aquele
  indicador só some quando o compositor associa uma ativação de janela ao
  lançamento, o que antes nunca acontecia numa segunda tentativa que só pinga
  a instância existente e sai sem abrir janela nenhuma.

## [0.57.0] — 2026-08-19

### Adicionado
- **`GlacierDaemon::single_instance(app_id)`** — trava de instância única por
  processo do usuário: uma segunda tentativa de lançar o app pinga a primeira
  instância (que reabre/foca a janela principal — mesmo caminho do "Open" da
  bandeja) e `run()` retorna de imediato **sem** construir motor nem abrir
  janela nessa segunda tentativa. Pensado para apps com bandeja
  ([`GlacierDaemon::tray`]) que sobrevivem à última janela: sem a trava, cada
  clique perdido no launcher enquanto o app já está recolhido na bandeja abre
  uma instância nova, e o usuário acumula N processos sem perceber. A trava é
  um `TcpListener` em loopback numa porta derivada do `app_id` (novo módulo
  interno `single_instance`, mesmo padrão de thread dedicada + subscription do
  `tray`) — ver `src/single_instance.rs` para a troca feita (sem depender de
  mais crates, ao custo de uma chance pequena de colisão de porta com outro
  processo qualquer).

## [0.56.0] — 2026-08-06

### Adicionado
- **`<ComboEdit>`** — dropdown editável/pesquisável (`combo_box` do iced),
  primeiro widget do catálogo (`PLANO_WIDGETS.md`) a sair do planejamento:
  ao contrário de `<Select>`/`pick_list` (sem estado, reconstruído do zero a
  cada frame a partir do contexto), o `combo_box` do iced é **com estado** —
  precisa de um `combo_box::State` que sobrevive entre frames pra não perder
  o texto/filtro em digitação. O motor mantém esse estado num mapa novo
  (`GlacierUI::combos`/`widget::ComboMap`), keyed pelo binding `value`, com o
  mesmo mecanismo de "própria edição vs. mudança externa" que o `<TextArea>`
  já usa pros seus buffers (`combo_synced`/`combo_options_synced`,
  reconciliados em `GlacierUI::sync_combos`). Atributos: `options`/
  `label_field`/`value_field` (mesma forma do `<Select>`, array JSON no
  contexto), `value` (texto atual, digitado ou selecionado), `onChange`
  (cada tecla digitada — o motor já escreve o texto em `value` sozinho,
  como o `<TextArea>` faz com o seu buffer) e `onSelect` (só quando um item
  *já existente* da lista é escolhido — clique ou Enter num item filtrado),
  para um script reagir diferente a "digitou algo novo" vs. "escolheu algo
  que já conhecíamos". Caso de uso motivador (ver `examples/combo_edit/`):
  um campo de servidor com uma lista de pares servidor/token salvos, onde
  escolher um servidor da lista preenche o token sozinho.
- **`<TimePicker>`** — builtin composto (`TextInput` + `Button`) para
  entrada de horário: digitação direta no campo (`value`/`onChange`) ou um
  botão de ação (`onPick`, tipicamente abrindo um modal/diálogo próprio do
  app) para um seletor visual. Puramente apresentacional — sem estado
  próprio, sem `init`; a validação de formato e o modal de seleção ficam a
  cargo do app consumidor. Ver `examples/timepicker/`.
- **`GlacierDaemon::antialiasing(bool)`** — liga/desliga o MSAAx4 default do
  renderer do iced. Default `true` (preserva o comportamento atual). Motivado
  por uma investigação de scroll travando num app: numa máquina sem GPU
  compatível com o `wgpu` (Vulkan "não-compliant" recusado pelo driver, GL não
  enumerando adapter algum — ver `wgpu_hal::vulkan::adapter`), todo o
  renderizador cai pra software (`llvmpipe` ou o `tiny-skia` do próprio iced),
  e o antialiasing multiplica esse custo várias vezes por não ter hardware pra
  absorvê-lo. Confirmado via `perf`: a maior parte do tempo de CPU durante o
  scroll estava dentro de funções do rasterizador por software
  (`tiny_skia::pipeline::highp::bicubic`/`gradient`).
- **`AssetSource::supports_reload(&self) -> bool`** — novo método com corpo
  default (`true`, não quebra implementações existentes). Uma fonte embutida
  (assets compilados no binário) deve sobrescrever para `false`: o tick de
  hot-reload do `GlacierDaemon` agora consulta isto para decidir se inclui a
  subscription do timer, em vez de rodá-la pra sempre sem nunca ter trabalho
  de verdade a fazer.

### Corrigido
- **Tickers de hot-reload e de expiração de toast só rodam quando podem ter
  efeito** — antes, `GlacierDaemon::subscription` sempre incluía
  `iced::time::every(reload_period)` e `iced::time::every(toast_period)`,
  mesmo sem nenhuma janela com asset recarregável ou toast ativo. Cada tick
  processado força o `iced` a reconstruir a tela inteira via `view()` (é assim
  que o loop do `iced_winit` funciona — qualquer `Message` implica um redraw
  completo, não só das janelas afetadas). Num fallback por software (ver
  acima), isso custava um redraw cheio a cada ~250-500ms mesmo com o app
  parado. Agora cada ticker só entra na subscription quando
  `AssetSource::supports_reload()` é `true` em pelo menos uma janela (reload) ou
  há pelo menos um toast em exibição (toast).

## [0.54.0] — 2026-07-19

### Adicionado
- **`GlacierUI::register` liga `<script>` Luau, quando houver** — um
  `Component` Rust registrado via `register(Box<dyn Component>)` deixa de ter
  seu `<script>` descartado silenciosamente: se o `Template::File` carrega um
  bloco `<script>`, `LuauComponent::wrap` (`src/luau/mod.rs`) o embrulha, e
  cada ação passa a resolver por precedência — a função Lua de mesmo nome
  vence se existir; senão o hook Rust correspondente (`update`, `init`,
  `on_form_submit`, `on_broadcast`) roda no lugar. Unifica os dois caminhos
  de registro (`register` vs. `register_component`) quanto a scripting: antes
  só `register_component(name, path)` (sem `Box<dyn Component>`) aceitava
  Lua. `Template::Inline` continua sem suporte a `<script>` (sem `path` para
  resolver `src`/`require`, mesma limitação que `LuauComponent` sempre teve).
  Não quebra nada existente: nenhum registro via `register` tinha `<script>`
  até aqui.

## [0.53.0] — 2026-07-19

### Adicionado
- **`<ProgressBar>` formalizado como primitiva** (`QProgressBar` do Qt):
  `value`/`valor` (chave de contexto numérica — ausente/não-numérica conta
  como `min`), `min`/`max` (padrão `0`/`100`), `vertical`, `showValue`
  (percentual centralizado, como o `QProgressBar::textVisible`, default do
  Qt) e `color`/`cor` (o preenchimento — a track usa o `background` genérico
  do nó). Os quatro estilos builtin (`crate::style`) ganharam uma regra de
  tag `ProgressBar { }` — verde no `FROST` (como o Windows/Vista), `primary`
  nos outros três (como o Fusion do Widget Gallery).
- **`<Spinner>`/`<BusyIndicator>`** — o `QProgressBar` indeterminado
  (`setRange(0, 0)`)/`BusyIndicator` do QML: um anel de pontos girando sem
  fim, para operações sem duração conhecida. Reclassificado de `●` para `—`
  no `PLANO_WIDGETS.md`: ao contrário de `Tabs`/`SpinBox`/`Calendar`, um
  indicador indeterminado não guarda **valor** algum — só uma fase de
  rotação, que mora no `tree::State` do próprio widget (mesmo mecanismo do
  `AnimatedToggler`, ver `ANIMACOES.md`), então N instâncias na tela giram
  cada uma com seu relógio sem tocar o contexto global. Desenhado com
  `fill_quad` (pontos num anel, opacidade decaindo pelo rastro), sem puxar o
  trait `canvas`/`geometry::Renderer`. `color`/`cor`; sem cor, usa o
  `primary` do tema ativo (inclusive o de um estilo builtin).
- Exemplo `galeria_estilos` ganhou a seção "Indicadores" (progress bar
  bindado + spinner + botão que avança 10% por clique, voltando a 0 ao
  passar de 100%).

### Corrigido
- **`<ProgressBar>` sem `width` explícito colapsava a quase-zero** — visível
  demais quando um estilo builtin (ou classe do app) declara `ProgressBar {
  background; border-radius }`: como esses são campos **genéricos** do nó, o
  wrap "embrulha em `Container` se tiver background/borda" (compartilhado por
  todo `render_node`) entrava em ação e envolvia a barra num `Container` sem
  `width` — ou seja, `Length::Shrink`. Um `Shrink` ao redor do `Length::Fill`
  (o default do `progress_bar` do iced) colapsa a barra a quase-nada, sobrando
  só o `<Spinner>` vizinho visível. `ProgressBar` agora fica de fora desse
  wrap (já pinta o próprio trilho/borda no `.style()`, então era redundante
  de qualquer forma). `Button`/`Select` nunca sofreram disso: seu tamanho
  natural já é `Shrink`, não têm o que colapsar.

---

## [0.52.0] — 2026-07-18

### Adicionado
- **Estilos builtin (`glacier_ui::style`)** — o análogo dos `QStyle` do Qt:
  quatro estilos prontos (`FROST` claro nativo, `FUSION` claro cinza,
  `FUSION_DARK` escuro azul e `PHANTOM` escuro grafite), cada um uma
  `const Style` com paleta (vira o `iced::Theme`) + GSS de regras de tag
  (`Button { }`, `Select { }`, com `:hover`/`:active`/`:disabled`/`:focus`)
  instalado como **underlay** — abaixo de qualquer `.gss` do app, então
  classes/ids/atributos inline e `<link rel="theme">` continuam vencendo.
  - `GlacierDaemon::style(style::FUSION_DARK)` define o default do app inteiro
    (todas as janelas: principal, reabertura pela bandeja e filhas de
    `open_window`) — o análogo do `QApplication::setStyle`.
  - `GlacierUI::set_style(&Style)` para apps de janela única (ou troca manual).
  - **Troca em runtime sem componente**: ação builtin `style:<nome>` num botão,
    ou `onChange="style:set"` num `<Select>` (o valor escolhido é o nome). O
    nome do estilo ativo fica no contexto em `glacier_style`
    (`style::CONTEXT_KEY`) — é o `value` que o `<Select>` exibe.
  - O GSS de cada estilo publica a paleta como variáveis (`var(--primary)`,
    `var(--surface)`, `var(--border)`, …) para o `.gss` do app se ancorar nas
    cores do estilo ativo.
  - Um app pode declarar o próprio `const Style { … }` (campos `&'static str`)
    e passá-lo aos mesmos pontos.
  - Novo exemplo `galeria_estilos` — widget gallery com o combo "Style:" do Qt.
  - Novo `RenderInputs::install_underlay_stylesheet` (folha na posição 0,
    substituída no lugar pela chave ao trocar de estilo).
- **`<Toggle>` com animação.** O toggler do iced desenha o knob teleportando;
  o novo widget `AnimatedToggler` (`crate::animated_toggler`, usado por todo
  `<Toggle>`) desliza a bolinha e mistura as cores do trilho em 200ms
  (easeOutCubic), via `iced::animation::Animation` — mesmo catálogo de
  estilo/paleta do toggler original, então temas e estilos builtin valem sem
  mudança. Anima só enquanto corre (um `request_redraw` por quadro); parado,
  custo zero.
- **`<Checkbox tristate="true">`** — três estados na variável de contexto,
  ciclados a cada clique na ordem do `Qt::CheckState`: `"false"` → `"mixed"` →
  `"true"`. O estado `"mixed"` desenha um traço (−) no lugar do check. Sem o
  atributo, nada muda (binário, como antes).

### Corrigido
- **`<Select>` sem `padding` ficava com o texto colado na borda** (o motor
  passava `Padding::ZERO` ao `pick_list`). Os quatro estilos builtin agora
  declaram `Select { padding: 6 10 }`, alinhando a altura do combo à dos
  inputs; um `padding` inline ou de classe continua vencendo.

### Quebras
- `NodeType::Checkbox` ganhou o campo `tristate: bool`. Só afeta quem
  desestrutura/constrói a variante sem `..` — inclua `tristate` (ou `..`) no
  padrão; `false` reproduz o comportamento anterior.

---

## [0.51.0] — 2026-07-18

### Adicionado
- **Descompressão gzip transparente no `fetch`.** As requisições agora mandam
  `Accept-Encoding: gzip` por padrão (a menos que o chamador já tenha definido
  um `Accept-Encoding`), e uma resposta com `Content-Encoding: gzip` é
  descomprimida antes de chegar ao Lua — que recebe o mesmo `body` de texto de
  sempre. Um servidor que não comprima ignora o header; o ganho aparece em
  conexões remotas (JSON comprime bem), não em localhost. Só o `fetch`
  unário: o **SSE** (`sse`) continua sem compressão de propósito (stream de
  vida longa exigiria gzip com flush por evento, com taxa pior e atrito com
  proxies). Teste: `gunzip_round_trip`.

## [0.50.1] — 2026-07-18

### Performance
- **`<image>`/`<svg>` agora memoizam o `Handle` por caminho.** Como o
  `render_node` roda a cada redraw, a leitura via `AssetSource` da 0.50.0
  releria o arquivo do disco (dev/`DiskAssets`) ou recopiaria+re-hashearia os
  bytes embutidos (release) a cada quadro, por nó de imagem. Um cache por thread
  (identidade = caminho; assets binários são imutáveis no processo) constrói o
  handle uma vez e o reusa — inclusive entre motores/janelas. Sem mudança de
  API.

## [0.50.0] — 2026-07-18

### Adicionado
- **Camada de resolução de assets (`AssetSource`) — binários standalone.** Todo
  asset que o motor lê em runtime (templates `.gv`/`.kdl`, estilos `.gss`, JSON de
  tema/dados, scripts Luau e binários SVG/imagem) passa por um
  [`AssetSource`](crate::AssetSource) em vez de tocar o `std::fs` direto. O default
  [`DiskAssets`] lê do disco exatamente como antes (com hot-reload), então nada
  muda para quem não fizer nada.
  - Injete uma fonte **embutida** com `GlacierDaemon::assets(Arc<dyn AssetSource>)`
    (ou `GlacierUI::with_asset_source`, `LuauComponent::from_file_with`/
    `from_source_with`) para um binário **100% desacoplado dos arquivos**: um app
    de release pode empacotar todos os seus assets em tempo de compilação (ex.:
    `include_dir!`) atrás de um `AssetSource` e rodar sem a árvore de assets no
    disco. O padrão recomendado é injetar só em release
    (`#[cfg(not(debug_assertions))]`), deixando o dev com disco + hot-reload.
  - Numa fonte embutida, `AssetSource::modified` devolve `None`, o que **desliga o
    hot-reload naturalmente** (`check_reload` vira no-op — não há arquivo a vigiar).
  - Os `<svg>`/`<image>` agora carregam via `from_memory`/`from_bytes` (bytes da
    fonte de assets) em vez de `Handle::from_path`, para funcionarem embutidos.

### Quebras
- `render_node` ganhou um parâmetro final `assets: &dyn AssetSource`. Quem chama o
  motor pela API pública (`GlacierDaemon`/`GlacierApp`/`GlacierUI::render_current`)
  não é afetado; só quem chamava `render_node` diretamente precisa passar a fonte
  (use `&DiskAssets` para o comportamento anterior).

## [0.49.1] — 2026-07-18

### Corrigido
- **`remember_window_geometry` não persistia nada sem um gancho `on_close`.** O
  fechamento da principal só **consultava** a geometria (e, portanto, só disparava
  a gravação) quando havia um `on_close` registrado — a persistência nativa
  ligada por `remember_window_geometry` era ignorada, o `window-geometry.json`
  nunca era escrito e o app reabria sempre no tamanho default. Agora o fechamento
  consulta a geometria quando há `on_close` **ou** a persistência nativa está
  ligada. Regressão: `remember_geometry_consulta_geometria_ao_fechar_sem_on_close`.

## [0.49.0] — 2026-07-18

### Adicionado
- **I/O de arquivo local na camada Luau.** Antes, um `<script>` não tinha como
  ler nem gravar arquivo — a única persistência era o `storage` (JSON chaveado
  gerenciado pelo motor). Agora:
  - **Leitura** via `fetch("file://<caminho>")`: em vez de uma requisição HTTP,
    lê o arquivo local e devolve o **mesmo** formato de sempre
    (`{ ok, status, body, error }` — `200`/`ok` com o conteúdo no `body`,
    `404`/`error` quando não existe). Um script lê um arquivo com a mesma chamada
    com que faria um GET. (Ao contrário do browser, que bloqueia `file://` de
    propósito por ser código remoto; aqui o Luau é código do próprio app.)
  - **Escrita** via o global `write_file(path, conteúdo)` → `true` no sucesso ou
    `false, "<mensagem>"` na falha (cria o diretório pai se preciso; nunca derruba
    o script). Síncrono, como o `storage`.
- **Persistência automática da geometria da janela principal**, opt-in via
  `GlacierDaemon::remember_window_geometry(true)`. Com ela, o tamanho (e a
  posição, onde a plataforma a expõe) é gravado ao fechar e restaurado ao abrir,
  reabrindo o app onde parou — **sem flash** (a janela já nasce no tamanho certo)
  e sempre respeitando o `min_size`. O arquivo (`window-geometry.json`) mora sob o
  `storage_dir`; sem um `storage_dir` a opção é no-op. No Wayland só o tamanho
  volta (o protocolo não expõe a posição ao cliente). Substitui o padrão de um app
  fazer isso à mão via `on_close` + `window::Settings` montadas na inicialização.

## [0.48.0] — 2026-07-17

### Mudado
- **Bandeja: fechar a janela principal agora a recolhe SEM matar o motor.**
  Antes, fechar a última janela (com bandeja) encerrava a janela e **descartava o
  motor** dela — junto com o login e qualquer stream `sse`/`websocket` vivo. Um
  app de bandeja que precisa continuar recebendo eventos (ex.: notificar quando um
  deploy termina) ficava sem conexão nenhuma enquanto recolhido.

  Agora, com bandeja configurada, fechar a principal **destaca** o motor: a janela
  do SO é destruída (no Wayland esconder/minimizar-restaurar é impossível pelo
  toolkit — destruir é a única forma de a janela sumir de verdade), mas o **motor
  segue vivo e headless** — SSE conectado, login intacto, notificações do SO
  continuam disparando. O `open_main()` (item "abrir" da bandeja) **religa esse
  mesmo motor** numa janela nova, preservando a sessão; o `main_id` migra para a
  janela nova (o recipe do `sse`/`websocket` inclui o id da janela, então há um
  breve reconnect do stream no instante da reabertura — irrelevante).

  Sem bandeja, nada muda: fechar a última janela encerra o app como sempre.

### Compatibilidade
- Sem quebras de API. É uma mudança de **comportamento** restrita a apps que usam
  `.tray(...)`: a principal passa a recolher (motor vivo) em vez de destruir. Apps
  sem bandeja não são afetados.

---

## [0.47.0] — 2026-07-17

### Adicionado
- **Ícone de bandeja (system tray) + app que sobrevive à última janela**, atrás
  da feature **`tray`** (opcional; sem ela nada de GTK/tray-icon é arrastado). No
  builder do `GlacierDaemon`:
  - `.tray(TrayConfig { icon, tooltip, items })` — habilita a bandeja. Com ela
    configurada, **fechar a última janela não encerra mais o app**: ele recolhe
    para a bandeja. Sem bandeja, o comportamento é o de sempre (encerra na última
    janela).
  - `.on_tray(|id, &mut TrayActions| { … })` — gancho de clique nos itens do
    menu. `TrayActions` oferece `open_main()` (reabre/foca a principal), `quit()`
    (encerra), `set_label(id, text)` e `set_checked(id, bool)`.
  - `TrayItem::button/check/separator` para montar o menu.
  - Funções globais `notifications_enabled()` / `set_notifications_enabled(bool)`:
    interruptor de processo que o `notify()` consulta antes de emitir — o gancho
    da bandeja liga/desliga as notificações do SO sem passar pela camada Luau.

  A bandeja roda numa **thread dedicada** (o `iced`/`winit` é dono do loop
  principal): Linux via libappindicator+GTK, Windows via message-loop Win32.
  **macOS não é suportado** (exige a thread principal) — lá `.tray(...)` é
  ignorada e o app volta a encerrar na última janela. No Linux não há evento de
  clique no ícone (o clique abre o menu); no Windows o clique esquerdo reabre a
  principal. Ver `src/tray.rs` e `examples/bandeja`.

### Compatibilidade
- Sem quebras: a feature `tray` é opt-in e toda a API nova é aditiva. Quem não a
  habilita compila exatamente como antes (nenhuma dep nova).

---

## [0.46.0] — 2026-07-15

### Mudado
- **`confirm()` (Luau) agora é SÍNCRONO e retorna um booleano**, no espírito do
  `fetch`: suspende a corrotina, exibe o diálogo e só retoma quando o usuário
  escolhe um botão — devolvendo `true` (confirmou) ou `false`
  (cancelou/dispensou). Deixa o fluxo linear, sem callback separado:
  ```lua
  if confirm({ title = "Remover?", message = "…", confirm_label = "Remover",
               destructive = true }) then
      -- fazer a ação aqui mesmo
  end
  ```

### Quebras
- **`confirm{ confirm_action = "…" }` deixou de existir.** Antes, `confirm` não
  suspendia e o botão de confirmação despachava a função nomeada em
  `confirm_action` como um clique à parte. Agora não há `confirm_action`: trate
  o retorno booleano. Migração:
  ```lua
  -- antes
  confirm({ title = "T", message = "M", confirm_action = "do_x" })
  function do_x() ... end
  -- depois
  if confirm({ title = "T", message = "M" }) then ... end
  ```
  Diálogos abertos pela API Rust (`Context::show_dialog` com botões que roteiam
  ações) não mudam — só o `confirm()` da camada Luau passou a suspender.

---

## [0.45.0] — 2026-07-15

### Adicionado
- **`GlacierDaemon::storage_dir(dir)`**: define o diretório onde o global
  `storage` (persistência local em JSON, análoga a `localStorage`) grava seus
  arquivos, aplicado a todas as janelas do app. Sem isto, `storage` mantém o
  comportamento legado — grava em `.glacier-storage/` **relativo ao diretório do
  script**, o que falha silenciosamente quando os assets moram num caminho
  read-only (um app empacotado rodando de `/usr/share`). Passe um diretório
  gravável pelo usuário (ex.: o data dir do XDG) e o `storage` passa a gravar
  em `<dir>/.glacier-storage/<componente>.json`. Também exposto o helper de
  baixo nível `luau::set_storage_root(path)` que o builder usa por baixo.

---

## [0.44.0] — 2026-07-15

### Mudado
- **`notify()` no Linux passa a emitir via `notify-send`** (subprocesso), com
  fallback automático para o `notify-rust` in-process se o `notify-send` não
  estiver instalado. Em outros SOs (Windows/macOS) nada muda — segue in-process.
  Motivo: alguns ambientes de desktop (observado num GNOME 46) **suprimem
  silenciosamente** notificações fdo enviadas *in-process* por um app que tem
  janela — o compositor associa a notificação ao app pelo PID→janela (`app_id`)
  e a descarta mesmo com o app habilitado nas configurações (`.show()` retorna
  `Ok`, nada aparece). Um subprocesso **sem janela** não é associado a nenhum app
  e é exibido. `app_name`/`icon`/título/corpo são repassados aos flags do
  `notify-send`. Ver `emit_os_notification` em `lib.rs`.

---

## [0.43.0] — 2026-07-15

### Adicionado
- **`notify()` ganhou `app_name` e `icon`** (ambos opcionais, na tabela Luau e em
  `NotificationSpec`). `app_name` sobrescreve o padrão do `notify-rust` (que usa o
  nome do executável); `icon` é um nome de ícone do tema ou caminho. Motivação
  real: alguns ambientes de desktop **filtram/descartam** notificações pela
  identidade do app — um GNOME em que o `app_name` casando com um `.desktop`
  instalado fazia a notificação ser descartada silenciosamente (o nome do binário
  virava o `app_name` por padrão). Poder setar um nome de exibição que não casa
  com um `.desktop` contorna isso. `NotificationSpec` agora deriva `Default`.

---

## [0.42.0] — 2026-07-15

### Adicionado
- **`notify(...)` (camada Luau) / `Context::notify` + `NotificationSpec` (Rust)**
  — notificações **nativas do sistema operacional**, entregues à central de
  notificações do SO (freedesktop/D-Bus no Linux/BSD, WinRT no Windows,
  `NSUserNotification` no macOS) via `notify-rust`. Diferente de `toast`, que é
  efêmero e desenhado dentro da própria janela, a notificação sobrevive ao app
  estar minimizado ou em outro workspace — para eventos que o usuário quer saber
  sem olhar para o app (ex.: um deploy terminou). Na Luau: `notify({ title, body })`
  ou `notify("mensagem")` (string vira o corpo). O motor a entrega fora da thread
  de UI (o backend é síncrono), é acumulativa como o toast e não realimenta nada
  ao componente. Novo exemplo: `cargo run --example notificacoes`.

---

## [0.41.0] — 2026-07-14

Rodada de robustez: o que faltava para a lib ser defensável fora do app que a
criou. Ver `RELATORIO_0.38_A_0.40.md` para o processo (inclusive os erros).

### Adicionado
- **`render_inputs::RenderInputs`** — as entradas de render (folhas de estilo,
  templates parseados, viewport) atrás de um portão que conta as mudanças numa
  `epoch`. O cache de avaliação guarda a época em que foi construído e se
  descarta sozinho quando ela avança.
- **CI** (GitHub Actions): build, testes, `clippy -D warnings`, `fmt --check` e
  `cargo doc -D warnings`.
- Este `CHANGELOG.md`.

### Corrigido
- **Invalidação do cache deixou de depender de memória humana.** A 0.40 usava
  oito chamadas manuais de `invalidate_eval_cache()` espalhadas pelos call-sites
  — e uma delas estava furada: o hot-reload de `.gss` escrevia direto em
  `stylesheets[idx]` e só não servia estilo velho porque um `invalidate` genérico
  vinha depois, por acaso. Agora os campos são privados noutro módulo e a época é
  incrementada pelos próprios métodos de mutação.
- **`cargo doc`**: 5 links quebrados na documentação, incluindo um
  `EngineMessage::LuaStream` que não existe (é `LuauStream`).
- **Clippy: 62 → 0**, com `-D warnings`.
- Um resize que **não** cruza breakpoint de `@media` deixou de poder invalidar o
  cache (arrastar a borda da janela custaria uma reconstrução por pixel).

### Quebras
- **`LuauComponent::from_file`/`from_source`** passam a devolver
  `Result<Self, GlacierError>` em vez de `Result<Self, String>`.
  *Migração:* o `Display` do erro traz a mesma mensagem de antes; se você fazia
  `.map_err(|s| ...)` com a `String`, use `.to_string()`.
- Todo o código foi passado por **rustfmt** (commit isolado, sem mudança de
  comportamento) — relevante só para quem mantém um fork.

---

## [0.40.1] — 2026-07-14

### Corrigido
- **Dirty-tracking não funcionava em nenhuma tela com lista.** As variáveis de um
  item de `for-each` (`{l.nome}`) só existem na camada daquele item, mas subiam
  até o conjunto de dependências do *template*. Lá em cima o motor perguntava "o
  contexto ainda tem `l.nome` = a?" e ouvia **não** para sempre, porque `l.nome`
  nunca esteve no contexto — então a tela ficava eternamente suja e o cache
  existia sem nunca acertar. Cada leitura agora registra a profundidade da camada
  que a resolveu, e ao fechar um quadro só sobem as leituras resolvidas fora dele.

## [0.40.0] — 2026-07-14

### Adicionado
- **Dirty-tracking**: o motor rastreia as chaves de contexto que cada subárvore
  lê e **não reconstrói o que não mudou**. Memoiza nas duas fronteiras que pagam:
  o uso de um componente (props bem definidas) e cada item de `for-each`.
  `reevaluate_all` nem entra na árvore se nada que a tela lê mudou.
- `eval::EvalCache`, `eval::evaluate_template`, `eval::Deps`.

Medido na árvore real de um app (600 nós):

| cenário | antes | depois |
|---|---|---|
| muda uma chave que ninguém lê | 6,3 ms | 3,5 µs |
| muda uma chave lida, lista de 45 linhas intacta | 6,3 ms | 1,6 ms |

### Quebras
- **`UiNode` ganhou o campo `node_id`** (identidade estável, é a chave do cache).
  *Migração:* quem constrói `UiNode` à mão precisa preenchê-lo.

### Notas
- Listas **reordenáveis** ficam fora do cache de propósito: o corpo do item
  carrega `drag_order` *injetado* (não lido do contexto), então o rastreamento
  não perceberia uma mudança de ordem. São listas pequenas.

## [0.39.0] — 2026-07-14

### Melhorado
- **`EvalCtx`: contexto em camadas.** Cada item de `for-each` fazia
  `context.clone()` — uma cópia do contexto inteiro (com strings grandes dentro,
  como um log vindo de SSE) por linha renderizada, a cada reavaliação. Agora as
  variáveis do item e as props de componente entram numa cadeia de camadas
  encadeada na pilha, sem copiar a base.

  *Nota honesta:* isto sozinho rendeu pouco (6,5 ms → 6,0 ms). O gargalo era
  outro — ver 0.40.0.

## [0.38.1] — 2026-07-14

### Corrigido
- **Pânico ao parsear qualquer template com caractere multi-byte logo após um
  `<`.** A varredura por `<style>` fatiava o `&str` por byte (`&tail[..5]`); uma
  régua `──` num comentário XML caía no meio de um caractere. Comparação passou a
  ser feita em bytes.

## [0.38.0] — 2026-07-14

### Adicionado
- **Erro tipado (`error::GlacierError`)** com **`Diagnostic`** posicional:
  arquivo, linha, coluna, o trecho ofensor com um `^` embaixo e uma dica
  acionável. Sem dependência nova (`Display`/`Error` à mão).
- **`GlacierDaemon`** ganhou o que faltava para um app real não precisar
  reimplementar o runtime: `.font()`, `.default_font()`, `.main_window(Settings)`,
  `.child_window()`, `.on_message()` (persistência), `.on_close(WindowGeometry)`,
  `.reload_period()`, `.toast_period()`.
- **GSS: lista de seletores por vírgula** (`.a, .b { }`).
- `GlacierUI::keep_evaluated`, `evaluated`, `context`, `current_screen`,
  `history`, `dialog`, `custom_theme`, `stylesheets`, `parsed`, `is_registered`.

### Corrigido
- **Corpo de `<style>` era lido como XML.** Uma tag citada num comentário do CSS
  (`/* o card vira <Text> */`) virava um elemento de verdade, e o erro apontava o
  `</style>` reclamando de uma tag que o autor nunca abriu. Agora o corpo é
  blindado com CDATA e nunca passa pelo parser de XML.
- **`strip_script` comia as linhas do bloco**, deslocando para cima *todo* erro
  abaixo dele. Agora preserva a contagem de linhas.
- **`.a, .b { }` no GSS virava UMA classe de nome literal `"a, .b"`** — sem erro,
  sem aviso, sem estilo, e nenhum nó jamais a casava.
- Comentário de bloco multi-linha no `.gss` deixou de deslocar as linhas
  seguintes.
- Propriedade GSS desconhecida agora avisa com `arquivo:linha` e sugere a certa
  (`colr` → *"você quis dizer 'color'?"*).
- **`window:drag` era um no-op silencioso no Wayland.** O motor resolvia as ações
  `window:*` via `window::latest()`, cujo round-trip perde o pointer-grab serial.
  Passaram a ser tratadas no runner, contra o `Id` da janela em roteamento.

### Melhorado
- **Avaliação escopada.** `reevaluate_all` avaliava **todo template registrado**,
  cada um como raiz — e como avaliar inlina recursivamente os componentes, um app
  com 15 componentes reconstruía a árvore inteira 16 vezes por tecla digitada, 15
  delas para árvores que ninguém renderiza. Agora só a tela ativa (e os
  `keep_evaluated`) é construída.

### Quebras
- **Os campos de `GlacierUI` são privados.** *Migração:* use os getters
  (`context()`, `evaluated(name)`, `current_screen()`, …).
- **`render(name)`** de um template fora de uso devolve `GlacierError::NotEvaluated`
  em vez de uma árvore obsoleta. *Migração:* `set_initial_screen(name)` ou
  `keep_evaluated(name)`.
- **`NodeType::Style` ganhou o campo `line`** (posiciona erros de `.gss` inline).
- Toda a API pública passou de `Result<_, String>` para `Result<_, GlacierError>`.
  *Migração:* o `Display` é compatível; `format!("{e}")` segue funcionando.

---

[0.41.0]: https://github.com/antoniofernandodj/glacier-ui/releases/tag/v0.41.0
[0.40.1]: https://github.com/antoniofernandodj/glacier-ui/releases/tag/v0.40.1
[0.40.0]: https://github.com/antoniofernandodj/glacier-ui/releases/tag/v0.40.0
[0.39.0]: https://github.com/antoniofernandodj/glacier-ui/releases/tag/v0.39.0
[0.38.1]: https://github.com/antoniofernandodj/glacier-ui/releases/tag/v0.38.1
[0.38.0]: https://github.com/antoniofernandodj/glacier-ui/releases/tag/v0.38.0
