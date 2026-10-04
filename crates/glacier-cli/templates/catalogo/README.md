# {{titulo}}

Um **catálogo navegável de widgets** do [glacier-ui](https://crates.io/crates/glacier-ui):
uma barra lateral de categorias, cada uma abrindo uma tela com dezenas de
widgets do motor — cada um com um exemplo mínimo e vivo, feito para ser copiado
para o seu `.gvb`.

```
cargo run
```

## O mapa

```
src/main.rs                      a casca: sobe o runner, registra views/app.gvb
views/
├── app.gvb                       a JANELA: <drawer> de categorias + roteador (um <Categoria> por rota)
├── components/nav_item.gvb       o item da gaveta lateral
├── categorias/                  UMA tela por seção da tabela de widgets
│   ├── botoes.gvb                button, toolbutton, radio, checkbox, toggle, buttonbox, …
│   ├── texto.gvb                 textinput, textarea, maskedinput, comboedit, autocomplete, form, …
│   ├── numericos.gvb             spinbox, slider, rangeslider, dial, gauge, lcdnumber, rating, …
│   ├── selecao.gvb               select, listview, tableview, treeview, pagination, fontselect, …
│   ├── data_hora.gvb             calendar, monthyearpicker, daterangepicker, dateedit, timeedit, …
│   ├── displays.gvb              badge, card, avatar, chip, frame, skeleton, qrcode, canvas (+ path/polygon), …
│   ├── containers.gvb            groupbox, grid, flow, space, splitter, accordion, toolbox, reveal, mdiarea, dock
│   ├── navegacao.gvb             tabbar, tabs, stackview, wizard, wizardnav, swipeview, drawer
│   ├── barras.gvb                menubar, contextmenu, toolbar, statusbar, sizegrip
│   ├── overlays.gvb              tooltip=, whats_this=, popover, popup, stack, notificationdot, splashscreen, rubberband, …
│   ├── graficos.gvb              linechart (+ série múltipla), barchart, piechart/donut, sparkline
│   └── dialogos.gvb              <dialog> declarativo + confirm{}/prompt{}/pick_color{}/toast()
├── scripts/
│   ├── app.luau                 init(): SEMEIA toda chave que os demos leem
│   └── handlers/                demo.luau (ir/dizer/def) + dialogos.luau (os modais)
└── styles/
    ├── theme.json               as cores base do tema
    └── app.gss                  a paleta :root + as classes de papel dos demos
```

## Como funciona

- **Um registro só.** `src/main.rs` registra `views/app.gvb`; cada `categorias/*.gvb`
  entra por `<link rel="import">` e é carregado em cascata.
- **Sem escada de `se`.** `app.gvb` guarda a categoria atual em `{view}` e o
  roteador é uma linha por tela: `<Botoes if="{view}" equals="botoes" />`.
- **A barra lateral é um `<drawer>`.** Ele empurra o conteúdo (não cobre); o
  botão `☰` faz `drawer::toggle:menu`, e `init()` semeia `menu = "true"` para
  ela começar aberta.
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

Com o app aberto, salve qualquer `.gvb`, `.gss` ou `.luau`: o motor relê e
redesenha. Só `src/main.rs` exige recompilar.
