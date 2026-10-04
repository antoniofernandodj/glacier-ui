//! Servidor SSE + WebSocket **reais**, numa thread própria, só com a std e o
//! `tungstenite` síncrono (já dependência do crate). Serve aos dois exemplos de
//! ciclo de vida (`ciclo_vida_luau` e `ciclo_vida_rust`).
//!
//! Uma porta só, dois endpoints, escolhidos olhando o cabeçalho da requisição:
//!   GET /sse  -> `text/event-stream`, um `data: tick N` a cada 500 ms
//!   GET /ws   -> WebSocket: manda `tick N` a cada segundo e ecoa o que receber
//!
//! Cada conexão que abre ou fecha é impressa no terminal — é assim que se vê,
//! do lado do servidor, que sair da tela de fato derrubou o stream.

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio_tungstenite::tungstenite::{self, Message};

/// Conexões abertas agora, por tipo (para o log do servidor).
#[derive(Default)]
pub struct Contagem {
    pub sse: AtomicUsize,
    pub ws: AtomicUsize,
}

/// Sobe o servidor em `127.0.0.1:<porta livre>` e devolve a porta.
pub fn iniciar() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let porta = listener.local_addr().expect("addr").port();
    let contagem = Arc::new(Contagem::default());
    std::thread::spawn(move || {
        for conexao in listener.incoming().flatten() {
            let contagem = Arc::clone(&contagem);
            std::thread::spawn(move || atender(conexao, &contagem));
        }
    });
    println!("[servidor] ouvindo em 127.0.0.1:{porta} (/sse e /ws)");
    porta
}

fn atender(stream: TcpStream, contagem: &Contagem) {
    // `peek` lê o cabeçalho sem consumi-lo: o handshake do WebSocket precisa
    // receber a requisição inteira.
    let mut buf = [0u8; 1024];
    let Ok(n) = stream.peek(&mut buf) else { return };
    let cabecalho = String::from_utf8_lossy(&buf[..n]).to_lowercase();
    if cabecalho.starts_with("get /ws") && cabecalho.contains("upgrade: websocket") {
        servir_ws(stream, contagem);
    } else if cabecalho.starts_with("get /sse") {
        servir_sse(stream, contagem);
    }
}

fn servir_sse(mut stream: TcpStream, contagem: &Contagem) {
    // Consome o cabeçalho que o `peek` deixou na fila.
    let mut descarte = [0u8; 1024];
    let _ = std::io::Read::read(&mut stream, &mut descarte);

    let cab = "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\n\
               cache-control: no-cache\r\nconnection: close\r\n\r\n";
    if stream.write_all(cab.as_bytes()).is_err() {
        return;
    }
    let ativas = contagem.sse.fetch_add(1, Ordering::Relaxed) + 1;
    println!("[servidor] SSE aberto   (ativos: {ativas})");
    let mut n = 0u64;
    // O fim da conexão do cliente aparece como erro de escrita no tick seguinte.
    while stream
        .write_all(format!("data: tick {n}\n\n").as_bytes())
        .and_then(|_| stream.flush())
        .is_ok()
    {
        n += 1;
        std::thread::sleep(Duration::from_millis(500));
    }
    let ativas = contagem.sse.fetch_sub(1, Ordering::Relaxed) - 1;
    println!("[servidor] SSE fechado  (ativos: {ativas})");
}

fn servir_ws(stream: TcpStream, contagem: &Contagem) {
    let Ok(mut ws) = tungstenite::accept(stream) else {
        return;
    };
    // Leitura com prazo: entre uma mensagem do cliente e outra, o laço ainda
    // consegue mandar o `tick` do servidor.
    let _ = ws.get_mut().set_read_timeout(Some(Duration::from_millis(250)));
    let ativas = contagem.ws.fetch_add(1, Ordering::Relaxed) + 1;
    println!("[servidor] WS aberto    (ativos: {ativas})");
    let _ = ws.send(Message::text("olá do servidor"));

    let mut n = 0u64;
    let mut ultimo = std::time::Instant::now();
    loop {
        match ws.read() {
            Ok(Message::Text(t)) => {
                if ws.send(Message::text(format!("eco: {t}"))).is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(_) => break,
        }
        if ultimo.elapsed() >= Duration::from_secs(1) {
            ultimo = std::time::Instant::now();
            n += 1;
            if ws.send(Message::text(format!("tick {n}"))).is_err() {
                break;
            }
        }
    }
    let ativas = contagem.ws.fetch_sub(1, Ordering::Relaxed) - 1;
    println!("[servidor] WS fechado   (ativos: {ativas})");
}
