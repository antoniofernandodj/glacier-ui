# {{titulo}}

Aplicação desktop multi-janela com
[glacier-ui](https://crates.io/crates/glacier-ui), com ícone de bandeja.

```
cargo run
```

## O modelo

Cada janela é um **motor Glacier independente**: contexto e estado isolados,
hot-reload próprio, e fechar uma não afeta as outras. Elas se coordenam por três
funções, nenhuma delas implícita:

| Função | O que faz |
|---|---|
| `open_window({ file = ..., data = ... })` | abre uma janela; `data` semeia o contexto dela antes do `init` |
| `broadcast(evento, payload)` | manda uma mensagem para as OUTRAS janelas (nunca para a própria) |
| `close_window()` | fecha a própria janela |

A janela receptora trata em `function on_broadcast(evento, payload)`, com o
payload já decodificado.

## A bandeja

Com a feature `tray` ligada (ver `Cargo.toml`), **fechar a última janela não
encerra o app**: ele se recolhe para a bandeja, e o menu passa a controlar o
ciclo de vida. Daí a necessidade de `single_instance`: sem ele, clicar no
lançador de novo abriria um segundo processo enquanto o primeiro segue vivo e
invisível.

No Linux a bandeja usa libappindicator + GTK em runtime — num pacote `.deb`,
declare `libgtk-3-0` e `libayatana-appindicator3-1` nas dependências. macOS não
é suportado pela bandeja (exige a thread principal).

## O mapa

```
src/main.rs                     runner, menu da bandeja, instância única
assets/icone.png                ícone da janela e da bandeja (embutido no binário)
views/
├── painel.gv                   janela principal
├── detalhe.gv                  janela filha (aberta por open_window)
├── scripts/
│   ├── painel.luau             abrir filha, notificar, receber broadcast
│   ├── detalhe.luau            contador próprio, broadcast + close_window
│   └── glacier.d.luau          tipos dos globais do motor (só para o luau-lsp)
└── styles/app.gss              tokens :root + classes das duas janelas
```
