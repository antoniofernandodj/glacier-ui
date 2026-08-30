//! {{titulo}} — o menor projeto glacier-ui que ainda mostra a ideia.
//!
//! Este arquivo é só a casca: ele sobe o runner, registra a tela e diz qual
//! abre primeiro. Tudo o que a tela É (layout, estilo e comportamento) mora em
//! `views/contador.gv` — e muda sem recompilar, porque o motor recarrega o
//! template a quente quando o arquivo é salvo.

use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            // O caminho é relativo ao diretório de onde o app roda — a raiz do
            // projeto, se for `cargo run`. Título e tamanho da janela NÃO estão
            // aqui: quem os declara é o `<screen>` do próprio .gv.
            if let Err(erro) = motor.register_component("contador", "views/contador.gv") {
                // O Display do erro já traz arquivo:linha:coluna, o trecho e a
                // dica — reembrulhar só esconderia isso.
                eprintln!("{erro}");
            }
            motor.set_initial_screen("contador");
        })
        .run()
}
