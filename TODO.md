# TODO — tipos, estado, escopo, componentes, handlers, sinais, formatação, cor e botão ocupado

**Estado:** implementado no motor em 2026-11-27 (Frentes 1, 2, 3, 4, 6, 7, 8, 10,
11 — ver **Status da entrega** no fim). **Os testes foram escritos e não foram
executados**: rodar `make test` (ou `cargo test --lib` + `cargo test --test
linguagem_frentes`) antes de publicar. O que falta está listado no fim com o
motivo.
Origem: uma análise da linguagem `.gv` feita a partir do `rustploy-gui`, o maior app sobre
o glacier (68 `.gv` de view, ~9,8k linhas, ~500 scripts Luau).

As frentes abaixo são **uma entrega só**, porque se apoiam umas nas outras: o botão
ocupado precisa de booleano de verdade, o estado local precisa de tipo, e o componente
fechado precisa de estado local. A entrega só termina com a **migração do rustploy**
(Frente 9): o motor ganhar o recurso e o app continuar com a solução caseira é
justamente o problema que motivou este plano.

## Por que isto existe: o que o rustploy mostra

| Sintoma no rustploy-gui | Número | Causa no motor |
|---|---|---|
| Chaves globais distintas lidas pelas views (`@service_form_healthcheck_interval`…) | 656 | estado global e plano; prefixo faz papel de namespace |
| Componentes que declaram `props` | 13 de 68 | componente lê o global em vez de receber o que precisa |
| Comparações `== "true"` / `!= "true"` | 153 | o contexto é `String → String` |
| `json.encode` nos handlers para mandar lista à view | 29 | idem |
| Trio `form_control` + `:value` + `on_change = field:x` por campo | 91 / 161 | não há vínculo único de campo |
| Chaves `busy_*` + `disabled = @busy_*` à mão | 225 | o `busy_buttons` do motor não cobre o que o app precisa (Frente 4) |

## Frente 1 — Contexto tipado de ponta a ponta

**Hoje:** `ContextMap = HashMap<String, String>` (`src/lib.rs`). O `src/ctx_types.rs` guarda
**ao lado** um mapa `chave → (tipo, texto)` para o Luau receber de volta `true`/`3`/tabela,
mas o avaliador de template e os widgets continuam lendo texto. Por isso:

- `@x == "true"` é a forma natural de testar um booleano;
- argumento de ação vindo de uma chave chega como texto (desvio 7 do
  `PLANO_LINGUAGEM_GV_GSS.md`);
- a regra de verdade é textual: `"no"`, `"não"`, `"off"`, `"0"` são falsos. Um valor
  **digitado pelo usuário** ("não") muda o que um `if` faz.

**Fazer:**

- [ ] O valor do contexto passa a ser um `Value` (`Null | Bool | Number | String | List | Object`), *(parcial: o avaliador já lê o tipo pela tag; o armazenamento ainda é texto + `ctx_types` ao lado)*
      com o texto só como forma de exibição. O `ctx_types` deixa de ser um mapa paralelo
      e vira o próprio armazenamento.
- [x] O avaliador de expressão opera sobre `Value`: `@ativo` testa o bool, `@n > 3` o
      número, `@lista` é lista sem `json.encode`/`decode`.
- [x] Argumento de ação vindo de chave chega com o tipo da chave.
- [x] Verdade — **decidido pelo dono (2026-11-25):** depois que a tela estiver migrada para os
      tipos, **todo texto não vazio é verdadeiro**, inclusive `"não"`, `"no"`, `"off"`,
      `"false"` e `"0"` digitados ou gravados como texto. Falso passa a ser só o bool `false`,
      `null`/ausente, o número `0`, `""`, `[]` e `{}`.
      **A troca acontece só junto com a adaptação da tela, nunca antes:**
      - tela não migrada continua com a regra textual de hoje (`no`/`não`/`off`/`"false"`/`"0"`
        falsos), sem mudar nada no comportamento;
      - a regra nova é ligada pela própria migração (o `fmt --migrate` desta frente liga a
        marca no app/tela **no mesmo passo** em que reescreve os testes de verdade dela), e não
        por mudança de padrão do motor;
      - antes de ligar, o `glacier check` lista todo `if`/`hidden`/`disabled`/`class:x` cuja
        chave pode receber texto que muda de valor com a regra nova, e a migração só fecha
        quando essa lista está vazia ou revista;
      - o padrão do motor só vira a regra nova numa `feat!` depois que o rustploy (Frente 9)
        estiver migrado e verde.
