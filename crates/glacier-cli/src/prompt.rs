//! Perguntas de terminal, em std puro.
//!
//! Menu numerado em vez de seleção por setas: setas exigem modo raw (termios),
//! e o custo disso seria uma dependência — justamente o que esta CLI evita para
//! que `cargo install glacier-cli` leve segundos. Um menu numerado ainda tem a
//! vantagem de funcionar com a entrada redirecionada (`echo 2 | glacier new`).

use std::io::{self, BufRead, IsTerminal, Write};

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

/// Sim/não com default. Aceita s/sim/y/yes e n/nao/não/no.
pub fn confirmar(e: &Estilo, pergunta: &str, padrao: bool) -> bool {
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

/// Menu numerado. Devolve o índice escolhido; Enter aceita `padrao`.
pub fn escolher(e: &Estilo, pergunta: &str, opcoes: &[(&str, &str)], padrao: usize) -> usize {
    println!("{} {}", e.verde("?"), e.negrito(pergunta));
    for (i, (titulo, descricao)) in opcoes.iter().enumerate() {
        let marca = if i == padrao { "›" } else { " " };
        println!(
            "  {marca} {}  {}",
            e.ciano(&format!("{}", i + 1)),
            e.negrito(titulo)
        );
        println!("      {}", e.fraco(descricao));
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
