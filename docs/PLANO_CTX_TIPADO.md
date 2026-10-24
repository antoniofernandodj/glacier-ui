# Plano: `ctx` tipado (e a saída do MicroPython)

**Estado:** **concluído** — Fases 0 a 5 implementadas (0.134 a 0.138; rustploy-gui 0.1.19); Fase 5 (rustploy-gui) concluída — ver "Fase 5" abaixo. Nasce de uma pergunta: *o tipo que o
script Luau grava em `ctx` se perde, porque tudo vira string — dá para preservar
e devolver com o tipo certo?* Dá. Este plano cobre o caminho inteiro, inclusive o
que ficaria para depois.

Duas decisões já tomadas, que moldam o resto:

1. **O MicroPython sai do motor.** Deixa de existir a segunda linguagem de
   `<script>`; sobra só o Luau. Isso tira do caminho a distinção `int`/`float`:
   no lado Luau só existe `number`.
2. **Os widgets também gravam com tipo.** Um `<checkbox value="aceito">` faz
   `ctx.aceito` chegar ao script como `boolean`, não como `"true"`.

## Decisões (2026-10-22)

- O `ctx` tipado entra **inteiro** (Fases 0 a 4) e o `rustploy-gui` o adota na
  Fase 5. O `"[]"` em string é o sintoma que motivou a adoção.
- **Não** se cria um tipo opaco "json". O valor JSON no Luau é a **tabela** com uma
  **marca do motor** (metatable): `json.array(t)` e `json.object(t)`. Motivo: uma
  tabela mantém `#`, `ipairs`, `t[i]` e `table.insert`; um userdata teria de
  reimplementar tudo isso. Um método `to_string()` colidiria com chaves do dado, então
  a conversão em texto é `__tostring` na marca (e o `json.encode` de sempre).
- A marca viaja nos dois sentidos: o que o script grava como `Json` volta ao script
  como tabela **com a mesma marca**, então `ctx.x = ctx.x or json.array({})` e
  `table.insert(ctx.x, v)` funcionam sem texto.
- Atribuir uma tabela **vazia sem marca** ao `ctx` grava `[]` (e não `{}`): o `{}` em
  `ctx` nunca significou "objeto vazio" para widget nenhum, e `json.decode` devolve a
  mesma tabela dos dois. `json.encode({})` continua `{}` (corpos de API).
- Releases renumerados: a 0.133.0 já saiu; a 0.134.0 abre a Fase 0 junto com a
  correção de `tela:set`/`component(...)` aceitarem tabelas.

---

## 1. O problema, com precisão

O contexto do motor é `ContextMap = HashMap<String, String, Fx>`
(`src/lib.rs:683`, ~112 usos). A fronteira com o Luau é:

| sentido | função | o que faz |
|---|---|---|
| script → motor | `luau_value_to_string` (`src/luau/mod.rs:1725`) | `true`→`"true"`, `3`→`"3"`, tabela→`json.encode` |
| motor → script | `sync_to_luau` (`:416`) | `ctx_table.set(k, v.as_str())` — **tudo string** |

Consequências observadas:

- `ctx.flag = false` volta como `"false"`, que é **truthy** em Luau. Sem erro,
  sem aviso, comportamento invertido.
- `ctx.lista = {…}` volta como texto; o script precisa de `json.decode`.
- Número só funciona porque o Luau coage string em aritmética; `tonumber(ctx.x)`
  espalhado (14 ocorrências nos exemplos/templates).
- A string `"42"` e o número `42` são indistinguíveis.
- **Bug latente achado de passagem:** `luau_value_to_string` faz `n as i64` para
  todo `Number` de parte fracionária zero. `1e300` satura em `i64::MAX`. Entra na
  Fase 1 como correção.

O que **não** muda: markup, `eval`, widgets e `if=` continuam lendo `&str`.
`is_truthy` (`eval.rs:1128`) segue aceitando `"true"|"1"|"yes"|"on"|"sim"`.

---

## 2. A ideia: o tipo viaja ao lado da string

`ContextMap` continua `String → String`. Ao lado dele vive um **mapa de tags**:

```rust
enum Tag { Bool, Number, Json }          // ausente = string (o padrão)
struct Tipado { tag: Tag, snapshot: String }
type TagMap = HashMap<String, Tipado, Fx>;
```

Regras:

- **Gravar com tipo:** escreve a string canônica em `ContextMap` e
  `(tag, snapshot = a mesma string)` em `TagMap`.
- **Ler com tipo:** a tag só vale se `data[k] == snapshot`. Se qualquer outro
  caminho sobrescreveu a chave (widget sem tag, `Context::set` do Rust, patch de
  efeito assíncrono), a tag está obsoleta e a chave volta a ser string. **Nunca
  se ressuscita um tipo errado.**
- **Canônicos:** `Bool` = `"true"`/`"false"`; `Number` = inteiro sem `.0` quando
  cabe em 2^53, senão o shortest round-trip do `f64`; `Json` = `serde_json`
  compacto.
- **Não finito:** `NaN`/`±inf` não têm representação; avisar e descartar (hoje
  gravaria `"NaN"`).
- Remover a chave (`ctx.x = nil`, `data.remove`) remove a tag junto.

Por que sidecar e não `enum Value` no lugar da string: ver a Fase 6. Resumo — o
sidecar não toca o caminho quente (`{chave}` lê `&str` como hoje) e entrega 100%
do comportamento pedido.

### Lado Luau: cache de tabelas

Decodificar todo `Json` a cada evento seria o regresso de desempenho que já
mordeu este repositório. Mitigação: um cache `chave → (snapshot, valor Luau)`:

- `sync_to_luau`: se a tag é `Json` e o snapshot bate com o do cache, **reusa o
  mesmo objeto Luau** (sem decodificar); senão decodifica e atualiza o cache.
- `sync_from_luau`: re-encoda cada tabela `Json` do cache e compara com o
  snapshot; igual → não escreve (e não invalida dependência de `eval_deps`).
  Diferente → mutação no lugar (`ctx.lista[1].x = 2`) é gravada.
- Bônus: referências a tabelas guardadas em corrotinas suspensas continuam
  válidas entre turnos (a tabela `ctx` já é mantida in-place por esse motivo).

---

## 3. Fases

Cada fase é um release (minor, pela convenção do projeto: bump de minor pode
quebrar API, com a seção **Quebras** do CHANGELOG dizendo como migrar).

### Fase 0 — Remover o MicroPython (0.134.0)

Primeiro, porque simplifica tudo depois: um só runtime de script para tipar.

**O que sai**

- `src/micropython/` (`mod.rs`, `prelude.py`), `vendor/micropython_embed/` (3 MB)
  e `vendor/glacier_mp_shim/`.
- `build.rs`: hoje só compila o MicroPython — apagar o arquivo, o `build =` e a
  dependência de build `cc` (conferir no `Cargo.lock` se algo mais usa `cc`).
- `Cargo.toml`: a feature `micropython`, o `[[example]] contador_micropython`
  (gva e gvb) e os comentários associados (~linhas 207, 409, 465, 552).
- Exemplos `examples/gva/contador_micropython/` e
  `examples/gvb/contador_micropython/` (`.py`, `.gss`, `.gva`/`.gvb`, `main.rs`).
- `src/lib.rs`: `pub mod micropython` (97), os desvios de `has_script` (~3475 e
  ~4725), o erro de tela inline `lang="python"` (~1451) e os comentários.
- `src/error.rs`: a variante `GlacierError::MicroPython`.
- `src/luau_web.rs:41`: o `!= Some("micropython")`.

**O que muda de comportamento (cuidado)**

- `eval::script_lang` normaliza `"python"` → `"micropython"` (`eval.rs:86-103`).
  Depois da remoção, um `<script lang="python">` **não pode** cair em Luau em
  silêncio (rodaria Python como Luau e falharia de forma obscura). Deve dar um
  erro claro: *"lang=\"python\" não é mais suportado; só Luau"*. Vale para
  qualquer `lang` desconhecido.
- `tests/gvb_sintaxe.rs:47` usa `script(lang = python)` como amostra de sintaxe;
  trocar por outro atributo.

**Auditar** (podem ter referência a Python que não seja o motor): `crates/glacier-cli`
(`scaffold.rs`, `serve.rs`, `extensions.rs`, `config.rs`, `ANDROID_TODO.md`),
`editors/` (gramática/realce de `lang`), `.github/` (jobs que ligam a feature),
`README.md`, `docs/PLANO_APP_TELAS.md`, `docs/PLANO_SCRIPT_APP.md`.

