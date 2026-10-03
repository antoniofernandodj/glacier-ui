# Android — o que falta adaptar

O glacier-ui nasceu desktop-first. Este preset entrega o **caminho de build**
(setup, APK, instalação, logcat, o entry point e o patch do `iced_winit`); o
**motor** ainda não foi adaptado. Esta lista é o que se sabe que falta.

## Estado deste preset

- [x] Verificado na criação do preset: o projeto gerado passa no `cargo check`
      do **desktop**; o `patches/iced_winit` compila para
      `aarch64-linux-android`; `make doctor` e `make -n build` rodam.
- [ ] **Nunca rodou num aparelho.** O Makefile/`fazer.bat` e o patch foram
      escritos a partir de um projeto de referência (iced 0.13 + cargo-apk) e
      de um `cargo check --target aarch64-linux-android` do motor **sem NDK
      instalado**. O `make setup` e o `make build` deste preset **nunca foram
      executados**: o primeiro `make build` é o primeiro teste de verdade.
- [ ] `fazer.bat` (Windows) nunca foi executado.
- [ ] CI que gere o APK do preset — hoje nada impede o preset de quebrar.

Quando o `make build` falhar, o erro provavelmente está em um dos itens
**bloqueantes** abaixo.

## Bloqueantes: o motor não compila para Android

Medido com `cargo check --lib --target aarch64-linux-android` no motor:

- [ ] **`rfd`** (diálogo nativo de arquivo) não tem backend Android — **não
      compila** (12 erros). Precisa de `cfg` no `Cargo.toml` do motor
      (`[target.'cfg(not(target_os = "android"))'.dependencies]`) e em
      `src/file_dialog.rs`. Afeta `open_file`, `open_files`, `save_file` e
      `pick_folder` da camada Luau. No Android o equivalente é o Storage Access
      Framework (`ACTION_OPEN_DOCUMENT` / `ACTION_CREATE_DOCUMENT`) por JNI.
- [ ] **`notify-rust`** (`notify()` do Luau) fala D-Bus/freedesktop — não existe
      no Android. *Não confirmado se compila*; de qualquer forma não funciona.
      O equivalente é `NotificationManager` por JNI (+ permissão
      `POST_NOTIFICATIONS` no Android 13+).
- [ ] **`ring` e `mlua-sys` (Luau, C++)** exigem o clang do NDK. O Makefile
      exporta `CC_<alvo>`/`CXX_<alvo>`/`AR_<alvo>` por isso; **não verificado**
      que o Luau compila e linka com esse NDK.
- [ ] **`libc++_shared.so`**: o Luau é C++; o crate `cc` liga a `c++_shared` no
      Android, e a biblioteca precisa ir **dentro do APK**. Verificar se o
      `cargo-apk` a empacota; senão, `CXXSTDLIB=c++_static` ou copiá-la.
- [ ] O motor sozinho não passa no `cargo check` para Android: o
      `android-activity` exige que alguém escolha `native-activity` ou
      `game-activity`. O preset escolhe (feature do `winit` no `Cargo.toml`).
      Um `cargo check` do motor no CI precisaria fazer o mesmo.
- [ ] Features `tray`, `webview`, `system-fonts` e `single_instance`: **não
      ligar** no Android (Linux/desktop). Garantir que ligá-las dá erro claro,
      não um erro de linker.

## `open_window` → navegação em pilha (a mudança de modelo)

Hoje `open_window` abre **uma janela do SO** (um `GlacierUI` por janela,
`src/daemon.rs`). No Android há **uma** superfície, e isso não se traduz.

- [ ] No Android, `open_window(...)` deve **empilhar** uma tela: a nova entra
      no topo, a anterior fica viva por baixo (estado, scroll, foco).
- [ ] `close_window()` — e o **botão Voltar** do sistema — **desempilham**,
      voltando para a tela anterior; só a última da pilha encerra a activity.
- [ ] O botão Voltar chega como `Key::Named(NamedKey::GoBack)` no winit 0.30:
      mapear para "fechar a janela do topo".
- [ ] Decidir o que as chamadas que **assumem janelas separadas** viram:
      `broadcast` (todas as telas da pilha? só as vivas?), trazer uma janela
      para o foco, `close_window(id)` de uma tela que não é a do topo.
- [ ] Opções de `open_window` sem sentido no celular — **ignorar**, não falhar:
      posição, `decorations`, `always_on_top`, `resizable`, tamanho,
      `remember_window_geometry`.
- [ ] Relação com o ciclo de vida de tela já existente: `on_enter`/`on_leave` e
      o fechamento de streams do `navigate_to=` (0.112.0) devem valer também
      para empilhar/desempilhar.
- [ ] Documentar o contrato: o mesmo `.gv` multi-janela deve funcionar nos dois
      alvos — janelas no desktop, pilha no celular.

## Plataforma

- [ ] **Diretório de armazenamento.** `.glacier-storage` é relativo ao
      diretório de trabalho — no Android o cwd é `/`, somente leitura.
      `storage_dir` precisa vir de `AndroidApp::internal_data_path()`
      (`filesDir`). Afeta o global `storage` do Luau e o SQLite do usuário.
- [ ] **Assets sem hot-reload.** Só `embed_assets!` funciona (a lista é manual).
      Ideias: `embed_assets_dir!("views")` recursivo; e um `glacier serve
      android` que empurra `views/` por `adb` para um diretório do app e liga
      o hot-reload no aparelho.
- [ ] **APIs de filesystem do Luau** (`fetch("file://…")`, `zip_dir`, caminhos
      absolutos): semântica de caminhos no Android (sandbox, `filesDir`,
      `cacheDir`, URIs `content://`).
