//! `glacier serve wasm` e `glacier serve desktop`: rodar o projeto do diretório
//! atual sem decorar a sequência de comandos de cada alvo.
//!
//! O `wasm` é o que justifica o comando. Para o navegador são quatro passos, e
//! cada um tem uma armadilha que não dá erro na hora:
//!
//! 1. `cargo build --target wasm32-unknown-unknown`. Sem o target instalado, o
//!    erro do rustc fala de `core`, não de rustup.
//! 2. `wasm-bindgen` na MESMA versão do crate `wasm-bindgen` do `Cargo.lock`.
//!    Com outra versão, o `.js` gerado não casa com o `.wasm`, e o erro só
//!    aparece no console do navegador.
//! 3. Copiar a página (`web/`) para junto do `.js` gerado.
//! 4. Servir por HTTP (`file://` não carrega módulo ES) e com
//!    `Cache-Control: no-store`. Um servidor comum responde 304, e o navegador
//!    segue rodando o `.wasm` ANTERIOR ao rebuild, sem aviso nenhum.
//!
//! O servidor é std puro, como o resto da CLI (ver o Cargo.toml): estático,
//! só em 127.0.0.1, uma thread por conexão. Serve para desenvolver, não para
//! publicar.
//!
//! ## `--watch`
//!
//! No navegador não existe hot-reload: os `.gv` e `.gss` entram DENTRO do
//! `.wasm` pelo `embed_assets!`, então ver uma mudança exige recompilar. O
//! `--watch` automatiza esse ciclo — varre `src/`, `views/`, `web/` e o
//! `Cargo.toml`, recompila quando algo muda e faz a página se recarregar.
//!
//! Três decisões que ele carrega:
//!
//! - **varredura, não `inotify`**: a CLI não tem dependências, e o conjunto
//!   vigiado é pequeno e escolhido a dedo (nunca `target/`);
//! - **cada build é montada à parte e só depois troca de lugar**: um erro de
//!   compilação deixa a página aberta funcionando com a build anterior, em vez
//!   de derrubar o servidor e obrigar a redigitar o comando;
//! - **a recarga é injetada na `index.html` servida**, não no arquivo do
//!   projeto: o `--watch` não deixa resíduo no disco de ninguém.

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use crate::prompt::Estilo;

const ALVO_WASM: &str = "wasm32-unknown-unknown";
const PORTA_PADRAO: u16 = 8080;

/// Nome fixo dos arquivos gerados (`app.js`, `app_bg.wasm`): a página importa
/// `./app.js` sem precisar saber o nome do crate.
const NOME_SAIDA: &str = "app";

/// Onde a página montada fica, sob a raiz do projeto. Dentro de `target/`
/// para já estar coberta pelo `.gitignore` e sumir num `cargo clean`.
const DIR_SAIDA: &str = "target/glacier-web";

/// Onde cada build é MONTADA, antes de virar a pasta servida.
///
/// A build antiga só é substituída depois que a nova terminou inteira. É o que
/// permite ao `--watch` sobreviver a um erro de compilação: a pasta servida
/// continua com a build que funcionava, e a página aberta no navegador não cai.
const DIR_MONTAGEM: &str = "target/glacier-web-next";

/// Intervalo entre duas varreduras do disco no modo `--watch`.
///
/// Varredura de `mtime`, e não `inotify`: esta CLI não tem dependências (ver o
/// Cargo.toml), e as três pastas vigiadas são pequenas — o `views/` de um
/// projeto tem dezenas de arquivos, não milhares.
const INTERVALO_VIGIA: Duration = Duration::from_millis(700);

/// Espera depois de ver a primeira mudança. Um editor que salva escrevendo um
/// temporário e renomeando produz duas mudanças seguidas, e sem esta pausa a
/// segunda dispararia uma segunda build já obsoleta.
const ACALMAR: Duration = Duration::from_millis(250);

/// A rota que a página consulta para saber se saiu build nova. Só existe com
/// `--watch`.
const ROTA_RECARGA: &str = "/__glacier/recarregar";

/// O script que o servidor injeta na `index.html` **servida** quando o
/// `--watch` está ligado. O arquivo no disco não é tocado.
///
/// É consulta em laço, e não WebSocket nem SSE: o servidor é uma thread por
/// conexão, e uma conexão pendurada por aba aberta custaria mais que um GET de
/// três bytes a cada meio segundo em `127.0.0.1`.
const SCRIPT_RECARGA: &str = r#"<script>
// Injetado pelo `glacier serve wasm --watch` — não está no seu arquivo.
(async () => {
  let atual = null;
  for (;;) {
    try {
      const r = await fetch("/__glacier/recarregar", { cache: "no-store" });
      const geracao = await r.text();
      if (atual === null) atual = geracao;
      else if (geracao !== atual) location.reload();
    } catch (_) {
      // Servidor parado ou reiniciando: tenta de novo no próximo laço.
    }
    await new Promise((f) => setTimeout(f, 500));
  }
})();
</script>
"#;

#[derive(Debug, Clone, PartialEq)]
struct Opcoes {
    porta: u16,
    /// Na web o padrão é release (`--dev` desliga): um `.wasm` de debug tem
    /// centenas de MB e roda devagar no navegador. No desktop o padrão é o do
    /// `cargo run` (`--release` liga).
    release: bool,
    features: Option<String>,
    /// `--watch`: recompila a cada mudança em `src/`, `views/`, `web/` e
    /// `Cargo.toml`, e faz a página se recarregar (só wasm).
    watch: bool,
    /// Argumentos depois de `--`, repassados ao app (só desktop).
    resto: Vec<String>,
}

