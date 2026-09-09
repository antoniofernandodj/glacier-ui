# Plano — Extensões VS Code (Glacier)

Suporte de editor para as duas linguagens do glacier-ui. Instalação **local**
apenas (publicação no Marketplace abandonada — burocracia de publisher/PAT do
Azure, ver seção final).

## Estado atual

### `editors/vscode/` — Glacier GSS (`.gss`) — v0.1
- Realce de sintaxe específico do GSS (espelha `src/stylesheet.rs`):
  seletores `.classe` e pseudo-estados, `:root`/`var()`, `@media`, propriedades
  conhecidas vs. typos, cores hex, keywords de valor, comentários `//` e `/* */`.
- Snippets: `class`, `hover`, `root`, `media`, `var`, `card`.
- Ícone de arquivo estilo CSS.
- Verificado: `vsce package` + tokenização real (vscode-textmate) 12/12 escopos.

### `editors/vscode-gv/` — Glacier View (`.gv`) — v0.2
- Realce: tags (componente vs. primitiva), atributos (ações destacadas),
  interpolação `{var|default}`, cores; **Lua embutido** em `<script>` e
  **GSS embutido** em `<style>`.
- **DocumentLink + Go to Definition** — toda referência a arquivo fica
  sublinhada (Ctrl/Cmd+Click) e responde a F12:
  - `<script src="app.luau">` → o arquivo `.luau`.
  - `on_click="fn"` → `function fn()` no `.luau` externo ou no `<script>` inline;
    sem `<script>`, o braço `"fn" =>` (ou `if action == "fn"`,
    `strip_prefix("fn:")`) do `Component::update` em Rust do bloco `impl` que
    declara **este** template.
  - `onToggle="escolher_tipo:roadmap"` → a convenção `nome:sufixo` do
    `run_inner`: tenta o valor inteiro e depois só o nome, sublinhando só a
    metade que o motor chama.
  - **Ações built-in** do `GlacierUI::dispatch` são categoria à parte, nunca
    tratadas como função: `clipboard:<chave>`, `open:<chave>`,
    `textarea_end:<b>`, `textarea_top:<b>` linkam o **sufixo** para onde a
    chave de contexto é escrita (`ctx.chave = …` no Lua, `ctx.set("chave", …)`/
    `define_data` no Rust do componente que renderiza este template);
    `window:*` e `style:*` linkam para a tabela de ações built-in da
    referência, assim como uma chave que não é escrita em lugar nenhum.
  - `<link rel="stylesheet" href>` / `<style href>` → a folha `.gss`;
    `rel="theme"`/`rel="data"` → o JSON.
  - `<link rel="import">`, `<import from>`, `<Include src>` → o template
    importado; `<Image src>`/`<Svg src>` → o asset.
  - **Variáveis de contexto** → onde a chave é escrita, **script primeiro**:
    `{chave}`/`{chave|default}` em qualquer lugar da markup (fora de
    `<script>`/`<style>`/comentário) e os atributos que bindam contexto
    (`value`, `checked`, `items`/`options`, `cond`, `for-each`, …). A cadeia,
    na ordem: `ctx.chave = …`/`ctx["chave"] = …` no `<script>` do template →
    nos módulos que ele `require` (transitivo, resolvido como
    `luau::resolve_module`: `dir`, `dir/lib`, `GLACIER_LUAU_PATH`, `.luau`
    antes de `.lua`, `x.luau` antes de `x/init.luau`) → `ctx.set`/`define_data`
    no Rust que renderiza este template → qualquer outro script do workspace
    (um app, um contexto: uma tela lê o que a irmã escreveu). Chave que ninguém
    escreve não vira link.
  - **Variável de laço** (`{c.titulo}` sob `for-each="…" var="c"`) → a
    declaração `var="c"` mais próxima acima; ela sombreia o contexto, então
    responde antes da cadeia acima.
  - `<Componente/>` e `navigateTo="tela"` → o `.gv`/`.xml` que o declara
    (`<import>` do próprio arquivo, `<import>` de qualquer template do
    workspace, `register_component("Nome","path")`/`nome:`+`template:` no Rust,
    ou convenção de nome snake_case).
  - Tag nativa/builtin → seção no doc de referência embutido
    (`references/glacier-view.md`), que cobre as 30 tags do `parser.rs` e a
    tabela de ações built-in.
