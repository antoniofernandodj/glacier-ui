# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

O crate está em **0.x**: pela convenção do Cargo, um bump de *minor* (`0.40` →
`0.41`) **pode quebrar API**, e é o que este projeto usa para mudanças
incompatíveis. Toda quebra vem listada em **Quebras** com o que fazer para migrar.

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