pub fn serve(e: &Estilo, argv: Vec<String>) -> io::Result<()> {
    let mut args = argv.into_iter();
    match args.next().as_deref() {
        Some("wasm") => wasm(e, opcoes(args, Alvo::Wasm)?),
        Some("desktop") => desktop(e, opcoes(args, Alvo::Desktop)?),
        Some(outro) => Err(invalido(format!(
            "alvo desconhecido de `serve`: '{outro}' (use `wasm` ou `desktop`)"
        ))),
        None => Err(invalido(
            "`serve` precisa do alvo: `glacier serve wasm` ou `glacier serve desktop`".into(),
        )),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Alvo {
    Wasm,
    Desktop,
}

fn opcoes(mut args: impl Iterator<Item = String>, alvo: Alvo) -> io::Result<Opcoes> {
    let wasm = alvo == Alvo::Wasm;
    let mut op = Opcoes {
        porta: PORTA_PADRAO,
        release: wasm,
        features: None,
        watch: false,
        resto: Vec::new(),
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" if wasm => {
                let valor = args
                    .next()
                    .ok_or_else(|| invalido("`--port` precisa de um número".into()))?;
                op.porta = valor
                    .parse()
                    .map_err(|_| invalido(format!("porta inválida: '{valor}'")))?;
            }
            "--dev" if wasm => op.release = false,
            "--watch" | "-w" if wasm => op.watch = true,
            "--release" if !wasm => op.release = true,
            "--features" | "-F" => {
                // Sem valor não pode virar "sem features" em silêncio: quem
                // digitou `--features` queria alguma.
                let valor = args
                    .next()
                    .ok_or_else(|| invalido(format!("`{arg}` precisa da lista de features")))?;
                op.features = Some(valor);
            }
            "--" if !wasm => op.resto.extend(args.by_ref()),
            outro => {
                let nome = if wasm { "wasm" } else { "desktop" };
                return Err(invalido(format!(
                    "opção desconhecida de `serve {nome}`: '{outro}'"
                )));
            }
        }
    }
    Ok(op)
}

// ── desktop ───────────────────────────────────────────────────────────────────

fn desktop(e: &Estilo, op: Opcoes) -> io::Result<()> {
    let raiz = raiz_do_projeto()?;

    // `current_dir` na raiz, e não onde o comando foi digitado: os caminhos de
    // `views/` são relativos ao diretório de onde o app roda, e um `cargo run`
    // de dentro de `src/` abriria uma janela vazia.
    let mut cmd = Command::new("cargo");
    cmd.arg("run").current_dir(&raiz);
    if op.release {
        cmd.arg("--release");
    }
    if let Some(features) = &op.features {
        cmd.args(["--features", features]);
    }
    if !op.resto.is_empty() {
        cmd.arg("--").args(&op.resto);
    }

    passo(e, "cargo run");
    exigir_sucesso(cmd.status(), "cargo run")
}

// ── wasm ──────────────────────────────────────────────────────────────────────

fn wasm(e: &Estilo, op: Opcoes) -> io::Result<()> {
    let raiz = raiz_do_projeto()?;

    let pagina = raiz.join("web");
    if !pagina.join("index.html").is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "falta {} — é a página que carrega o app no navegador. \
                 Um projeto novo com ela: `glacier new <nome> --preset wasm32`",
                pagina.join("index.html").display()
            ),
        ));
    }
    exigir_target()?;

    // A porta ANTES do build: descobrir que ela está ocupada depois de minutos
    // de compilação é o pior momento possível.
    let listener = TcpListener::bind(("127.0.0.1", op.porta)).map_err(|erro| {
        io::Error::new(
            erro.kind(),
            format!(
                "não consegui abrir 127.0.0.1:{}: {erro} — tente `--port <outra>`",
                op.porta
            ),
        )
    })?;

    // A versão do `wasm-bindgen` também ANTES do build, pelo mesmo motivo da
    // porta: um projeto novo resolve a versão mais recente do crate, e descobrir
    // que o binário do PATH é outro depois de seis minutos de compilação é o
    // pior momento possível.
    let versao = versao_do_wasm_bindgen(&raiz)?;
    exigir_wasm_bindgen(&versao)?;

    let montagem = montar(e, &raiz, &op, &versao)?;

    let servido = Arc::new(Servido {
        pasta: raiz.join(DIR_SAIDA),
        geracao: AtomicU64::new(0),
        troca: RwLock::new(()),
        watch: op.watch,
    });
    trocar(&servido, &montagem)?;

    let endereco = listener.local_addr()?;
    println!();
    println!(
        "  {} {}",
        e.verde("servindo em"),
        e.negrito(&format!("http://{endereco}/"))
    );
    println!(
        "  {}",
        e.fraco(&format!("pasta: {}", servido.pasta.display()))
    );
    if op.watch {
        println!(
            "  {}",
            e.fraco("vigiando src/, views/, web/ e Cargo.toml — salve um arquivo e a página se recarrega.")
        );
        println!("  {}", e.fraco("Ctrl+C para parar."));
        println!();
        let vigia = Vigia {
            e: *e,
            raiz,
            op,
            versao,
            servido: Arc::clone(&servido),
        };
        std::thread::spawn(move || vigia.rodar());
    } else {
        println!(
            "  {}",
            e.fraco("Ctrl+C para parar. Mudou o código? Rode com `--watch`, ou o comando de novo e recarregue a página.")
        );
        println!();
    }
    atender_para_sempre(listener, servido);
    Ok(())
}

