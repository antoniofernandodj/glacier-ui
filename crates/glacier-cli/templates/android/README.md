# {{titulo}}

App Android com [glacier-ui](https://crates.io/crates/glacier-ui): o layout mora
em XML (`.gv`), o estilo num `.gss` e o comportamento num `<script>` Luau — o
mesmo código roda no desktop (onde se desenvolve) e no celular.

> **Preset experimental.** O glacier-ui ainda não foi adaptado ao Android: este
> projeto existe para ter o caminho de build pronto enquanto o motor é
> adaptado. A lista do que falta está em [`ANDROID_TODO.md`](ANDROID_TODO.md) —
> leia antes de esperar que uma API do motor funcione no aparelho.

## Começando

```
make setup      # uma vez: Java 17, Android SDK + NDK, alvos Rust, cargo-apk
make doctor     # confere o que falta
make run        # roda no DESKTOP, com hot-reload de .gv/.gss
make build      # APK de debug → target/debug/apk/{{nome_projeto}}.apk
make launch     # compila, instala no aparelho (USB, depuração ativa) e abre
```

No Windows não há `make`: `fazer setup`, `fazer doctor`, `fazer build`,
`fazer launch`… (`fazer` sem argumento lista tudo).

`make help` lista todos os alvos. Para emulador: `make setup-emulator`,
`make emulator`, e `make launch ABI=x86_64`.

## O mapa

| Arquivo | O que é |
|---|---|
| `src/lib.rs` | o app (`run()`) + `android_main`, o ponto de entrada do `NativeActivity` |
| `src/main.rs` | o desktop: chama o mesmo `run()` |
| `views/app.gv` | a tela: `<screen>` + `<resources>` + layout + `<script>` |
| `views/styles/` | `theme.json` (fundo da janela) e `app.gss` (paleta e classes) |
| `patches/iced_winit/` | o `iced_winit` 0.14.1 + patch de Android (ver o README de lá) |
| `Makefile` / `fazer.bat` | setup da máquina, build, instalação, logcat |
| `ANDROID_TODO.md` | o que ainda não funciona no Android |

## Como as peças se ligam

**No Android não há `views/` em disco.** O `.so` não tem diretório de trabalho
com arquivos, então `src/lib.rs` embute `views/` com `embed_assets!` (só no
alvo `android`). Consequências:

- **Arquivo novo em `views/` entra na lista do `embed_assets!`.** Um esquecido
  funciona no desktop e falha no celular, com "não está entre os assets
  embutidos" no `make logcat`.
- **Não há hot-reload no aparelho** — mudou o `.gv`, recompile (`make launch`).
  Por isso se desenvolve no desktop (`make run`) e se confere no aparelho.

**`patches/iced_winit` é obrigatório.** O `iced_winit` publicado não consegue
criar o `EventLoop` no Android (o winit exige o `AndroidApp`) nem compila o
teclado lá. O `[patch.crates-io]` do `Cargo.toml` troca por uma cópia com a
correção atrás de `cfg(target_os = "android")` — fora do Android ela é idêntica
à original, então o desktop não muda.

**O pacote tem `lib` e `bin`**, por isso o build usa `cargo apk build --lib`: o
APK carrega a `cdylib`.

## Depurar

`make logcat` segue o log (`log::info!` e o `println!`/pânicos do Rust). Um app
que abre e fecha na hora quase sempre deixou um pânico lá.

`make doctor` falha dizendo o que falta e qual alvo resolve.

## Chave de assinatura

`make release` assina com `release.keystore`, criada por `make keystore` com a
senha `{{nome_crate}}` (escrita no `Cargo.toml`, de propósito). **É uma chave de
teste** — está no `.gitignore` e não serve para publicar na Play Store.
