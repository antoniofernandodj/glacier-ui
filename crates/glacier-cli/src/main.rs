//! `glacier` — a CLI do glacier-ui.
//!
//! Existe por um motivo só: um projeto glacier tem um `Cargo.toml`, um
//! `src/main.rs`, um `.gvb` com cabeçalho, um `.gss`, um `.luaurc` e uma árvore
//! de scripts Luau — e descobrir essa disposição lendo o README, arquivo por
//! arquivo, é a parte mais chata de começar. `glacier new` pergunta o que
//! precisa saber e entrega tudo isso já ligado e rodando.
//!
//! ```text
//! glacier new [nome]          cria um projeto (questionário)
//! glacier serve wasm          compila para o navegador e serve em localhost
//!                             (`--watch` recompila e recarrega a cada mudança)
//! glacier serve desktop       roda o app no desktop (`cargo run` na raiz)
//! glacier test [arquivo]      roda os testes Luau (`*.test.luau`)
//!
//! Os padrões de `serve` e `test` (porta, release, features, configurações
//! nomeadas com `-c`) ficam no `glacier.jsonc` do projeto — ver `config`.
//! glacier install-extensions  só as extensões de VS Code
//! glacier presets             lista os presets
//! ```

mod config;
mod embedded;
mod extensions;
mod json;
mod prompt;
mod scaffold;
mod serve;
mod vsix;
mod wizard;

use std::process::ExitCode;

use prompt::Estilo;
use wizard::{Argumentos, Opcao};

fn main() -> ExitCode {
    let e = Estilo::detectar();
    let mut args = std::env::args().skip(1);

    let resultado = match args.next().as_deref() {
        None | Some("-h") | Some("--help") | Some("help") => {
            ajuda(&e);
            Ok(())
        }
        Some("-V") | Some("--version") | Some("version") => {
            println!(
                "glacier {} (motor glacier-ui {})",
                env!("CARGO_PKG_VERSION"),
                embedded::VERSAO_MOTOR
            );
            Ok(())
        }
        Some("presets") => {
            listar_presets(&e);
            Ok(())
        }
        Some("new") => novo(&e, args.collect()),
        Some("install-extensions") => instalar_extensoes(&e),
        Some("serve") => serve::serve(&e, args.collect()),
        Some("test") => serve::testar(&e, args.collect()),
        Some(outro) => {
            eprintln!("{} comando desconhecido: '{outro}'", e.vermelho("erro:"));
            eprintln!("  rode `glacier --help` para ver os comandos.");
            return ExitCode::FAILURE;
        }
    };

    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("{} {erro}", e.vermelho("erro:"));
            ExitCode::FAILURE
        }
    }
}

fn novo(e: &Estilo, argv: Vec<String>) -> std::io::Result<()> {
    let mut args = Argumentos::default();
    let mut resto = argv.into_iter();

    while let Some(arg) = resto.next() {
        match arg.as_str() {
            // Um `--preset` sem valor não pode cair no default em silêncio: quem
            // o digitou tem um preset em mente, e receber outro sem aviso é pior
            // do que a mensagem de erro.
            "--preset" | "-p" => match resto.next() {
                Some(id) => args.preset = Some(id),
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("`{arg}` precisa do id de um preset (veja `glacier presets`)"),
                    ));
                }
            },
            "--extensions" => args.extensoes = Some(Opcao::Sim),
            "--no-extensions" => args.extensoes = Some(Opcao::Nao),
            "--git" => args.git = Some(Opcao::Sim),
            "--no-git" => args.git = Some(Opcao::Nao),
            "--build" => args.build = Some(Opcao::Sim),
            "--no-build" => args.build = Some(Opcao::Nao),
            "-y" | "--yes" => args.sim_para_tudo = true,
            outro if outro.starts_with('-') => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("opção desconhecida de `new`: '{outro}'"),
                ));
            }
            nome => args.nome = Some(nome.to_string()),
        }
    }

    match wizard::perguntar(e, args)? {
        Some(plano) => wizard::executar(e, &plano, embedded::VERSAO_MOTOR),
        None => {
            println!("  {}", e.fraco("cancelado — nada foi criado."));
            Ok(())
        }
    }
}