/// Compila e monta a pasta da build em [`DIR_MONTAGEM`], devolvendo o caminho.
///
/// Montar à parte e trocar depois (ver [`trocar`]) é o que deixa a pasta
/// servida sempre inteira: uma build que falha no meio não apaga a anterior, e
/// no `--watch` a página aberta segue funcionando com ela.
fn montar(e: &Estilo, raiz: &Path, op: &Opcoes, versao: &str) -> io::Result<PathBuf> {
    let perfil = if op.release { "--release" } else { "debug" };
    passo(e, &format!("cargo build --target {ALVO_WASM} ({perfil})"));
    let binario = compilar_wasm(raiz, op)?;

    let montagem = raiz.join(DIR_MONTAGEM);
    if montagem.exists() {
        fs::remove_dir_all(&montagem)?;
    }
    fs::create_dir_all(&montagem)?;

    passo(e, &format!("wasm-bindgen {versao}"));
    let status = Command::new("wasm-bindgen")
        .args([
            "--target",
            "web",
            "--no-typescript",
            "--out-name",
            NOME_SAIDA,
        ])
        .arg("--out-dir")
        .arg(&montagem)
        .arg(&binario)
        .status();
    exigir_sucesso(status, "wasm-bindgen")?;
    copiar_dir(&raiz.join("web"), &montagem)?;
    Ok(montagem)
}

/// Põe a build montada no lugar da servida e conta a geração nova.
///
/// A troca acontece com o `RwLock` fechado para escrita: um pedido que caísse
/// no meio dela leria um `.wasm` truncado, e o sintoma no navegador — módulo
/// que não instancia — não diria nada sobre a causa.
fn trocar(servido: &Servido, montagem: &Path) -> io::Result<()> {
    let _guarda = servido.troca.write().unwrap_or_else(|e| e.into_inner());
    if servido.pasta.exists() {
        fs::remove_dir_all(&servido.pasta)?;
    }
    fs::rename(montagem, &servido.pasta)?;
    servido.geracao.fetch_add(1, Ordering::Release);
    Ok(())
}

// ── vigia (`--watch`) ─────────────────────────────────────────────────────────

/// O estado que a thread do vigia precisa para reconstruir do zero.
struct Vigia {
    e: Estilo,
    raiz: PathBuf,
    op: Opcoes,
    versao: String,
    servido: Arc<Servido>,
}

impl Vigia {
    /// Laço infinito: varre, compara, reconstrói. Só termina com o processo
    /// (Ctrl+C), como o servidor.
    fn rodar(self) {
        let mut antes = Impressao::tirar(&self.raiz);
        loop {
            std::thread::sleep(INTERVALO_VIGIA);
            let agora = Impressao::tirar(&self.raiz);
            if agora == antes {
                continue;
            }
            // Deixa o editor terminar de salvar antes de ler a árvore de novo:
            // o que vale é o estado depois da pausa, não o do meio da escrita.
            std::thread::sleep(ACALMAR);
            let depois = Impressao::tirar(&self.raiz);
            let so_pagina = depois.codigo == antes.codigo;
            antes = depois;

            if so_pagina {
                // `web/` não entra no `.wasm` — é cópia, não compilação.
                passo(&self.e, "web/ mudou, recopiando");
                if let Err(erro) = self.recopiar_pagina() {
                    self.falhou(&erro);
                    continue;
                }
            } else {
                passo(&self.e, "mudou, recompilando");
                match montar(&self.e, &self.raiz, &self.op, &self.versao)
                    .and_then(|montagem| trocar(&self.servido, &montagem))
                {
                    Ok(()) => {}
                    Err(erro) => {
                        self.falhou(&erro);
                        continue;
                    }
                }
            }
            println!(
                "  {} {}",
                self.e.verde("pronto"),
                self.e
                    .fraco("— a página aberta se recarrega em até meio segundo")
            );
        }
    }

    /// Copia só `web/` para dentro da pasta servida. Mesma trava da [`trocar`],
    /// e a geração sobe igual: quem está com a página aberta recebe o HTML novo.
    fn recopiar_pagina(&self) -> io::Result<()> {
        let _guarda = self
            .servido
            .troca
            .write()
            .unwrap_or_else(|e| e.into_inner());
        copiar_dir(&self.raiz.join("web"), &self.servido.pasta)?;
        self.servido.geracao.fetch_add(1, Ordering::Release);
        Ok(())
    }

    /// Erro de build no `--watch` NÃO derruba o servidor: a pasta servida
    /// continua com a build anterior, a página aberta segue de pé, e o próximo
    /// salvamento tenta outra vez. Um `exit` aqui obrigaria a redigitar o
    /// comando depois de cada erro de sintaxe.
    fn falhou(&self, erro: &io::Error) {
        eprintln!("  {} {erro}", self.e.vermelho("erro:"));
        eprintln!(
            "  {}",
            self.e
                .fraco("a página segue com a build anterior; corrija e salve de novo")
        );
    }
}

/// O estado do disco que decide se há build nova a fazer, separado pelo que
/// cada parte exige: `web/` é cópia, o resto é compilação.
#[derive(Debug, Default, PartialEq)]
struct Impressao {
    /// `web/` — a página, que não entra no `.wasm`.
    pagina: Vec<Arquivo>,
    /// `src/`, `views/` e `Cargo.toml` — tudo que o `.wasm` embute ou compila.
    codigo: Vec<Arquivo>,
}