- [x] Luau: `ctx.x = true` grava bool, `ctx.lista = { … }` grava lista; nada de `json.encode`
      para falar com o template.
- [x] `glacier check` avisa `== "true"`/`== "false"` e sugere a forma tipada; `fmt --migrate`
      reescreve `@x == "true"` → `@x` e `@x != "true"` → `!@x` **só** quando a chave é bool
      conhecida (senão só avisa).

## Frente 2 — Estado local e escopo por componente

**Hoje:** todo componente lê e escreve o mesmo contexto global. Não há estado próprio de
instância, colisão de nome não é detectada, e dois usos do mesmo componente com estados
diferentes exigem trocar o prefixo à mão (o mesmo vale para os nomes importados e o
`style` inline, que também são globais).

**Fazer:**

- [x] `state { … }` no `component`: chaves locais à instância, com tipo e `default`
      (mesma sintaxe do `props`). Dentro do componente, `@x` resolve primeiro no estado
      local, depois em `props`, e só então no global.
- [x] Acesso ao global explícito e visível: `@app.x` (ou outro prefixo a decidir). O
      `glacier check` avisa componente que lê global sem declarar.
- [ ] O script do componente (`script(src = …)`) recebe o estado da **instância**
      (`self.x` / `ctx.x` local) — duas instâncias não se veem.
- [x] Vínculo único em formulário: substitui `form_control` + `:value` +
      `on_change = field:x`, resolvido contra o estado do `form`/componente. A escrita é
      `::value = @x` (duas vias, como o `v-model` do Vue / `[( )]` do Angular), definida na
      Frente 10 (`docs/PLANO_SINAIS_UNICOS.md`), e não um atributo `bind = campo` à parte.
- [ ] Colisão: duas fontes escrevendo a mesma chave global com tipos diferentes é aviso;
      nome importado registrado duas vezes com arquivos diferentes é erro (hoje vence quem
      importou primeiro, em silêncio).

## Frente 3 — Componentes que fecham a abstração

**Hoje:** existem `props` tipados (0.169) e `slot` (builtins e componentes), mas:

