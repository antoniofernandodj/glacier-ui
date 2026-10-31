//! Painel de referência da Fase 0 — ver `docs/MEDICOES.md`.
//!
//!   WGPU_BACKEND=gl cargo run --release --example painel
//!
//! O gráfico vem de um `fetch` a `http://127.0.0.1:8098/dados.json`, e este `main` **sobe esse
//! servidor sozinho**: uma thread que serve o `dados.json` e morre junto com o processo (não
//! há nada para desligar depois). Se a porta já estiver ocupada — o `make medir` sobe o dele
//! (`python3 -m http.server`) antes de abrir o painel —, não faz nada e o painel usa o que
//! estiver lá.

use std::io::{Read, Write};
use std::net::TcpListener;

use glacier_ui::GlacierDaemon;

const PORTA: u16 = 8098;
/// Os mesmos bytes que o `python3 -m http.server` serviria da pasta do painel.
const DADOS: &str = include_str!("dados.json");

/// Sobe o servidor de dados numa thread. `false` se a porta já tem dono.
fn servidor_de_dados() -> bool {
    let Ok(escuta) = TcpListener::bind(("127.0.0.1", PORTA)) else {
        return false;
    };
    // Thread de fundo: quando o processo sai, ela vai junto.
    std::thread::spawn(move || {
        for conexao in escuta.incoming().flatten() {
            let mut c = conexao;
            let mut pedido = [0u8; 1024];
            let n = c.read(&mut pedido).unwrap_or(0);
            let linha = String::from_utf8_lossy(&pedido[..n]);
            let caminho = linha.split_whitespace().nth(1).unwrap_or("/");
            let resposta = if caminho.split('?').next() == Some("/dados.json") {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{DADOS}",
                    DADOS.len()
                )
            } else {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
            };
            let _ = c.write_all(resposta.as_bytes());
        }
    });
    true
}

fn main() -> iced::Result {
    if !servidor_de_dados() {
        eprintln!("painel: a porta {PORTA} já está em uso; usando o servidor que está lá");
    }
    GlacierDaemon::new()
        .locales_dir("examples/gvb/painel/locales")
        .default_locale("pt-BR")
        .main_template("examples/gvb/painel/app.gvb")
        .run()
}
