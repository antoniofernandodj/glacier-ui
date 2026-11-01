# Plano: checar o render pelo Luau, via a API da tela

> **ARQUIVADO — CONCLUÍDO** (2026-10-25). Mantido como registro histórico.

**Estado:** **concluído** (2026-10-22). Fases 1, 2 e 3 saíram na glacier-ui 0.133.0 e o
`templates_render.rs` do `rustploy-gui` foi migrado para `tests/tela/render.test.luau`
(commit `7e95b02`). Escrito a partir de um experimento no `rustploy-gui` (seção 1); as perguntas
da seção 6 estão respondidas lá. Dois itens ficaram de fora de propósito — ver "Fora do escopo".

## 1. O problema

O `glacier test` testa **lógica**: chama handlers, lê o contexto, confere toasts. O `rustploy-gui`
ainda mantém testes em **Rust** (`tests/templates_render.rs`, 19 testes) só para o que o Luau não
enxerga: se as telas sobem e como ficam na tela. Isso obriga o projeto a ter dois runners, e o
`glacier test` ainda compila o crate inteiro (`cargo run` em modo de teste), então nem o custo de
CI é menor do que o do `cargo test`.

O experimento que mostra a lacuna (2026-10-22, `rustploy-gui`): quebrar um arquivo de view de
propósito e rodar os dois runners.

| Quebra injetada | `glacier test` | `cargo test --test templates_render` |
|---|---|---|
| `<tagquebrada` solta no fim de `views/app.gv` | passou (60 testes) | **falhou** |
| propriedade inexistente em `views/styles/app.gss` | passou | passou |

Por que o Luau não pega: `component("views/app.gv")` só carrega as views que o teste chama, e o
objeto `tela` expõe apenas `call`, `get`, `set`, `data`, `missing` e `toasts` (ver
`src/luau/testing.rs`, `impl UserData for TestComponent`). Ele olha o **estado**, nunca a **árvore
avaliada**. O Rust consegue porque chama `GlacierUI::register_app` e percorre os `UiNode`.

O `tela:missing()` já é um primeiro passo: avalia o template contra os dados e devolve as chaves
que o markup lê e o contexto não tem. A proposta é generalizar isso — expor o resultado da
avaliação, não só o rastreamento de leituras.

## 2. O que os testes Rust de render conferem hoje

Lendo `rustploy-gui/tests/templates_render.rs`, são quatro tipos de coisa:

1. **Sobe sem erro.** Toda tela, aba e janela parseia e avalia (`all_screens_and_service_tabs_render`,
   `*_window_renders`).
2. **Texto e botões presentes.** Percorre a árvore atrás de um texto ou rótulo de botão
   (`textos_avaliados`, `botoes_avaliados`).
3. **Estrutura condicional.** Só a tela ativa é avaliada; o `if=` esconde o que deve esconder;
   `disconnect_clears_the_session_context`.
4. **Estilo resolvido.** A sidebar vira drawer, as ações viram ícones em largura estreita, o item
   de navegação aceso recolore os filhos por combinador, o grid usa `spread`.

Os tipos 1 a 3 só precisam da árvore. O tipo 4 precisa também do estilo resolvido e de uma largura
de janela.

## 3. A proposta

### 3.1 Carregar o app inteiro

```lua
function test_o_app_todo_carrega()
    assert_renders("views/app.gv")   -- parseia o app e todos os imports, avalia a tela inicial
end
```

Falha com a mensagem do parser (arquivo e linha) se qualquer view importada estiver quebrada,
mesmo uma que nenhum outro teste carrega. É a linha da tabela da seção 1 que hoje só o Rust pega, e
o item de **menor custo e maior valor** deste plano.

### 3.2 A árvore avaliada na `tela`

```lua
local tela = component("views/service_export_window.gv", { ... })
tela:call("init")

local arvore = tela:render()                 -- tabela aninhada
tela:texts()                                 -- { "Exportar serviço", "api", ... }, em ordem
tela:buttons()                               -- { "Cancelar", "Exportar" }
tela:find({ kind = "button", text = "Exportar" })   -- primeiro nó que casa, ou nil
tela:find_all({ class = "card" })
```

Um nó é `{ kind, text, class, attrs, children }`. `kind` é o nome da tag (`text`, `button`,
`row`…). `attrs` traz o que o autor escreveu (`id`, `disabled`, `on_press`), já com `{chave}`
resolvido contra o contexto. Os dados vêm do mesmo `UiNode` que o Rust percorre hoje, convertido
para tabelas Luau.

Asserções novas, no estilo das de `PLANO_TESTES.md`:

```lua
assert_has_text(tela, "Exportar serviço")
assert_has_button(tela, "Exportar")
assert_no_text(tela, "Erro")
```

### 3.3 Estilo resolvido e largura

```lua
tela:set_viewport(480, 800)         -- largura/altura lógicas; padrão: a do screen
local nav = tela:find({ id = "nav-projects" })
tela:style(nav)                     -- { color = "#fff", padding = 8, ... } já com @media e combinadores
```

`tela:style(no)` devolve as propriedades **depois** de aplicar seletor, combinador e `@media` para
o viewport atual. Isso cobre o tipo 4 da seção 2. É a parte mais cara (o resolvedor de estilo hoje
roda junto da construção dos widgets), então fica por último.

