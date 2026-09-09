# {{titulo}}

Um **painel de dados** com [glacier-ui](https://crates.io/crates/glacier-ui):
KPIs e gráficos numa tela só, alimentados por um `every(1000, …)` no `<script>`.

```
cargo run
```

## O mapa

```
src/main.rs                 a casca: registra views/app.gv e abre a janela
views/
├── app.gv                  KPIs (<card>) + <linechart series>/<barchart>/<donut>/<gauge>/<sparkline>
├── scripts/app.luau        janelas rolantes por série; every(1000) as empurra e re-projeta no ctx
└── styles/
    ├── theme.json          cores base
    └── app.gss             layout + classes de papel
```

## A ideia

O `ctx` é a **vitrine**, não o depósito. As séries de verdade são arrays Luau
(`req_api`, `cpu`, …); a cada tique o `tick()` empurra um ponto novo, descarta o
mais antigo e re-encoda tudo em `ctx.serie_req`, `ctx.spark_cpu`, … Os widgets
leem essas chaves e redesenham — nenhum deles sabe que existe um `every`.

`<linechart series="serie_req">` recebe `[{ name, points }]` — várias linhas com
legenda e cores do ciclo do tema. `<barchart>`, `<piechart>`/`<donut>` seguem
série única. `<gauge bands="g_bands" needle="true">` pinta as faixas inteiras e
deixa a agulha apontar a leitura.

## Trocar a fonte dos dados

`tick()` hoje sorteia números. Troque o corpo dele por um `fetch(...)` (que
suspende sem travar a UI) ou por leituras de `/proc`, mantendo o mesmo
`publicar()` no fim.

## Hot-reload

Salve `app.gv`, `app.gss` ou `app.luau` com a janela aberta: aplica na hora.
