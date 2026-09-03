//! O questionário do `glacier new` e a execução do que ele decidiu.
//!
//! A ordem é deliberada: **primeiro pergunta tudo, depois executa tudo**. Um
//! scaffolder que intercala pergunta e efeito deixa o usuário em meio caminho
//! quando ele desiste na terceira pergunta — aqui, até o "confirmar?" final,
//! nada foi escrito no disco.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::extensions::{self, Editor};
use crate::prompt::{self, Estilo};
use crate::scaffold::{self, PRESETS, Preset};

/// Resposta a uma pergunta que a linha de comando pode ter respondido antes.
/// `Auto` = ninguém decidiu ainda; o questionário (ou o default) decide.
#[derive(Clone, Copy, PartialEq)]
pub enum Opcao {
    Auto,
    Sim,
    Nao,
}

impl Opcao {
    fn decidir(self, e: &Estilo, interativo: bool, pergunta: &str, padrao: bool) -> bool {
        match self {
            Opcao::Sim => true,
            Opcao::Nao => false,
            Opcao::Auto if interativo => prompt::confirmar(e, pergunta, padrao),
            Opcao::Auto => padrao,
        }
    }
}

/// O que a linha de comando já fixou. O que ficar em aberto vira pergunta.
#[derive(Default)]
pub struct Argumentos {
    pub nome: Option<String>,
    pub preset: Option<String>,
    pub extensoes: Option<Opcao>,
    pub git: Option<Opcao>,
    pub build: Option<Opcao>,
    /// `--yes`: aceita todos os defaults sem perguntar nada.
    pub sim_para_tudo: bool,
}

/// A decisão completa, antes de qualquer efeito no disco.
pub struct Plano {
    pub nome: String,
    pub preset: &'static Preset,
    pub destino: PathBuf,
    /// `Some` = instalar as extensões neste editor.
    pub editor: Option<Editor>,
    pub git: bool,
    pub build: bool,
}

/// Roda o questionário. `Ok(None)` = o usuário respondeu "não" no resumo final.
pub fn perguntar(e: &Estilo, args: Argumentos) -> io::Result<Option<Plano>> {
    let interativo = prompt::interativo() && !args.sim_para_tudo;

    println!();
    println!("  {}", e.negrito("glacier — novo projeto"));
    println!(
        "  {}",
        e.fraco("Enter aceita o valor entre parênteses. Nada é escrito antes da confirmação.")
    );
    println!();

    // ── 1. Nome ──────────────────────────────────────────────────────────────
    let nome = match args.nome {
        Some(n) => n,
        None if interativo => prompt::texto(e, "Nome do projeto", "meu-app"),
        None => "meu-app".to_string(),
    };
    scaffold::validar_nome(&nome).map_err(|m| io::Error::new(io::ErrorKind::InvalidInput, m))?;

    let destino = PathBuf::from(&nome);
    if destino.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "{} já existe — escolha outro nome ou apague o diretório",
                destino.display()
            ),
        ));
    }

    // ── 2. Preset ────────────────────────────────────────────────────────────
    let preset = match args.preset {
        Some(id) => scaffold::preset(&id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("preset desconhecido: '{id}' (veja `glacier presets`)"),
            )
        })?,
        None if interativo => {
            println!();
            let opcoes: Vec<(&str, &str)> =
                PRESETS.iter().map(|p| (p.titulo, p.descricao)).collect();
            &PRESETS[prompt::escolher(e, "Qual preset?", &opcoes, 0)]
        }
        None => &PRESETS[0],
    };

    // ── 3. Extensões de VS Code ──────────────────────────────────────────────
    println!();
    let editor = escolher_editor(e, interativo, args.extensoes.unwrap_or(Opcao::Auto));

    // ── 4. git / build ───────────────────────────────────────────────────────
    let git = existe_no_path("git")
        && args.git.unwrap_or(Opcao::Auto).decidir(
            e,
            interativo,
            "Rodar `git init` no projeto?",
            true,
        );

    // Default `false`: a primeira compilação baixa e compila `iced` inteiro, e
    // alguns minutos de espera não deveriam ser o default de um `new`.
    let build = existe_no_path("cargo")
        && args.build.unwrap_or(Opcao::Auto).decidir(
            e,
            interativo,
            "Compilar agora (`cargo build`)?",
            false,
        );

    let plano = Plano {
        nome,
        preset,
        destino,
        editor,
        git,
        build,
    };

    // ── 5. Resumo e confirmação ──────────────────────────────────────────────
    resumo(e, &plano);
    if interativo && !prompt::confirmar(e, "Criar?", true) {
        return Ok(None);
    }
    Ok(Some(plano))
}

