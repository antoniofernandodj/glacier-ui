# {{titulo}}

Aplicação desktop multi-janela com
[glacier-ui](https://crates.io/crates/glacier-ui), com ícone de bandeja.

```
cargo run
```

## O mapa

```
src/main.rs                     só diz qual template abre
views/
├── painel.gvb                   o app(...): janela principal, tray e as telas
├── assets/icone.png            ícone da janela e da bandeja
├── detalhe.gvb                  tela da janela filha (aberta por open_window)
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
| `open_window("detalhe", { size = "380 300", data = … })` | abre uma **tela do app** em outra janela; `data` semeia o contexto dela **antes** do `init` |
| `broadcast(evento, payload)` | manda uma mensagem para as OUTRAS janelas (nunca para a própria) |
| `close_window()` | fecha a própria janela |

A janela receptora trata em `function on_broadcast(evento, payload)`, com o
payload já decodificado numa tabela.

`close_window()` existe porque o motor isolado não conhece o próprio id de
janela — quem fecha é o daemon. O par `broadcast` + `close_window` é o padrão
"janela auxiliar devolve um resultado e some".

O nome do `open_window` é o de uma `screen` do `app` (`screen(name = detalhe,
src = "detalhe.gvb")`), e a janela nova herda dele o ícone e as folhas de estilo.
O **tamanho vai na chamada**: a tela é só conteúdo, e `size` num `screen` sob o
`app` é erro de parse. O `title` fica na tela.

O contador do `detalhe.gvb` prova o isolamento: abra duas filhas e compare.

## A bandeja

O ícone, o menu e o que cada item faz estão no `resources` do `app` de
`views/painel.gvb`:

```
app(id = …, single_instance = true, remember_geometry = true, size = "520 460") {
  resources {
    tray(icon = "views/assets/icone.png", tooltip = "…") {
      item(label = "Abrir …", on_click = tray:open)
      check(label = Notificações, checked = @__notifications, on_click = notifications:toggle)
      separator
      item(label = Sair, on_click = tray:quit)
    }
  }
  …
}
```

Com a feature `tray` ligada (ver `Cargo.toml`), **fechar a última janela não
encerra o app**: ele se recolhe para a bandeja, e o menu passa a controlar o
ciclo de vida.

Daí o `single_instance`: sem ele, clicar no lançador de novo abriria um segundo
processo enquanto o primeiro segue vivo e invisível. A segunda tentativa pinga a
primeira e sai; a instância viva reabre e foca a janela principal.

`tray:open`, `tray:quit` e `notifications:toggle` o runner trata sozinho — o
último alterna o interruptor global das notificações do SO, e `{__notifications}`
é o estado dele, que o `<check>` mostra. Qualquer outra ação num item vai para o
script da janela principal, cujo motor continua vivo com a janela recolhida; um
`label` ou `checked` com `{chave}` acompanha o contexto dela.

No Linux a bandeja usa libappindicator + GTK em runtime — num pacote `.deb`,
declare `libgtk-3-0` e `libayatana-appindicator3-1` nas dependências. macOS não
é suportado pela bandeja (exige a thread principal).

## `toast` e `notify`

`toast` é efêmero e vive **dentro** da janela; `notify` é a notificação nativa
do SO, que sai pelo caminho do sistema e sobrevive à janela fechada.

## Persistência

`remember_geometry = true` no `app` grava tamanho e posição ao fechar e
restaura ao abrir. No Wayland só o tamanho volta.

O `id` do `app` dá nome ao diretório de dados (`~/.local/share/<id>`,
`%APPDATA%\<id>`), onde a geometria e o global `storage` do Luau gravam — fora
dos assets, que num app instalado costumam ser read-only.