/// Um arquivo vigiado, pelo que importa para decidir se ele mudou.
///
/// O CRC do **conteúdo** é o que decide, e não o `mtime`: a granularidade do
/// `mtime` é de um segundo em alguns sistemas de arquivos, e dois salvamentos
/// do mesmo tamanho dentro da mesma marca de tempo passariam batidos — foi
/// exatamente o que o teste desta impressão pegou. O `mtime` e o tamanho ficam
/// na chave porque são de graça (vêm do `metadata` que já foi lido) e porque
/// um arquivo grande que só teve o `mtime` mexido também conta como mudança
/// para o cargo.
type Arquivo = (PathBuf, Option<SystemTime>, u64, u32);

impl Impressao {
    fn tirar(raiz: &Path) -> Self {
        let mut imp = Self::default();
        coletar_impressao(&raiz.join("web"), &mut imp.pagina);
        coletar_impressao(&raiz.join("src"), &mut imp.codigo);
        coletar_impressao(&raiz.join("views"), &mut imp.codigo);
        coletar_impressao(&raiz.join("Cargo.toml"), &mut imp.codigo);
        imp.pagina.sort();
        imp.codigo.sort();
        imp
    }
}

/// Acumula os [`Arquivo`]s de `alvo` (arquivo ou diretório).
///
/// Ler o conteúdo de tudo a cada varredura é aceitável porque o conjunto
/// vigiado é pequeno e escolhido a dedo — `src/`, `views/`, `web/` e o
/// `Cargo.toml`, nunca `target/` — e são os mesmos arquivos que o
/// `embed_assets!` já embute no `.wasm`.
///
/// `.glacier-storage` é pulado: é estado de runtime que o app do desktop grava
/// enquanto roda, e um projeto aberto nos dois alvos recompilaria sem parar.
fn coletar_impressao(alvo: &Path, saida: &mut Vec<Arquivo>) {
    let Ok(meta) = fs::metadata(alvo) else {
        // Não existir é um estado como qualquer outro: o arquivo pode ter sido
        // apagado, e isso também é uma mudança a reconstruir.
        return;
    };
    if meta.is_file() {
        let crc = fs::read(alvo).map(|b| crate::vsix::crc32(&b)).unwrap_or(0);
        saida.push((alvo.to_path_buf(), meta.modified().ok(), meta.len(), crc));
        return;
    }
    let Ok(entradas) = fs::read_dir(alvo) else {
        return;
    };
    for entrada in entradas.flatten() {
        if entrada.file_name() == std::ffi::OsStr::new(".glacier-storage") {
            continue;
        }
        coletar_impressao(&entrada.path(), saida);
    }
}

fn exigir_target() -> io::Result<()> {
    // Sem `rustup` (toolchain da distro, por exemplo) não há como perguntar;
    // o próprio `cargo build` diz o que faltar.
    let Ok(saida) = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    else {
        return Ok(());
    };
    let instalados = String::from_utf8_lossy(&saida.stdout);
    if instalados.lines().any(|l| l.trim() == ALVO_WASM) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "o target {ALVO_WASM} não está instalado — rode: rustup target add {ALVO_WASM}"
            ),
        ))
    }
}

/// Compila e devolve o caminho do `.wasm`. O caminho sai das mensagens JSON do
/// cargo, e não de um `target/...` montado à mão, para respeitar
/// `CARGO_TARGET_DIR` e workspaces. `json-render-diagnostics` mantém os erros
/// de compilação legíveis no terminal.
fn compilar_wasm(raiz: &Path, op: &Opcoes) -> io::Result<PathBuf> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--target",
        ALVO_WASM,
        "--message-format=json-render-diagnostics",
    ])
    .current_dir(raiz)
    .stdout(Stdio::piped());
    if op.release {
        cmd.arg("--release");
    }
    if let Some(features) = &op.features {
        cmd.args(["--features", features]);
    }

    let mut filho = cmd.spawn().map_err(|erro| {
        io::Error::new(erro.kind(), format!("não consegui rodar o cargo: {erro}"))
    })?;
    let mut binario = None;
    if let Some(stdout) = filho.stdout.take() {
        for linha in BufReader::new(stdout).lines() {
            if let Some(caminho) = executavel_wasm(&linha?) {
                binario = Some(caminho);
            }
        }
    }
    if !filho.wait()?.success() {
        return Err(io::Error::other(
            "a compilação para wasm falhou (veja a saída acima)",
        ));
    }
    binario.ok_or_else(|| {
        io::Error::other("o cargo não gerou nenhum .wasm — o projeto tem um `src/main.rs`?")
    })
}

/// O `"executable"` de uma mensagem `compiler-artifact` do cargo, se for um
/// `.wasm`.
fn executavel_wasm(linha: &str) -> Option<PathBuf> {
    if !linha.contains("\"reason\":\"compiler-artifact\"") {
        return None;
    }
    let chave = "\"executable\":\"";
    let inicio = linha.find(chave)? + chave.len();
    let fim = inicio + linha[inicio..].find('"')?;
    // O JSON escapa a barra invertida dos caminhos do Windows.
    let caminho = linha[inicio..fim].replace("\\\\", "\\");
    caminho.ends_with(".wasm").then(|| PathBuf::from(caminho))
}

