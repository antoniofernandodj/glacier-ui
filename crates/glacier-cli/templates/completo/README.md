# {{titulo}}

Aplicação desktop com [glacier-ui](https://crates.io/crates/glacier-ui): a
interface é descrita em XML (`.gv`), o estilo num `.gss` CSS-like e o
comportamento em [Luau](https://luau.org) interpretado em runtime.

```
cargo run
```

## O mapa

```
src/main.rs                     a casca: só sobe o runner
views/
├── app.gv                      a JANELA: <screen>, titlebar própria, sidebar, roteador
├── home.gv                     rota "home"   — <component>, cards, for-each, if/else
├── sobre.gv                    rota "sobre"  — <component>
├── components/
│   ├── nav_item.gv             item de sidebar, com contrato em <props>
│   └── stat_card.gv            cartão de número, com prop opcional
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

- **Um registro só.** O runner abre `views/app.gv` sozinho; os outros templates
  entram por `<link rel="import">` e são carregados em cascata.
- **Um script só.** O contexto do motor é global, então `views/app.gv` é o único
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
| `<link rel="import">` | o próprio `.gv` que importa |
| `<script src="…">` | o próprio `.gv` |
| `require("…")` no Luau | o arquivo `.luau` que chama |

## O cabeçalho da janela

`src/main.rs` só sobe o runner: sem `.main`, ele abre `views/app.gv`. O que
descreve a janela e o aplicativo mora no cabeçalho desse arquivo:

```xml
<screen title="…" size="980 640" min_size="560 420" decorations="false">
  <resources>
    <app id="…" remember_geometry="true" />
    …
```

- **`title`, `size`, `min_size`** ficam junto da tela que descrevem — e assim o
  título recarrega a quente.
- **`decorations="false"`** troca a titlebar do SO pela que o template desenha.
  As ações `window:*` do `app.gv` são built-in do motor: não há handler para
  elas no Luau.
- **`<app id>`** dá nome ao diretório de dados (`~/.local/share/<id>`,
  `%APPDATA%\<id>`), a raiz gravável do global `storage` do Luau — fora dos
  assets, read-only num app instalado.
- **`remember_geometry="true"`** grava tamanho/posição ao fechar e restaura ao
  abrir. No Wayland só o tamanho volta. O pedido de fechar passa pelo runner
  sozinho, para a geometria ser salva antes de a janela sumir.
- Uma janela aberta por `open_window(...)` declara a própria moldura no
  `<screen>` do arquivo dela (`decorations="false"`), como esta.

## A janela sem decoração

O motor não empilha camadas, então a moldura de 5px que dá as alças de
redimensionar é montada como **linha de cima / faixa do meio / linha de baixo**.
Cada alça declara o próprio `cursor` e a direção (`window:resize:nw`, `:n`, …).

## Templates: `<screen>` e `<component>`

Todo `.gv` começa com um cabeçalho que envolve o arquivo inteiro:

- **`<screen>`** é uma **janela** — aceita `title`, `size`, `min_size`,
  `max_size`, `fixed_size`, `resizable`, `decorations` e `icon`.
- **`<component>`** é um pedaço de tela (o que um `<import>` traz) e **não
  aceita atributo nenhum**: `title`/`size` não teriam a quem se aplicar ali, e
  escrevê-los é erro de parse em vez de um atributo ignorado em silêncio.

Dentro do cabeçalho, `<resources>` guarda o que a tela precisa (estilos,
scripts, `<link>`) e o resto é o layout.

## `<props>`: o contrato de um componente

`components/nav_item.gv` e `components/stat_card.gv` declaram as props que
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

Os tokens em `:root` são a fonte única da paleta, e `var()` atravessa
stylesheets — os mesmos nomes resolvem dentro dos `<style scoped>` dos
templates. O `theme.json` é outra coisa: é o tema do iced, as cores base dos
widgets.

Num `Button`, `color` é o **fundo** e `text-color` é o texto.

`@media` casa contra o tamanho da janela em px lógicos. O motor **não reflui
`Row` para `Column`**, então em janelas estreitas o caminho é esconder o cromo
de menor prioridade (`hidden: true`) em vez de tentar recolocá-lo.

## Hot-reload

Com o app aberto, salve qualquer `.gv`, `.gss` ou `.luau`: o motor relê e
redesenha. Só `src/main.rs` exige recompilar — e ele quase não muda.

## Tipos no editor

O `.luaurc` declara os globais que o motor injeta, e
`views/scripts/glacier.d.luau` os tipa. Com o
[luau-lsp](https://github.com/JohnnyMorganz/luau-lsp):

```
luau-lsp analyze --definitions=views/scripts/glacier.d.luau views/scripts
```

Para realce e ir-para-definição nos `.gv`/`.gss`, instale as extensões de VS
Code com `glacier install-extensions`.
