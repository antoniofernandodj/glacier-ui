//! Teste do foco de teclado de `src/link_focus.rs`, SEM janela: monta o widget numa
//! `UserInterface` com o renderizador nulo (`()`) do iced e injeta Tab, Shift+Tab,
//! Enter e Espaço, conferindo o que sai.
//!
//! É um projeto à parte (não entra no `cargo test` do motor, que é pesado):
//!
//!     cd tools/link_focus_check && cargo run
//!
//! Cobre a lógica de foco e de teclado. NÃO cobre o desenho do destaque (precisa de
//! renderizador de verdade).
#[path = "../../../src/link_focus.rs"]
mod link_focus;

use iced::advanced::widget::operation::focusable::{focus_next, focus_previous};
use iced::advanced::{Clipboard, clipboard};
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::{rich_text, span};
use iced::{Element, Event, Font, Size, event, mouse};
use iced_runtime::UserInterface;
use iced_runtime::user_interface::Cache;

#[derive(Clone, Debug, PartialEq)]
struct Msg(usize);

type El<'a> = Element<'a, Msg, iced::Theme, ()>;

fn paragrafo<'a>(destaque: Option<usize>) -> El<'a> {
    let spans: Vec<iced::widget::text::Span<'a, usize, Font>> = (0..3)
        .flat_map(|i| {
            let mut link = span(format!("link{i}")).link(i);
            if destaque == Some(i) {
                link = link.underline(true);
            }
            [span("texto ".to_string()), link]
        })
        .collect();
    rich_text(spans).on_link_click(|i: usize| Msg(i + 100)).into()
}

fn embrulho<'a>() -> El<'a> {
    link_focus::FocusLinks::new(
        paragrafo(None),
        (0..3).map(|i| paragrafo(Some(i))).collect(),
        (0..3).map(Msg).collect(),
    )
    .into()
}

struct Nulo;
impl Clipboard for Nulo {
    fn read(&self, _: clipboard::Kind) -> Option<String> { None }
    fn write(&mut self, _: clipboard::Kind, _: String) {}
}

fn tecla(n: Named, shift: bool) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Named(n),
        modified_key: Key::Named(n),
        physical_key: keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: if shift { Modifiers::SHIFT } else { Modifiers::empty() },
        text: None,
        repeat: false,
    })
}

fn rodar(ui: &mut UserInterface<Msg, iced::Theme, ()>, mut op: Box<dyn iced::advanced::widget::Operation<()>>) {
    use iced::advanced::widget::operation::Outcome;
    loop {
        ui.operate(&(), op.as_mut());
        match op.finish() {
            Outcome::Chain(proximo) => op = proximo,
            _ => break,
        }
    }
}

fn main() {
    let mut r = ();
    let mut ui = UserInterface::build(embrulho(), Size::new(400.0, 200.0), Cache::default(), &mut r);
    let mut falhas = 0;
    let mut confere = |nome: &str, ok: bool| {
        println!("{} {nome}", if ok { "ok  " } else { "FALHA" });
        if !ok { falhas += 1; }
    };

    let enviar = |ui: &mut UserInterface<Msg, iced::Theme, ()>, e: Event| -> (Vec<Msg>, event::Status) {
        let mut msgs = Vec::new();
        let (_, st) = ui.update(&[e], mouse::Cursor::Unavailable, &mut (), &mut Nulo, &mut msgs);
        (msgs, st[0])
    };

    // 1. sem foco, Enter não faz nada
    let (m, _) = enviar(&mut ui, tecla(Named::Enter, false));
    confere("sem foco, Enter não publica", m.is_empty());

    // 2. foco por Tab global (a operação do motor)
    rodar(&mut ui, Box::new(focus_next::<()>()));
    let (m, st) = enviar(&mut ui, tecla(Named::Enter, false));
    confere("após focus_next, Enter ativa o link 0", m == vec![Msg(0)] && st == event::Status::Captured);

    // 3. Tab anda dentro do parágrafo, capturando
    let (m, st) = enviar(&mut ui, tecla(Named::Tab, false));
    confere("Tab 1 capturado, sem mensagem", m.is_empty() && st == event::Status::Captured);
    let (m, _) = enviar(&mut ui, tecla(Named::Enter, false));
    confere("Enter ativa o link 1", m == vec![Msg(1)]);
    let (_, st) = enviar(&mut ui, tecla(Named::Tab, false));
    confere("Tab 2 capturado", st == event::Status::Captured);
    let (m, _) = enviar(&mut ui, tecla(Named::Space, false));
    confere("Espaço ativa o link 2", m == vec![Msg(2)]);

    // 4. no último link o Tab NÃO é capturado (segue para o foco global)
    let (_, st) = enviar(&mut ui, tecla(Named::Tab, false));
    confere("Tab no último link não é capturado", st == event::Status::Ignored);

    // 5. Shift+Tab volta
    let (_, st) = enviar(&mut ui, tecla(Named::Tab, true));
    confere("Shift+Tab capturado", st == event::Status::Captured);
    let (m, _) = enviar(&mut ui, tecla(Named::Enter, false));
    confere("Enter ativa o link 1 (voltou)", m == vec![Msg(1)]);
    let (_, _) = enviar(&mut ui, tecla(Named::Tab, true));
    let (_, st) = enviar(&mut ui, tecla(Named::Tab, true));
    confere("Shift+Tab no primeiro link não é capturado", st == event::Status::Ignored);

    // 6. a operação global tira o foco do widget (próximo focável): sem outro, volta/permanece
    rodar(&mut ui, Box::new(focus_next::<()>()));
    let (m, _) = enviar(&mut ui, tecla(Named::Enter, false));
    println!("(após 2º focus_next com um único focável: Enter → {m:?})");

    // 7. clique fora tira o foco
    rodar(&mut ui, Box::new(focus_previous::<()>()));
    let _ = enviar(&mut ui, Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)));
    println!("(cursor indisponível: clique sem posição não tira o foco — coberto só com cursor)");

    if falhas > 0 { std::process::exit(1); }
    println!("tudo certo");
}