/// A versão do crate `wasm-bindgen` que ESTE projeto usa.
///
/// Um projeto recém-criado ainda não tem `Cargo.lock` — ele nasceria no build,
/// tarde demais para esta conferência. O `cargo generate-lockfile` resolve as
/// versões e escreve o lock sem compilar nada.
fn versao_do_wasm_bindgen(raiz: &Path) -> io::Result<String> {
    if localizar_acima(raiz, "Cargo.lock").is_none() {
        let status = Command::new("cargo")
            .arg("generate-lockfile")
            .current_dir(raiz)
            .status();
        exigir_sucesso(status, "cargo generate-lockfile")?;
    }
    let lock = localizar_acima(raiz, "Cargo.lock").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "não achei o Cargo.lock do projeto nem consegui gerá-lo",
        )
    })?;
    versao_no_lock(&fs::read_to_string(&lock)?, "wasm-bindgen").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "o Cargo.lock não tem o crate `wasm-bindgen` — o projeto depende do glacier-ui?",
        )
    })
}

/// A versão de um pacote no `Cargo.lock` (`name = "x"` seguido de `version`).
fn versao_no_lock(lock: &str, pacote: &str) -> Option<String> {
    let alvo = format!("name = \"{pacote}\"");
    let mut linhas = lock.lines();
    while let Some(linha) = linhas.next() {
        if linha.trim() == alvo {
            let versao = linhas
                .next()?
                .trim()
                .strip_prefix("version = \"")?
                .strip_suffix('"')?;
            return Some(versao.to_string());
        }
    }
    None
}

fn exigir_wasm_bindgen(versao: &str) -> io::Result<()> {
    // `binstall` baixa o binário pronto (segundos); o `install` recompila
    // (minutos). Quem está travado aqui quer o primeiro.
    let instalar = format!(
        "cargo binstall wasm-bindgen-cli@{versao}   (ou: cargo install wasm-bindgen-cli --version {versao})"
    );
    let saida = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("`wasm-bindgen` não está no PATH — rode: {instalar}"),
            )
        })?;
    // `wasm-bindgen 0.2.126`
    let texto = String::from_utf8_lossy(&saida.stdout);
    let instalada = texto.split_whitespace().nth(1).unwrap_or("?");
    if instalada == versao {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "o `wasm-bindgen` do PATH é {instalada}, mas o Cargo.lock pede {versao}: \
             versões diferentes geram um .js que não casa com o .wasm. Rode: {instalar}"
        )))
    }
}

// ── servidor estático ─────────────────────────────────────────────────────────

/// O que o servidor e o vigia compartilham.
struct Servido {
    /// A pasta servida.
    pasta: PathBuf,
    /// Sobe a cada build que chega ao disco. A página consulta este número na
    /// [`ROTA_RECARGA`] e se recarrega quando ele muda.
    geracao: AtomicU64,
    /// Fechado enquanto a pasta é trocada (ver [`trocar`]).
    troca: RwLock<()>,
    /// Com `--watch`, a rota de recarga existe e a `index.html` servida recebe
    /// o [`SCRIPT_RECARGA`].
    watch: bool,
}

fn atender_para_sempre(listener: TcpListener, servido: Arc<Servido>) {
    for conexao in listener.incoming() {
        let Ok(stream) = conexao else { continue };
        let servido = Arc::clone(&servido);
        std::thread::spawn(move || {
            // Uma conexão que cai no meio (aba fechada durante o download do
            // .wasm) não é problema de ninguém.
            let _ = atender(stream, &servido);
        });
    }
}

fn atender(stream: TcpStream, servido: &Servido) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut leitor = BufReader::new(&stream);

    let mut primeira = String::new();
    leitor.read_line(&mut primeira)?;
    // Os cabeçalhos não mudam a resposta de um servidor estático, mas precisam
    // ser lidos: fechar a conexão com dados não lidos faz o SO mandar RST, e o
    // navegador descarta a resposta.
    loop {
        let mut linha = String::new();
        if leitor.read_line(&mut linha)? == 0 || linha.trim_end().is_empty() {
            break;
        }
    }

    let mut partes = primeira.split_whitespace();
    let metodo = partes.next().unwrap_or("");
    let alvo = partes.next().unwrap_or("");
    let resposta = resolver(servido, metodo, alvo);
    escrever(&stream, &resposta, metodo == "HEAD")
}

#[derive(Debug)]
struct Resposta {
    status: &'static str,
    tipo: &'static str,
    corpo: Vec<u8>,
}

fn resolver(servido: &Servido, metodo: &str, alvo: &str) -> Resposta {
    if metodo != "GET" && metodo != "HEAD" {
        return texto("405 Method Not Allowed", "só GET e HEAD");
    }
    // A única rota que não sai do disco. A página pergunta por ela em laço; a
    // resposta é o número da build atual.
    if servido.watch && alvo.split(['?', '#']).next() == Some(ROTA_RECARGA) {
        let geracao = servido.geracao.load(Ordering::Acquire);
        return Resposta {
            status: "200 OK",
            tipo: "text/plain; charset=utf-8",
            corpo: geracao.to_string().into_bytes(),
        };
    }
    let Some(relativo) = caminho_seguro(alvo) else {
        return texto("400 Bad Request", "caminho inválido");
    };

    // Ler com a trava de leitura: enquanto o vigia troca a pasta, o pedido
    // espera os poucos milissegundos do `rename` em vez de ler um arquivo pela
    // metade.
    let _guarda = servido.troca.read().unwrap_or_else(|e| e.into_inner());

    let mut arquivo = servido.pasta.join(relativo);
    if arquivo.is_dir() {
        arquivo = arquivo.join("index.html");
    }
    match fs::read(&arquivo) {
        Ok(mut corpo) => {
            let ehindex = arquivo.file_name().and_then(|n| n.to_str()) == Some("index.html");
            if servido.watch
                && ehindex
                && let Ok(html) = std::str::from_utf8(&corpo)
            {
                corpo = injetar_recarga(html).into_bytes();
            }
            Resposta {
                status: "200 OK",
                tipo: tipo_mime(&arquivo),
                corpo,
            }
        }
        Err(_) => texto("404 Not Found", &format!("não encontrado: {alvo}")),
    }
}

