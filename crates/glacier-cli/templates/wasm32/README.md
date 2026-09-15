# {{titulo}}

O mesmo app no **desktop** e no **navegador**, com
[glacier-ui](https://crates.io/crates/glacier-ui). O comportamento está em Rust
(o trait `Component`), e o markup e o estilo estão em `views/`.

```sh
glacier serve desktop    # abre a janela (o mesmo que `cargo run`)
glacier serve wasm       # compila para o navegador e serve em http://127.0.0.1:8080
```

## Pré-requisitos da web (uma vez)

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <a do Cargo.lock>
```

O `glacier serve wasm` confere as duas coisas antes de começar. Se a versão do
`wasm-bindgen` não bater com a do `Cargo.lock`, ele para e mostra o comando
certo. Com versões diferentes, o build passaria e o erro só apareceria no
console do navegador.

## O que o `glacier serve wasm` faz

1. `cargo build --release --target wasm32-unknown-unknown`
2. `wasm-bindgen --target web --out-name app`, gerando `app.js` e `app_bg.wasm`
   em `target/glacier-web/`
3. copia `web/` (a página) para junto deles
4. serve essa pasta em `127.0.0.1:8080` com `Cache-Control: no-store`. Assim
   o navegador nunca roda o `.wasm` de antes do rebuild.

Opções: `--port <n>`, `--dev` (build de debug, maior e mais lento) e
`--features web-gpu`. O conteúdo de `target/glacier-web/` é estático e pode ir
para qualquer servidor de arquivos.

## O mapa

```
src/main.rs            a casca: embute views/ no alvo wasm e registra o componente
src/contador.rs        o Component: template + estado + update
views/contador.gv      o layout
views/styles/app.gss   a paleta (:root) e as classes
web/index.html         a página que carrega o app.js no navegador
```

## Desktop e web: o que muda

| | desktop | navegador |
| --- | --- | --- |
| `views/` | lida do disco, com hot-reload | embutida no `.wasm` por `embed_assets!` |
| desenho | GPU | software (tiny-skia), ou GPU com `--features web-gpu` |
| `<script>` Luau | funciona | **não existe** (o Luau é C++ e não compila para esse alvo) |
| fontes | as do sistema | só a Fira Sans Regular: **negrito some** sem registrar a face Bold |
| `fetch`, notificação, diálogo de arquivo | funcionam | não existem |

**Arquivo novo em `views/`** entra também na lista do `embed_assets!` em
`src/main.rs`. Sem isso ele funciona no desktop e falha na web, com a mensagem
"não está entre os assets embutidos" no console (F12).

**Tela vazia no navegador?** Abra o console (F12). Na web, os erros do motor,
os `panic!` e os avisos de `.gss` aparecem ali.