/// Decide em qual editor instalar. Sem editor no `PATH` (ou sem extensões
/// embutidas) a pergunta nem aparece: ela não teria resposta útil.
fn escolher_editor(e: &Estilo, interativo: bool, pedido: Opcao) -> Option<Editor> {
    if !extensions::disponiveis() || pedido == Opcao::Nao {
        return None;
    }

    let mut editores = extensions::detectar();
    if editores.is_empty() {
        if pedido == Opcao::Sim {
            println!(
                "  {}",
                e.fraco("nenhum editor da família VS Code no PATH — extensões puladas")
            );
        }
        return None;
    }

    let instalar = match pedido {
        Opcao::Sim => true,
        Opcao::Nao => false,
        Opcao::Auto if interativo => {
            for nome in extensions::nomes() {
                println!("  {} {}", e.fraco("·"), e.fraco(&nome));
            }
            prompt::confirmar(
                e,
                "Instalar as extensões de VS Code (realce de sintaxe e ir-para-definição em .gv/.gss)?",
                true,
            )
        }
        // Sem TTY, instalar num editor por conta própria seria um efeito
        // colateral fora do que foi pedido: `new` cria um projeto.
        Opcao::Auto => false,
    };
    if !instalar {
        return None;
    }

    if editores.len() == 1 || !interativo {
        return Some(editores.remove(0));
    }

    let opcoes: Vec<(&str, &str)> = editores
        .iter()
        .map(|ed| (ed.nome, ed.comando.as_str()))
        .collect();
    let i = prompt::escolher(e, "Em qual editor?", &opcoes, 0);
    Some(editores.remove(i))
}

fn resumo(e: &Estilo, plano: &Plano) {
    println!();
    println!("  {}", e.negrito("Resumo"));
    println!(
        "    projeto   {}",
        e.ciano(&plano.destino.display().to_string())
    );
    println!(
        "    preset    {} — {}",
        e.ciano(plano.preset.titulo),
        e.fraco(plano.preset.descricao)
    );
    for destaque in plano.preset.destaques {
        println!("              {}", e.fraco(&format!("· {destaque}")));
    }
    match &plano.editor {
        Some(ed) => println!(
            "    extensões {} {}",
            e.ciano("instalar em"),
            e.ciano(ed.nome)
        ),
        None => println!("    extensões {}", e.fraco("não instalar")),
    }
    println!(
        "    git init  {}",
        if plano.git {
            e.ciano("sim")
        } else {
            e.fraco("não")
        }
    );
    println!(
        "    build     {}",
        if plano.build {
            e.ciano("sim")
        } else {
            e.fraco("não")
        }
    );
    println!();
}

/// Executa o plano, na ordem em que uma falha custa menos: os arquivos
/// primeiro (é o que o usuário veio buscar), e só depois o que é acessório.
pub fn executar(e: &Estilo, plano: &Plano, versao_motor: &str) -> io::Result<()> {
    println!("{} {}", e.verde("▸"), e.negrito("Criando o projeto"));
    let escritos = scaffold::criar(&plano.destino, plano.preset.id, &plano.nome, versao_motor)?;
    for arquivo in &escritos {
        println!("  {} {}", e.verde("+"), arquivo.display());
    }

    if plano.git {
        println!();
        println!("{} {}", e.verde("▸"), e.negrito("git init"));
        match rodar("git", &["init", "-q"], &plano.destino) {
            Ok(true) => println!("  {} repositório iniciado", e.verde("✔")),
            // Nenhum dos dois é motivo para desfazer o projeto: os arquivos
            // estão lá, e `git init` é um comando que o usuário pode repetir.
            Ok(false) => println!("  {} `git init` falhou", e.vermelho("✘")),
            Err(erro) => println!("  {} `git init`: {erro}", e.vermelho("✘")),
        }
    }

    if let Some(editor) = &plano.editor {
        println!();
        println!(
            "{} {}",
            e.verde("▸"),
            e.negrito(&format!("Instalando as extensões em {}", editor.nome))
        );
        match extensions::instalar(editor, e) {
            Ok(0) => println!("  {}", e.fraco("nenhuma extensão instalada")),
            Ok(_) => println!(
                "  {}",
                e.fraco("reabra o editor para o realce valer nos .gv/.gss")
            ),
            Err(erro) => println!("  {} {erro}", e.vermelho("✘")),
        }
    }

    if plano.build {
        println!();
        println!("{} {}", e.verde("▸"), e.negrito("cargo build"));
        match rodar("cargo", &["build"], &plano.destino) {
            Ok(true) => println!("  {} compilado", e.verde("✔")),
            Ok(false) => println!(
                "  {} a compilação falhou (veja a saída acima)",
                e.vermelho("✘")
            ),
            Err(erro) => println!("  {} cargo: {erro}", e.vermelho("✘")),
        }
    }

    proximos_passos(e, plano);
    Ok(())
}

fn proximos_passos(e: &Estilo, plano: &Plano) {
    println!();
    println!("  {}", e.negrito("Pronto. A partir daqui:"));
    println!("    cd {}", plano.nome);
    println!("    cargo run");
    println!();
    println!(
        "  {}",
        e.fraco(
            "Com o app aberto, edite um .gv ou .gss e salve: o hot-reload aplica sem recompilar."
        )
    );
    println!(
        "  {}",
        e.fraco("Os caminhos de `views/` são relativos ao diretório de onde o app roda.")
    );
    println!();
}

/// Roda um comando em `dir` herdando a saída (o `git`/`cargo` fala direto com o
/// usuário). `Ok(false)` = rodou e falhou; `Err` = nem chegou a rodar.
fn rodar(programa: &str, args: &[&str], dir: &Path) -> io::Result<bool> {
    Ok(Command::new(programa)
        .args(args)
        .current_dir(dir)
        .status()?
        .success())
}

fn existe_no_path(programa: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    let sufixos: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };
    std::env::split_paths(&path).any(|d| {
        sufixos
            .iter()
            .any(|s| d.join(format!("{programa}{s}")).is_file())
    })
}
