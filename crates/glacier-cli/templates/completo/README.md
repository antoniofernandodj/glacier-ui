# {{titulo}}

Aplicação desktop com [glacier-ui](https://crates.io/crates/glacier-ui): a
interface é descrita em blocos (`.gvb`, ou `.gva` em XML), o estilo num `.gss` CSS-like e o
comportamento em [Luau](https://luau.org) interpretado em runtime.

```
cargo run
```

## O mapa

```
src/main.rs                     a casca: só sobe o runner
views/
├── app.gvb                      a JANELA: <screen>, titlebar própria, sidebar, roteador
├── home.gvb                     rota "home"   — <component>, cards, for-each, if/else
├── sobre.gvb                    rota "sobre"  — <component>
├── components/
│   ├── nav_item.gvb             item de sidebar, com contrato em <props>
│   └── stat_card.gvb            cartão de número, com prop opcional
├── scripts/
│   ├── app.luau                entrada: carrega os handlers e define init()
│   ├── state.luau              estado mutável compartilhado
│   ├── handlers/nav.luau       navegação
│   ├── handlers/dados.luau     dados, mutação e fetch
│   └── glacier.d.luau          tipos dos globais do motor (só para o luau-lsp)
└── styles/
    ├── theme.json              tema do iced (cores base dos widgets)
    └── app.gss                 tokens :root + as classes compartilhadas
```

## Como as peças se ligam

- **Um registro só.** O runner abre `views/app.gvb` sozinho; os outros templates
  entram por `<link rel="import">` e são carregados em cascata.
- **Um script só.** O contexto do motor é global, então `views/app.gvb` é o único
  template com `<script>`: os `on_click` dos templates importados resolvem para
  as funções globais declaradas em `views/scripts/`.
- **Duas memórias.** `ctx` guarda strings — é o que os `{marcadores}` do markup
  leem. `state.luau` guarda as estruturas de verdade; `Dados.publicar()` copia
  uma na outra.
- **Onde fica cada estilo.** Classe usada por mais de um template vai no
  `app.gss`; classe de um template só vai no `<style scoped="true">` dele.

## Caminhos: cada um resolve de um jeito

| Onde | Relativo a |
|---|---|
| `<link rel="theme">`, `<link rel="stylesheet">` | o diretório de onde o app roda (a raiz do projeto) |
| `<link rel="import">` | o próprio `.gvb` que importa |
| `<script src="…">` | o próprio `.gvb` |
| `require("…")` no Luau | o arquivo `.luau` que chama |

## O `app`: a raiz do arquivo

`src/main.rs` só sobe o runner: sem `.main`, ele abre `views/app.gvb`, cuja raiz
é o `app(...)`. O que descreve a janela e o aplicativo mora nele:

```
app(id = …, remember_geometry = true, size = "980 640", min_size = "560 420",
    decorations = false) {
  resources { … }          // tema, folhas .gss, componentes globais
  screen(name = app, initial = true, title = "…") { … }
}
```

- **`size`, `min_size`, `decorations`** são da janela principal, e ficam no `app`.
  O **`title`** é da `screen` e acompanha a navegação (recarrega a quente).
- **`decorations = false`** troca a titlebar do SO pela que o template desenha.
  As ações `window:*` do `app.gvb` são built-in do motor: não há handler para
  elas no Luau.
- **`id`** dá nome ao diretório de dados (`~/.local/share/<id>`,
  `%APPDATA%\<id>`), a raiz gravável do global `storage` do Luau — fora dos
  assets, read-only num app instalado.
- **`remember_geometry = true`** grava tamanho/posição ao fechar e restaura ao
  abrir. No Wayland só o tamanho volta. O pedido de fechar passa pelo runner
  sozinho, para a geometria ser salva antes de a janela sumir.
- Uma janela aberta por `open_window("tela", { size = "…" })` abre uma **tela do
  app**; o tamanho vai na chamada, não no markup da tela.

## A janela sem decoração

O motor não empilha camadas, então a moldura de 5px que dá as alças de
redimensionar é montada como **linha de cima / faixa do meio / linha de baixo**.
Cada alça declara o próprio `cursor` e a direção (`window:resize:nw`, `:n`, …).

## Templates: `app`, `screen` e `component`

Todo `.gvb` começa com uma casca que envolve o arquivo inteiro:

- **`app`** é a raiz do manifesto: o aplicativo (`id`, janela principal, `tray`,
  `resources` global) e as telas dele.
- **`screen`** é uma **tela**: sob o `app` aceita `name`, `initial`, `title` e
  `src`, e é só conteúdo — `size`, `icon` e o resto da janela são do `app`. Um
  arquivo avulso com raiz `screen` (o que o `src` aponta) é uma tela válida.
- **`component`** é um pedaço de tela (o que um `import` traz) e **não aceita
  atributo nenhum**.

Dentro da casca, `resources` guarda o que ela precisa (estilos, scripts, `link`)
e o resto é o layout.

## `<props>`: o contrato de um componente

`components/nav_item.gvb` e `components/stat_card.gvb` declaram as props que
aceitam, com default para as opcionais. Uma prop ausente e sem default vale
string vazia — por isso `destaque` (default `""`) nunca casa com `"1"` quando
não é passada.

O `nav_item` recebe a rota atual de fora (`ativo="{view}"`) em vez de ler o
contexto global por conta própria: ele só compara o que recebeu.

## Controle de fluxo

`if`/`else` e `for-each` são **atributos**, aplicáveis a qualquer elemento — não
é preciso uma tag invólucro para condicionar um nó só:

```xml
<text class="tag_ok"  if="{i.ativo}" equals="1">ativo</text>
<text class="tag_off" else>parado</text>
```

`for-each` itera um **array JSON** guardado no contexto (daí o `json.encode` em
`handlers/dados.luau`); `var` nomeia a variável de cada volta, e um objeto vira
`{i.campo}`.

## Luau: a convenção de `require`

`require` resolve relativo ao **arquivo que chama**, como em Node:

- irmão no mesmo diretório → nome nu: `require("dados")`
- pacote pai → prefixo `../`: `require("../state")`
- do script de entrada → caminho: `require("handlers/dados")`

O `app.luau` não define lógica própria: ele carrega cada pacote de `handlers/`
para que as funções globais deles estejam registradas antes de qualquer clique.
`init()` é chamada pelo motor quando a tela entra.

Os handlers parecem não usados para o linter (nada os chama de dentro do Luau) —
daí o `--!nolint FunctionUnused` no topo.

`require` é cacheado pelo **caminho resolvido**, então todos os handlers recebem
a mesma tabela de `state.luau`: o que um muda, os outros veem.

## `fetch` não trava a UI

`fetch` **suspende a corrotina** da ação e retoma quando a resposta chega. A
janela continua respondendo, e o código segue linear, com cara de `await`:

```lua
ctx.status = "buscando..."   -- já aparece na tela, antes da suspensão
local res = fetch("https://api.ipify.org?format=json")
```

O retorno é `{ ok, status, body, error }`.

## Estilo

Precedência do mais fraco ao mais forte: **tag < classe < id < inline**; num
`class="a b"`, `b` sobrepõe `a`.

Os seletores se combinam como no CSS: `row.servico > text.nome` (composto e
filho direto), `.lista text` (descendente), `*`. Um composto fica logo acima do
seu tier na escada (`row.x` vence `.x`), e `+`, `~` e `[attr]` são erro de
parse. O `AGENTS.md` tem a gramática inteira.

Os tokens em `:root` são a fonte única da paleta, e `var()` atravessa
stylesheets — os mesmos nomes resolvem dentro dos `<style scoped>` dos
templates. O `theme.json` é outra coisa: é o tema do iced, as cores base dos
widgets.

Num `Button`, `color` é o **fundo** e `text-color` é o texto.

`@media` casa contra o tamanho da janela em px lógicos. O motor **não reflui
`Row` para `Column`**, então em janelas estreitas o caminho é esconder o cromo
de menor prioridade (`hidden: true`) em vez de tentar recolocá-lo.

## Hot-reload

Com o app aberto, salve qualquer `.gvb`, `.gss` ou `.luau`: o motor relê e
redesenha. Só `src/main.rs` exige recompilar — e ele quase não muda.

## Tipos no editor

O `.luaurc` declara os globais que o motor injeta, e
`views/scripts/glacier.d.luau` os tipa. Com o
[luau-lsp](https://github.com/JohnnyMorganz/luau-lsp):

```
luau-lsp analyze --definitions=views/scripts/glacier.d.luau views/scripts
```

Para realce e ir-para-definição nos `.gvb`/`.gss`, instale as extensões de VS
Code com `glacier install-extensions`.
