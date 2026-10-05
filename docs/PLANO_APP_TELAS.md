# Plano: o markup descreve o app, e o app tem telas

**Estado:** proposta. Nada aqui existe ainda.

---

## 0. A ideia

Hoje a raiz de um arquivo é um `<screen>`, e o que é do **aplicativo** (`app`,
`tray`) vai pendurado no `<resources>` dele. Isso tem dois defeitos:

1. Um app com várias telas não tem onde declarar o que é do app. `<app>` e `<tray>`
   "só valem no template principal" (`parser.rs`, doc do `AppMeta`) — numa outra
   tela são **ignorados em silêncio**. É a família de bug que o `CLAUDE.md` chama
   de a mais cara: declaração que parece valer e não vale.
2. As telas do app (as outras além da principal) são declaradas **em Rust** —
   `register_component` + `set_initial_screen` dentro de `.main(|motor| …)` — ou
   espalhadas em `link(rel = import)`.

A proposta: o **app é a raiz**, e as telas são filhos dele.

```
app(
  id = "{{nome_projeto}}", single_instance = true, remember_geometry = true,
  size = "980 640", min_size = "560 420", icon = "views/assets/icone.png",
) {
  resources {
    link(rel = theme, href = "views/styles/theme.json")
    link(rel = stylesheet, href = "views/styles/app.gss")
    tray(icon = "views/assets/icone.png") { … }
  }

  screen(name = home, initial = true, title = "Início") {
    column { … }
  }

  screen(name = ajustes, title = "Ajustes", src = "views/ajustes.gvb")
}
```

O que isso garante **por construção**: só existe um `app`, ele só pode estar na raiz,
e portanto nenhuma tela tem como discordar dele (`id`, instância única, geometria,
bandeja, tema, folhas globais).

`app` não precisa de outro nome. Ele já existe, mas **dentro do `<resources>`**; a
posição não precisa desambiguar: `app` só existe na raiz (a forma dentro de `resources` é removida, ver 1.5).

---

## 1. Decisões de design

### 1.1 O que mora no `app` e o que mora na `screen`

| No `app` (vale para o app inteiro) | Na `screen` (vale para ela) |
|---|---|
| `id`, `single_instance`, `remember_geometry` | `name`, `initial` |
| **janela principal:** `size`, `min_size`, `max_size`, `fixed_size`, `resizable`, `decorations`, `icon` | `title` (acompanha a navegação) |
| `tray` | — |
| `resources` global: `link` de tema, folhas `.gss`, dados, **componentes** (`component(name = X) { … }` inline e `link(rel = import)`) | `resources` local: `script`, `style(scoped)`, `props`, `link` só dela, **componentes só dela** |
| — | o layout (os filhos que não são declaração) |

Regra de escopo: o que o `app` declara é somado ao que cada tela declara. Uma folha
`.gss` global carrega **uma vez** (a chave `gss::<path>` já deduplica por caminho).

### 1.1.1 Componentes globais

Um componente declarado no `resources` do `app` vale para **todas as telas**, sem
cada uma repetir o `link(rel = import)`:

```
app(id = meu_app) {
  resources {
    component(name = Rotulo) { props { prop(name = texto) } text(content = "@texto") }
    link(rel = import, href = "components/nav_item.gvb", as = NavItem)
  }
  screen(name = home) { column { Rotulo(texto = "oi") NavItem(…) } }
}
```

- as duas formas que existem hoje (inline e por arquivo) valem, com a mesma sintaxe
  do componente local — o corpo é o que iria num arquivo próprio;
- **precedência:** o componente declarado na tela vence o global de mesmo nome
  (sombreia); dois com o mesmo nome **no mesmo escopo** é erro de parse;
- a regra da convenção segue: componente do app é `CamelCase`, e a busca é sensível
  a caixa;
- um componente global não vê o `script` de nenhuma tela (só as props que recebe).

### 1.2 Tela inline ou em arquivo

As duas formas, porque um app grande em um arquivo só vira inviável:

- **inline:** `screen(name = home) { … }` — o corpo é o layout;
- **por arquivo:** `screen(name = ajustes, src = "views/ajustes.gvb")` — o arquivo é
  um `.gvb` de hoje (raiz `screen`/`component`). **Esses arquivos não mudam.**