fn instalar_extensoes(e: &Estilo) -> std::io::Result<()> {
    if !extensions::disponiveis() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "esta build da CLI não tem extensões embutidas",
        ));
    }

    let mut editores = extensions::detectar();
    if editores.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "nenhum editor da família VS Code no PATH (procurei por code, code-insiders, cursor, codium, windsurf)",
        ));
    }

    let editor = if editores.len() == 1 || !prompt::interativo() {
        editores.remove(0)
    } else {
        let opcoes: Vec<(&str, &str)> = editores
            .iter()
            .map(|ed| (ed.nome, ed.comando.as_str()))
            .collect();
        let i = prompt::escolher(e, "Em qual editor?", &opcoes, 0);
        editores.remove(i)
    };

    println!(
        "{} {}",
        e.verde("▸"),
        e.negrito(&format!("Instalando as extensões em {}", editor.nome))
    );
    let n = extensions::instalar(&editor, e)?;
    if n > 0 {
        println!(
            "  {}",
            e.fraco("reabra o editor para o realce valer nos .gvb/.gss")
        );
    }
    Ok(())
}

fn listar_presets(e: &Estilo) {
    println!();
    for p in scaffold::PRESETS {
        println!("  {}  {}", e.ciano(p.id), e.negrito(p.titulo));
        println!("      {}", e.fraco(p.descricao));
        for destaque in p.destaques {
            println!("      {}", e.fraco(&format!("· {destaque}")));
        }
        println!();
    }
}

fn ajuda(e: &Estilo) {
    let ids: Vec<&str> = scaffold::PRESETS.iter().map(|p| p.id).collect();
    println!(
        "
  {titulo}
  {sub}

  {uso}
    glacier new [nome]            cria um projeto — pergunta o resto
    glacier serve wasm            compila para o navegador e serve em http://127.0.0.1:8080
    glacier serve desktop         roda o app no desktop (`cargo run` na raiz do projeto)
    glacier test [arquivo|pasta]  roda os testes Luau (*.test.luau em views/ e tests/)
    glacier install-extensions    só instala as extensões de VS Code
    glacier presets               descreve os presets disponíveis
    glacier --version

  {opcoes_new}
    -p, --preset <id>             {presets}
        --extensions              instala as extensões sem perguntar
        --no-extensions           não instala as extensões
        --git / --no-git          `git init` no projeto criado
        --build / --no-build      `cargo build` ao final
    -y, --yes                     não pergunta nada: aceita todos os defaults

  {opcoes_serve}
    wasm     --port <n>           porta local (padrão 8080)
             --dev                build de debug (padrão: release)
         -w, --watch              recompila a cada mudança e recarrega a página
             --no-watch           desliga o `watch` que o glacier.jsonc ligou
    desktop  --release            build otimizado (padrão: debug, como `cargo run`)
             -- <args>            repassa os argumentos ao app
    ambos    -F, --features <l>   features do projeto (ex.: web-gpu)
         -c, --configuration <n>  configuração nomeada do glacier.jsonc

  {opcoes_test}
    -t, --filter <trecho>         só os testes cujo nome contém o trecho
    -v, --verbose                 mostra o nome de cada teste
    -vv                           + a documentação (`---`) de cada teste
    -vvv                          + tempo por teste e stacktrace das falhas
    -s, --silent                  esconde os logs dos testes (print, console.*)
                                  que aparecem por padrão; combina: -svvv
        --release                 build otimizado
    -F, --features <l>            features do projeto
    -c, --configuration <n>       configuração nomeada do glacier.jsonc

  {arquivo}
    glacier.jsonc na raiz do projeto: os padrões de `serve` e `test` (porta,
    release, features, configurações nomeadas). Opcional; a flag sempre vence.
    Campo desconhecido é erro. `glacier new` já o cria, comentado campo a campo.

  {exemplos}
    glacier new
    glacier new painel --preset completo --extensions --no-build
    glacier new teste -p minimo -y
    glacier new site -p wasm32 -y && cd site && glacier serve wasm
    glacier test
    glacier test -vv            nomes e documentação de cada teste
    glacier test -svvv          sem logs, com tempo e stacktrace das falhas
    glacier test views/scripts/handlers/settings.test.luau -t zip
    glacier serve wasm -c dev
",
        titulo = e.negrito("glacier — a CLI do glacier-ui"),
        sub = e.fraco("cria um projeto pronto (templates .gvb/.gss + scripts Luau) e instala as extensões de VS Code"),
        uso = e.negrito("USO"),
        opcoes_new = e.negrito("OPÇÕES DE `new`"),
        opcoes_serve = e.negrito("OPÇÕES DE `serve`"),
        opcoes_test = e.negrito("OPÇÕES DE `test`"),
        arquivo = e.negrito("CONFIGURAÇÃO"),
        exemplos = e.negrito("EXEMPLOS"),
        presets = ids.join(" | "),
    );
}
