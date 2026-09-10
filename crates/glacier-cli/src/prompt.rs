//! Perguntas de terminal, em std puro.
//!
//! O questionário é **navegável pelas setas** (↑/↓ movem, Enter escolhe): o
//! módulo [`raw`] entra em modo raw chamando o `stty` do sistema, sem
//! dependência de crate. Onde `stty` não existe (Windows, shell mínimo) ou o
//! stdin não é um tty, tudo cai no **menu numerado** de sempre — que também
//! funciona com a entrada redirecionada (`echo 2 | glacier new`).

use std::io::{self, BufRead, IsTerminal, Write};

mod raw;

/// Códigos ANSI, desligados quando a saída não é um terminal (log/pipe).
pub struct Estilo {
    ativo: bool,
}

impl Estilo {
    pub fn detectar() -> Self {
        // NO_COLOR é a convenção de fato (no-color.org).
        let ativo = io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
        Self { ativo }
    }
    pub fn negrito(&self, s: &str) -> String {
        self.envolver(s, "\x1b[1m")
    }
    pub fn ciano(&self, s: &str) -> String {
        self.envolver(s, "\x1b[36m")
    }
    pub fn fraco(&self, s: &str) -> String {
        self.envolver(s, "\x1b[2m")
    }
    pub fn verde(&self, s: &str) -> String {
        self.envolver(s, "\x1b[32m")
    }
    pub fn vermelho(&self, s: &str) -> String {
        self.envolver(s, "\x1b[31m")
    }
    fn envolver(&self, s: &str, codigo: &str) -> String {
        if self.ativo {
            format!("{codigo}{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
}

/// `true` quando dá para perguntar: sem TTY (CI, pipe) o questionário é pulado
/// e todo mundo fica com o default, que é o comportamento previsível.
pub fn interativo() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// Lê uma linha; `None` no EOF (Ctrl-D), que a chamada trata como "aborta".
fn ler_linha() -> Option<String> {
    let mut buf = String::new();
    let n = io::stdin().lock().read_line(&mut buf).ok()?;
    if n == 0 {
        return None;
    }
    Some(buf.trim().to_string())
}

/// Texto livre com default. Enter aceita o default.
pub fn texto(e: &Estilo, pergunta: &str, padrao: &str) -> String {
    print!("{} {} ", e.verde("?"), e.negrito(pergunta));
    print!("{} ", e.fraco(&format!("({padrao})")));
    let _ = io::stdout().flush();

    // Sem repetição: qualquer texto serve como nome, e a validação de verdade
    // (`scaffold::validar_nome`) acontece uma vez, com a mensagem certa.
    match ler_linha() {
        Some(s) if !s.is_empty() => s,
        _ => padrao.to_string(),
    }
}

/// Sim/não com default. Setas ↑/↓ + Enter; sem modo raw, cai no `[S/n]` de texto.
pub fn confirmar(e: &Estilo, pergunta: &str, padrao: bool) -> bool {
    let opcoes = [("Sim", ""), ("Não", "")];
    let inicial = if padrao { 0 } else { 1 };
    match menu_setas(e, pergunta, &opcoes, inicial) {
        Some(i) => i == 0,
        None => confirmar_texto(e, pergunta, padrao),
    }
}

fn confirmar_texto(e: &Estilo, pergunta: &str, padrao: bool) -> bool {
    let dica = if padrao { "S/n" } else { "s/N" };
    loop {
        print!(
            "{} {} {} ",
            e.verde("?"),
            e.negrito(pergunta),
            e.fraco(&format!("[{dica}]"))
        );
        let _ = io::stdout().flush();

        let Some(resposta) = ler_linha() else {
            return padrao;
        };
        match resposta.to_lowercase().as_str() {
            "" => return padrao,
            "s" | "sim" | "y" | "yes" => return true,
            "n" | "nao" | "não" | "no" => return false,
            _ => println!("  {}", e.vermelho("responda s ou n.")),
        }
    }
}

/// Menu de escolha. Setas ↑/↓ + Enter; sem modo raw, cai no menu numerado.
/// Devolve o índice escolhido; o default é `padrao`.
pub fn escolher(e: &Estilo, pergunta: &str, opcoes: &[(&str, &str)], padrao: usize) -> usize {
    menu_setas(e, pergunta, opcoes, padrao).unwrap_or_else(|| escolher_numerado(e, pergunta, opcoes, padrao))
}

/// O menu navegável. `None` = não deu para entrar em modo raw (a chamada usa o
/// caminho de texto/numerado).
///
/// **Uma linha física por item** (o rótulo, truncado à largura do terminal) e
/// mais uma linha para a descrição do item selecionado — nada quebra em duas,
/// então o redesenho pode subir um número FIXO de linhas (`\x1b[<n>A`) sem
/// drift. Só a *entrada* vira raw; a saída segue com o `\n` normal.
fn menu_setas(e: &Estilo, pergunta: &str, opcoes: &[(&str, &str)], padrao: usize) -> Option<usize> {
    let mut raw = raw::Raw::ativar()?;

    let com_descricao = opcoes.iter().any(|(_, d)| !d.is_empty());
    // linhas redesenhadas: uma por opção + a da descrição (se houver alguma).
    let total_linhas = opcoes.len() + usize::from(com_descricao);
    // Largura para truncar; sem `stty size`, um teto conservador.
    let largura = raw::colunas().unwrap_or(80).max(24);

    println!("{} {}", e.verde("?"), e.negrito(pergunta));
    println!("{}", e.fraco("  ↑/↓ move · Enter escolhe · Esc cancela"));
    let mut sel = padrao.min(opcoes.len().saturating_sub(1));
    desenhar(e, opcoes, sel, com_descricao, largura);

    loop {
        match raw.ler_tecla().ok()? {
            raw::Tecla::Cima => sel = (sel + opcoes.len() - 1) % opcoes.len(),
            raw::Tecla::Baixo => sel = (sel + 1) % opcoes.len(),
            raw::Tecla::Enter => return Some(sel),
            raw::Tecla::Sair => {
                drop(raw);
                println!("{}", e.vermelho("  cancelado."));
                std::process::exit(130);
            }
            raw::Tecla::Interromper => {
                drop(raw);
                std::process::exit(130);
            }
            raw::Tecla::Outra => continue,
        }
        print!("\x1b[{total_linhas}A");
        desenhar(e, opcoes, sel, com_descricao, largura);
    }
}

/// Corta `s` para caber em `max` COLUNAS (aprox.: 1 char = 1 coluna), com `…` se
/// sobrou texto. A cor é aplicada DEPOIS do corte, para o código ANSI não
/// contar como largura.
fn cortar(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let corte = max.saturating_sub(1);
    let mut r: String = s.chars().take(corte).collect();
    r.push('…');
    r
}

/// (Re)desenha o menu: uma linha por item + a descrição do selecionado.
/// `\x1b[K` no fim de cada linha apaga o que sobrou de um render anterior mais
/// longo.
fn desenhar(e: &Estilo, opcoes: &[(&str, &str)], sel: usize, com_descricao: bool, largura: usize) {
    let mut buf = String::new();
    for (i, (titulo, _)) in opcoes.iter().enumerate() {
        let rotulo = cortar(titulo, largura.saturating_sub(4));
        if i == sel {
            buf.push_str(&format!("{} {}\x1b[K\n", e.ciano("❯"), e.ciano(&rotulo)));
        } else {
            buf.push_str(&format!("  {rotulo}\x1b[K\n"));
        }
    }
    if com_descricao {
        let desc = cortar(opcoes[sel].1, largura.saturating_sub(6));
        buf.push_str(&format!("    {}\x1b[K\n", e.fraco(&desc)));
    }
    print!("{buf}");
    let _ = io::stdout().flush();
}

/// Menu numerado — o caminho sem modo raw. Enter aceita `padrao`.
fn escolher_numerado(e: &Estilo, pergunta: &str, opcoes: &[(&str, &str)], padrao: usize) -> usize {
    println!("{} {}", e.verde("?"), e.negrito(pergunta));
    for (i, (titulo, descricao)) in opcoes.iter().enumerate() {
        let marca = if i == padrao { "›" } else { " " };
        println!(
            "  {marca} {}  {}",
            e.ciano(&format!("{}", i + 1)),
            e.negrito(titulo)
        );
        if !descricao.is_empty() {
            println!("      {}", e.fraco(descricao));
        }
    }

    loop {
        print!(
            "  {} ",
            e.fraco(&format!("escolha 1-{} ({})", opcoes.len(), padrao + 1))
        );
        let _ = io::stdout().flush();

        let Some(resposta) = ler_linha() else {
            return padrao;
        };
        if resposta.is_empty() {
            return padrao;
        }
        match resposta.parse::<usize>() {
            Ok(n) if (1..=opcoes.len()).contains(&n) => return n - 1,
            _ => println!("  {}", e.vermelho("número fora da lista.")),
        }
    }
}
