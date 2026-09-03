//! Dirigir a UI **de outra thread**, sem tocar no teclado nem no mouse.
//!
//! O loop do iced é dono da thread principal, e tudo o que acontece na UI nasce
//! de um evento dele. Este exemplo mostra a via de mão contrária: uma thread
//! qualquer do programa — aqui um `sleep` em laço; num app de verdade, um
//! servidor HTTP local, um watcher de arquivos, uma integração com o SO —
//! injetando ações no motor da janela principal.
//!
//! O vocabulário é o mesmo dos templates:
//!
//! - `patch` escreve pares no contexto (o que um `<textinput>` faria);
//! - `click` dispara uma ação pelo nome (o que um `<button on_click>` faria);
//! - `action` dispara uma ação com valor (o que um `on_change` faria).
//!
//! Por isso **toda** ação que a UI declara já é alcançável de fora, inclusive
//! as que forem adicionadas depois — não há lista para manter em dia.
//!
//! ```bash
//! cargo run --example controle_externo
//! ```
//!
//! A janela abre "esperando a outra thread…" e, sem nenhuma interação, se
//! preenche sozinha em três passos.

use std::thread;
use std::time::Duration;

use glacier_ui::{external, GlacierDaemon};

fn main() -> iced::Result {
    // O canal precisa existir ANTES de `run()`: é a existência dele que faz o
    // daemon registrar a subscription que o drena.
    let ui = external::sender();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        // 1. Escreve direto no contexto.
        ui.patch(vec![("nome".into(), "vindo de outra thread".into())]);

        thread::sleep(Duration::from_secs(1));
        // 2. Dispara uma ação com valor (como um campo de busca).
        ui.action("buscar", "nginx");

        thread::sleep(Duration::from_secs(1));
        // 3. Dispara uma ação sem valor (como um botão).
        ui.click("conectar");
    });

    GlacierDaemon::new()
        .title("Glacier - Controle externo")
        .main(|motor| {
            if let Err(e) = motor.register_component(
                "controle_externo",
                "examples/controle_externo/controle_externo.gv",
            ) {
                eprintln!("Erro ao registrar: {e}");
            }
            motor.set_initial_screen("controle_externo");
        })
        .run()
}
