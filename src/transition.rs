//! `transition` do `.gss`: a troca de estilo de um `<button>` — repouso, hover,
//! pressionado — passa a ser **animada** em vez de instantânea.
//!
//! ```gss
//! .botao          { background: #5865F2; }
//! .botao:hover    { background: #7983F5; }
//! .botao          { transition: background 150ms ease-out; }
//! ```
//!
//! # Por que é um embrulho, e não só uma conta
//!
//! O `iced` redesenha **sob demanda** e reconstrói a `view` a cada mensagem, então o
//! progresso de uma animação não pode morar no widget (ele morre e renasce) nem em
//! lugar nenhum que a `view` reconstrua. O padrão é o de `docs/ANIMACOES.md`: o
//! estado vai para a árvore de widgets — aqui, o [`Anim`] dentro do `tree::State` do
//! embrulho [`Transicionado`] — e o embrulho pede o próximo quadro enquanto a
//! animação não termina.
//!
//! O estilo de um botão é uma função do estado (`button::Status`), avaliada no
//! `draw`. O closure de estilo, em `widget.rs`, calcula o estilo **de destino** do
//! estado atual e passa por [`Anim::estilo`], que devolve o valor interpolado entre o
//! estilo anterior e o de destino. O embrulho não conhece o `Status` (ele é interno do
//! botão), então refaz a conta mínima pelo cursor — dentro/fora e botão apertado —
//! só para saber **quando pedir um quadro**.
//!
//! # O que anima
//!
//! `background` (quando as duas pontas são cores sólidas; um gradiente troca de uma
//! vez), `color`/`text-color` (a cor do rótulo), `border-color` e `opacity` (que o
//! motor já aplicou nas cores). Espessura e raio da borda trocam de uma vez.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::widget::button;
use iced::{Background, Color, Element, Event, Length, Rectangle, Size, Vector, window};

/// A curva da animação.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curva {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Curva {
    fn aplica(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            // O `ease` do CSS (cubic-bezier(.25,.1,.25,1)) é perto de um ease-out suave.
            Self::Ease => 1.0 - (1.0 - t).powf(2.4),
        }
    }
}

/// `transition: <propriedades> <duração> [curva]`, já lida.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transicao {
    pub fundo: bool,
    pub texto: bool,
    pub borda: bool,
    pub duracao: Duration,
    pub curva: Curva,
}

/// Lê `background 150ms`, `all 0.2s ease-in-out`, `background, color 120ms`. Uma só
/// duração vale para todas as propriedades (a primeira que aparecer). `None` se não
/// houver duração ou propriedade reconhecida — e o botão fica sem transição.
pub fn parse(s: &str) -> Option<Transicao> {
    let mut t = Transicao {
        fundo: false,
        texto: false,
        borda: false,
        duracao: Duration::ZERO,
        curva: Curva::Ease,
    };
    let mut achou_duracao = false;
    for tok in s.split(|c: char| c.is_whitespace() || c == ',').filter(|x| !x.is_empty()) {
        let tl = tok.to_ascii_lowercase();
        if let Some(ms) = tl.strip_suffix("ms").and_then(|n| n.parse::<f32>().ok()) {
            if !achou_duracao {
                t.duracao = Duration::from_secs_f32((ms / 1000.0).max(0.0));
                achou_duracao = true;
            }
            continue;
        }
        if let Some(sec) = tl.strip_suffix('s').and_then(|n| n.parse::<f32>().ok()) {
            if !achou_duracao {
                t.duracao = Duration::from_secs_f32(sec.max(0.0));
                achou_duracao = true;
            }
            continue;
        }
        match tl.as_str() {
            "all" => {
                t.fundo = true;
                t.texto = true;
                t.borda = true;
            }
            "background" | "bg" | "fill" => t.fundo = true,
            "color" | "text-color" | "text_color" => t.texto = true,
            "border-color" | "border_color" | "border" => t.borda = true,
            // A opacidade já virou alfa nas cores; animá-la é animar as três.
            "opacity" => {
                t.fundo = true;
                t.texto = true;
                t.borda = true;
            }
            "linear" => t.curva = Curva::Linear,
            "ease" => t.curva = Curva::Ease,
            "ease-in" => t.curva = Curva::EaseIn,
            "ease-out" => t.curva = Curva::EaseOut,
            "ease-in-out" => t.curva = Curva::EaseInOut,
            _ => {}
        }
    }
    (achou_duracao && (t.fundo || t.texto || t.borda) && !t.duracao.is_zero()).then_some(t)
}