Isso faz a migração ser aditiva: todo arquivo com raiz `screen` continua válido; o
que muda é que agora existe um arquivo com raiz `app` que os lista.

### 1.3 Nome e tela inicial

- `name` obrigatório, **exceto** com `src` (aí vale o nome do arquivo sem extensão,
  como o `link rel=import` faz hoje; `name` o troca, como o `as`).
- `initial = true`: no máximo uma (erro de parse se duas). Sem nenhuma, vale a
  **primeira** declarada. Um app de uma tela só não escreve `initial`.
- O nome é o mesmo que `navigate:<nome>`, `navigate_to` e `open_window` já usam. Não
  há conceito novo de navegação.

### 1.4 Janela × tela (decidido: opção B)

Hoje `<screen>` mistura **destino de navegação** com **dados da janela do SO**
(`size`, `decorations`…), e o atributo de janela numa tela que não é a inicial era
ignorado ao navegar — declaração que parece valer e não vale. A separação:

- **`screen` é só conteúdo:** `name`, `initial`, `title`, `src`, layout e
  `resources` locais. `size`, `min_size`, `max_size`, `fixed_size`, `resizable`,
  `decorations` e `icon` no `screen` são **erro de parse**, com mensagem apontando
  para o `app`. Não existe entidade `window`.
- **`app` leva os atributos da janela principal** — a única que o daemon abre sem
  ninguém pedir. É o lugar natural: o `remember_geometry` já é dela.
  O `app` ainda leva `icon`, que as janelas filhas herdam.
- **Janelas filhas** são dadas por chamada: `open_window("ajustes", { size = "420 520" })`
  em Luau e `WindowSpec` em Rust (já existe, `component.rs`). O nome é o de uma tela.
- **`title` continua na tela** e acompanha a navegação.
- **Consequência assumida:** o tamanho não muda ao navegar (já não mudava). Quem
  precisa de uma janela pequena e outra grande abre duas janelas.
- **Fora do plano:** entidade `window`, `single`, geometria lembrada por janela,
  `modal`, `always_on_top`, `position`. Voltam como plano próprio se fizerem falta.

### 1.5 Quebra de compatibilidade (decidido)

Só o autor usa o framework hoje, então **não se mantém código para as duas formas**:

- `app` e `tray` dentro de `resources` deixam de existir. O parser os rejeita com
  um erro que diz "`app`/`tray` só na raiz `app`" — não há aviso de depreciação nem
  caminho duplo.
- `app` e `tray` só existem como filhos da raiz `app`. O `NodeType::App` lido de
  dentro de `resources` e a regra "só vale no template principal" são **apagados**.
- Um arquivo com raiz `screen` continua sendo uma tela válida (é o que `src` aponta
  e o que uma app de uma tela só, sem `app`/`tray`, pode ser). Um arquivo com raiz
  `screen` que declare `app`/`tray` é erro.
- `<component>` (arquivo que só é pedaço de tela) não muda.
- Todo template, exemplo e doc que usa a forma antiga é migrado **no mesmo
  trabalho** (passo 4); nada fica quebrado no fim.
- `.gv` (XML), `.gva` e `.gvb` usam a **mesma estrutura** (`<app><screen/></app>` e
  equivalentes); o parser é um só, então não há implementação por formato — só
  conferir com um teste por extensão.

### 1.6 Rust

Depois do plano, o `GlacierDaemon` só precisa de:

```rust
GlacierDaemon::new().run()                                    // views/app.gvb, raiz `app`
GlacierDaemon::new().main_template("views/app.gvb").run()
```

`.main(|motor| …)` fica **só** para o que não tem forma em markup:
`motor.register(Box::new(MeuComponente))` (lógica em Rust). Uma tela servida por
`impl Component` aparece no manifesto como `screen(name = contador)` **sem corpo e
sem `src`**; o `.main` registra o componente com esse nome (decidido). Tela sem
corpo/`src` cujo nome ninguém registrou: erro ao subir, citando o nome.

