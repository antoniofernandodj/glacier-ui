# glacier-cli

A CLI do [glacier-ui](https://crates.io/crates/glacier-ui). Ela existe por um
motivo só: um projeto glacier tem um `Cargo.toml`, um `src/main.rs`, um `.gv`
com cabeçalho, um `.gss`, um `.luaurc` e uma árvore de scripts Luau — e
descobrir essa disposição lendo o README, arquivo por arquivo, é a parte mais
chata de começar.

```bash
cargo install glacier-cli
glacier new
```

O `new` faz um questionário — navegável pelas setas (↑/↓ movem, Enter escolhe;
sem um `stty` no sistema, cai num menu numerado) —, mostra um resumo e **só
então** escreve alguma coisa: até a confirmação final, nada foi criado.

```
glacier new [nome]            cria um projeto — pergunta o resto
glacier serve wasm            compila para o navegador e serve em http://127.0.0.1:8080
glacier serve desktop         roda o app no desktop (`cargo run` na raiz do projeto)
glacier install-extensions    só instala as extensões de VS Code
glacier presets               descreve os presets disponíveis
glacier --version
```

## Presets

| id | O que é |
|---|---|
| `completo` | Janela sem decoração com titlebar própria, tema + `.gss`, componentes com `<props>`, navegação, `fetch`, toasts, `@media` |
| `minimo` | Uma tela, um `.gss` e um bloco de script Luau — o menor projeto que ainda mostra a ideia |
| `janelas` | Multi-janela (`open_window`/`broadcast`/`close_window`), ícone de bandeja, instância única, geometria lembrada |
| `rust` | O trait `Component` com estado tipado em Rust, em vez de comportamento em Luau |
| `wasm32` | O mesmo app no desktop e no navegador: `Component` em Rust, `views/` embutida no `.wasm` por `embed_assets!` e uma `web/index.html`. Roda com `glacier serve wasm`, e publica com o `Dockerfile` multi-stage que vem junto |
| `catalogo` | Catálogo navegável de widgets: uma sidebar de categorias, cada tela com dezenas de widgets do motor num exemplo mínimo e vivo, feito para copiar |
| `dashboard` | Painel de KPIs e gráficos que andam sozinhos (`linechart` de série múltipla, `barchart`, `donut`, `gauge`, `sparkline`) via `every(1000)` |
| `formulario` | App de cadastro: `maskedinput`, `spinbox`, `dateedit`, `select`, `buttonbox` + validação em Luau que publica `erro_<campo>` |
| `crud` | Model/view: uma `tableview` ligada a um array do contexto, com Novo/Editar/Excluir por `prompt{}`/`confirm{}` (dados em memória) |

Todos herdam o `.gitignore`, o `.luaurc`, o `views/scripts/glacier.d.luau` (os
tipos dos globais que o motor injeta, para o luau-lsp) e a camada de build e
empacotamento abaixo.

## Compilar, empacotar e instalar

Todo projeto criado já sai com `Makefile` (Linux) e `fazer.bat` (Windows, onde
não há make), cobrindo os dois sistemas dos dois lados:

| | Makefile | fazer.bat |
|---|---|---|
| compilar | `make build` | `fazer build` |
| **para Windows** | `make windows` (cross-compile via cargo-xwin) | `fazer build` (MSVC nativo) |
| empacotar Windows | `make windows-dist` → `.zip` | `fazer dist` |
| empacotar Linux | `make linux-dist` → `.tar.gz`, `make deb` → `.deb` | — |
| instalar | `make install` (`~/.local`), `make install-sistema` | `fazer instalar` |

O `.exe` sai com `+crt-static`: sem isso ele exige o Visual C++ Redistributable
na máquina de destino e falha com uma caixa de erro que não diz qual DLL faltou.

Os pacotes levam `packaging/{windows,linux}/`: no Windows um `instalar.bat` que
copia para `%LOCALAPPDATA%\Programs` e cria o atalho no menu Iniciar **sem pedir
administrador**; no Linux um `instalar.sh` que instala em `~/.local` (ou
`--sistema` para `/usr/local`) e gera a entrada `.desktop`.

### Por que todo alvo de pacote termina numa conferência

O app lê `views/` em **runtime** — é o que dá o hot-reload. Um pacote sem essa
pasta compila, empacota, instala e abre: numa janela vazia, na máquina de quem
baixou, sem nenhuma mensagem que aponte a causa. Então `conferir-pacote` compara
a contagem de arquivos e falha alto antes de o `.zip` existir.

Pelo mesmo motivo, o que vai para `/usr/bin` (no `.deb`) e para `~/.local/bin`
(no `instalar.sh`) é um **wrapper de três linhas** que faz `cd` para a pasta de
instalação antes de executar. O programa de verdade fica ao lado do `views/`.
Sem isso, rodar o app de qualquer outro diretório o faria procurar os templates
onde eles não estão.

## Opções de `new`

```
-p, --preset <id>       completo | minimo | janelas | rust | wasm32 | catalogo | dashboard | formulario | crud
    --extensions        instala as extensões sem perguntar
    --no-extensions     não instala as extensões
    --git / --no-git    `git init` no projeto criado
    --build / --no-build  `cargo build` ao final
-y, --yes               não pergunta nada: aceita todos os defaults
```

Sem TTY (num pipe, em CI) o questionário é pulado e valem os defaults — e as
extensões **não** são instaladas, porque mexer no editor de alguém não é o que
`new` foi chamado para fazer.

## `serve`: rodar o projeto do diretório atual

`glacier serve` procura o `Cargo.toml` subindo a partir do diretório atual e
roda o app a partir da raiz do projeto. Isso importa porque os caminhos de
`views/` são relativos a ela.

### `glacier serve desktop`

`cargo run` na raiz do projeto. `--release` para o build otimizado,
`--features <lista>`, e `-- <args>` repassa argumentos ao app.

### `glacier serve wasm`

Compila para o navegador e serve, num comando só:

1. confere o target `wasm32-unknown-unknown` (senão mostra o `rustup target add`);
2. abre a porta **antes** de compilar, para não descobrir que ela está ocupada
   depois de minutos de build;
3. `cargo build --release --target wasm32-unknown-unknown`, lendo o caminho do
   `.wasm` pela saída JSON do cargo (respeita `CARGO_TARGET_DIR` e workspaces);
4. confere que o `wasm-bindgen` do `PATH` tem a **mesma versão** do crate
   `wasm-bindgen` no `Cargo.lock`. Com versões diferentes o build passaria e o
   erro só apareceria no console do navegador;
5. `wasm-bindgen --target web --out-name app` em `target/glacier-web/`, mais
   uma cópia de `web/` (a página);
6. serve essa pasta em `127.0.0.1:<porta>` com `Content-Type: application/wasm`
   e `Cache-Control: no-store`, para o navegador nunca rodar o `.wasm` de antes
   do rebuild.

```
--port <n>            porta local (padrão 8080)
--dev                 build de debug (padrão: release)
-w, --watch           recompila a cada mudança e recarrega a página
-F, --features <l>    features do projeto (ex.: web-gpu)
```

### `--watch`: o ciclo de edição no navegador

No navegador **não existe hot-reload**: os `.gv` e `.gss` entram dentro do
`.wasm` pelo `embed_assets!`, então ver uma mudança exige recompilar. O
`--watch` faz esse ciclo sozinho — varre `src/`, `views/`, `web/` e o
`Cargo.toml`, recompila quando algo muda, e a página aberta se recarrega.

```sh
glacier serve wasm --watch
```

- **Varredura de conteúdo (CRC), não `inotify`**: a CLI não tem dependências, e
  o conjunto vigiado é pequeno e escolhido a dedo — nunca `target/`. O CRC é o
  que decide, e não o `mtime`: dois salvamentos do mesmo tamanho dentro da mesma
  marca de tempo passariam batidos num filesystem de granularidade de 1 s.
- **Mudança só em `web/` não recompila**: a página é cópia, não `.wasm`.
- **Erro de compilação não derruba o servidor.** Cada build é montada em
  `target/glacier-web-next/` e só depois toma o lugar de `target/glacier-web/`,
  então a página aberta segue com a última build que funcionou; corrija e salve
  de novo.
- **A recarga é injetada na `index.html` servida**, antes do `</body>` — o
  arquivo do projeto não é tocado. É consulta em laço a `/__glacier/recarregar`
  (meio segundo), não WebSocket: o servidor é uma thread por conexão, e uma
  conexão pendurada por aba custaria mais que um GET de três bytes em
  `127.0.0.1`. Sem `--watch`, nem a rota nem o script existem.

O `.gv` e o `.gss` **no desktop** continuam com hot-reload de verdade (o app lê
`views/` do disco): para iterar em markup e estilo, o caminho mais rápido segue
sendo `glacier serve desktop`, e o navegador para conferir.

O servidor é estático, só escuta em `127.0.0.1` e é std puro, como o resto da
CLI. Serve para desenvolver; para publicar, suba o conteúdo de
`target/glacier-web/` para qualquer servidor de arquivos.

O projeto precisa de uma `web/index.html` que importe `./app.js`. O preset
`wasm32` já vem com ela.

### A imagem de produção

O preset `wasm32` também vem com um `Dockerfile` de dois estágios:

```sh
docker build -t meu-app-web . && docker run --rm -p 8080:80 meu-app-web
```

O primeiro estágio (`rust:1-slim`) instala o target, instala o
`wasm-bindgen-cli` **na versão que o `Cargo.lock` do projeto pede** — a mesma
conferência que o `serve wasm` faz — compila e roda o `wasm-bindgen`. O segundo
(`nginx:alpine-slim`, ~19 MB) recebe só a página, o `app.js` e o `app_bg.wasm`.
O `.wasm` vai sem as seções de nome e de produtor e gravado **apenas** em
`.gz`; o `docker/nginx.conf` tem `gzip_static always` + `gunzip on`, então
ninguém recebe resposta quebrada por isso. Os detalhes — e as duas pegadinhas
da configuração — estão no `README.md` do projeto criado.

## As extensões de VS Code

`glacier install-extensions` instala a **Glacier View** (`.gv`) e a **Glacier
GSS** (`.gss`): realce de sintaxe, mais links clicáveis e ir-para-definição dos
`src`/`href`/ações para os arquivos (e funções) que eles nomeiam.

As duas vêm embutidas neste binário e são empacotadas em `.vsix` na hora, sem
Node nem `vsce` — quem rodou `cargo install` não precisa ter nenhum dos dois.
Editores procurados no `PATH`, nessa ordem: `code`, `code-insiders`, `cursor`,
`codium`, `windsurf`.

## Sem dependências

Este crate não tem nenhuma dependência — nem `clap`, nem `iced`, nem o próprio
`glacier-ui`. É deliberado: a CLI existe para tirar alguém do zero, e um
`cargo install` que leva minutos derrotaria o propósito. O questionário, o zip
do `.vsix` e a escrita dos arquivos são std puro.

## Instalar sem o crates.io (.deb)

Para exercitar a CLI como o usuário final a vê — no `PATH`, longe do `target/`:

```bash
make deb-cli        # constrói em target/debian/ e confere as dependências
make install-cli    # constrói e instala (usa sudo)
make uninstall-cli
```

O pacote é só o binário (~270 KB). O `Depends` sai como `libc6 (>= 2.39)`: o
binário exige `libc.so.6` e `libgcc_s.so.1`, e o `dpkg-shlibdeps` omite o
segundo porque `libgcc-s1` já vem por dependência do `libc6`. Nada de GTK nem
de Node — as extensões vão embutidas e o `.vsix` é montado em tempo de execução.
`make check-deb` falha se o binário ganhar uma dependência nativa fora da glibc.

O piso `>= 2.39` é o da glibc da máquina que compilou (Ubuntu 24.04+ / Debian
13+). Para distribuir mais amplamente, compile num ambiente de glibc mais antiga.

## Desenvolvimento

Os presets vivem em `templates/<id>/` e são embutidos pelo `build.rs`; editar um
arquivo lá recompila a CLI. Dois arquivos ficam ali disfarçados, e a CLI desfaz
o disfarce ao criar o projeto (ver `scaffold::renomear`):

| No template | No projeto criado | Por quê |
|---|---|---|
| `gitignore` | `.gitignore` | com o ponto, ele valeria para o repositório do glacier e esconderia os próprios presets |
| `Cargo.toml.template` | `Cargo.toml` | `cargo package` **pula** todo subdiretório com um `Cargo.toml` — com o nome real, os presets ficariam de fora do `.crate` publicado |

As extensões vêm de `../../editors/` num checkout —
para publicar, `make sync-extensions` copia essa árvore para dentro do crate
(o Cargo não empacota nada de fora dele):

```bash
make publish-cli       # sync-extensions + cargo publish -p glacier-cli
```

Os presets são cobertos por dois testes no crate do motor: `tests/exemplos_gv.rs`
(todo `.gv` parseia) e `tests/presets_cli.rs` (cada preset carrega num
`GlacierUI` de verdade — imports, stylesheets e Luau incluídos).
