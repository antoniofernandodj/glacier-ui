//! Liga `tools/servidor_rede.py` (porta 8096) para os exemplos de rede e o desliga ao sair.
//!
//! `let _servidor = servidor_teste::ligar();` no começo do `main`. Fechar a janela solta o
//! guard (que mata o servidor); o Ctrl+C no terminal manda SIGINT ao grupo de processos
//! inteiro, então o servidor também cai. Se já houver um servidor na porta (você o ligou à
//! mão), ele é reaproveitado e deixado em paz.

use std::net::{SocketAddr, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const ENDERECO: &str = "127.0.0.1:8096";

pub struct Servidor(Option<Child>);

fn no_ar() -> bool {
    let addr: SocketAddr = ENDERECO.parse().unwrap();
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

pub fn ligar() -> Servidor {
    if no_ar() {
        eprintln!("[servidor_teste] já há um servidor em {ENDERECO}; vou usá-lo");
        return Servidor(None);
    }
    let filho = Command::new("python3")
        .args(["tools/servidor_rede.py", "8096", "--sair-com-o-pai"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let filho = match filho {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[servidor_teste] não consegui rodar python3 tools/servidor_rede.py: {e}");
            return Servidor(None);
        }
    };
    let limite = Instant::now() + Duration::from_secs(5);
    while !no_ar() && Instant::now() < limite {
        std::thread::sleep(Duration::from_millis(100));
    }
    eprintln!("[servidor_teste] servidor ligado em {ENDERECO}");
    Servidor(Some(filho))
}

impl Drop for Servidor {
    fn drop(&mut self) {
        if let Some(f) = &mut self.0 {
            let _ = f.kill();
            let _ = f.wait();
        }
    }
}
