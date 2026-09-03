# {{titulo}}

Aplicação desktop multi-janela com
[glacier-ui](https://crates.io/crates/glacier-ui), com ícone de bandeja.

```
cargo run
```

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

## O modelo

Cada janela é um **motor Glacier independente**: contexto e estado isolados,
hot-reload próprio, e fechar uma não afeta as outras. Elas se coordenam por três
funções, nenhuma delas implícita:

| Função | O que faz |
|---|---|
| `open_window({ file = …, data = … })` | abre uma janela; `data` semeia o contexto dela **antes** do `init` |
| `broadcast(evento, payload)` | manda uma mensagem para as OUTRAS janelas (nunca para a própria) |
| `close_window()` | fecha a própria janela |

A janela receptora trata em `function on_broadcast(evento, payload)`, com o
payload já decodificado numa tabela.

`close_window()` existe porque o motor isolado não conhece o próprio id de
janela — quem fecha é o daemon. O par `broadcast` + `close_window` é o padrão
"janela auxiliar devolve um resultado e some".

`open_window` não precisa repetir título e tamanho: o `<screen>` do arquivo os
declara. A chamada só os sobrepõe quando sabe algo que o arquivo não sabe.

O contador do `detalhe.gv` prova o isolamento: abra duas filhas e compare.

## A bandeja

Com a feature `tray` ligada (ver `Cargo.toml`), **fechar a última janela não
encerra o app**: ele se recolhe para a bandeja, e o menu passa a controlar o
ciclo de vida.

Daí a necessidade de `single_instance`: sem ele, clicar no lançador de novo
abriria um segundo processo enquanto o primeiro segue vivo e invisível. A
segunda tentativa pinga a primeira e sai; a instância viva reabre e foca a
janela principal.

Os `id` dos itens do menu (`abrir`, `notificacoes`, `sair`) são o que chega ao
gancho `on_tray`. `abrir` e `sair` são ações do runner; `notificacoes` alterna o
interruptor global do SO e reflete o novo estado no rótulo do próprio item — o
menu é a única superfície onde esse estado aparece. O rótulo começa em
"Desligar" porque as notificações começam ligadas.

No Linux a bandeja usa libappindicator + GTK em runtime — num pacote `.deb`,
declare `libgtk-3-0` e `libayatana-appindicator3-1` nas dependências. macOS não
é suportado pela bandeja (exige a thread principal).

## `toast` e `notify`

`toast` é efêmero e vive **dentro** da janela; `notify` é a notificação nativa
do SO, que sai pelo caminho do sistema e sobrevive à janela fechada.

## Persistência

`remember_window_geometry(true)` grava tamanho e posição ao fechar e restaura ao
abrir. No Wayland só o tamanho volta.

`storage_dir` é a raiz gravável do global `storage` do Luau. Sem ele, o
`storage` gravaria relativo aos assets — que num app instalado costuma ser
read-only.
