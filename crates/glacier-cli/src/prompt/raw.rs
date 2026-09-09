//! Modo raw do terminal para o questionário navegável por setas — **sem
//! dependência de crate**: entra e sai do modo raw chamando o `stty` do
//! sistema (POSIX), e lê o stdin byte a byte.
//!
//! `stty` está em qualquer Linux/macOS/BSD. Onde ele não existe (Windows, um
//! shell mínimo) [`Raw::ativar`] devolve `None` e o `prompt` cai no menu
//! numerado de sempre.

use std::io::{self, Read};
use std::process::{Command, Stdio};

/// Enquanto viva, o terminal está em modo raw (sem eco, sem buffer de linha).
/// O `Drop` restaura os ajustes originais — inclusive num `panic`.
pub struct Raw {
    original: String,
    /// Bytes já lidos do stdin e ainda não consumidos por [`Raw::ler_tecla`].
    /// Um autorepeat de seta (ou colar) chega como uma rajada; sem isto, o
    /// resto da rajada seria descartado a cada `read`.
    pendentes: Vec<u8>,
}

impl Raw {
    /// `Some` se o terminal entrou em modo raw; `None` se não deu (sem `stty`,
    /// stdin não é um tty, etc.) — a chamada então usa o caminho numerado.
    pub fn ativar() -> Option<Self> {
        let original = stty(&["-g"])?.trim().to_string();
        if original.is_empty() {
            return None;
        }
        // -echo: não repete o que foi digitado. -icanon: entrega cada tecla na
        // hora, sem esperar o Enter. -isig: Ctrl-C/Ctrl-Z chegam como BYTE
        // (`\x03`/`\x1a`) em vez de sinal — assim o `Drop` restaura o terminal
        // antes de sair (um SIGINT mataria o processo com o terminal ainda
        // sem eco). `opost` fica LIGADO: a saída segue traduzindo `\n`.
        // min 1 / time 0: `read` bloqueia até haver ao menos 1 byte.
        stty(&["-echo", "-icanon", "-isig", "min", "1", "time", "0"])?;
        Some(Self {
            original,
            pendentes: Vec::new(),
        })
    }

    /// Lê a próxima tecla, consumindo a rajada de bytes byte a byte. Bloqueia
    /// só quando não há nada pendente.
    pub fn ler_tecla(&mut self) -> io::Result<Tecla> {
        if self.pendentes.is_empty() {
            let mut buf = [0u8; 16];
            let n = io::stdin().read(&mut buf)?;
            if n == 0 {
                return Ok(Tecla::Sair); // EOF (Ctrl-D)
            }
            self.pendentes.extend_from_slice(&buf[..n]);
        }
        Ok(self.proxima())
    }

    /// Decodifica e remove uma tecla do início de `pendentes`.
    fn proxima(&mut self) -> Tecla {
        let p = &self.pendentes;
        let (tecla, consumir) = match p.as_slice() {
            [b'\r', ..] | [b'\n', ..] => (Tecla::Enter, 1),
            [3, ..] | [26, ..] => (Tecla::Interromper, 1), // Ctrl-C / Ctrl-Z
            [0x1b, b'[', b'A', ..] | [0x1b, b'O', b'A', ..] => (Tecla::Cima, 3),
            [0x1b, b'[', b'B', ..] | [0x1b, b'O', b'B', ..] => (Tecla::Baixo, 3),
            [0x1b, b'[', ..] => (Tecla::Outra, 3), // outra seta / F-key
            [0x1b, ..] => (Tecla::Sair, p.len()),  // Esc sozinho
            [b'q', ..] | [b'Q', ..] => (Tecla::Sair, 1),
            [b'k', ..] => (Tecla::Cima, 1),
            [b'j', ..] => (Tecla::Baixo, 1),
            _ => (Tecla::Outra, 1),
        };
        self.pendentes.drain(..consumir.min(self.pendentes.len()));
        tecla
    }
}

impl Drop for Raw {
    fn drop(&mut self) {
        let _ = stty(&[self.original.as_str()]);
    }
}

/// Roda `stty ARGS` contra o terminal (o `stdin` herdado é o tty que ele
/// ajusta). Devolve a saída em texto, ou `None` se o comando falhou/sumiu.
fn stty(args: &[&str]) -> Option<String> {
    let saida = Command::new("stty")
        .args(args)
        .stdin(Stdio::inherit())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    saida
        .status
        .success()
        .then(|| String::from_utf8_lossy(&saida.stdout).into_owned())
}

/// As teclas que o menu entende. Tudo mais é [`Tecla::Outra`] (ignorada).
pub enum Tecla {
    Cima,
    Baixo,
    Enter,
    /// Esc, `q` ou EOF — cancela.
    Sair,
    /// Ctrl-C — aborta o processo.
    Interromper,
    Outra,
}