- [ ] **Tela cheia e áreas seguras**: status bar, barra de navegação, notch e
      teclado ocupam pixels. O motor não conhece *insets* — precisa de um
      padding de sistema (algo como `env(safe-area-inset-*)` no `.gss`).
- [ ] **Rotação e redimensionamento**: o preset declara `config_changes` para a
      activity não reiniciar; confirmar que `@media (max-width: …)` reage.
- [ ] **Escala (DPI)**: conferir que `size: 14`/`padding: 16` do `.gss` têm um
      tamanho legível em densidades 2×–3×.
- [ ] **Ciclo de vida do app**: ao ir para segundo plano a superfície wgpu é
      destruída e recriada; `every()`, `after()`, `sse()` e `websocket()`
      precisam **pausar** (bateria, e o Android mata sockets em background). O
      `iced` trata a superfície; **não verificado** que o motor sobrevive.
- [ ] **Fechar a última tela**: o loop do daemon termina e o processo sai; no
      Android isso deve terminar a activity, e o app não deve morrer no
      `SIGKILL` do sistema sem chance de salvar estado.
- [ ] **Backend gráfico**: wgpu/Vulkan funciona na maioria dos aparelhos
      recentes; aparelhos antigos precisam de GL. `ICED_BACKEND` não é
      configurável num APK — expor `GlacierDaemon::backend(...)`.

## Entrada

- [ ] **Toque ≠ mouse.** `:hover` não existe no toque (e "gruda" depois do
      toque em alguns casos); `:active` precisa valer no `touch down`.
- [ ] **Rolagem por gesto** com inércia em `<scrollable>` e `<tableview>`; arrastar não pode disparar `on_click`.
- [ ] **Pressionar e segurar** como clique direito (menus de contexto) e como "mostrar tooltip".
- [ ] **Teclado virtual / IME.** O `NativeActivity` do winit tem suporte fraco a
      IME; `<textinput>`/`<textarea>`/`<maskedinput>` podem não abrir o
      teclado. Avaliar `GameActivity` (`android-game-activity`), que exige uma
      parte Java/Kotlin no APK, ou JNI direto (`showSoftInput`).
- [ ] **Seleção de texto** com as alças do Android; copiar/colar (a
      `window_clipboard` no Android: não verificado).
- [ ] Alvos de toque: os widgets do motor (`<checkbox>`, `<slider>`, `<switch>`…) têm área de acerto de desktop (~20px). Um modo
      "touch" no tema que aumente o `padding` mínimo.

## Widgets e APIs sem equivalente direto

- [ ] `<tray>` / bandeja, `single_instance`: não se aplicam — ignorar com aviso.
- [ ] Barra de título própria (`completo`: janela sem decoração): não se aplica;
      no Android quem manda é a status bar.
- [ ] `webview_url` em `open_window` → `android.webkit.WebView` por JNI.
- [ ] `open_file`, `open_files`, `save_file`, `pick_folder`: ver `rfd` acima.
- [ ] `<menubar>`/`<menu>` estilo desktop: pensar em *bottom sheet* / *app bar*.
- [ ] `prompt{}`/`confirm{}` desenhados pelo motor devem funcionar
      (são widgets), mas conferir foco e teclado virtual dentro deles.
- [ ] Fontes: o preset embute Fira Sans (feature `fira-sans` do `iced`). Texto
      fora do latino (CJK, árabe, emoji) depende de fonte de sistema —
      `/system/fonts` — e o caminho de descoberta no Android não foi testado.
- [ ] MicroPython (`feature = "micropython"`): compila C com o NDK — não
      testado.

## Permissões (AndroidManifest)

O preset só declara `INTERNET`. O motor precisa de uma forma declarativa de
pedir o resto — algo como `<app permissions="camera location">` no `.gv`, ou
uma chave em `[package.metadata.android]` documentada:

- [ ] Pedir permissão **em tempo de execução** (Android 6+) a partir do Luau.
- [ ] `POST_NOTIFICATIONS` (Android 13+), câmera, localização, armazenamento.
- [ ] Referência pronta: o repositório `rust-android` já tem *crates* JNI para
      bateria, haptics, rede, notificações, permissões e sensores — candidatos
      a `GlacierDaemon::lua_extension` (a ponte Rust → Luau que já existe).

## Build, tamanho, distribuição

- [ ] **Tamanho**: o APK de debug passa de 150 MB (iced + wgpu + Luau). Medir o
      de release (`opt-level`, `lto`, `strip`, `panic = "abort"`); separar
      ABIs (`aab`/splits).
- [ ] **`patches/iced_winit` é uma cópia** do 0.14.1. Ao subir o `glacier-ui`
      para outro `iced`, refazer a cópia e reaplicar `ANDROID.patch`. O fim
      disso é o patch entrar no `iced`/`winit` upstream, ou o motor expor um
      `EventLoop` próprio no Android.
- [ ] **AAB e Play Store**: `cargo-apk` gera APK; para a loja é preciso um
      `.aab` (`cargo-apk2`, `xbuild` ou Gradle) e uma chave de publicação de
      verdade — a `release.keystore` do preset é só de teste.
- [ ] Ícone do app, `adaptive-icon`, nome por locale (`res/` do APK).
- [ ] Gradle/`GameActivity` como alternativa ao `cargo-apk` se o IME exigir.
- [ ] `glacier serve android` (build + instala + abre + logcat) na CLI, no mesmo
      espírito de `glacier serve wasm`.
- [ ] iOS: fora do escopo deste preset, mas o desenho de "pilha em vez de
      janela" serve aos dois.
