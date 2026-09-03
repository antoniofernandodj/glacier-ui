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

O `new` faz um questionário, mostra um resumo e **só então** escreve alguma
coisa: até a confirmação final, nada foi criado.

```
glacier new [nome]            cria um projeto — pergunta o resto
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
-p, --preset <id>       completo | minimo | janelas | rust
    --extensions        instala as extensões sem perguntar
    --no-extensions     não instala as extensões
    --git / --no-git    `git init` no projeto criado
    --build / --no-build  `cargo build` ao final
-y, --yes               não pergunta nada: aceita todos os defaults
```

Sem TTY (num pipe, em CI) o questionário é pulado e valem os defaults — e as
extensões **não** são instaladas, porque mexer no editor de alguém não é o que
`new` foi chamado para fazer.

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
