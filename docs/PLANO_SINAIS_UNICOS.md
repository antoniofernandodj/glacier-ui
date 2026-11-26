# Plano: cada sinal do `.gv` com um significado só

**Estado:** plano, **nada implementado** (2026-11-25, sobre o glacier-ui 0.169.2). Faz parte
da entrega do `TODO.md` (Frente 10) e migra o rustploy-gui junto (Frente 9).

## O problema

O `docs/LINGUAGEM.md` tem uma tabela de sigilos, e ela é necessária: hoje a mesma escrita
significa coisas diferentes conforme o lugar, e coisas iguais têm várias escritas. O
manual ensina, mas não tira o custo — quem lê precisa parar e consultar. O objetivo é que
**cada sinal tenha um significado, em qualquer lugar**, para a tabela caber numa tela e ser
lembrada sem abrir.

### O que confunde hoje (com contagem nos 68 `.gv` do rustploy-gui)

| Confusão | Exemplo | Usos |
|---|---|---|
| Um `@` a mais muda tudo | `:value = campo` liga à chave `campo`; `:value = @campo` liga à chave **cujo nome está em** `campo` | 298 `:attr` |
| `:` com três papéis | ligação (`:value = x`), qualificador (`class:ativo = …`), prefixo de embutido (`field:x`, `window:resize:nw`) | 298 + 123 + 176 |
| `:` ao contrário do Vue/Angular | no Vue `:value="x"` é **uma via** (o atributo recebe a expressão); aqui `:value = x` é **duas vias** | 298 |
| Palavra sem aspas com quatro papéis | `title = Rustploy` (texto), `class = btn` (classe), `on_click = salvar` (handler), `align_x = center` (enum) | — |
| Quatro formas de mostrar um valor | `text(@m)`, `text("@m")`, `text("{m}")`, `text("{@m}")` | 352 `"…@x…"`, 7 `"{x}"` |
| `{x}` sem `@` lê a chave `x`; fora de `{}` é preciso `@` | `"{__dialog.message}"` × `@__dialog.message` | 7 |
| Aspas que não são texto | `class = "btn btn--primary"` (lista de classes), `size = "1280 820"` (par de números) | centenas |
| `@` dentro de texto interpola | `"contato@empresa.com"` precisa de `@@` | — |
| Valor padrão com sintaxe própria | `{chave\|padrão}` | 0 (mas existe) |
| O motor adivinha se um atributo é expressão | "valor começando por `@`, `!`, `(`, `[` ou `t(`, `len(`… é expressão" | 824 `attr = @x` |

## Decisão do dono (2026-11-25): o `:` fica, com o sentido do Vue e do Angular

O `:` é mantido **na frente do nome do atributo**, como no Vue (`:title="expr"`, o
`v-bind`) e no Angular (`[title]="expr"`): **o atributo recebe o resultado de uma
expressão, numa via, do estado para a tela.** Sem `:`, o atributo recebe um literal.
A ligação de duas vias, que hoje usa o `:`, ganha o `::` (o "dois sentidos" do
`[( )]` do Angular / `v-model` do Vue). Uma versão anterior deste plano trocava o `:` por
`bind(…)`; foi descartada.

| | Vue | Angular | `.gv` |
|---|---|---|---|
| literal | `title="Rustploy"` | `title="Rustploy"` | `title = "Rustploy"` |
| uma via (expressão) | `:title="nome"` | `[title]="nome"` | `:title = @nome` |
| duas vias | `v-model="nome"` | `[(ngModel)]="nome"` | `::value = @nome` |
| classe condicional | `:class="{ativo: c}"` | `[class.ativo]="c"` | `:class.ativo = @c` |
| evento | `@click="salvar()"` | `(click)="salvar()"` | `on_click = salvar()` |
| condição | `v-if="c"` | `@if (c)` | `if = @c` / `if @c { … }` |

**O que não se copia do Vue:** o `@click`. No `.gv` o `@` já é "o valor de uma chave" (824
usos no rustploy) e ficaria com dois sentidos. O evento continua `on_click = salvar()`.

## O desenho: a tabela inteira