`.title`, `.main_size`, `.main_window`, `.single_instance`, `.remember_window_geometry`, `.tray`
seguem existindo e **vencendo** o markup, como hoje (`DAEMON.md`, regra 2).

---

## 2. Implementação, em ordem

Cada passo termina compilando; o passo 1 já remove a forma antiga, então ele e o 4 (migração) andam juntos.

### Passo 1 — parser: reconhecer a raiz `app`

`src/parser.rs`, junto ao bloco que "abre a casca" (~6490, hoje só trata
`Screen | ComponentRoot` com `roots.len() == 1`).

- Nova estrutura `AppManifest { meta: AppMeta, tray: Option<TrayMeta>, decls: Vec<UiNode>, screens: Vec<ScreenDecl> }`.
- `ScreenDecl { name, initial, meta: ScreenMeta, source: Inline(UiNode) | File(String) }`.
- Os atributos de janela saem da `ScreenMeta` e vão para o `AppMeta` (que passa a
  ter `size`/`min_size`/`max_size`/`fixed_size`/`resizable`/`decorations`/`icon`);
  a `ScreenMeta` fica só com `title`, e a `ScreenDecl` carrega `name`/`initial`/`src`.
  Atributo de janela num `screen` é erro de parse (1.4).
- `decls` globais incluem os `component` inline e os `link(rel = import)` do `resources` do `app`.
- Cada tela inline passa **pelo mesmo caminho de hoje** (um header `Screen` + filhos
  → a lógica existente de dissolver `resources`), uma vez por tela. Reaproveitar,
  não reescrever: o resto do motor nunca soube que existiu cabeçalho.
- `app`/`tray` dentro de `resources` viram erro (1.5), e o ramo que os lia de lá
  é apagado. Validar: dois `app`, `app`/`tray` fora da raiz, duas `initial`,
  nomes de tela repetidos, tela sem `name` nem `src`.
- Os preprocessamentos do `.gvb`/XML preservam a contagem de linhas de propósito:
  manter, para o erro apontar a linha do arquivo do autor.

### Passo 2 — `app_manifest` e registro de telas

`src/lib.rs` (`app_manifest`, ~4367) hoje faz parse inteiro e devolve
`(AppMeta, TrayMeta, título)`. Passa a devolver o `AppManifest`.

- Novo `GlacierUI::register_app(manifest, path)`: para cada `ScreenDecl`, registra o
  componente com o nome da tela (inline: a árvore já parseada; `src`: o mesmo
  caminho do `link rel=import`), carrega as `decls` globais uma vez (inclusive os componentes globais, registrados no mesmo espaço de nomes que o dos locais, com a precedência de 1.1.1), e chama
  `set_initial_screen`.
- `template_setup` (`daemon.rs` ~93) vira: se o arquivo tem raiz `app`,
  `register_app`; senão, o comportamento atual. O `run` já sabe se o template vem do
  padrão/`main_template` (é o que hoje habilita `<app>`/`<tray>`); a regra "só no
  principal" deixa de existir, porque a raiz `app` é por definição a principal.

### Passo 3 — hot-reload

Hoje as chaves de `file_mod_times` são por componente. Com telas inline, várias
telas partilham um arquivo: mudou o arquivo → **re-registrar todas as telas dele**
(mesmo padrão de `inline_style_key`, que já resolve o mesmo problema para `<style>`).
Garantir que a tela ativa e o histórico de navegação sobrevivem ao reload.

### Passo 4 — documentação, ferramentas e modelos

- `docs/DAEMON.md`: reescrever as seções 1–3 em torno do `app`; a tabela
  "No markup" aponta para o manifesto; "Antes e depois" ganha o caso de várias telas
  e a tabela do `.main(…)` perde as linhas de `register_component`/`set_initial_screen`.
- `README.md`, `CHANGELOG.md`, `docs/PRIMITIVAS.md`/`BUILTINS.md` onde citarem `<app>`.
- `open_window` em Luau passa a aceitar a tabela de opções de janela (`size`, …),
  espelhando o `WindowSpec`; `docs/DIALOGS.md`/`DAEMON.md` e o template `janelas`
  deixam de pôr `size` no `screen` da janela filha.
