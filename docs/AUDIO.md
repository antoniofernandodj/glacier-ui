# Áudio (`audio`)

Fase 6 do `PLANO_SUBSTITUIBILIDADE.md`, **só o áudio**: o `video` não foi feito (ver o fim).
Exemplo: `examples/audio` — `cargo run --example audio --features audio`.

## Ligar

O `audio` mora atrás da feature **`audio`** (desligada por padrão): ela traz o `rodio`
(MP3, WAV, OGG/Vorbis, FLAC), e o `cpal` por baixo precisa de `libasound2-dev` para
compilar no Linux. Quem não toca som não paga nada disso.

```toml
glacier-ui = { version = "0.146", features = ["audio"] }
```

Sem a feature o `audio` ainda existe: o player mostra `erro: feature `audio` desligada` e
o log avisa uma vez.

## No markup

```gv
audio(name = musica, src = "sons/tema.mp3")  // player: ▶/⏸, barra, tempo
audio(name = aviso, src = "sons/ding.wav", controls = false)  // só a fonte
audio(name = fundo, src = "sons/loop.ogg", autoplay = true, loop = true, volume = 0.3)
```

```
audio(name = musica, src = "sons/tema.mp3", on_ended = proxima)
```

| Atributo | O que faz |
|---|---|
| `src` | caminho do arquivo, **local** (relativo ao diretório de trabalho do processo) |
| `name` | identifica a fonte; sem ele vale o `src`. É o que as ações e o Luau usam |
| `controls` | `false` não desenha nada (só registra a fonte). Padrão: `true` |
| `autoplay` | começa a tocar ao carregar (uma vez por `name`+`src`) |
| `loop` | recomeça ao terminar (há um intervalo curto: o arquivo é reaberto) |
| `volume` | `0` a `1` (acima de `1` amplifica) |
| `on_ended` | ação chamada quando termina (não em `loop`) |
| `on_error` | ação chamada se o arquivo não abre/decodifica ou não há dispositivo |
| `width`, `spacing` | do player (largura padrão 360) |

O player é um botão, uma barra de posição (arrastar ou clicar faz *seek*) e `m:ss / m:ss`.
Um formato que não informa a duração (um MP3 sem cabeçalho de tamanho) mostra só o tempo
decorrido e a barra fica parada.

Registrar é **idempotente**: reavaliar a tela não reabre o arquivo. Mudar o `src` de um
`name` troca a fonte. `on_ended`/`on_error` recebem o namespace do componente, como qualquer ação.

## Controle de fora

Ações embutidas, em qualquer `on_click`: `audio:toggle:NOME`, `audio:play:NOME`,
`audio:pause:NOME`, `audio:stop:NOME`. (`audio:seek:NOME` e `audio:volume:NOME` levam o valor
e são o que a barra usa.)

Em Luau, o global `audio`:

```lua
audio.play("musica")   audio.pause("musica")   audio.toggle("musica")   audio.stop("musica")
audio.seek("musica", 30.5)          -- segundos
audio.volume("musica", 0.5)
audio.position("musica")            -- número, segundos
audio.duration("musica")            -- número ou nil
audio.playing("musica")             -- boolean
audio.error("musica")               -- string ou nil
audio.available()                   -- false se a feature `audio` está desligada
```

Em Rust: `glacier_ui::audio::{play, pause, toggle, stop, seek, set_volume, status}`.

## Como funciona

O dispositivo de saída e os `Player`s do `rodio` moram numa **thread própria**
(`glacier-audio`); a UI manda comandos por um canal e lê um retrato (`Status`) que a
thread publica a cada 50 ms. O dispositivo só é aberto no **primeiro som**. Enquanto algo
toca (e por 600 ms depois de um comando) o daemon assina um tick de 100 ms que redesenha a
barra e entrega `on_ended`/`on_error`; parado, não há tick nem custo.

## Medido

Numa janela real (Xvfb, ponteiro por XTEST, dispositivo de áudio real da máquina): o ▶ vira
⏸ e a barra anda (0:03 / 0:08 após 3 s); um clique na barra faz *seek*; no fim o ícone volta
a ▶ com 0:08 / 0:08 e o `on_ended` chega ao handler (contador `fins: 1`); tocar de novo depois do
fim recomeça do zero; pausar segura a posição. Tanto com o MP3 quanto com o WAV.

## O que NÃO está feito

- **`video` não foi feito.** A fase pedia `audio` primeiro; decodificar vídeo é outro
  projeto (decodificador, sincronia com áudio, textura por quadro) e a `webview` que servia de
  saída foi removida. Fica pendente no plano.
- **Só arquivo no disco.** Não lê de `embed_assets!` nem do `AssetSource`; o `src` é um
  caminho do sistema de arquivos. (O `rodio` decodifica de memória; falta ligar isso ao
  `AssetSource`.)
- **`loop` não é sem emenda** (reabre o arquivo ao terminar). Sem streaming por URL, sem fila,
  sem equalizador, sem `playbackRate`.
- **Vários windows:** o tick vai a todas as janelas, e o `on_ended` é entregue à primeira que
  o recebe. Um app multi-janela que precise do evento numa janela específica deve
  usar uma só janela dona do áudio.
- **Web (wasm):** sem som. O módulo existe sem a feature (o `audio` mostra o erro), mas ligar
  `audio` no alvo `wasm32` não é suportado — o `rodio` é dependência só do nativo. Nem este
  caminho foi compilado para wasm aqui.
- Verificado em **Linux** (ALSA/PipeWire). macOS e Windows usam o mesmo `cpal`, mas não foram
  rodados.
