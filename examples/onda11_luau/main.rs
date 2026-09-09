//! **Onda 11, em Luau**: o que fica por cima, do lado do script.
//!
//! Rode com: `cargo run --example onda11_luau`
//!
//! # O que este exemplo mostra e a versão Rust não mostra
//!
//! Nada de API nova — e **é essa a observação**, a mesma que a Onda 9 já
//! tinha registrado do lado do script.
//!
//! O habilitador A (`<stack>`, `x=`/`y=`/`anchor=`) é markup puro: nenhuma das
//! duas formas de posicionar um filho pede uma linha de `<script>`. O
//! habilitador B (`grip::Alvo::Ponto`) escreve direto nas quatro chaves que
//! um `<mdisubwindow>` nomeia — o mesmo binding que um `<slider>` já usava
//! desde a 0.63, só que em duas dimensões de uma vez.
//!
//! O que sobra para o script fazer são quatro coisas pequenas: semear
//! (`init`), alternar um booleano (`NotificationDot`), reagir a um clique com
//! payload (`remover_tag:<tag>`, a convenção `nome:sufixo` do dispatcher) e
//! reagir a um clique sem payload (`SplashScreen`, `CommandLink`,
//! `RoundButton`). O campo de texto do `QrCode` nem isso: sem uma função
//! `qr_texto`, o binding legado do motor (`ctx[acao] = valor`) resolve
//! sozinho.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 11 (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) =
                motor.register_component("onda11_luau", "examples/onda11_luau/app.gv")
            {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda11_luau");
        })
        .run()
}