**Documentação:** CHANGELOG com **Quebras** (feature removida, `lang="python"`
agora é erro); histórico antigo do CHANGELOG fica intacto. Atualizar/remover a
memória `micropython-script-lang.md` do projeto.

**Verificação:** o crate compila sem a feature; `cargo publish --dry-run`; o
pacote publicado encolhe ~3 MB; os exemplos `gva`/`gvb` restantes carregam.

### Fase 1 — Núcleo: bool e number (0.135.0)

Só o que corrige o bug do `false` truthy, ainda sem tabelas nem widgets.

1. `Tag` + `TagMap`; o `TagMap` mora ao lado de `context_data` em `GlacierUI` e
   `Context` ganha uma referência (`types: &mut TagMap`) — um ponto só, onde o
   `Context` é construído.
2. `luau_value_to_string` passa a devolver `(String, Option<Tag>)`; corrige o
   `n as i64`; rejeita não-finitos.
3. `sync_to_luau`: `Bool` → `Value::Boolean`, `Number` → `Value::Number`
   (validando snapshot). Tabela e string seguem como estão.
4. `sync_from_luau`: grava tag; apaga tag de chave removida.
5. **API Rust do host:** `Context::set_bool/set_number` e
   `get_bool/get_number` (retornam `Option`, parseando a string só quando a
   tag bate; `get` continua devolvendo `&String`, inalterado).
6. `semeia` (`mod.rs:668`) e os pontos que escrevem `ctx_table` direto precisam
   respeitar a tag, senão a primeira leitura volta a ser string.
7. Harness de teste (`testing.rs`, `component(path, initial_ctx)`): o
   `initial_ctx` com `true`/`3` vira tag; a leitura de `ctx` no teste devolve o
   tipo. Hoje converte com `to_string_lossy` (`testing.rs:637`).

**Quebras reais (e a mitigação):**

- Quem faz `ctx.x = true` e depois lê `ctx.x == "true"` passa a ver `boolean`.
  Quem grava a *string* `"true"` não muda. No repo: 14 comparações
  `== "true"`/`== "false"` (exemplos, docs, templates) e `tonumber(ctx.x)`
  (continua funcionando — `tonumber(5)` é 5).
