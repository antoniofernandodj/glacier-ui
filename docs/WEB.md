# Glacier no navegador (WebAssembly)

O motor compila para `wasm32-unknown-unknown` e roda num `<canvas>` do
navegador pelo mesmo caminho que o iced usa na web: winit cria o canvas, o wgpu
desenha por WebGL (ou WebGPU, onde houver) e o executor de tarefas é o
`wasm-bindgen-futures`. Um app é o mesmo `GlacierDaemon`, os mesmos
`Component`s Rust, o mesmo `.gv` e o mesmo `.gss`.

O exemplo de referência é `examples/web_contador`:

```sh
make web-contador          # compila, gera o JS e serve em http://localhost:8080
```

## Pré-requisitos (uma vez)

```sh
rustup target add wasm32-unknown-unknown
cargo binstall wasm-bindgen-cli@0.2.126   # ou: cargo install wasm-bindgen-cli --version 0.2.126
```

A versão do `wasm-bindgen-cli` **tem de ser igual** à do crate `wasm-bindgen` no
`Cargo.lock` (`grep -A1 'name = "wasm-bindgen"' Cargo.lock`). Com versões
diferentes o build passa e o erro só aparece no console do navegador.

Nenhuma feature precisa ser ligada: o `Cargo.toml` já soma `iced/webgl` e as
dependências web só no alvo `wasm32`.

## O que muda num app

**1. Os arquivos entram no binário.** No navegador não há disco para o
`DiskAssets` ler. Troque a fonte por `embed_assets!`, listando os mesmos
caminhos que o markup usa:

```rust
let daemon = GlacierDaemon::new();

#[cfg(target_arch = "wasm32")]
let daemon = daemon.assets(std::sync::Arc::new(glacier_ui::embed_assets![
    "views/app.gv",
    "views/app.gss",
    "views/theme.json",
]));
```

Os caminhos são relativos à raiz do crate que chama a macro. Um arquivo que
ficou fora da lista falha como um arquivo ausente no disco, e a mensagem diz
"não está entre os assets embutidos". O `EmbeddedAssets` também serve para um
binário nativo standalone. Com ele não há hot-reload.

**2. O HTML.** O iced troca o elemento de `id="iced"` pelo canvas da janela
principal. Veja `examples/web_contador/index.html`: um `<div id="iced">`, o canvas
com `width/height: 100%` e o `import init from "./<nome>.js"`.

**3. O build.**

```sh
cargo build --release --example web_contador --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir dist \
  target/wasm32-unknown-unknown/release/examples/web_contador.wasm
cp examples/web_contador/index.html dist/
```

`dist/` é estático e pode ir para qualquer servidor de arquivos. Ele precisa
ser servido por HTTP, porque `file://` não carrega módulo ES.

## O que NÃO existe na web

| Recurso | No navegador | Por quê |
| --- | --- | --- |
| `<script>` Luau | **erro no registro do componente** | O Luau é C++ e o `luau0-src` só gera wasm para emscripten; o iced roda em `wasm32-unknown-unknown`. |
| `LuaExtension`, `glacier_ui::mlua` | não compilam | Idem. |
| `fetch`, SSE, WebSocket | erro com a URL | O `net.rs` é hyper/rustls sobre socket. Hoje só a camada Luau chega aqui. |
| `notify()` | vai para o console | Sem D-Bus/WinRT. |
| Diálogo de arquivo | responde "cancelado" e avisa no console | Sem `rfd`. |
| Bandeja, webview, instância única | desligados | Cada aba é a própria instância. |
| Hot-reload | não há | Os assets são embutidos. |
| Várias janelas | não testado | Cada janela do daemon vira outro canvas no `<body>`. |

A regra de desenho é **nunca ignorar em silêncio**. Um `<script>` num template
não é descartado: o registro devolve `GlacierError::Luau` explicando o motivo,
porque uma tela com os botões mortos é o pior bug que este motor já teve.

## Fontes e renderização

- **Texto.** No navegador não existe fonte de sistema. O `Cargo.toml` liga
  `iced/fira-sans` só no alvo wasm, e essa passa a ser a fonte padrão. Sem ela
  a tela abre com fundo, botões e cores, **sem nenhuma letra** e sem erro.
  Uma fonte própria entra do jeito de sempre (`GlacierDaemon::font_named`),
  porque os bytes são embutidos no binário.