### 3.4 O que **não** entra

- Pixels, fontes e posição real. O Luau confere estrutura e propriedades resolvidas, não desenho. O
  smoke visual continua sendo outra coisa (Xvfb/Xephyr).
- Interação por clique de mouse. O teste dispara a ação pelo nome (`tela:call`), como hoje.

## 4. Fases

| Fase | Entrega | Substitui no rustploy-gui |
|---|---|---|
| 1 | `assert_renders(path)` | os testes `*_window_renders`, `all_screens_and_service_tabs_render` |
| 2 | `tela:render()`, `texts`, `buttons`, `find`, `find_all` + `assert_has_*` | os testes de texto, botão e `if=` (tipos 2 e 3) |
| 3 | `tela:set_viewport` e `tela:style` | sidebar drawer, ações em ícone, recolor por combinador (tipo 4) |

Cada fase se entrega e se usa sozinha. Só quando a fase 3 existir é que o `templates_render.rs`
pode sair do `rustploy-gui`; até lá ele fica (e o `embedded.rs` fica de qualquer jeito, porque
confere o empacotamento do binário, que não é render).

## 5. Custo no CI

Esta proposta **não** reduz o tempo de compilação: o `glacier test` continua rodando o crate em modo
de teste. O ganho é de **manutenção** (um runner só, mesma sintaxe, mensagem de falha com arquivo e
linha), não de minutos. Se o objetivo for CI mais curto, o caminho é outro: cache do `target/` e
compilar uma vez só para os dois runners.

## 6. Perguntas (respondidas na implementação)

**Como ficou:** `assert_renders`, `assert_has_text`/`assert_no_text`, `assert_has_button`/
`assert_no_button`, e em `tela`: `render`, `texts`, `buttons`, `find`, `find_all`,
`set_viewport`, `style`. Código em `src/luau/testing.rs` (`carregar`, `com_arvore`,
`no_para_lua`) e `testing.luau`. Não precisou do "resolvedor de estilo" separado que a seção 3.3
temia: a avaliação já resolve seletor, combinador e `@media` nos campos do `UiNode`, e o
`set_viewport` só reavalia com outro viewport.

Desvio do desenho: o avaliador **zera `class` e `id`** dos nós (já foram resolvidos nos campos de
estilo), então `find({ class = … })` não tinha o que olhar. Um interruptor por thread
(`eval::manter_identidade`, ligado só pelo runner de testes) os devolve; fora dos testes o custo é a
leitura de um `thread_local` por nó. Limites conhecidos: o `text` de um nó só existe para `text`,
`button`, `checkbox` e `toggle`; `attrs` cobre `button`, `textinput`, `textarea`, `checkbox`,
`toggle`, `image`/`svg` e `progressbar`; `class` de um componente aparece na raiz dele só pela
classe do template, não pela escrita no uso. Cada chamada reavalia a tela do zero.

1. **Propriedade GSS desconhecida.** Já **não** derruba a folha: o motor a ignora e avisa em
   `stderr` (`propriedade GSS desconhecida 'border-bottom' em '.botao' (ignorada)`). O
   `assert_renders` não a trata como falha (o aviso não chega ao teste). Se for desejável, o caminho
   é capturar esses avisos durante a carga e expô-los — fica como melhoria.
2. **Formato do nó.** `attrs` plano (`no.attrs.on_click`), com o estilo separado em `style`.
3. **Estabilidade.** O formato é novo e pode crescer (mais variantes em `attrs`/`text`); tratar
   como experimental até o `rustploy-gui` migrar.
4. **Janelas auxiliares.** Em aberto: `component(path)` carrega uma tela/app; uma janela aberta por
   `open_window` se testa carregando a tela dela (`component("views/x.gv")`). `tela:windows()`
   não foi feito.

### Migração do rustploy-gui (feita)

`rustploy-gui/tests/templates_render.rs` virou `tests/tela/render.test.luau` (commit `7e95b02`,
glacier-ui 0.133). O app hoje depende da glacier-ui 0.138.0. O `embedded.rs`/`app_manifest.rs`
ficam em Rust, porque conferem o empacotamento do binário, que não é render.

### Fora do escopo

- `tela:windows()` — uma janela aberta por `open_window` se testa carregando a tela dela.
- Expor ao teste os avisos de propriedade GSS desconhecida (hoje só vão para `stderr`).

Se algum dia fizerem falta, entram como melhoria nova, não como pendência deste plano.

### Perguntas originais

1. **Propriedade GSS desconhecida.** Nenhum runner a pegou no experimento. O comentário antigo do
   `templates_render.rs` diz que ela derruba a folha inteira em silêncio. Isso ainda vale? Se sim,
   `assert_renders` deveria falhar nesse caso (ou o motor deveria reportar aviso).
2. **Formato do nó.** `attrs` plano ou separado por tipo? Plano é mais simples de escrever
   (`no.attrs.disabled`), mas perde a distinção entre atributo escrito e atributo herdado.
3. **Estabilidade.** A árvore avaliada é detalhe interno do motor. Expô-la congela um formato.
   Vale marcar `render()` como experimental até a fase 3 fechar.
4. **Janelas auxiliares.** Cada uma é um motor isolado (`open_window`). `tela:render()` deveria
   funcionar por janela, e `tela:windows()` listar as abertas?