/// Enfia o [`SCRIPT_RECARGA`] antes do `</body>` do HTML **servido** — o
/// arquivo no disco não é tocado, e por isso o `--watch` não deixa resíduo no
/// projeto de ninguém.
///
/// Sem `</body>` (uma página mínima é HTML válido sem ele), vai no fim.
fn injetar_recarga(html: &str) -> String {
    match html.rfind("</body>") {
        Some(i) => format!("{}{SCRIPT_RECARGA}{}", &html[..i], &html[i..]),
        None => format!("{html}{SCRIPT_RECARGA}"),
    }
}

fn texto(status: &'static str, mensagem: &str) -> Resposta {
    Resposta {
        status,
        tipo: "text/plain; charset=utf-8",
        corpo: mensagem.as_bytes().to_vec(),
    }
}

fn escrever(mut saida: impl Write, resposta: &Resposta, so_cabecalho: bool) -> io::Result<()> {
    write!(
        saida,
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        resposta.status,
        resposta.tipo,
        resposta.corpo.len()
    )?;
    if !so_cabecalho {
        saida.write_all(&resposta.corpo)?;
    }
    saida.flush()
}

/// `/a/b%20c.js?v=1` → `a/b c.js`. `None` para qualquer coisa que tente sair da
/// pasta servida (`..`, caminho absoluto, prefixo de unidade do Windows).
fn caminho_seguro(alvo: &str) -> Option<PathBuf> {
    let sem_query = alvo.split(['?', '#']).next()?;
    if !sem_query.starts_with('/') {
        return None;
    }
    let decodificado = decodificar(sem_query)?;
    let mut relativo = PathBuf::new();
    for parte in Path::new(decodificado.trim_start_matches('/')).components() {
        match parte {
            Component::Normal(nome) => relativo.push(nome),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(relativo)
}

/// Decodificação de `%XX`. Uma sequência malformada recusa o caminho inteiro.
fn decodificar(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut saida = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            saida.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            saida.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(saida).ok()
}

/// `application/wasm` é o que importa: sem ele o `WebAssembly.instantiateStreaming`
/// do `app.js` falha e cai num caminho mais lento, com aviso no console.
fn tipo_mime(caminho: &Path) -> &'static str {
    let extensao = caminho
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match extensao.as_deref() {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

// ── utilitários ───────────────────────────────────────────────────────────────

fn raiz_do_projeto() -> io::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    localizar_acima(&cwd, "Cargo.toml")
        .and_then(|manifesto| manifesto.parent().map(Path::to_path_buf))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "nenhum Cargo.toml em {} nem acima — rode dentro de um projeto glacier",
                    cwd.display()
                ),
            )
        })
}

/// O primeiro `nome` em `dir` ou num diretório acima dele (um `Cargo.lock` de
/// workspace mora na raiz do workspace, não na do pacote).
fn localizar_acima(dir: &Path, nome: &str) -> Option<PathBuf> {
    dir.ancestors()
        .map(|d| d.join(nome))
        .find(|caminho| caminho.is_file())
}

fn copiar_dir(origem: &Path, destino: &Path) -> io::Result<()> {
    fs::create_dir_all(destino)?;
    for entrada in fs::read_dir(origem)? {
        let entrada = entrada?;
        let alvo = destino.join(entrada.file_name());
        if entrada.file_type()?.is_dir() {
            copiar_dir(&entrada.path(), &alvo)?;
        } else {
            fs::copy(entrada.path(), alvo)?;
        }
    }
    Ok(())
}

fn passo(e: &Estilo, texto: &str) {
    println!("{} {}", e.verde("▸"), e.negrito(texto));
}

fn exigir_sucesso(status: io::Result<ExitStatus>, programa: &str) -> io::Result<()> {
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Err(io::Error::other(format!(
            "`{programa}` falhou (veja a saída acima)"
        ))),
        Err(erro) => Err(io::Error::new(
            erro.kind(),
            format!("não consegui rodar `{programa}`: {erro}"),
        )),
    }
}

