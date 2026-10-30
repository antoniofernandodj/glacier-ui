//! Teste, SEM janela, de `src/transition.rs`: monta um botão embrulhado numa
//! `UserInterface` com o renderizador nulo (`()`), injeta o cursor sobre ele e quadros
//! de redesenho com o tempo andando, e registra as cores que o estilo devolve.
//!
//!     cd tools/transition_check && cargo run
//!
//! Cobre a LÓGICA (a rampa de cores, o pedido de quadros, o fim da animação). Não cobre
//! o desenho de verdade.
#[path = "../../../src/transition.rs"]
mod transition;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use iced::advanced::{Clipboard, clipboard};
use iced::widget::button;
use iced::{Background, Color, Element, Event, Gradient, Point, Size, Theme, mouse, window};
use iced_runtime::UserInterface;
use iced_runtime::user_interface::{Cache, State};

struct Nulo;
impl Clipboard for Nulo {
    fn read(&self, _: clipboard::Kind) -> Option<String> { None }
    fn write(&mut self, _: clipboard::Kind, _: String) {}
}

const AZUL: Color = Color { r: 0.345, g: 0.396, b: 0.949, a: 1.0 };
const LARANJA: Color = Color { r: 0.878, g: 0.478, b: 0.373, a: 1.0 };

/// O fundo do hover, montado como o `widget.rs` monta: um gradiente de paradas iguais.
fn gradiente_unico(c: Color) -> Background {
    Background::Gradient(Gradient::Linear(
        iced::gradient::Linear::new(iced::Radians(0.0)).add_stop(0.0, c).add_stop(1.0, c),
    ))
}

fn main() {
    let tr = transition::parse("background 400ms linear").expect("parse");
    let slot = transition::novo_slot();
    let vistas: Rc<RefCell<Vec<Color>>> = Rc::new(RefCell::new(Vec::new()));
    let relogio: Rc<RefCell<Instant>> = Rc::new(RefCell::new(Instant::now()));

    let (s2, v2, r2) = (slot.clone(), vistas.clone(), relogio.clone());
    let btn = button("x").width(300).height(100).on_press(()).style(move |_t: &Theme, status| {
        let alvo = match status {
            button::Status::Hovered => iced::widget::button::Style {
                background: Some(gradiente_unico(LARANJA)),
                ..Default::default()
            },
            _ => iced::widget::button::Style {
                background: Some(Background::Color(AZUL)),
                ..Default::default()
            },
        };
        let estilo = match s2.borrow().clone() {
            Some(anim) => anim.borrow_mut().estilo(alvo, &tr, *r2.borrow()),
            None => alvo,
        };
        if let Some(Background::Color(c)) = estilo.background {
            v2.borrow_mut().push(c);
        }
        estilo
    });
    let el: Element<'_, (), Theme, ()> = transition::Transicionado::new(btn.into(), slot).into();

    let mut r = ();
    let mut ui = UserInterface::build(el, Size::new(300.0, 100.0), Cache::default(), &mut r);
    let tema = Theme::Dark;
    let est = iced::advanced::renderer::Style { text_color: Color::WHITE };
    let mut falhas = 0;
    let mut confere = |nome: &str, ok: bool| {
        println!("{} {nome}", if ok { "ok  " } else { "FALHA" });
        if !ok { falhas += 1; }
    };

    let fora = mouse::Cursor::Unavailable;
    let dentro = mouse::Cursor::Available(Point::new(20.0, 15.0));
    let t0 = Instant::now();
    let quadro = |ui: &mut UserInterface<(), Theme, ()>, cursor, quando: Duration, evento: Event| {
        *relogio.borrow_mut() = t0 + quando;
        let mut m = Vec::new();
        let (st, _) = ui.update(&[evento], cursor, &mut (), &mut Nulo, &mut m);
        ui.draw(&mut (), &tema, &est, cursor);
        matches!(st, State::Updated { redraw_request: window::RedrawRequest::NextFrame, .. })
    };
    let redesenho = |q: Duration| Event::Window(window::Event::RedrawRequested(iced::time::Instant::now() + q));

    // 1. repouso: nasce assentado no azul, sem animar a aparição
    quadro(&mut ui, fora, Duration::ZERO, redesenho(Duration::ZERO));
    confere("repouso: azul", vistas.borrow().last() == Some(&AZUL));

    // 2. o cursor entra: o embrulho pede quadro e o estilo NÃO salta para o laranja
    let pediu = quadro(&mut ui, dentro, Duration::from_millis(100),
        Event::Mouse(mouse::Event::CursorMoved { position: Point::new(20.0, 15.0) }));
    quadro(&mut ui, dentro, Duration::from_millis(101), redesenho(Duration::from_millis(101)));
    let c = *vistas.borrow().last().unwrap();
    confere("cursor entrou: o embrulho pediu um quadro", pediu);
    confere("logo após entrar, a cor ainda é ~azul (não saltou)", (c.r - AZUL.r).abs() < 0.05);

    // 3. a rampa: 100, 200, 300, 400, 500 ms depois do início
    let mut rampa = Vec::new();
    let mut ainda_pede = Vec::new();
    for ms in [150u64, 200, 300, 400, 450, 600] {
        let p = quadro(&mut ui, dentro, Duration::from_millis(100 + ms), redesenho(Duration::from_millis(100 + ms)));
        rampa.push((ms, *vistas.borrow().last().unwrap()));
        ainda_pede.push((ms, p));
    }
    for (ms, c) in &rampa {
        println!("      +{ms:>3}ms  r={:.3} g={:.3} b={:.3}", c.r, c.g, c.b);
    }
    let r: Vec<f32> = rampa.iter().map(|(_, c)| c.r).collect();
    confere("a rampa de vermelho é crescente até o laranja", r.windows(2).all(|w| w[1] >= w[0]) && r[0] > AZUL.r && r[0] < LARANJA.r);
    confere("passado o tempo, termina exatamente no laranja", (r.last().unwrap() - LARANJA.r).abs() < 0.002);
    let pede = |ms: u64| ainda_pede.iter().find(|(m, _)| *m == ms).map(|(_, p)| *p).unwrap_or(false);
    confere("enquanto anima (150 ms), continua pedindo quadro", pede(150));
    confere("depois de terminar (600 ms), para de pedir quadro", !pede(600));

    // 4. o cursor sai: volta ao azul animando
    quadro(&mut ui, fora, Duration::from_millis(1000),
        Event::Mouse(mouse::Event::CursorMoved { position: Point::new(500.0, 500.0) }));
    quadro(&mut ui, fora, Duration::from_millis(1001), redesenho(Duration::from_millis(1001)));
    quadro(&mut ui, fora, Duration::from_millis(1200), redesenho(Duration::from_millis(1200)));
    let meio = *vistas.borrow().last().unwrap();
    quadro(&mut ui, fora, Duration::from_millis(1700), redesenho(Duration::from_millis(1700)));
    let fim = *vistas.borrow().last().unwrap();
    confere("ao sair, anima de volta (meio do caminho)", meio.r < LARANJA.r && meio.r > AZUL.r);
    confere("ao sair, termina no azul", (fim.r - AZUL.r).abs() < 0.002);

    if falhas > 0 { std::process::exit(1); }
    println!("tudo certo");
}