/// A cor de um fundo, se ele é de uma cor só: uma cor sólida, ou um gradiente cujas
/// paradas são todas iguais — que é como o botão monta o fundo de um estado
/// (`:hover { background: … }`), sempre como gradiente.
fn cor_unica(b: Background) -> Option<Color> {
    match b {
        Background::Color(c) => Some(c),
        Background::Gradient(iced::Gradient::Linear(l)) => {
            let mut cores = l.stops.iter().flatten().map(|s| s.color);
            let primeira = cores.next()?;
            cores.all(|c| c == primeira).then_some(primeira)
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_cor(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

/// O estado de uma animação de botão. Mora na árvore de widgets, atrás de um `Rc`
/// para o closure de estilo (criado a cada `view`) e o embrulho enxergarem o mesmo.
#[derive(Debug, Default)]
pub struct Anim {
    alvo: Option<button::Style>,
    de: Option<button::Style>,
    atual: Option<button::Style>,
    t0: Option<Instant>,
    /// Há uma animação em curso: o embrulho pede quadro até ela terminar.
    pub animando: bool,
    /// O embrulho viu uma mudança de estado e o `draw` ainda não a processou.
    pub pendente: bool,
    /// Quantas vezes o estilo de destino mudou — só para os testes e o diagnóstico.
    pub transicoes: u32,
}

impl Anim {
    /// O estilo a desenhar **agora**, dado o de destino do estado atual.
    pub fn estilo(&mut self, alvo: button::Style, tr: &Transicao, agora: Instant) -> button::Style {
        self.pendente = false;
        match self.alvo {
            None => {
                // Primeira vez: nasce assentada no destino, sem animar a aparição.
                self.alvo = Some(alvo);
                self.atual = Some(alvo);
                return alvo;
            }
            Some(anterior) if anterior != alvo => {
                self.de = Some(self.atual.unwrap_or(anterior));
                self.alvo = Some(alvo);
                self.t0 = Some(agora);
                self.animando = true;
                self.transicoes += 1;
            }
            _ => {}
        }
        if !self.animando {
            self.atual = Some(alvo);
            return alvo;
        }
        let t0 = self.t0.unwrap_or(agora);
        let p = (agora.saturating_duration_since(t0).as_secs_f32() / tr.duracao.as_secs_f32())
            .clamp(0.0, 1.0);
        if p >= 1.0 {
            self.animando = false;
            self.atual = Some(alvo);
            return alvo;
        }
        let e = tr.curva.aplica(p);
        let de = self.de.unwrap_or(alvo);
        let mut s = alvo;
        if tr.fundo
            && let (Some(a), Some(b)) = (
                de.background.and_then(cor_unica),
                alvo.background.and_then(cor_unica),
            )
        {
            s.background = Some(Background::Color(lerp_cor(a, b, e)));
        }
        if tr.texto {
            s.text_color = lerp_cor(de.text_color, alvo.text_color, e);
        }
        if tr.borda {
            s.border.color = lerp_cor(de.border.color, alvo.border.color, e);
        }
        self.atual = Some(s);
        s
    }
}

/// O estado que sobrevive entre as reconstruções da `view`.
struct Persist {
    anim: Rc<RefCell<Anim>>,
    pressionado: Cell<bool>,
    /// A última chave de estado (0 repouso, 1 hover, 2 pressionado) que o embrulho viu.
    chave: Cell<u8>,
}

/// O elo entre o closure de estilo e o estado da árvore: a `view` cria o `Slot`, o
/// closure o lê no `draw`, e o embrulho o preenche em `state`/`diff`.
pub type Slot = Rc<RefCell<Option<Rc<RefCell<Anim>>>>>;

pub fn novo_slot() -> Slot {
    Rc::new(RefCell::new(None))
}

/// Embrulha um botão e pede quadros enquanto a transição dele anda.
pub struct Transicionado<'a, Message, Theme, Renderer> {
    filho: Element<'a, Message, Theme, Renderer>,
    slot: Slot,
}

impl<'a, Message, Theme, Renderer> Transicionado<'a, Message, Theme, Renderer> {
    pub fn new(filho: Element<'a, Message, Theme, Renderer>, slot: Slot) -> Self {
        Self { filho, slot }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Transicionado<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Persist>()
    }

    fn state(&self) -> tree::State {
        let anim = Rc::new(RefCell::new(Anim::default()));
        *self.slot.borrow_mut() = Some(anim.clone());
        tree::State::new(Persist {
            anim,
            pressionado: Cell::new(false),
            chave: Cell::new(0),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.filho)]
    }

    fn diff(&self, tree: &mut Tree) {
        // O widget desta `view` é novo; o estado da animação é o da árvore.
        let p = tree.state.downcast_ref::<Persist>();
        *self.slot.borrow_mut() = Some(p.anim.clone());
        tree.diff_children(std::slice::from_ref(&self.filho));
    }

    fn size(&self) -> Size<Length> {
        self.filho.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.filho.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.filho
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.filho.as_widget().draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, op: &mut dyn Operation) {
        self.filho
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, op);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.filho.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        let p = tree.state.downcast_ref::<Persist>();
        let dentro = cursor.is_over(layout.bounds());
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if dentro => {
                p.pressionado.set(true);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => p.pressionado.set(false),
            _ => {}
        }
        let chave = if dentro && p.pressionado.get() {
            2
        } else if dentro {
            1
        } else {
            0
        };
        let mut a = p.anim.borrow_mut();
        if chave != p.chave.get() {
            p.chave.set(chave);
            a.pendente = true;
            shell.request_redraw();
        }
        // O estilo só muda no `draw`: enquanto ele não rodou ou a animação anda, pede o
        // próximo quadro — senão ela congelaria até o próximo evento do usuário.
        if matches!(event, Event::Window(window::Event::RedrawRequested(_))) && (a.animando || a.pendente) {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.filho
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.filho
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Transicionado<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(w: Transicionado<'a, Message, Theme, Renderer>) -> Self {
        Element::new(w)
    }
}

