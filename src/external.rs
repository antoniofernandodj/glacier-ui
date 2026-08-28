//! Canal externo — injeta ações no motor da janela principal a partir de
//! **qualquer thread**.
//!
//! O loop do `iced` é dono da thread principal, e tudo o que acontece na UI
//! nasce de um evento dele: um clique, uma tecla, um tick. Isso deixa de fora
//! um caso legítimo: o app tem uma thread própria — um servidor HTTP local, um
//! watcher de arquivos, uma integração com o SO — que precisa **acionar** a UI,
//! e não só ler o estado dela.
//!
//! Antes disto, um app nessa situação só tinha saídas ruins: espelhar o estado
//! num `Arc<Mutex<…>>` paralelo (que diverge do contexto do motor na primeira
//! ação que esquecerem de replicar), ou simular eventos de entrada no servidor
//! gráfico. Este módulo dá a via direta.
//!
//! ## O que dá para mandar
//!
//! Um [`EngineMessage`] — o mesmo tipo que um clique produz. Na prática as três
//! formas úteis são [`ExternalSender::click`] (dispara uma ação pelo nome, como
//! um botão), [`ExternalSender::action`] (idem, com valor, como um campo de
//! texto) e [`ExternalSender::patch`] (escreve pares no contexto). Como o
//! vocabulário é o mesmo dos templates, **toda** ação que a UI declara já é
//! alcançável — inclusive as que forem adicionadas depois.
//!
//! ## Para onde vai
//!
//! Sempre para o motor da janela **principal**, mesmo quando ela está recolhida
//! na bandeja: nesse estado o motor continua vivo (só a janela sumiu), então um
//! app de bandeja segue inteiramente dirigível de fora.
//!
//! ## Uso
//!
//! ```no_run
//! # use glacier_ui::{GlacierDaemon, external};
//! // Antes de `run()`: cria o canal e guarda o remetente.
//! let ui = external::sender();
//!
//! std::thread::spawn(move || {
//!     // De outra thread, a qualquer momento:
//!     ui.patch(vec![("url".into(), "https://exemplo.dev".into())]);
//!     ui.click("connect");
//! });
//!
//! GlacierDaemon::new().main(|_| {}).run().unwrap();
//! ```
//!
//! O canal nasce na primeira chamada de [`sender`]; o daemon só registra a
//! subscription quando ele existe, então quem não usa não paga nada.

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, OnceLock};

use crate::EngineMessage;

/// Lado receptor, drenado pelo daemon. Global porque `Subscription::run` exige
/// um `fn` não-capturante (mesma restrição que a bandeja resolve assim).
static RX: OnceLock<Mutex<Receiver<EngineMessage>>> = OnceLock::new();

/// Lado emissor guardado para que chamadas repetidas de [`sender`] devolvam um
/// remetente para o **mesmo** canal, em vez de criar um canal órfão que
/// ninguém drena.
static TX: OnceLock<Mutex<Sender<EngineMessage>>> = OnceLock::new();

/// Remetente clonável e `Send`, para mover a qualquer thread.
#[derive(Clone)]
pub struct ExternalSender {
    tx: Sender<EngineMessage>,
}

impl std::fmt::Debug for ExternalSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExternalSender")
    }
}

impl ExternalSender {
    /// Dispara uma ação pelo nome — equivalente a clicar o botão que a declara.
    ///
    /// Devolve `false` se o app já encerrou (o receptor sumiu). Nenhum método
    /// aqui entra em pânico: quem manda de outra thread costuma estar num
    /// caminho que não pode derrubar o processo por causa da UI.
    pub fn click(&self, action: impl Into<String>) -> bool {
        self.send(EngineMessage::UiClick(action.into()))
    }

    /// Dispara uma ação **com valor** — equivalente ao `on_change` de um campo.
    pub fn action(&self, action: impl Into<String>, value: impl Into<String>) -> bool {
        self.send(EngineMessage::UiInputChanged {
            action: action.into(),
            value: value.into(),
        })
    }

