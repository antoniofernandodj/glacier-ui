//! O servidor SSE + WebSocket dos exemplos `ciclo_vida_*` fala de verdade:
//! sem isto, um erro de protocolo ali só apareceria abrindo a janela.

#[path = "../examples/ciclo_vida_luau/servidor.rs"]
mod servidor;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use tokio_tungstenite::tungstenite::{self, Message};

#[test]
fn sse_manda_ticks_em_text_event_stream() {
    let porta = servidor::iniciar();
    let mut s = TcpStream::connect(("127.0.0.1", porta)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    s.write_all(b"GET /sse HTTP/1.1\r\nhost: x\r\naccept: text/event-stream\r\n\r\n")
        .unwrap();

    let mut recebido = String::new();
    let mut buf = [0u8; 256];
    // Cabeçalho + pelo menos dois eventos (500 ms entre eles).
    while recebido.matches("data: tick").count() < 2 {
        let n = s.read(&mut buf).expect("o servidor parou de mandar");
        assert!(n > 0, "conexão fechada cedo: {recebido:?}");
        recebido.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    assert!(recebido.starts_with("HTTP/1.1 200"), "{recebido:?}");
    assert!(recebido.contains("text/event-stream"));
    assert!(recebido.contains("data: tick 0\n\n"));
    assert!(recebido.contains("data: tick 1\n\n"));
}

#[test]
fn ws_saúda_ecoa_e_manda_tick() {
    let porta = servidor::iniciar();
    let (mut ws, _) = tungstenite::connect(format!("ws://127.0.0.1:{porta}/ws")).unwrap();
    assert_eq!(ws.read().unwrap().into_text().unwrap(), "olá do servidor");

    ws.send(Message::text("oi")).unwrap();
    let mut viu_eco = false;
    let mut viu_tick = false;
    for _ in 0..6 {
        let m = ws.read().unwrap().into_text().unwrap();
        viu_eco |= m.as_str() == "eco: oi";
        viu_tick |= m.as_str().starts_with("tick ");
        if viu_eco && viu_tick {
            break;
        }
    }
    assert!(viu_eco, "faltou o eco");
    assert!(viu_tick, "faltou o tick periódico");
    ws.close(None).unwrap();
}
