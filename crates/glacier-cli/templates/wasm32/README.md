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

Opções: `--port <n>`, `--dev` (build de debug, maior e mais lento),
`--watch` e `--features web-gpu`.

**`glacier serve wasm --watch`** fecha o ciclo de edição no navegador: ele varre
`src/`, `views/`, `web/` e o `Cargo.toml`, recompila a cada mudança e a página
aberta se recarrega sozinha. Não é hot-reload — no navegador o `.gv` está dentro
do `.wasm` (ver a tabela abaixo) e recompilar é obrigatório; o `--watch` só tira
isso da sua mão. Um erro de compilação não derruba o servidor: a página segue
com a última build que funcionou. Mudança só em `web/` não recompila nada.

Para iterar em markup e estilo, o caminho mais rápido continua sendo o desktop
(`glacier serve desktop`), onde o hot-reload aplica sem recompilar. O conteúdo de `target/glacier-web/` é estático e pode ir
para qualquer servidor de arquivos.

O servidor do `glacier serve wasm` é para desenvolver: ele só escuta em
`127.0.0.1` e responde `Cache-Control: no-store`. Para publicar, use a imagem.

## Publicar: a imagem Docker

```sh
docker build -t {{nome_projeto}}-web .
docker run --rm -p 8080:80 {{nome_projeto}}-web    # http://localhost:8080
```

O `Dockerfile` tem dois estágios. O primeiro é um `rust:1-slim` que instala o
target `wasm32-unknown-unknown`, instala o `wasm-bindgen-cli` **na versão que o
`Cargo.lock` deste projeto pede** (versões diferentes geram um `.js` que não
casa com o `.wasm`, e o erro só apareceria no console do navegador), compila e
roda o `wasm-bindgen`. O segundo é um `nginx:alpine-slim` que recebe **só** a
página, o `app.js` e o `app_bg.wasm`: nada de rustc, cargo, registry ou
`target/` viaja para produção.

O que sobra de tamanho é o `.wasm`, e ele sai daqui de três maneiras menor:

- build de release;
- `--remove-name-section --remove-producers-section`, que tiram os nomes dos
  símbolos e a etiqueta do gerador — megabytes que só servem para depurar;
- gravado **apenas** em `.gz`. O `docker/nginx.conf` tem `gzip_static always`
  (manda o `.gz` mesmo para quem não pediu) e `gunzip on` (descomprime na hora
  para o cliente raro que não aceita gzip), então a imagem carrega um terço do
  peso sem responder coisa quebrada para ninguém.

Os nomes dos artefatos são fixos, então o `nginx.conf` responde
`Cache-Control: no-cache`: o navegador revalida (304 curto) em vez de servir o
`.wasm` de antes do último deploy. `GET /healthz` devolve `ok` para quem
orquestra a imagem.

Duas coisas que a configuração faz de propósito e que é fácil desfazer sem
perceber: **não** há `try_files` (ele procura o arquivo exato e daria 404 num
asset que só existe como `.gz`), e o `index.html` é o único arquivo **sem**
comprimir (a diretiva `index` olha o arquivo de verdade, e com só o
`index.html.gz` no disco a raiz daria 404).

Para encolher mais, o caminho é o `wasm-opt -Oz` do binaryen — o comentário no
`Dockerfile` diz por que ele não está ligado por padrão.

## O mapa

```
src/main.rs            a casca: embute views/ no alvo wasm e registra o componente
src/contador.rs        o Component: template + estado + update
views/contador.gv      o layout
views/styles/app.gss   a paleta (:root) e as classes
web/index.html         a página que carrega o app.js no navegador
Dockerfile             compila o .wasm num estágio e serve num nginx mínimo
docker/nginx.conf      a configuração desse nginx (gzip, tipo do .wasm, cache)
.dockerignore          o que não vai no contexto do build (target/ são gigabytes)
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