    /// Mescla pares no contexto da janela principal e reavalia a árvore.
    pub fn patch(&self, pairs: Vec<(String, String)>) -> bool {
        self.send(EngineMessage::ContextPatch(pairs))
    }

    /// Manda um [`EngineMessage`] cru, para o que os atalhos acima não cobrem.
    pub fn send(&self, msg: EngineMessage) -> bool {
        self.tx.send(msg).is_ok()
    }
}

/// Devolve o remetente do canal externo, criando-o na primeira chamada.
///
/// Chame **antes** de [`crate::GlacierDaemon::run`]: é a existência do canal
/// que faz o daemon registrar a subscription que o drena.
pub fn sender() -> ExternalSender {
    let tx = TX.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        // `RX` é preenchido junto: os dois lados nascem na mesma chamada, então
        // não existe janela em que o canal esteja "meio criado".
        let _ = RX.set(Mutex::new(rx));
        Mutex::new(tx)
    });

    ExternalSender {
        // O `Sender` do mpsc é clonável e cada clone alimenta o mesmo receptor.
        tx: tx.lock().expect("canal externo envenenado").clone(),
    }
}

/// Se alguém já pediu um [`sender`]. O daemon usa para decidir se registra a
/// subscription — sem isto, todo app pagaria um poll a mais para nada.
pub(crate) fn is_active() -> bool {
    RX.get().is_some()
}

/// Stream que drena o canal e emite as mensagens para o daemon.
///
/// `fn` (não closure) porque `Subscription::run` deriva a chave da subscription
/// do tipo da função. Faz poll não-bloqueante com `sleep`, como a bandeja: o
/// receptor do `mpsc` é síncrono e a ponte para o mundo async do iced é este
/// laço. 120 ms é curto o bastante para parecer imediato e longo o bastante
/// para não acordar o loop à toa.
pub(crate) fn event_stream() -> impl futures::Stream<Item = EngineMessage> {
    use futures::SinkExt;
    use std::time::Duration;

    iced::stream::channel(64, |mut output: futures::channel::mpsc::Sender<EngineMessage>| async move {
        loop {
            // O lock é solto entre uma mensagem e outra de propósito: `send`
            // abaixo é `await`, e segurar o mutex através dele bloquearia todo
            // emissor pelo tempo de um redraw.
            let proxima = RX
                .get()
                .and_then(|rx| rx.lock().ok().and_then(|g| g.try_recv().ok()));

            match proxima {
                Some(msg) => {
                    let _ = output.send(msg).await;
                }
                None => tokio::time::sleep(Duration::from_millis(120)).await,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um teste só, e não três: o canal é ESTÁTICO (um por processo), então
    /// testes paralelos disputariam o mesmo receptor e falhariam por ordem, não
    /// por defeito.
    #[test]
    fn canal_externo() {
        // Antes de qualquer `sender()`, o daemon não deve registrar a
        // subscription — quem não usa não paga o poll.
        assert!(!is_active(), "o canal não pode existir antes de alguém pedir");

        let a = sender();
        assert!(is_active());

        // Dois remetentes alimentam o MESMO canal. Sem isto, o segundo
        // chamador mandaria para um receptor que ninguém drena e as ações
        // sumiriam sem erro nenhum.
        let b = sender();

        assert!(a.click("connect"));
        assert!(b.action("busca", "nginx"));
        assert!(a.patch(vec![("view".into(), "projects".into())]));

        let rx = RX.get().unwrap().lock().unwrap();
        let msgs: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();

        assert_eq!(msgs.len(), 3, "as três chegaram no mesmo receptor");
        assert!(matches!(&msgs[0], EngineMessage::UiClick(a) if a == "connect"));
        assert!(
            matches!(&msgs[1], EngineMessage::UiInputChanged { action, value }
                if action == "busca" && value == "nginx")
        );
        assert!(matches!(&msgs[2], EngineMessage::ContextPatch(p) if p[0].0 == "view"));
    }
}
