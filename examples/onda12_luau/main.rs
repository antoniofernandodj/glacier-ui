//! **Onda 12, em Luau**: o `<dock>` do lado do script.
//!
//! Rode com: `cargo run --example onda12_luau` (com `WGPU_BACKEND=gl` nesta
//! máquina).
//!
//! # O que este exemplo mostra
//!
//! A mesma observação das ondas 9 a 11: o `<dock>` inteiro — o `<splitter>`
//! acoplado, o `<stack>` flutuante, os botões do cabeçalho e o arrasto de
//! reancoragem (`grip::Alvo::Zona`) — é **markup e motor**. O que sobra para o
//! Luau é semear e, se quiser, um atalho que escreve a chave `mode`. Nada aqui
//! desenha nem posiciona.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — Onda 12 (dock, Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) =
                motor.register_component("onda12_luau", "examples/onda12_luau/app.gv")
            {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda12_luau");
        })
        .run()
}