| Escrita | Significa | Só isso |
|---|---|---|
| `attr = literal` | o atributo recebe **um literal**: texto entre aspas, nome, número, cor, lista | sempre |
| `:attr = expr` | o atributo recebe **o resultado de uma expressão**, numa via (Vue `:attr`, Angular `[attr]`) | sempre e só na frente do nome |
| `::attr = alvo` | o atributo **liga nos dois sentidos** a uma chave (`@x` ou `key(expr)`) | só em widget de valor |
| `:attr.sub = expr` | **parte** de um atributo por expressão: `:class.ativo = @c` (Angular `[class.ativo]`) | `class` hoje; `style` se vier |
| `"…"` | **texto**. Nada entre aspas é nome, lista, número ou ligação | sempre |
| `{expr}` | **buraco de expressão dentro de um texto**: `"Olá, {@nome}"` | só dentro de aspas; `{{`/`}}` é a chave literal |
| `@x` | **o valor da chave `x`** (com `@x.campo`, `@x[0]`) | só dentro de expressão; num texto, `{@x}` |
| `nome` | **um nome**: classe, valor de enum, componente, prop, handler | nunca texto |
| `nome(…)` | **uma chamada**: função de expressão (`t`, `len`…), ação (`salvar()`) | sempre |
| `[a, b]` | **lista** | sempre |

Literais de valor sem aspas: número (`12`, `0.5`), cor (`#7F849C`), `true`/`false`/`null`.
Tudo o que não cabe na tabela é erro de parse com `arquivo:linha` e a forma certa.

### Onde a expressão vem sem `:`

Três posições **são** expressão por natureza e não levam `:` — como no Vue, em que
`v-if` e `@click` também não levam:

- **diretivas:** `if = expr`, `each = @lista as v`, `key = expr` (e o `else`);
- **eventos:** `on_x = acao(args)`;
- **o conteúdo posicional** de `text`/`button` (`text(@mensagem)`, `text("Olá, {@n}")`),
  o equivalente ao `{{ }}` do Vue: é conteúdo, não atributo.

Em qualquer outro atributo, **é o `:` que decide**: com `:` é expressão, sem `:` é literal.
Some a heurística "começa com `@`/`!`/`(`… é expressão": `disabled = @busy` sem `:` é erro
("use `:disabled = @busy`").

## Cada conceito, uma escrita

| Conceito | Hoje (todas aceitas) | Depois (a única) |
|---|---|---|
| Mostrar um valor | `@m`, `"@m"`, `"{m}"`, `"{@m}"` | `text(@m)`; num texto composto, `"Nota: {@m}"` |
| Atributo vindo do estado | `disabled = @busy`, `label = @x` | `:disabled = @busy`, `:label = @x` |
| Texto literal | `"Rustploy"`, `Rustploy`, `'Rustploy'` | `"Rustploy"` (`'…'` só dentro de `{}` num texto) |
| Valor padrão | `{x\|padrão}` | `@x ?? "padrão"` (operador novo na expressão) |
| Ligação de duas vias | `:value = x` | `::value = @x` |
| Ligação dinâmica (nome da chave vem de outra) | `:value = @x`, `:value = "{x}"` | `::value = key(@x)` |
| Leitura de lista/opções por nome | `:items = x`, `:options = x` (quando só leem) | `:items = @x` |
| Campo de formulário | `form_control = x` + `:value = x` + `on_change = field:x` | `::value = @x` (dentro de `form`, o controle do form vem do `::`) |
| Lista de classes | `class = "btn btn--primary"` | `class = [btn, btn--primary]` (uma: `class = btn`) |
| Classe condicional | `class:ativo = expr` | `:class.ativo = expr` |
| Par/quádrupla de números | `size = "1280 820"`, `padding = "9 16"` | `size = [1280, 820]`, `padding = [9, 16]` |
| Ação | `salvar`, `salvar()`, `copiar:@id` | `salvar()`, `copiar(@id)` (Frente 6) |
| Embutido | `window:resize:nw`, `field:x`, `tray:open` | `window.resize("nw")`, `tray.open()`; `field:x` some (vira `::`) |
| `@` literal num texto | `"a@@b.com"` | `"a@b.com"` (dentro de aspas `@` não interpola mais) |
| i18n | `t("k")`, `"{t('k')}"` | `:label = t("k")`; conteúdo `text(t("k"))`; num texto composto `"◆ {t('k')}"` |

Com isso o `:` tem **um** sentido (prefixo: "expressão aqui"), e o `::` é o mesmo sentido
nos dois sentidos. O `:` no meio do nome (`class:ativo`) e nos embutidos (`window:resize`)
some. O `:` do ternário (`a ? b : c`) é sintaxe de expressão e fica — a mesma convivência
do Vue/JS.

