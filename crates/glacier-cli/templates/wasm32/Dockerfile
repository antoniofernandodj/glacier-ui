# A imagem web deste app: um estágio compila o `.wasm`, e o de produção recebe
# só a página, o `app.js` e o `app_bg.wasm` — nada de rustc, cargo, registry
# do crates.io ou `target/`.
#
#   docker build -t {{nome_projeto}}-web .
#   docker run --rm -p 8080:80 {{nome_projeto}}-web    # http://localhost:8080
#
# O que decide o tamanho final não é o servidor (o nginx:alpine-slim são ~19 MB):
# é o `.wasm`. Ele sai daqui sem as seções de nome e de produtor e é gravado
# **só** em `.gz` — o nginx o descomprime na hora para quem não aceitar gzip
# (ver `docker/nginx.conf`).

# ── Estágio 1: compilar ───────────────────────────────────────────────────────

# `rust:1-slim` é o último 1.x (hoje o 1.90, Debian bookworm). Para que a
# imagem de hoje e a de seis meses saiam da mesma toolchain, fixe a versão:
# `rust:1.90-slim`. O projeto é edition 2024, então o piso é o 1.85.
FROM rust:1-slim AS build

WORKDIR /app
RUN rustup target add wasm32-unknown-unknown

# O manifesto ANTES do código: este passo e o `cargo install` abaixo (que leva
# minutos) só refazem quando as dependências mudam, e não a cada edição de
# `src/` ou `views/`.
#
# `Cargo.lock*` com asterisco porque um projeto recém-criado ainda não tem o
# lock — nesse caso ele é gerado aqui, que resolve as versões sem compilar nada.
#
# O `src/main.rs` de mentira é só para o cargo conseguir LER o manifesto: sem
# alvo nenhum ele recusa com "no targets specified in the manifest" e o
# `generate-lockfile` para aqui. O `COPY src` abaixo passa por cima dele.
COPY Cargo.toml Cargo.lock* ./
RUN set -eu; \
    if [ ! -f Cargo.lock ]; then \
        mkdir -p src; \
        echo 'fn main() {}' > src/main.rs; \
        cargo generate-lockfile; \
    fi

# O binário do `wasm-bindgen` tem de ser da MESMA versão do crate
# `wasm-bindgen` que este projeto resolveu. Com versões diferentes o build passa
# e o erro só aparece no console do navegador — é a mesma conferência que o
# `glacier serve wasm` faz antes de compilar.
RUN set -eu; \
    versao="$(sed -n '/^name = "wasm-bindgen"$/{n;s/^version = "\(.*\)"$/\1/p;}' Cargo.lock)"; \
    if [ -z "$versao" ]; then \
        echo "o Cargo.lock não tem o crate wasm-bindgen — o projeto depende do glacier-ui?" >&2; \
        exit 1; \
    fi; \
    cargo install wasm-bindgen-cli --version "$versao" --locked

# `views/` é obrigatória no build da web: no navegador não há disco, e o
# `embed_assets!` do `src/main.rs` embute esses arquivos no `.wasm`.
COPY src ./src
COPY views ./views
COPY web ./web

RUN cargo build --release --target wasm32-unknown-unknown

# `--remove-name-section` e `--remove-producers-section` tiram do `.wasm` os
# nomes dos símbolos e a etiqueta de quem o gerou: são megabytes que só servem
# para depurar, e este é um build de release.
#
# Para encolher mais, o caminho é o `wasm-opt -Oz` do binaryen. Ele fica de
# fora de propósito: um binaryen mais antigo que o rustc que gerou o `.wasm`
# recusa as extensões novas do formato (reference-types), e aí o build para
# aqui — trocando tamanho de imagem por uma imagem que não sai.
RUN set -eu; \
    wasm="target/wasm32-unknown-unknown/release/{{nome_projeto}}.wasm"; \
    if [ ! -f "$wasm" ]; then \
        echo "não achei $wasm — o nome do [package] no Cargo.toml mudou?" >&2; \
        exit 1; \
    fi; \
    wasm-bindgen --target web --no-typescript \
        --remove-name-section --remove-producers-section \
        --out-name app --out-dir /dist "$wasm"; \
    cp -R web/. /dist/; \
    gzip -9 /dist/app.js /dist/app_bg.wasm; \
    ls -lh /dist

# ── Estágio 2: servir ─────────────────────────────────────────────────────────

FROM nginx:alpine-slim

# Só a configuração deste app; o `nginx.conf` da imagem (e o `mime.types` dele)
# continua valendo.
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /dist /usr/share/nginx/html

EXPOSE 80