- **Negrito e itálico somem na web.** A `fira-sans` do iced embute **só a face
  Regular**. Um `bold: true` (ou `font="bold"`) pede peso Bold na família
  padrão. No desktop o SO tem essa face; no navegador não há nenhuma, e o texto
  ocupa o espaço mas não desenha nenhum glifo. Para usar negrito, registre a face:
  `GlacierDaemon::new().font(include_bytes!("fonts/FiraSans-Bold.ttf"))`.
  O mesmo vale para itálico e para qualquer outro peso.
- **GPU: desligada por padrão na web.** O build para o navegador compila o iced
  **sem wgpu**, só com o **tiny-skia**, que desenha por software num canvas 2D.
  Funciona em qualquer navegador, mas fica mais lento em tela grande e animada.
  O motivo: o iced 0.14 escolhe o backend pela variável `ICED_BACKEND`, e no
  navegador não existe ambiente, então o app não tem como escolher. O fallback
  wgpu → tiny-skia só acontece quando o wgpu **não acha adaptador**. Um Chrome
  que expõe WebGPU entrega o adaptador e, com driver ruim, falha ao criar a
  superfície. O resultado é canvas vazio sem erro nenhum, e foi isso que
  aconteceu no primeiro teste desta porta.
- **Com GPU:** `make web-contador WEB_FEATURES=web-gpu`, ou
  `cargo build … --features web-gpu`. Liga WebGPU, e WebGL2 onde não houver
  WebGPU. Só use se a tela aparecer no navegador de destino.

## Onde olhar quando algo não aparece

- **Console do navegador (F12).** Na web, `eprintln!`/`println!` do motor são
  redirecionados para `console.error`/`console.log` (macros no topo de
  `src/lib.rs`), e o `console_error_panic_hook` dá mensagem e linha a um
  `panic!`. Aviso de `.gss`, erro de parse de `.gv` e asset não embutido
  aparecem ali.
- **Tela branca e nada no console.** Confira se a versão do `wasm-bindgen-cli`
  casa com o `Cargo.lock`, e se a página está sendo servida por HTTP.
- **Tela branca com WebGPU.** O wgpu consegue o adaptador, mas o processo de GPU
  do Chrome não cria a swap chain. No log do Chrome aparece
  `Could not find SharedImageBackingFactory … WebgpuSwapChainTexture`. É o
  navegador ou o driver, não o app. Nesta máquina (Ivy Bridge, Vulkan
  incompleto) isso acontece com `--enable-unsafe-webgpu`. Sem essa flag o
  Chrome não expõe adaptador, e o fallback tiny-skia desenha normalmente.

### Verificação feita até aqui

Nesta máquina, só o caminho **tiny-skia** (software) foi visto desenhando a
tela inteira, em Chrome headless. Os caminhos WebGPU e WebGL por GPU **ainda
não foram vistos** funcionando: o Vulkan da GPU integrada está quebrado e o
Chrome não entrega WebGL aqui. Rode `make web-contador` numa máquina com GPU
funcional antes de dar esse caminho como bom.

## Como foi feito (para quem mexe no motor)

- `Cargo.toml`: mlua, hyper, tokio, rustls, tungstenite, notify-rust, rfd e zip
  moraram em `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. No
  wasm entram `iced/webgl`, `web-sys`, `wasmtimer` e
  `console_error_panic_hook`.
- `src/lib.rs` troca `luau` por `src/luau_web.rs` e `net` por `src/net_web.rs`
  via `#[path]`. Os dois mantêm a superfície que o resto do motor chama.
- `Instant`/`SystemTime` vêm de `iced::time` (reexport do `web-time`). No nativo
  são os mesmos tipos do `std`; no navegador, o `now()` do `std` panica.
- `tokio::time::sleep` virou `crate::timer::sleep`.
- `single_instance`, `file_dialog` e as notificações têm um ramo `wasm32`.

Para conferir que o nativo não quebrou e que a web compila:

```sh
cargo check --lib --examples
cargo check --lib --target wasm32-unknown-unknown
```