## O que precisa ser conferido antes de implementar

- [ ] Quais `:attr` de hoje são de **duas vias** de fato e quais só leem por nome. No
      rustploy: `:value` (178), `:checked` (54), `:options` (31), `:items` (23), `:sort` (12)…
      `:options`/`:items` parecem só leitura → `:options = @x`; `:value`/`:checked` → `::`.
- [ ] Os builtins que usam ligação dinâmica (`wizard`, `tabs`, `command`, com
      `:value = "{value}"`) passam a `::value = key(@value)`; conferir que nenhum outro uso
      depende do `"{x}"` sem `@`.
- [ ] Classificar cada atributo do catálogo como **texto**, **nome**, **número/lista** ou
      **valor de widget** (aceita `::`). É o catálogo que torna "nome onde se espera texto" e
      "`::` em atributo que não é de valor" erros.
- [ ] Componente do app: `Comp(:titulo = @x)` passa prop por expressão e
      `Comp(titulo = "Fixo")` por literal, a mesma regra. Prop que aceita `::` é declarada
      (`prop(name = valor, type = string, model = true)`), como o `defineModel` do Vue.
- [ ] O `.gss` não entra: ele segue a sintaxe do CSS (`padding: 8 15;`, `var(--x)`), outra
      linguagem com seus próprios sinais. Só se garante que nenhum sinal do `.gv` mude de
      sentido dentro de `style """…"""`.

## Migração

- [ ] `glacier fmt --migrate` ganha as trocas abaixo, todas mecânicas e idempotentes:
  - `attr = @x` / `attr = expr` fora de diretiva, evento e conteúdo → `:attr = …`
    (no rustploy: 266 `disabled`, 36 `current`, 20 `label`…; os `disabled = @busy_*`
    somem antes, pela Frente 4);
  - `:attr = x` de duas vias → `::attr = @x`; de leitura → `:attr = @x`;
    `:attr = @x` / `"{x}"` → `::attr = key(@x)`;
  - trio de formulário → `::value = @x`;
  - `class:nome = e` → `:class.nome = e`;
  - `"@x"` (o valor inteiro) → `@x`; `"texto @x"` → `"texto {@x}"`; `"{x}"` → `"{@x}"`;
  - `"a@@b"` → `"a@b"`; `{x|p}` → `{@x ?? "p"}`;
  - `class = "a b"` → `class = [a, b]`; `size = "1 2"` → `size = [1, 2]` nos atributos
    numéricos do catálogo;
  - texto sem aspas em atributo de texto → entre aspas.
- [ ] Período de convivência: as formas antigas carregam com aviso de depreciação
      (`glacier check` lista); viram erro de parse numa `feat!` depois do rustploy migrado.
      **Atenção:** `:value = x` muda de sentido (duas vias → uma via). Durante a
      convivência, `:attr = nome_sem_@` continua sendo lido como a forma antiga com aviso;
      a forma nova de uma via sempre tem expressão (`@x`, chamada, literal entre aspas), e o
      parser distingue as duas por isso. Na `feat!`, `:attr = nome` passa a ser o literal
      `nome` avaliado como expressão (um nome), e o `check` acusa.
- [ ] Reescrever na forma nova: `docs/LINGUAGEM.md` (a tabela de sigilos vira a deste
      plano), `CLAUDE.md`, preset `AGENTS.md` do glacier-cli, `BUILTINS.md`,
      `PRIMITIVAS.md`, exemplos, templates dos builtins e blocos ```gv``` de `.md` (o
      `fmt --migrate` já migra `.md`).
- [ ] Corpus: reimprimir todo `.gv` do repositório e do rustploy numa cópia, conferir que a
      árvore de widgets avaliada é a mesma antes e depois para o mesmo ctx.

## Critério de pronto

- A tabela de sinais do `docs/LINGUAGEM.md` é a deste plano, sem exceções.
- No rustploy-gui: zero `:attr = nome` na forma antiga, zero `class:`, zero `field:`, zero
  `"@x"` como valor inteiro, zero `"{x}"` sem `@`, zero `class = "a b"`, zero `attr = @x`
  sem `:` fora de diretiva/evento/conteúdo, zero texto sem aspas em atributo de texto
  (`glacier check --strict` sem aviso de sinal).
- O motor não tem mais a heurística "começa com `@`/`!`/`(`… é expressão": o `:` decide.