- **Lint de carga:** ao carregar um script, varrer o texto por
  `ctx.<nome> ==/~= "true"|"false"` e emitir um aviso *uma vez* ("`ctx.x` agora
  pode ser boolean; compare com `true`/`false`"). É heurístico e barato.
- **Válvula de escape temporária:** `<script ctx_types="off">` mantém o
  comportamento antigo por script. Removida na versão seguinte à da Fase 3.
  *(Decisão aberta — ver seção 6.)*

### Fase 2 — Tabelas e a marca JSON (0.136.0)

1. `Tag::Json` + o cache da seção 2.
2. `ctx.lista = {…}` volta como **tabela** (objeto Luau reaproveitado entre
   turnos). `ctx.lista = json.encode(t)` continua sendo *string* — o padrão
   atual da documentação segue funcionando e dando o mesmo efeito no markup.
3. Casos de borda a cobrir com teste:
   - `{}` ambíguo (objeto vs. array): a tag `Json` não basta, o *snapshot*
     (`{}` ou `[]`) já resolve na volta; `json.array({})` grava `[]`.
   - `null` em JSON → `nil`; buracos em array; chaves não-string; tabela
     cíclica (hoje `from_value` falha → avisar e descartar, como já faz).
   - Dois nomes para a mesma tabela (aliasing): cada chave tem o seu cache.
   - Widget sobrescrevendo uma chave `Json` com string: snapshot diverge →
     cache descartado → chave vira string.
4. **Medido (build de depuração, lista de 10 000 objetos, 300 eventos):** re-codificar
   toda tabela a cada turno custava **222 ms/evento** (contra 1,1 ms da string antiga) —
   inviável. Solução implementada: chaves JSON **preguiçosas** (`__index`/`__newindex` no
   `ctx`; decodifica no primeiro acesso, recodifica só o tocado). Resultado: handler que
   não toca na lista **0,12 ms**; handler que a lê ~224 ms (custo de quem usa).
   Medição original exigida antes de fechar a fase: lista de 10 000 itens, mudança
   de um campo, N eventos seguidos. Critério: nenhum regresso mensurável frente
   ao estado atual (há `src/perf.rs`). Se o re-encode por turno pesar, o plano B
   é uma tabela com `__newindex` marcando "suja" (mais complexo; só se medir mal).
5. `Context::set_json<T: Serialize>` / `get_json<T: DeserializeOwned>` para o Rust.
6. **A marca JSON** (decisão acima):
   - `json.array(t)` (já existe), `json.object(t)` (novo, marca `t` como objeto) e
     `__tostring` nas duas marcas, que devolve o JSON compacto (`tostring(lista)`).
   - Ler uma chave `Json` devolve a tabela **com a marca** (o cache da seção 2
     guarda a tabela já marcada); o `sync_from_luau` a recodifica com a marca.
   - Tabela **vazia sem marca** atribuída ao `ctx` grava `[]`. Uma tabela com a
     marca de array e chaves que não são sequência (`ctx.x.foo = 1`) vira objeto na
     escrita — a marca não pode produzir JSON inválido.
   - `null` num JSON lido vira `nil`, como hoje; um array com furo (`nil` no meio)
     é recodificado preenchendo o furo com `null`. `json.null` fica de fora até
     alguma API exigir.
   - A string `"[]"` gravada à mão continua sendo **string** (sem tag): nada da
     compatibilidade se perde.

### Fase 3 — Widgets gravam com tipo (0.137.0)

O que torna o `<checkbox>` útil de verdade para quem escreve `if ctx.aceito then`.

**Implementado (0.137.0):** `EngineMessage::UiInputTyped { action, value, kind }` (o mesmo
roteamento de `UiInputChanged`, com o tipo como dica que o `Context::set_input` usa no
fallback de escrita) e `ContextPatchTyped` (para a escrita direta sem `on_change`).
Tipados: `checkbox`, `toggle` (Bool); `slider`, `spinbox`, `rating`, `dial`,
`pagination` (Number). O argumento de um handler próprio continua texto. Um número
que o script devolve sem mudar mantém o texto do widget (`0.50`). Ficam texto, por
ora: `radiogroup`, `select`, `tabbar`/`tabs` (id), `accordion`/`drawer` (`""`/`"true"`),
`splitter`/`grip`/`scroll` (escritas diretas do motor), `rangeslider`, `colorwheel`.

**Mecanismo (desenho original).** Hoje os widgets emitem `EngineMessage::ContextPatch(Vec<(String,
String)>)` (50 usos; tratado em `lib.rs:2174`) e `EffectOutcome.patch`. Em vez de
mexer em todos os sítios, acrescentar uma variante irmã:

```rust
ContextPatchTipado(Vec<(String, String, Tag)>)
```

O `ContextPatch` antigo permanece e **limpa a tag** da chave (a string escrita
quase nunca coincide com o snapshot; se coincidir, continua sendo o mesmo valor).
Só os widgets com saída de tipo conhecido migram para a variante nova.

**Levantamento de widgets** (a confirmar um a um na implementação; só a camada
`widget.rs` e `src/builtins/` gravam valor):

| tipo | widgets candidatos |
|---|---|
| `Bool` | `checkbox`, `toggler`, `animated_toggler`, switch/`round_button` de estado, `groupbox` checável, `accordion` (aberto/fechado), `drawer` |
| `Number` | `slider`, `spinbox`, `progressbar` (entrada), `rating`, knob/dial, `splitter`, `scroll`, `tab_bar` (índice) |
| string (fica) | `textinput`, `textarea`, `combobox`, `radiogroup`, `dateedit`/`timeedit` (ISO é texto por desenho — ver o comentário do módulo `date` no `prelude.luau`), `colorpicker`, `fontselect` |
| `Json` | seleção múltipla/`listview`, ordenação (`sort_var`) |

**Quebras.** Um script que lia `ctx.aceito == "true"` de um checkbox quebra — é a
parte mais visível da mudança. Migrar os exemplos e templates; o lint da Fase 1
cobre o texto.

**Teste (regra do `CLAUDE.md`):** não basta escrever e reler a chave. Para cada
widget migrado, um teste que monte a árvore avaliada e verifique (a) o ramo
`if="{aceito}"` existe/some, (b) `value_var` continua sendo o *nome* da chave, não
o valor interpolado, e (c) o script enxerga `type(ctx.aceito) == "boolean"`.
Rodar o exemplo com `WGPU_BACKEND=gl`.

### Fase 4 — Acabamento (0.138.0)

**Feita.** Verificado: `AGENTS.md` e presets do CLI (glacier-cli 0.18.0), cabeçalho de
`src/luau/mod.rs`, `README.md` sem o modelo "tudo é texto"; `ctx_types="off"` não existe mais;
`storage` e o payload de `broadcast` passam por JSON e já preservam o tipo (`storage.set("k", true)`
volta `true`). O `ctx` inicial de uma janela filha segue como texto (decisão: não leva tag).
Exemplos: `tem_notificacao` e `cancelado` viraram booleano em `gvb/` e `gva/`. Ficaram de
propósito `__dialog.negrito == "true"` (chave de diálogo, não conferida em execução) e o
`restart == "true"` do `main.rs` (a API Rust de `Context` é texto). O comentário do
`prelude.luau` agora diz que o `<dateedit>` grava texto, sem afirmar que todo `ctx` é string.

- Reescrever as "três consequências" em `crates/glacier-cli/templates/_comum/AGENTS.md`
  (hoje: *"Tudo é texto… compare com `tonumber`"*), o cabeçalho de `src/luau/mod.rs`
  e `README.md`; tirar o comentário `ctx é {[string]: string}` do `prelude.luau`.
- `glacier.d.luau` já declara `ctx: { [string]: any }`; o `.luaurc` lista `ctx` em
  `globals`, o que vira `any` de qualquer jeito (memória `luaurc-globals-anulam-tipos`).
  Nenhuma mudança necessária, só documentar.
- Remover a válvula `ctx_types="off"` e o aviso de lint se já não fizerem sentido.
- Migrar os 14 `== "true"` e os `tonumber(ctx…)` supérfluos dos exemplos, trocando
  `"true"` por `true` onde o valor é lógico. *Mecânico; basta carregar* (memória
  `exemplos-nao-precisam-ser-identicos`).
- Verificar `storage` (já é JSON): confirmar que `storage.set("k", true)` e
  `storage.get` preservam o tipo hoje; se não, alinhar.
- Conferir `open_window`/`broadcast`: o contexto inicial de uma janela filha e o
  payload de broadcast passam por string. Decidir se levam tag (provável: sim para
  `Bool`/`Number`, `Json` já viaja como JSON — `mod.rs:1023`).

### Fase 5 — Adotar no rustploy-gui

**Concluída** (rustploy-gui 0.1.19, commit `37d4b0a`). O app usa a glacier-ui 0.138.0. As
listas viraram tabela (`17a65e3`); os seeds de `jobs`, `services` e `wizard` passaram a
`json.array({})` e o `log_window` lê o seed com `Helpers.decode_list`. Os `== "true"` de
Luau já tinham migrado (`ecc7159`: `Helpers.is_true/is_false`); os que sobram são markup
`.gvb` e não mudam. A camada `format/` segue devolvendo `"[]"` em texto de propósito, e os
testes dela o afirmam: `decode_list` aceita as duas formas. Suíte: 377 testes passam.

Só depois de a 0.136.0 (Fase 2) estar publicada, e a 0.137.0 (Fase 3) para os
widgets. O app é um repositório à parte (`../rustploy/rustploy-gui`); commits lá
só quando o usuário pedir.

- **Listas vazias.** Os ~44 `"[]"` de `views/` viram `json.array({})` (ou `{}`, pela
  regra da tabela vazia). `ctx.x = ctx.x or "[]"` vira `ctx.x = ctx.x or json.array({})`.
- **Leituras.** `pcall(json.decode, ctx.x or "[]")` vira o acesso direto à tabela;
  comparações como `ctx.new_job_git_providers == "[]"` viram `#ctx.x == 0`.
- **Booleanos e números** (Fases 1 e 3): os ~80 `== "true"`/`== "false"` dos
  `.luau` migram caso a caso (quem escreve a chave: checkbox → `boolean`; script →
  o que gravou). Os `if @x == "true"` dos `.gvb` são markup e **não mudam**.
- Bump da dependência `glacier-ui` no `Cargo.toml` e no `Cargo.lock`; testes
  (`glacier test`, `cargo test --test app_manifest`) e telas de `settings`,
  `new_service` e `docker`.

### Fase 6 — O que ficou de fora de propósito (condicional)

Registrado para não ser reinventado nem esquecido.

**5a. `ContextMap` com `enum Value` no lugar de `String`.** A forma "pura", mas:
toca ~112 usos, todo o `eval`, todos os widgets e a API pública
(`Context::get -> &String`); cada `{chave}` passaria a formatar o valor em vez de
emprestar a string, e a leitura de contexto é o caminho mais quente do motor.
*Critério para reabrir:* se o sidecar mostrar-se insuficiente (por exemplo, se
surgirem muitas chaves tipadas escritas por caminhos que o sidecar não alcança),
ou se o motor ganhar uma segunda fonte de dados tipados (ex.: um `<datasource>`
que já nasça estruturado). Antes de decidir: medir o custo de formatação no
`perf.rs`.

**5b. Inferir tipo pela forma da string** (`"true"`→bool, `"42"`→número).
Rejeitada em definitivo: destrói strings legítimas (CEP `"007"`, id `"123"`, o
texto `"true"`) e não se desfaz.

**5c. Esquema de `ctx` declarado** (um bloco que nomeia as chaves e seus tipos, para
validar escrita e dar tipo ao Luau estrito). Interessante, mas é feature à parte e
depende do `.luaurc` deixar de anular os tipos de `ctx`. Só depois da Fase 4.

**5d. Tipar `{x}` no markup** (ex.: `if="{n > 3}"` com número de verdade). O
avaliador compara strings por desenho; não faz parte deste plano.

**5e. Atualização parcial do contexto** (diff por chave em vez de reavaliar tudo).
Ortogonal; só se a medição da Fase 2 apontar o re-encode como gargalo.

### Depois de cada release: adaptar o rustploy-gui

Regra: **publicar a glacier-ui primeiro, adaptar o app depois.** O app depende da
versão publicada (`glacier-ui = { version = "0.132.0", features = ["tray"] }` em
`../rustploy/rustploy-gui/Cargo.toml:35`), então ele só consegue testar a mudança
depois do `cargo publish`. Cada fase que publica gera um passo de adaptação; o app
é um repositório à parte (`../rustploy/rustploy-gui`) e as mudanças lá ficam em
commit próprio, só quando o usuário pedir.

**Levantamento já feito (contra a 0.132.0):**

- **MicroPython:** o app não usa (nenhum `lang="python"`, nenhum `.py`, sem a
  feature). A Fase 0 só pede o bump de versão.
- **Escritas de `"true"`/`"false"` como string:** ~82 em `views/`. Seguem
  funcionando — string gravada como string continua string. Não é preciso mexer,
  mas na Fase 3 vale trocar por `true`/`false` onde o valor é lógico.
- **Comparações `== "true"`/`== "false"`:** ~80 no total. Dois grupos:
  - em `.gvb` (`if @x == "true"`, ex.: `new_service.gvb:101`): é **markup**, o motor
    compara strings. **Não muda.**
  - em `.luau` (concentrados em `scripts/handlers/settings.luau` — 13, além de
    `stream`, `services`, `bundle_import`, `projects`, `wizard`, `docker`): este é o
    grupo que **quebra** quando a chave passa a vir tipada. Exemplo real:
    `settings.luau:337` `local prune = ctx.manifest_prune == "true"` — se
    `manifest_prune` for escrita por um `checkbox` (Fase 3), passa a ser `boolean` e
    a comparação vira `false` **em silêncio**. Para cada ocorrência: descobrir quem
    escreve a chave (checkbox → `boolean` após a Fase 3; script → o que o script
    gravou) e trocar por `ctx.x == true` ou simplesmente `ctx.x`.
- **Widgets que gravam valor:** ~20 `checkbox(`, 8 `spinbox(`, 6 `radiogroup`, 6
  `tabbar`, `tabs`, `drawer`. Os `checkbox` e `spinbox` são os afetados pela Fase 3
  (`Bool` e `Number`); `radiogroup` e `tabbar` precisam ser confirmados contra a
  tabela de widgets da Fase 3 antes da adaptação.
- **`tonumber(ctx…)`:** 5 usos — continuam corretos (`tonumber(5)` é `5`); trocar
  por uso direto só onde ficar mais limpo.
- **`json.decode(ctx…)`:** 1 uso — após a Fase 2, vira acesso direto **se** a chave
  tiver sido gravada como tabela; se foi gravada com `json.encode`, continua string
  e o `json.decode` permanece.
- **Testes Rust:** `tests/templates_render.rs` (51 KB) lê o contexto por
  `ctx.get(k)`, que continua devolvendo `&String` — ex.:
  `ctx.get("connected") == Some("false")` segue passando. Revisar apenas os
  testes de widgets migrados na Fase 3 e, se houver, os que montam `initial_ctx`.
- **Docs do app:** `AGENTS.md` (68 KB), `README.md` e `CONTINUE.md` repetem o
  modelo "tudo é texto"; atualizar junto com o `AGENTS.md` do template da CLI
  (Fase 4).

**Passo a passo, por release:**

1. Publicar a glacier-ui (e a `glacier-cli`, se mudou) e esperar o índice do
   crates.io aceitar a versão.
2. No rustploy-gui: subir `glacier-ui` no `Cargo.toml` para a versão nova e
   regenerar o `Cargo.lock`.
3. Rodar a lista acima (`rg '== "(true|false)"' views/scripts`, widgets
   `checkbox`/`spinbox`) e corrigir; o aviso de carga da Fase 1 aponta os casos que
   o `rg` não pegar.
4. Rodar os testes do app (`tests/`), `glacier test` e subir a GUI
   (`WGPU_BACKEND=gl`, e o Xvfb descrito no `AGENTS.md` do app, para headless) para
   olhar as telas de `settings`, `new_service` e `docker`, que concentram as
   comparações.
5. Só então propor o commit do app, citando a versão da glacier-ui que o exige.

**Ordem sugerida:** o app pode ficar em versões antigas durante as Fases 0 e 2 sem
quebrar nada além do próprio bump; a adaptação que realmente exige trabalho é a da
**Fase 3** (e, em menor grau, a da Fase 1 se algum script gravar booleano Luau e
comparar com `"true"`). Para não deixar o app quebrado entre uma fase e outra, o
bump do app da Fase 3 e a adaptação dos `.luau` entram no **mesmo commit**.

---

## 4. Riscos

| risco | mitigação |
|---|---|
| Quebra silenciosa de `== "true"` | lint de carga, `Quebras` no CHANGELOG, válvula temporária, migração dos exemplos |
| Regresso de desempenho em listas grandes | cache de snapshot; medição obrigatória na Fase 2 |
| Tag obsoleta devolvendo tipo errado | validação por snapshot; teste que sobrescreve a chave por widget sem tag |
| Widget migrado que o script lia como string | tabela de levantamento revisada widget a widget; teste de árvore avaliada |
| `lang="python"` rodar como Luau depois da Fase 0 | erro explícito para qualquer `lang` desconhecido |
| rustploy-gui quebra em silêncio após a Fase 3 (`ctx.x == "true"` em `settings.luau` e afins) | levantamento acima; adaptar o app no mesmo commit do bump; rodar as telas de `settings`/`new_service`/`docker` |
| Arquivos órfãos do MicroPython no pacote | conferir `include`/`exclude` do `Cargo.toml` e o `cargo publish --dry-run` |

## 5. Verificação (restrições desta máquina)

- Não usar `cargo test` aqui (memória `sem-cargo-test`): verificar com
  `cargo publish --dry-run` e, para módulos isolados, `rustc --test` num
  scratchpad (memória `cli-glacier-jsonc-e-verificacao`).
- Exemplos: `WGPU_BACKEND=gl cargo run --example <nome>` — o Vulkan da GPU
  integrada está quebrado.
- Cada fase fecha com: teste que olha a **árvore avaliada** (não só escreve/relê),
  exemplo rodando e CHANGELOG com **Quebras**.

## 6. Decisões em aberto

1. **Válvula de escape.** `<script ctx_types="off">` por script (recomendado,
   remove em 1 release) ou nenhuma, apoiando-se só no lint e nas **Quebras**?
   A convenção do projeto aceita quebra em minor; a válvula só suaviza a
   migração dos apps que já existem.
2. **Fases 1–3 em releases separados** (recomendado, cada um reversível e
   medido) ou um release único? O custo de juntar é perder o ponto de bisect
   quando o desempenho ou um widget regredir.
3. **Contexto inicial de janela filha e `broadcast`** (Fase 4) carregam tag?
   Recomendação: sim, para `Bool`/`Number`.