fn invalido(mensagem: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, mensagem)
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::io::Read;

    fn args(v: &[&str]) -> std::vec::IntoIter<String> {
        v.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn wasm_e_release_por_padrao_e_dev_desliga() {
        let op = opcoes(args(&[]), Alvo::Wasm).unwrap();
        assert!(op.release);
        assert_eq!(op.porta, PORTA_PADRAO);

        let op = opcoes(
            args(&["--dev", "--port", "9000", "-F", "web-gpu"]),
            Alvo::Wasm,
        )
        .unwrap();
        assert!(!op.release);
        assert_eq!(op.porta, 9000);
        assert_eq!(op.features.as_deref(), Some("web-gpu"));
    }

    #[test]
    fn desktop_repassa_o_que_vem_depois_do_separador() {
        let op = opcoes(args(&["--release", "--", "--dev", "x"]), Alvo::Desktop).unwrap();
        assert!(op.release);
        assert_eq!(op.resto, ["--dev", "x"]);
    }

    #[test]
    fn opcao_de_um_alvo_e_recusada_no_outro() {
        assert!(opcoes(args(&["--port", "1"]), Alvo::Desktop).is_err());
        assert!(opcoes(args(&["--release"]), Alvo::Wasm).is_err());
        assert!(opcoes(args(&["--port"]), Alvo::Wasm).is_err());
        assert!(opcoes(args(&["--port", "abc"]), Alvo::Wasm).is_err());
        assert!(opcoes(args(&["--features"]), Alvo::Wasm).is_err());
    }

    #[test]
    fn caminho_seguro_nunca_sai_da_pasta() {
        assert_eq!(caminho_seguro("/"), Some(PathBuf::new()));
        assert_eq!(caminho_seguro("/app.js?v=2"), Some(PathBuf::from("app.js")));
        assert_eq!(
            caminho_seguro("/a/b%20c.png"),
            Some(PathBuf::from("a/b c.png"))
        );
        assert_eq!(caminho_seguro("/../Cargo.toml"), None);
        assert_eq!(caminho_seguro("/a/%2E%2E/%2E%2E/segredo"), None);
        assert_eq!(caminho_seguro("sem-barra"), None);
        assert_eq!(caminho_seguro("/%zz"), None);
    }

    #[test]
    fn wasm_sai_como_application_wasm() {
        assert_eq!(tipo_mime(Path::new("x/app_bg.wasm")), "application/wasm");
        assert_eq!(
            tipo_mime(Path::new("app.JS")),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            tipo_mime(Path::new("sem_extensao")),
            "application/octet-stream"
        );
    }

    #[test]
    fn le_a_versao_do_wasm_bindgen_no_lock() {
        let lock = "[[package]]\nname = \"wasm-bindgen-futures\"\nversion = \"0.4.76\"\n\n\
                    [[package]]\nname = \"wasm-bindgen\"\nversion = \"0.2.126\"\n";
        assert_eq!(
            versao_no_lock(lock, "wasm-bindgen").as_deref(),
            Some("0.2.126")
        );
        assert_eq!(versao_no_lock(lock, "iced"), None);
    }

    #[test]
    fn acha_o_wasm_na_mensagem_do_cargo() {
        let artefato = r#"{"reason":"compiler-artifact","target":{"kind":["bin"]},"executable":"/p/target/wasm32-unknown-unknown/release/meu-app.wasm","fresh":false}"#;
        assert_eq!(
            executavel_wasm(artefato),
            Some(PathBuf::from(
                "/p/target/wasm32-unknown-unknown/release/meu-app.wasm"
            ))
        );
        let lib = r#"{"reason":"compiler-artifact","executable":null}"#;
        assert_eq!(executavel_wasm(lib), None);
        assert_eq!(
            executavel_wasm(r#"{"reason":"build-finished","success":true}"#),
            None
        );
    }

    /// O servidor de verdade, numa porta livre: os cabeçalhos que o navegador
    /// precisa, e nada fora da pasta.
    #[test]
    fn servidor_responde_com_tipo_certo_e_sem_cache() {
        let pasta = std::env::temp_dir().join(format!("glacier-serve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&pasta);
        fs::create_dir_all(&pasta).unwrap();
        fs::write(pasta.join("index.html"), "<p>oi</p>").unwrap();
        fs::write(pasta.join("app_bg.wasm"), [0u8, 97, 115, 109]).unwrap();

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let endereco = listener.local_addr().unwrap();
        std::thread::spawn({
            let servido = servido(&pasta, false);
            move || atender_para_sempre(listener, servido)
        });

        let pedir = |linha: &str| {
            let mut s = TcpStream::connect(endereco).unwrap();
            write!(s, "{linha}\r\nHost: x\r\n\r\n").unwrap();
            let mut resposta = String::new();
            s.read_to_string(&mut resposta).unwrap();
            resposta
        };

        let raiz = pedir("GET / HTTP/1.1");
        assert!(raiz.starts_with("HTTP/1.1 200 OK"), "{raiz}");
        assert!(raiz.contains("Content-Type: text/html"), "{raiz}");
        assert!(raiz.contains("Cache-Control: no-store"), "{raiz}");
        assert!(raiz.ends_with("<p>oi</p>"), "{raiz}");

        let wasm = pedir("GET /app_bg.wasm HTTP/1.1");
        assert!(wasm.contains("Content-Type: application/wasm"), "{wasm}");
        assert!(wasm.contains("Content-Length: 4"), "{wasm}");

        assert!(pedir("GET /../Cargo.toml HTTP/1.1").starts_with("HTTP/1.1 400"));
        assert!(pedir("GET /nada.js HTTP/1.1").starts_with("HTTP/1.1 404"));
        assert!(pedir("POST / HTTP/1.1").starts_with("HTTP/1.1 405"));

        let head = pedir("HEAD / HTTP/1.1");
        assert!(
            head.contains("Content-Length: 9") && !head.contains("<p>"),
            "{head}"
        );

        let _ = fs::remove_dir_all(&pasta);
    }

    fn servido(pasta: &Path, watch: bool) -> Arc<Servido> {
        Arc::new(Servido {
            pasta: pasta.to_path_buf(),
            geracao: AtomicU64::new(7),
            troca: RwLock::new(()),
            watch,
        })
    }

    #[test]
    fn watch_e_so_do_wasm() {
        assert!(opcoes(args(&["--watch"]), Alvo::Wasm).unwrap().watch);
        assert!(opcoes(args(&["-w"]), Alvo::Wasm).unwrap().watch);
        assert!(!opcoes(args(&[]), Alvo::Wasm).unwrap().watch);
        assert!(opcoes(args(&["--watch"]), Alvo::Desktop).is_err());
    }

    /// O script entra antes do `</body>`, e o arquivo no disco não muda.
    #[test]
    fn a_recarga_e_injetada_antes_do_fecho_do_body() {
        let html = "<html><body><div id=\"iced\"></div></body></html>";
        let saida = injetar_recarga(html);
        assert!(saida.contains("__glacier/recarregar"), "{saida}");
        let script = saida.find("<script>").unwrap();
        let body = saida.find("</body>").unwrap();
        assert!(
            script < body,
            "o script tem de vir ANTES do </body>:\n{saida}"
        );
        assert!(saida.contains("<div id=\"iced\">"), "{saida}");

        // Sem `</body>` (HTML válido) o script vai no fim, não some.
        let solto = injetar_recarga("<div id=\"iced\"></div>");
        assert!(solto.trim_end().ends_with("</script>"), "{solto}");
    }

    /// Sem `--watch` não existe rota de recarga nem injeção: o `serve` de
    /// sempre serve a página exatamente como ela está no disco.
    #[test]
    fn sem_watch_nao_ha_rota_nem_script() {
        let pasta = std::env::temp_dir().join(format!("glacier-sem-watch-{}", std::process::id()));
        let _ = fs::remove_dir_all(&pasta);
        fs::create_dir_all(&pasta).unwrap();
        fs::write(pasta.join("index.html"), "<body>oi</body>").unwrap();

        let s = servido(&pasta, false);
        let pagina = resolver(&s, "GET", "/index.html");
        assert_eq!(pagina.corpo, b"<body>oi</body>");
        assert!(resolver(&s, "GET", ROTA_RECARGA).status.starts_with("404"));

        let _ = fs::remove_dir_all(&pasta);
    }

    /// Com `--watch`, a rota devolve a geração (é o que a página compara) e a
    /// `index.html` sai com o script.
    #[test]
    fn com_watch_a_rota_devolve_a_geracao_e_o_index_leva_o_script() {
        let pasta = std::env::temp_dir().join(format!("glacier-com-watch-{}", std::process::id()));
        let _ = fs::remove_dir_all(&pasta);
        fs::create_dir_all(&pasta).unwrap();
        fs::write(pasta.join("index.html"), "<body>oi</body>").unwrap();

        let s = servido(&pasta, true);
        let recarga = resolver(&s, "GET", ROTA_RECARGA);
        assert_eq!(recarga.corpo, b"7");
        assert_eq!(recarga.tipo, "text/plain; charset=utf-8");
        // Com query, que é como o `fetch` costuma chegar depois de um proxy.
        assert_eq!(resolver(&s, "GET", "/__glacier/recarregar?t=1").corpo, b"7");

        let pagina = String::from_utf8(resolver(&s, "GET", "/").corpo).unwrap();
        assert!(pagina.contains("__glacier/recarregar"), "{pagina}");
        assert!(pagina.contains("oi"), "{pagina}");

        // O `.wasm` não é tocado pela injeção.
        fs::write(pasta.join("app_bg.wasm"), [0u8, 97, 115, 109]).unwrap();
        assert_eq!(resolver(&s, "GET", "/app_bg.wasm").corpo, [0, 97, 115, 109]);

        let _ = fs::remove_dir_all(&pasta);
    }

    /// A impressão do disco separa `web/` (cópia) do resto (compilação), e vê
    /// mudança de conteúdo do mesmo tamanho — é o `mtime` que a pega.
    #[test]
    fn a_impressao_separa_a_pagina_do_codigo() {
        let raiz = std::env::temp_dir().join(format!("glacier-impressao-{}", std::process::id()));
        let _ = fs::remove_dir_all(&raiz);
        fs::create_dir_all(raiz.join("web")).unwrap();
        fs::create_dir_all(raiz.join("views/styles")).unwrap();
        fs::write(raiz.join("Cargo.toml"), "[package]").unwrap();
        fs::write(raiz.join("web/index.html"), "<body>a</body>").unwrap();
        fs::write(raiz.join("views/app.gv"), "<column/>").unwrap();

        let antes = Impressao::tirar(&raiz);
        assert_eq!(antes.pagina.len(), 1);
        assert_eq!(antes.codigo.len(), 2, "views/app.gv + Cargo.toml");

        // Mudança só na página, do MESMO tamanho e possivelmente no mesmo
        // `mtime` — é o CRC do conteúdo que a pega. O código fica igual, e é o
        // que faz o vigia recopiar em vez de recompilar.
        fs::write(raiz.join("web/index.html"), "<body>b</body>").unwrap();
        assert_eq!(
            fs::metadata(raiz.join("web/index.html")).unwrap().len(),
            14,
            "as duas versões têm de ter o mesmo tamanho, é o que o teste cobre"
        );
        let depois = Impressao::tirar(&raiz);
        assert_ne!(depois.pagina, antes.pagina);
        assert_eq!(depois.codigo, antes.codigo);

        // Arquivo novo em views/ conta como mudança de código.
        fs::write(raiz.join("views/styles/app.gss"), ".x{}").unwrap();
        let terceira = Impressao::tirar(&raiz);
        assert_ne!(terceira.codigo, depois.codigo);
        assert_eq!(terceira.codigo.len(), 3);

        // E o storage do desktop é ignorado: ele muda com o app rodando, e
        // sozinho faria o vigia recompilar sem parar.
        fs::create_dir_all(raiz.join("views/.glacier-storage")).unwrap();
        fs::write(raiz.join("views/.glacier-storage/dados.json"), "{}").unwrap();
        assert_eq!(Impressao::tirar(&raiz).codigo, terceira.codigo);

        let _ = fs::remove_dir_all(&raiz);
    }
}
