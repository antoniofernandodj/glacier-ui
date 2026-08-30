//! {{titulo}} — comportamento em Rust, com estado tipado.
//!
//! O outro caminho do glacier-ui é pôr o comportamento num `<script>` Luau
//! dentro do próprio template, que muda sem recompilar. Este preset mostra o
//! caminho oposto: um `impl Component` em Rust, onde o estado é um campo de
//! struct com o tipo que ele merece, e o compilador cobre as trocas.
//!
//! Os dois convivem: o template continua recarregando a quente (é markup), e um
//! `<script>` pode ser acrescentado depois sem tirar o Component do lugar.

mod contador;

use glacier_ui::GlacierDaemon;

use crate::contador::Contador;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            // `register` (e não `register_component`) é o registro de um
            // Component: o motor pega o template dele por `Component::template`.
            if let Err(erro) = motor.register(Box::new(Contador::new())) {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("contador");
        })
        .run()
}