- `crates/glacier-cli/templates/{completo,janelas}`: `app.gvb`/`painel.gvb` e o
  `main.rs`/`README.md`/`AGENTS.md` passam para a forma nova; **o `glacier new`
  gera a forma nova**.
- `examples/gvb/gvb_convert` (usa `app`): ensinar o conversor `.gv` → `.gvb` a
  emitir a raiz `app` quando houver `<app>`.
- `editors/vscode-gv` (`extension.js`, `references/glacier-view.md`): completions,
  snippets e validação do `app`/`screen(name, initial, src)`; subir a versão
  (padrão dos últimos commits).
- Migrar só os exemplos que têm `app`/`tray` (ex.: `bandeja`) e criar **um**
  exemplo novo com duas telas e navegação. Os demais seguem como estão
  (“exemplos não precisam ser idênticos”).

### Passo 5 — enxugar o Rust

Com tudo acima, os `main.rs` de exemplos com `.main(|motor| register_component…)`
sem `impl Component` viram `GlacierDaemon::new().run()`.

---

## 3. Como verificar

O padrão de bug deste repositório é teste que passa com widget vazio. Aqui o
equivalente é o manifesto "carregar" e a tela não estar registrada. Testes devem
olhar o **resultado**, não só o parse:

- após `register_app`, `current_screen` é a `initial`; `navigate_to("ajustes")`
  avalia a tela certa (a árvore avaliada **tem** o layout dela, não um buraco);
- `AppMeta` e `TrayMeta` vêm do manifesto mesmo com a tela inicial **não** sendo a
  primeira do arquivo;
- erros: dois `app`, `app` dentro de `resources` de uma tela sob raiz `app`, duas
  `initial`, nome repetido — cada um com a **linha** do arquivo;
- a folha `.gss` global carrega uma única vez com N telas;
- um componente global é resolvido em todas as telas; o local de mesmo nome o
  sombreia só na tela dele; nome repetido no mesmo escopo dá erro com a linha;
- hot-reload mantém a tela ativa.

Regras do ambiente: **não rodar `cargo test`** aqui (memória do projeto); verificar
com `cargo check` / `cargo publish --dry-run`, e rodar o exemplo novo com
`WGPU_BACKEND=gl cargo run --example <nome>`.

---

## 4. Riscos

- **`app`/`tray` esquecidos na forma antiga** em algum arquivo: viram erro de parse
  com mensagem clara; o passo 4 migra todos os conhecidos (`grep` por `app(`/`<app`).
- **Tela com `src` cuja raiz não é `screen`/`component`**: erro claro.
- **`size`/`decorations` esquecidos no `screen`** de arquivos antigos: erro de parse
  apontando para o `app` (1.4); o passo 4 os migra (`grep` por `size =` em `screen`).
- **Nome `app` já ocupado** pelo arquivo padrão `views/app.gvb`: sem conflito, é nome
  de arquivo, não de tag.

---

## 5. Decisões tomadas

1. **Sem compatibilidade com a forma antiga** — ver 1.5.
2. **Tela servida por `impl Component`:** `screen(name = x)` sem corpo; o `.main`
   registra o componente — ver 1.6.
3. **Sem `initial`:** vale a primeira tela — ver 1.3.
4. **`.gva`/`.gv`/`.gvb`:** mesma estrutura — ver 1.5.
5. **`script` global do app:** fora deste plano, mas **vira o próximo plano**
   (item 6).

## 6. Depois deste plano: gerar `docs/PLANO_SCRIPT_APP.md`

Quando os passos 1–5 estiverem prontos e verificados, o último entregável é um
**novo plano** para o `script` no nível do `app` (estado e lógica que vivem o app
inteiro, hoje impossíveis de declarar sem repetir em cada tela). Ele deve levantar,
já com o manifesto funcionando:

- onde o `script` do `app` roda (um runtime Luau/Python por app × um por tela) e o
  que as telas enxergam dele (`ctx`, `storage`, funções);
- a ordem de inicialização (`init` do app antes do `init` da primeira tela);
- como convive com o `script` local de cada tela e com o hot-reload;
- relação com `lua_extension` e com o global `storage`.
