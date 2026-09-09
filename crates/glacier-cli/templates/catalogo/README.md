# {{titulo}}

Um **catálogo navegável de widgets** do [glacier-ui](https://crates.io/crates/glacier-ui):
uma barra lateral de categorias, cada uma abrindo uma tela com dezenas de
widgets do motor — cada um com um exemplo mínimo e vivo, feito para ser copiado
para o seu `.gv`.

```
cargo run
```

## O mapa

```
src/main.rs                      a casca: sobe o runner, registra views/app.gv
views/
├── app.gv                       a JANELA: sidebar + roteador (um <Categoria> por rota)
├── components/nav_item.gv       o item da barra lateral
├── categorias/                  UMA tela por seção da tabela de widgets
│   ├── botoes.gv                button, toolbutton, radio, checkbox, toggle, buttonbox, …
│   ├── texto.gv                 textinput, textarea, maskedinput, comboedit, autocomplete, …
│   ├── numericos.gv             spinbox, slider, rangeslider, dial, gauge, lcdnumber, rating, …
│   ├── selecao.gv               select, listview, tableview, treeview, pagination, fontselect, …
│   ├── data_hora.gv             calendar, monthyearpicker, daterangepicker, dateedit, timeedit, …
│   ├── displays.gv              badge, card, avatar, chip, frame, skeleton, qrcode, canvas, …
│   ├── containers.gv            groupbox, grid, flow, splitter, toolbox, accordion, mdiarea, dock
│   ├── navegacao.gv             tabbar, tabs, stackview, wizard, swipeview, drawer
│   ├── barras.gv                menubar, contextmenu, toolbar, statusbar, sizegrip
│   ├── overlays.gv              tooltip=, whats_this=, popover, popup, stack, rubberband, …
│   ├── graficos.gv              linechart (+ série múltipla), barchart, piechart/donut, sparkline
│   └── dialogos.gv              <dialog> declarativo + confirm{}/prompt{}/pick_color{}/toast()
├── scripts/
│   ├── app.luau                 init(): SEMEIA toda chave que os demos leem
│   └── handlers/                demo.luau (ir/dizer/def) + dialogos.luau (os modais)
└── styles/
    ├── theme.json               as cores base do tema
    └── app.gss                  a paleta :root + as classes de papel dos demos
```

## Como funciona

- **Um registro só.** `src/main.rs` registra `views/app.gv`; cada `categorias/*.gv`
  entra por `<link rel="import">` e é carregado em cascata.
- **Sem escada de `se`.** `app.gv` guarda a categoria atual em `{view}` e o
  roteador é uma linha por tela: `<Botoes if="{view}" equals="botoes" />`.
- **Todo widget grava numa chave.** O nome dela está no `value`/`checked`/
  `group`/`start`/`open` — nunca `{interpolado}`. `app.luau::init()` semeia
  todas com um valor plausível; um widget sem semente aparece vazio.
- **`on_change` é opcional.** Sem uma função Luau de mesmo nome, o motor grava
  `ctx[nome] = valor` sozinho — é por isso que a maioria dos demos não tem
  handler.

## Para o seu projeto

Copie o bloco `<column class="demo"> … </column>` do widget que você quer,
troque os nomes de chave (`sb_qtd` → `quantidade`) e semeie essas chaves no seu
`init()`. Apague o resto do catálogo.

A referência completa de cada tag — atributos, defaults, formato do JSON que ela
lê — está em `AGENTS.md`, seção *O catálogo de widgets*.

## Hot-reload

Com o app aberto, salve qualquer `.gv`, `.gss` ou `.luau`: o motor relê e
redesenha. Só `src/main.rs` exige recompilar.
