# TODO — estado, tipos, componentes e botão ocupado

**Estado:** plano, **nada implementado** (escrito em 2026-11-25, sobre o glacier-ui 0.169.2).
Origem: uma análise da linguagem `.gv` feita a partir do `rustploy-gui`, o maior app sobre
o glacier (68 `.gv` de view, ~9,8k linhas, ~500 scripts Luau).

As frentes abaixo são **uma entrega só**, porque se apoiam umas nas outras: o botão
ocupado precisa de booleano de verdade, o estado local precisa de tipo, e o componente
fechado precisa de estado local. A entrega só termina com a **migração do rustploy**
(Frente 6): o motor ganhar o recurso e o app continuar com a solução caseira é
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

- [ ] O valor do contexto passa a ser um `Value` (`Null | Bool | Number | String | List | Object`),
      com o texto só como forma de exibição. O `ctx_types` deixa de ser um mapa paralelo
      e vira o próprio armazenamento.
- [ ] O avaliador de expressão opera sobre `Value`: `@ativo` testa o bool, `@n > 3` o
      número, `@lista` é lista sem `json.encode`/`decode`.
- [ ] Argumento de ação vindo de chave chega com o tipo da chave.
- [ ] Verdade — **decidido pelo dono (2026-11-25):** depois que a tela estiver migrada para os
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
      - o padrão do motor só vira a regra nova numa `feat!` depois que o rustploy (Frente 6)
        estiver migrado e verde.
- [ ] Luau: `ctx.x = true` grava bool, `ctx.lista = { … }` grava lista; nada de `json.encode`
      para falar com o template.
- [ ] `glacier check` avisa `== "true"`/`== "false"` e sugere a forma tipada; `fmt --migrate`
      reescreve `@x == "true"` → `@x` e `@x != "true"` → `!@x` **só** quando a chave é bool
      conhecida (senão só avisa).

## Frente 2 — Estado local e escopo por componente

**Hoje:** todo componente lê e escreve o mesmo contexto global. Não há estado próprio de
instância, colisão de nome não é detectada, e dois usos do mesmo componente com estados
diferentes exigem trocar o prefixo à mão (o mesmo vale para os nomes importados e o
`style` inline, que também são globais).

**Fazer:**

- [ ] `state { … }` no `component`: chaves locais à instância, com tipo e `default`
      (mesma sintaxe do `props`). Dentro do componente, `@x` resolve primeiro no estado
      local, depois em `props`, e só então no global.
- [ ] Acesso ao global explícito e visível: `@app.x` (ou outro prefixo a decidir). O
      `glacier check` avisa componente que lê global sem declarar.
- [ ] O script do componente (`script(src = …)`) recebe o estado da **instância**
      (`self.x` / `ctx.x` local) — duas instâncias não se veem.
- [ ] `bind = campo` único em formulário: substitui `form_control` + `:value` +
      `on_change = field:x`, resolvido contra o estado do `form`/componente.
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

- [ ] `slot` repassável: um componente pode encaminhar o que recebeu num slot para o
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

- [ ] Cobrir `on_submit`: o botão `type = submit` de um `form` fica ocupado pelo handler do
      `on_submit` (e o `form` inteiro, opcionalmente, via `busy` no `form`).
- [ ] Guarda de reentrada **no dispatch**: com `busy_buttons` ligado, uma ação cujo handler
      está ocupado é descartada antes de rodar (não só desenhada como desabilitada). Opt-out
      por nó: `busy = false`.
- [ ] Liberação antecipada: `busy.release()` no Luau solta o handler corrente; e uma opção de
      app `busy_release = toast` que reproduz o comportamento do rustploy (o primeiro `toast`
      da corrotina libera).
- [ ] Identidade pela corrotina, como no `busy.luau`: toast de outra ação não libera esta;
      ação chamada de dentro de outra segura as duas chaves.
- [ ] `__busy.<handler>` vira bool tipado (Frente 1) e fica legível no template para
      spinners (`spinner(if = @__busy.salvar)`).
- [ ] Testes no motor espelhando o `rustploy-gui/tests/lib/busy.test.luau` (duplo clique, toast libera,
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

## Frente 6 — Migração do rustploy-gui (parte da mesma entrega)

Quando as frentes acima forem publicadas, **o rustploy migra na mesma tacada**, como foi
feito com a 0.169 (`fmt --migrate` numa cópia, depois no repo). A entrega não está pronta
enquanto o rustploy usar a solução caseira.

- [ ] `glacier fmt --migrate` cobre o que é mecânico: `== "true"` em chave bool,
      trio de campo → `bind`, `disabled = @busy_<handler>` no botão do próprio handler → nada.
- [ ] No rustploy-gui:
  - [ ] ligar `app(busy_buttons = true, busy_release = toast)` no `views/app.gv`;
  - [ ] apagar `views/scripts/busy.luau`, `views/scripts/busy_actions.luau`,
        `tools/busy_actions.py` (no agregador `rustploy`), os testes `tests/lib/busy*.test.luau` e os `require("busy").install(…)` dos scripts de tela;
  - [ ] apagar os `disabled = @busy_*` (as linhas que sobrarem são as que combinam
        condições e ficam com `busy = expr`);
  - [ ] portar os casos de `tests/lib/busy.test.luau` para testes do motor (Frente 4) e conferir os de `tests/windows/` que dependem de `busy_*`;
  - [ ] trocar `json.encode` de lista para o ctx por atribuição direta;
  - [ ] converter os componentes extraídos (`views/service/*`, `views/home/*`, `views/shell/*`)
        para `props`/`state` onde hoje leem global com prefixo.
- [ ] Critério de pronto (medido com `grep` no rustploy-gui):
  - `busy_` em `views/` = 0;
  - `== "true"` / `!= "true"` em `.gv` = 0;
  - `json.encode` em `views/scripts/handlers/` só onde o destino é a rede, não o ctx;
  - `glacier check --strict` e `glacier test -vvv` verdes;
  - a regra de verdade nova (Frente 1) ligada no `app.gv`, e não antes dos itens acima.
- [ ] Atualizar o `AGENTS.md` do agregador rustploy (seção "Armadilhas de template") e o
      preset `crates/glacier-cli/templates/_comum/AGENTS.md`.

## Ordem sugerida

1 (tipos) → 2 (estado local) → 3 (componentes) → 4 (ocupado) → 5 (recursos) → 6 (migração).
A Frente 4 depende só da 1 e pode ir antes da 2/3 se for preciso tirar o `busy.luau` cedo;
a Frente 5 é independente.