- quase ninguém declara `props` (o idioma vigente é "componente sem argumentos lê o
  global");
- `slot` **não atravessa** um componente aninhado: a partição acontece uma vez, sobre os
  filhos crus, e o resultado é página em branco **sem erro** (`docs/PRIMITIVAS.md`,
  "Um `slot` não atravessa a fronteira de um componente aninhado");
- `spread = @c` é a saída para passar um objeto inteiro, e esconde quais campos o
  componente usa.

**Fazer:**

- [ ] `slot` repassável *(não feito: continua sem atravessar o componente aninhado)*: um componente pode encaminhar o que recebeu num slot para o
      slot de um componente filho (`slot(name = x, forward = true)` ou equivalente). Enquanto não
      existir, o caso de hoje vira **erro** de montagem, não página em branco.
- [ ] Slots nomeados documentados para componente **do app** (não só builtin), com
      `fallback` e verificação: slot preenchido que o componente não declara é erro.
- [ ] `spread` valida contra `props`: campo do objeto que não é prop é ignorado com aviso
      uma vez; prop obrigatória faltando no objeto é erro.
- [ ] Eventos de saída tipados (`prop(type = action)` já existe): documentar o padrão
      "componente emite, quem usa decide" como a forma de devolver dado ao pai.

## Frente 4 — Botão ocupado no motor (substituir o `busy.luau` do rustploy)

**Hoje:** o `app(busy_buttons = true)` (`src/eval.rs` `ocupado()`, `src/luau/mod.rs`
`marca_ocupado`/`solta_ocupados`) desabilita um `button` com `on_click` enquanto a corrotina
do handler está suspensa. O rustploy **não usa** e mantém o seu próprio
(`rustploy-gui/views/scripts/busy.luau` + `busy_actions.luau` + `tools/busy_actions.py --check` no agregador),
porque o do motor faz menos:

| | `busy.luau` (rustploy) | `busy_buttons` (motor) |
|---|---|---|
| Botão de submit (`button(type = submit)` → `on_submit` do `form`) | trava | **não trava** (só olha `on_click`) |
| Segundo disparo da mesma ação enquanto a primeira roda | **ignorado** | despachado de novo; só o desenho fica desabilitado |
| Quando libera | no **primeiro `toast`** da própria ação (a resposta chegou) | quando a corrotina **termina** (inclusive as releituras depois da resposta) |
| Erro no handler | libera e relança | libera quando a corrotina morre |

**Fazer:**

- [x] Cobrir `on_submit`: o botão `type = submit` de um `form` fica ocupado pelo handler do
      `on_submit` (e o `form` inteiro, opcionalmente, via `busy` no `form`).
- [x] Guarda de reentrada **no dispatch**: com `busy_buttons` ligado, uma ação cujo handler
      está ocupado é descartada antes de rodar (não só desenhada como desabilitada). Opt-out
      por nó: `busy = false`.
- [x] Liberação antecipada: `busy.release()` no Luau solta o handler corrente; e uma opção de
      app `busy_release = toast` que reproduz o comportamento do rustploy (o primeiro `toast`
      da corrotina libera).
- [x] Identidade pela corrotina, como no `busy.luau`: toast de outra ação não libera esta;
      ação chamada de dentro de outra segura as duas chaves.
- [x] `__busy.<handler>` vira bool tipado (Frente 1) e fica legível no template para
      spinners (`spinner(if = @__busy.salvar)`).
- [x] Testes no motor espelhando o `rustploy-gui/tests/lib/busy.test.luau` (duplo clique, toast libera,
      erro libera, ação aninhada).

## Frente 5 — Recursos que faltam

- [ ] **`keyed_column` de verdade.** Hoje `key =` no `each` é só apelido de `reorder_key`
      (desvio 5 do plano anterior); inserir ou remover um item no meio recria os widgets
      de baixo e perde estado por item (foco, texto digitado, scroll). Trocar a coluna do
      `each` por `keyed_column` do iced quando houver `key`.
- [ ] **`:hover` (e `:active`/`:focus`) num ancestral.** Hoje é erro de parse porque o estado
      vem do widget do iced no desenho. Fazer um embrulho que rastreia ponteiro por nó
      **só** quando o `.gss` usa estado em ancestral (sem custo no caso comum). Caso de
      uso: cartão clicável que recolore os filhos no hover.
- [ ] **`letter_spacing`.** Hoje é erro porque o iced 0.14 não desenha. Investigar o
      caminho (cosmic-text expõe espaçamento? desenho por glifo?) e decidir: implementar
      ou manter o erro com a explicação. Registrar a decisão aqui.

## Frente 6 — Ligação handler ↔ template verificada e tipada

**Hoje:** `on_click = salvar` (a forma sem argumento, 189 usos no rustploy) e
`on_click = exportar(@item.key, true)` (a forma de chamada da 0.169) resolvem o handler
**pelo nome, na hora do clique**, como uma função global do script. A 0.169 mudou a
**forma de passar argumentos** (`handler:arg` → `handler(args)`), não a ligação: nada
confere que `salvar` existe, quantos argumentos recebe nem de que tipo. `on_click = salvr`
carrega, desenha e só falha (ou não faz nada) quando alguém clica.

**Decisão do dono (2026-11-25): ação é sempre chamada, com parênteses.** O nome solto
(`on_click = salvar`) parece um valor, não uma ação, e convive com a forma de chamada só
por herança. A forma única passa a ser `on_click = salvar()`, com ou sem argumentos.

**Fazer:**

- [x] **Forma única `handler()`** em todo atributo de ação (`on_click`, `on_submit`,
      `on_change`, `on_press`, `on_double_click`, `on_*`):
  - `on_click = salvar()` é a forma canônica; confirmar por teste que o dispatch chama o
    handler com **zero** argumentos (não com um `""`) e que o botão ocupado (Frente 4)
    reconhece o nome nessa forma;
  - `on_click = salvar` (nome solto) fica **depreciado**: `glacier check` avisa com
    `arquivo:linha`, `glacier fmt --migrate` reescreve para `salvar()`, e numa `feat!`
    depois da migração do rustploy (Frente 9) vira erro de parse;
  - os embutidos seguem a mesma regra: `window:minimize` → `window.minimize()`,
    `window:resize:nw` → `window.resize("nw")`, `field:nome` → `field("nome")`,
    `tray:open` → `tray.open()` (a forma de chamada dos embutidos já existe desde a 0.169);
  - `prop(type = action)` recebe a ação na mesma forma (`on_salvar = salvar()`, ou
    `salvar(@id)` com argumento já preso), e o componente a dispara como `@on_salvar`;
  - a documentação (`docs/LINGUAGEM.md`, `CLAUDE.md`, o preset `AGENTS.md` do glacier-cli,
    exemplos e `.md` com blocos ```gv```) passa a mostrar só `handler()`.
- [x] O motor extrai dos scripts de cada tela/componente o conjunto de handlers
      (funções globais declaradas pelo script e pelos `require` dele) e a assinatura do
      Luau `--!strict` (`function exportar(key: string, on: boolean)`).
- [x] `glacier check` (e o carregamento em modo de desenvolvimento) acusa, com
      `arquivo:linha`:
  - handler inexistente (`on_click = salvr()`), com sugestão do nome mais próximo;
  - número de argumentos diferente da assinatura;
  - argumento de tipo errado: literal contra o tipo anotado, e chave tipada (Frente 1)
    contra o tipo anotado;
  - handler definido e nunca referenciado por nenhum `.gv` (aviso, desligável por
    handler: os chamados só do Luau).
- [ ] Os embutidos (`window:*`, `field:*`, `tray:*`, `notifications:*`) entram com
      assinatura própria no mesmo verificador. *(a lista `EMBUTIDAS` já existe no verificador; a
      assinatura de cada um não)*
- [ ] Ação passada como `prop(type = action)` é conferida no ponto em que o componente
      a dispara, contra o handler que quem usou o componente passou.
- [ ] O dono da ação (`Comp::handler`, hoje resolvido em silêncio para a tela atual quando
      o prefixo não é um componente com script) também é verificado: prefixo que não
      resolve é erro.
- [ ] Substitui o `tools/busy_actions.py --check` do rustploy na parte de "o handler do
      template existe no script".

## Frente 7 — `glacier fmt` que reindenta sem perder comentários

**Hoje:** o `fmt` faz só trocas pontuais (migração) e normaliza espaço no fim da linha
e a quebra final (desvio 8 do `PLANO_LINGUAGEM_GV_GSS.md`): o parser descarta os
comentários, e reimprimir a partir da árvore os apagaria.

**Fazer:**

- [ ] Árvore concreta (CST) ao lado da AST: o parser do `.gv` e do `.gss` guarda
      comentários (`//`, `/* */`) e linhas em branco presos ao nó vizinho (antes, no fim da
      linha, depois do último filho).
- [ ] Impressor a partir da CST com a convenção do `CLAUDE.md`: indentação de 2, um
      atributo por linha quando não cabe na largura (100), vírgula final nos atributos em
      várias linhas, uma linha em branco entre irmãos de bloco preservada mas nunca duas.
- [ ] Strings `"""…"""` e `l"""…"""`, blocos `style """…"""` e o conteúdo de `script`
      inline são copiados byte a byte.
- [ ] Idempotente (`fmt` duas vezes = uma) e sem perda: teste de corpus que reimprime
      todo `.gv`/`.gss` do repositório e dos exemplos e confere que o conjunto de
      comentários e a AST não mudaram.
- [ ] `glacier fmt --check` falha no CI quando um arquivo não está formatado.

## Frente 8 — Cor do CSS como padrão (`gss_color = css`)

**Hoje:** a semântica do CSS (`background` é o fundo, `color` é o conteúdo, em todo
widget) é opt-in por `app(gss_color = css)`. O padrão ainda é `legacy`, em que o `color`
de `button`, `select`, `comboedit` e `menu` é o **fundo** e o rótulo é `text_color`
(desvio 3 do `PLANO_LINGUAGEM_GV_GSS.md`). Quem vem do CSS erra a cor de todo botão.

**Fazer:**

- [ ] Passar os 82 `.gss` de exemplo e os estilos do repositório por
      `glacier fmt --migrate-color` e conferir em janela (screenshot) os que têm botão,
      select, comboedit e menu.
- [ ] Migrar o rustploy-gui (Frente 9) com o mesmo comando.
- [ ] Trocar o padrão para `css` numa `feat!`; `legacy` continua aceito por
      `app(gss_color = legacy)` com aviso de depreciação, e some numa versão seguinte.
- [ ] `glacier check` avisa `text_color` num app em modo `css` (sobra da migração).

## Frente 10 — Cada sinal do `.gv` com um significado só

Plano completo em **[`docs/PLANO_SINAIS_UNICOS.md`](docs/PLANO_SINAIS_UNICOS.md)**.

**Hoje:** a mesma escrita significa coisas diferentes conforme o lugar (`:value = x` ×
`:value = @x`; palavra sem aspas ser texto, classe, handler ou enum; `:` com três papéis) e
a mesma coisa tem várias escritas (`@m`, `"@m"`, `"{m}"`, `"{@m}"`). O manual tem uma tabela
de sigilos, mas quem lê precisa consultá-la.

**Fazer (resumo; os itens estão no plano):**

- [ ] **O `:` fica, com o sentido do Vue e do Angular (decisão do dono):** `:attr = expr` é
      o atributo recebendo uma expressão numa via (Vue `:attr`, Angular `[attr]`); sem `:`
      é literal. `::attr = @x` liga nos dois sentidos (`v-model` / `[( )]`), e
      `:class.ativo = expr` é a classe condicional (Angular `[class.ativo]`).
- [ ] Cada sinal com um significado só: `"…"` é sempre texto, `{expr}` só é buraco dentro
      de texto, `@x` é sempre o valor de uma chave, `nome` é sempre nome, `nome(…)` é sempre
      chamada, `[…]` é sempre lista.
- [ ] **O `:` decide** se um atributo é expressão; some a heurística "começa com
      `@`/`!`/`(`… é expressão". Diretivas (`if`, `each`), eventos (`on_*`) e o conteúdo
      posicional de `text`/`button` são expressão sem `:`, como `v-if`/`@click`/`{{ }}`.
- [ ] Lista de classes e pares numéricos como lista (`class = [btn, btn--primary]`,
      `size = [1280, 820]`); `@x ?? "padrão"` no lugar de `{x|padrão}`.
- [x] `glacier fmt --migrate` faz todas as trocas; formas antigas com aviso até a `feat!`.

## Frente 11 — Escopo local por padrão e aviso de sombra

**Hoje:**

- **Nome importado é global.** `link(rel = import, href = …, as = Nome)` registra `Nome` para
  o app inteiro; quem importou primeiro vence, e os arquivos extraídos usam `StatCard`,
  `LoadingRow`… sem reimportar (o `AGENTS.md` do rustploy documenta isso como armadilha).
  A Frente 2 só transforma a colisão em erro; o nome continua global.
- **`style` inline é global.** Só `style(scoped = true)` restringe ao arquivo; o padrão vaza
  regra para todas as telas.
- **Sombra calada.** Com a Frente 2, `@x` dentro de um componente procura no estado local,
  depois em `props`, depois no global. Uma chave de `state` ou `props` com o mesmo nome de
  uma global esconde a global sem aviso.

**Decisão do dono (2026-11-25): escopo local é o padrão.**

**Fazer:**

- [ ] **Import local ao arquivo.** O nome dado por `as = Nome` vale só no arquivo que o
      importou (e no que esse arquivo declara dentro dele). Usar um componente exige
      importá-lo no arquivo que usa; nome não importado é erro com `arquivo:linha` e a
      sugestão do `link(rel = import, …)` a acrescentar.
- [ ] Componente compartilhado por muitas telas: `link(rel = import, …, global = true)` no
      `app.gv` como **única** forma de tornar um nome global, explícita e num lugar só.
      Dois globais com o mesmo nome continuam erro.
- [x] **`style` local ao arquivo por padrão** (o `scoped = true` de hoje vira o
      comportamento padrão). Regra para o app inteiro vai no `.gss` do `app.gv`
      (`link(rel = stylesheet)`) ou em `style(global = true)`. `scoped = true` fica aceito
      com aviso de redundância.
- [ ] **Aviso de sombra** no `glacier check` e no carregamento em desenvolvimento: chave de
      `state` ou prop com o mesmo nome de uma chave global que o app escreve (no Luau ou em
      outro `.gv`) → aviso com os dois lugares. Silenciável por chave
      (`prop(name = x, shadow = true)`) quando a sombra é intencional.
- [ ] O mesmo aviso para `each @lista as v` quando `v` esconde uma chave global, um prop ou
      a variável de um `each` de fora.
- [ ] `glacier fmt --migrate`: para cada arquivo, acrescenta os `link(rel = import)` dos
      componentes que ele usa e hoje vêm do registro global; `style` sem `scoped` que é
      usado fora do próprio arquivo vira `style(global = true)` (o `check` mede quais
      classes de um `style` são usadas em outros arquivos para decidir).

## Frente 9 — Migração do rustploy-gui (parte da mesma entrega)

Quando as frentes acima forem publicadas, **o rustploy migra na mesma tacada**, como foi
feito com a 0.169 (`fmt --migrate` numa cópia, depois no repo). A entrega não está pronta
enquanto o rustploy usar a solução caseira.

- [ ] `glacier fmt --migrate` cobre o que é mecânico: `on_x = nome` → `on_x = nome()` e
      embutidos `window:x` → `window.x()` (Frente 6), as trocas de sinal da Frente 10
      (`attr = @x` → `:attr = @x`, `:value = x` → `::value = @x`, `"@x"` → `@x`,
      `class = "a b"` → `[a, b]`…), `== "true"` em chave bool,
      trio de campo → `::value = @x`, `disabled = @busy_<handler>` no botão do próprio handler → nada.
- [ ] No rustploy-gui:
  - [ ] ligar `app(busy_buttons = true, busy_release = toast)` no `views/app.gv`;
  - [ ] apagar `views/scripts/busy.luau`, `views/scripts/busy_actions.luau`,
        `tools/busy_actions.py` (no agregador `rustploy`), os testes `tests/lib/busy*.test.luau` e os `require("busy").install(…)` dos scripts de tela;
  - [ ] apagar os `disabled = @busy_*` (as linhas que sobrarem são as que combinam
        condições e ficam com `busy = expr`);
  - [ ] portar os casos de `tests/lib/busy.test.luau` para testes do motor (Frente 4) e conferir os de `tests/windows/` que dependem de `busy_*`;
  - [ ] trocar `json.encode` de lista para o ctx por atribuição direta;
  - [ ] `glacier fmt --migrate-color` e `app(gss_color = css)` (Frente 8), conferido em janela;
  - [ ] `glacier fmt` (Frente 7) em todo `views/`, num commit só de formatação;
  - [ ] imports e `style` locais (Frente 11): o `fmt --migrate` acrescenta os imports por
        arquivo; os componentes usados em muitas telas (`StatCard`, `LoadingRow`,
        `StateCell`…) podem ir como `global = true` no `app.gv`, decisão caso a caso;
  - [ ] resolver todo aviso de sombra (renomear ou marcar `shadow = true` com motivo);
  - [ ] converter os componentes extraídos (`views/service/*`, `views/home/*`, `views/shell/*`)
        para `props`/`state` onde hoje leem global com prefixo.
- [ ] Critério de pronto (medido com `grep` no rustploy-gui):
  - `busy_` em `views/` = 0;
  - `== "true"` / `!= "true"` em `.gv` = 0;
  - `json.encode` em `views/scripts/handlers/` só onde o destino é a rede, não o ctx;
  - `glacier check --strict` e `glacier test -vvv` verdes, **incluindo** a verificação de
    handlers e assinaturas (Frente 6) sem nenhum erro;
  - `glacier fmt --check` limpo em `views/`;
  - nenhuma ação na forma de nome solto ou `prefixo:arg` em `.gv` (`glacier check` sem
    aviso de depreciação de ação);
  - `text_color` em `views/styles/` = 0;
  - `glacier check` sem erro de nome não importado e sem aviso de sombra;
  - os zeros de sinal do critério de pronto de `docs/PLANO_SINAIS_UNICOS.md` (zero `:attr = nome`
    na forma antiga, zero `class:`, `field:`, `"@x"` como valor inteiro, `"{x}"` sem `@`, `class = "a b"`);
  - a regra de verdade nova (Frente 1) ligada no `app.gv`, e não antes dos itens acima.
- [ ] Atualizar o `AGENTS.md` do agregador rustploy (seção "Armadilhas de template") e o
      preset `crates/glacier-cli/templates/_comum/AGENTS.md`.

## Ordem sugerida

1 (tipos) → 10 (sinais) → 2 (estado local) → 3 (componentes) → 4 (ocupado) → 6 (handlers)
→ 11 (escopo) → 5 (recursos) → 9 (migração).
A Frente 4 depende só da 1 e pode ir antes da 2/3 se for preciso tirar o `busy.luau` cedo;
a Frente 6 depende da 1 (tipos das chaves) para conferir argumentos vindos de chave, mas a
parte "handler existe / aridade / literal" pode sair antes. As Frentes 5, 7 e 8 são
independentes; a 7 deve vir antes da migração, para o rustploy ser reformatado uma vez só.
A 10 vem logo depois da 1 porque define a escrita (`:`/`::`, `[…]`, aspas) que as Frentes
2, 3 e 6 usam. A 11 depende da 2 (o aviso de sombra precisa do `state`/`props` por
instância); o rustploy passa pelo `fmt --migrate` uma vez, com todas as trocas juntas.


---

## Status da entrega — 2026-11-27 (glacier-ui 0.170.0, não publicada)

Implementado no motor, com o `glacier check`/`fmt` acompanhando cada frente:

| Frente | O que está no código | O que ainda falta |
|---|---|---|
| 1 — tipos | `expr::avalia_tipado` + `EvalCtx::valor` (a tag do `ctx_types` chega ao avaliador: `if @bool`, `@n > 3`, `@lista is empty`, `json(@lista)` com o tipo); `app(truthy = typed)` (`expr::set_verdade_tipada`); `check` lista `== "true"` e os testes cuja chave recebe texto; `fmt --migrate` troca `@x == "true"` → `@x` **só** em bool conhecido e liga a marca no `app` | o `Value` próprio como armazenamento (a tag continua ao lado do texto, agora lida pelo avaliador); a `feat!` que inverte o padrão |
| 2 — estado local | `state { … }` no `component` (`gv.rs` → `prop(..., estado)`), default semeado no contexto **com tipo** no registro (`GlacierUI::semeia_estado`), `@app.x` (`EvalCtx::lookup`), vínculo `::attr = @x` no lugar do trio | **separação por instância**: dois componentes do mesmo nome compartilham a chave. O passo é levar o id da instância até o despacho (`LuauComponent` + `route_to_owner`) e namespacear a chave |
| 3 — componentes | `prop(type = action)` documentado e conferido; `on_abrir = "app:..."` como forma de emitir ao pai | `slot` repassável (`forward`) — hoje o caso continua sem atravessar; o erro de montagem que substitui a página em branco; `spread` conferindo campo a campo |
| 4 — ocupado | `button(type = submit)` ocupado pelo `on_submit`; guarda de reentrada no despacho (`run_inner`), com opt-out `busy = false`; `busy.release()` (prelúdio) e `app(busy_release = toast)`, por corrotina; `__busy.<handler>` bool tipado; testes do rustploy portados (`src/luau/mod.rs`) | nada estrutural; falta **rodar** os testes e conferir em janela |
| 5 — recursos | decisão registrada: `letter_spacing` continua erro (o iced 0.14 não desenha — ver abaixo) | `keyed_column` real (hoje `key =` é `reorder_key`), `:hover` em ancestral |
| 6 — handlers | `src/handlers.rs`: assinas `--!strict` extraídas do `.luau` (nome, aridade, tipos, variádica, `local function` fora), ações extraídas do `.gv`, `conferir` (existe / aridade / tipo) e `sugerir`; aviso de forma nua no `.gv`; `fmt --migrate` reescreve `nome` → `nome()` e os embutidos | a verificação no **ponto de disparo** de uma ação passada por prop; o dono `Comp::handler` como erro de carregamento; o aviso de handler nunca chamado só roda em `check`, não no load |
| 7 — fmt | `reindentar_gv` (tokenizador: strings `"""`/`l"""` e comentários byte a byte, recuo de 2, idempotente, não muda o número de linhas) ligado ao `fmt` e ao `fmt --check` | a CST ao lado da AST (o caminho para dividir um atributo por linha quando a linha passa de 100) — hoje o `fmt` reindentou mas não quebra linha |
| 8 — cor | `check` avisa `text_color` num app `gss_color = css` e deprecia `app(gss_color = legacy)` | migrar os 82 `.gss` de exemplo e inverter o padrão na `feat!` |
| 10 — sinais | `:attr = expr`, `::attr = @x` (com erro quando o atributo não é campo), `:class.x = expr`, `class = [a, b]`, `size = [1280, 820]`, `@x ?? "padrão"` (`Expr::Senao`); avisos das formas antigas + `fmt --migrate` | a heurística "começa com `@`/`!`/`(` é só avisada; remover na `feat!` |
| 11 — escopo | `check`: componente usado sem import no arquivo, sombra de `state`/prop, `link(..., global = true)` como única forma global | o registro ainda é plano em tempo de execução (o `check` é que fecha a porta); `style` local por padrão (o `check` mede, o padrão não virou) |
| 9 — rustploy | — | **não executável aqui**: precisa da versão publicada no crates.io e do repositório do app. É a próxima entrega depois de `make test` + publicação. |

**Decisão registrada (Frente 5, `letter_spacing`):** o iced 0.14 não expõe
espaçamento de traço no `Text` e o `cosmic-text` só o aceita na composição do
layout, não como atributo do widget; sem um caminho curto, **mantém-se o erro**
com a explicação, e o item volta se o iced subir de versão.

**Executar os testes (nada foi rodado nesta entrega):**

```sh
make test                      # núcleo
cargo test --lib               # expr, gv, ferramentas, handlers, luau
cargo test --test linguagem_frentes
cargo run --example linguagem_frentes   # conferir em janela (busy, submit, typed)
```

Depois de verdes: `cargo test --all-features` (a suíte completa, inclusive
`exemplos_gv`, que passa os `.gv` do repositório pelo parser).

### `glacier check --tipos` → o `ctx` tipado (2026-11-27)

`views/scripts/ctx.d.luau` gerado com o tipo de cada chave que a árvore conhece
(`boolean` do que um `.luau` gravou como bool, `string` de um `prop(type = string)`,
`table` de uma lista, `any` do que só o markup lê), com a reserva
`[string]: any` no fim. É o que faz o `ctx` injetado pelo motor ser visível no
editor — hoje ele é `declare ctx: { [string]: any }` no `glacier.d.luau` do preset,
e nada nele confere nome nem tipo de chave.

**Vai quebrar, e é preciso adaptar o `../rustploy` quando isto for usado lá:**

- [ ] **Remover o `declare ctx` do `crates/glacier-cli/templates/_comum/views/scripts/glacier.d.luau`
      do app** (o `check --tipos` avisa quando os dois existem): dois `declare ctx`
      na mesma árvore de sources é erro do luau-lsp.
- [ ] **Rodar `glacier check --tipos` uma vez e revisar o arquivo**: os ~500 scripts
      do rustploy gravam chaves que o scanner vê com um tipo e escrevem com outro
      (`ctx.x = "0"` numa chave que nasceu `ctx.x = 0` vira `number` contra
      `string`). Cada divergência é um erro NO EDITOR, não em tempo de execução —
      nada muda no app, mas a equipe passa a ver vermelho.
- [ ] **Fechar o contrato quando o vermelho zerar**: tirar a linha
      `[string]: any,` do `ctx.d.luau`. A partir daí, escrever numa chave que
      nenhum `.gv`/`.luau` declara é erro de tipo — o que é justamente o fim das
      656 chaves globais soltas, e é a quebra real desta linha de trabalho.
- [ ] **Opcional, e é o passo seguinte da Frente 2**:** transformar o `ctx` em
      parâmetro do handler (`function init(ctx)`), porque um valor passado por
      parâmetro é o carregador natural do **id da instância** — o que falta para
      "duas instâncias não se veem". Quebra todos os `.luau` de app, então precisa
      de `app(handler_ctx = param)` + `fmt --migrate` antes de tocar no rustploy.
- [ ] Rodar `glacier check --tipos` no CI (o arquivo gerado deve dar diff zero).