- Caminho resolvido como o motor resolve: relativo ao arquivo declarante
  primeiro, depois à raiz do workspace. Referência que não resolve (arquivo
  inexistente, handler que não existe em lugar nenhum, caminho interpolado)
  **não vira link** — a ausência do sublinhado é a dica.
- Índice do workspace (nome→arquivo, handler Rust→arquivo) construído sob
  demanda, cacheado e invalidado por `FileSystemWatcher`; leitura de arquivo
  cacheada por mtime.
- Verificado: lógica do provider 31/31 (harness com stub de `vscode`, rodando
  contra os 34 templates do repo: 165 links, 0 apontando para lugar nenhum;
  e contra o `roadmapia`, um app 100% Luau: as 49 chaves referenciadas pelas
  3 telas resolvem, ~2,6 ms por chamada do provider) +
  gramática 7/7 escopos (vscode-textmate, incl. `meta.embedded.block.lua`).

## Roadmap

### Curto prazo
- [x] **Fechamento automático de tag** — terminar `<element>` com `>` insere
      `</element>` e deixa o cursor no meio. Fora as folhas (`<Image>`,
      `<Badge>`, `<Radio>`, …), que o markup escreve com `/>`. *(v0.9.0)*
- [ ] **Hover** — assinatura/props do widget nativo; corpo/1ª linha da função Lua
      referenciada por uma ação.
- [ ] **Diagnóstico** — sublinhar `on_click="x"` quando não existe `function x`
      (no `<script>` inline nem no `src`); e `<import from="…">` com caminho
      inexistente.
- [ ] **Completion** — tags nativas + builtins, atributos por tag, e nomes de
      ações já definidas no `<script>`.

### Médio prazo
- [x] **DocumentLink** visível (sublinhado) nos valores de ação e nos nomes de
      componente, além do go-to-definition. *(v0.2)*
- [x] **Resolução de componente mais forte** — indexar `register_component`/
      `<import>` do workspace num mapa nome→arquivo, em vez de varrer a cada
      chamada; cachear e invalidar em `onDidChange`. *(v0.2)*
- [ ] **GSS**: go-to-definition de `class="card"` no `.gv` → regra `.card` no
      `.gss` linkado; e de `var(--x)` → declaração em `:root`.
- [ ] **Migração `.xml` → `.gv`** — decidir se os templates viram `.gv` (o Rust
      referencia por caminho; renomear exige atualizar os `register_component`).

### Longo prazo
- [ ] **Unificar** as duas extensões numa só "Glacier UI" (um install, um
      Makefile, um publisher) — contribui as duas linguagens + providers.
- [ ] **Formatter** (`.gv` e `.gss`).
- [ ] **Preview** ao vivo da tela (reaproveitar o hot-reload do motor).

## Instalação (local)

Cada extensão tem um `Makefile`:

```bash
# Glacier View (.gv)
cd editors/vscode-gv && make install

# Glacier GSS (.gss)
cd editors/vscode && make install
```

`make reinstall` após editar gramática/JS; `make uninstall` para remover.
Requer `code` no PATH e `npx` (Node).

## Nota sobre publicação no Marketplace

Bloqueada do lado da Microsoft: criação de publisher retornou
"Publisher Metadata has suspicious content" e depois rate limit
`Count/VSID` (conta nova). Decisão: **ficar em instalação local** via `.vsix`.
Se retomar: `vsce login <publisher>` + `vsce publish` (precisa de publisher
registrado e PAT com escopo *Marketplace → Manage*).
