//! `<Reveal>`: o corpo que **abre e fecha animado** — a altura do filho
//! crescendo de 0 até a natural (e voltando), com o que transborda recortado.
//!
//! É a peça que faltava para `<accordion>`/`<toolbox>` (0.84) não abrirem "de
//! estalo": antes, o corpo era um `<template if>` que existia ou não existia, e
//! a seção pulava de 0 para a altura final entre dois quadros. Agora o corpo
//! está **sempre** na árvore, dentro de um `<Reveal>` que sabe quanto dele
//! mostrar neste instante.
//!
//! ```xml
//! <Reveal open="{mostrar}" duration="180">
//!     <Column padding="12">…</Column>
//! </Reveal>
//! ```
//!
//! # Por que uma primitiva, e não um builtin
//!
//! Pelo primeiro dos três sinais do `PRIMITIVAS.md` visto pelo avesso: não é o
//! markup que falta, é o **relógio**. Interpolar altura exige um estado que
//! sobreviva ao rebuild da view e um quadro que se agende sozinho — as duas
//! coisas que só existem dentro de um `Widget` do `iced` (ver `ANIMACOES.md`).
//!
//! # A diferença para o `AnimatedToggler` e para o `Spinner`
//!
//! Os dois primeiros animam o que **desenham**; este anima o que **mede**. Um
//! knob que escorrega mexe só no `draw()`; uma seção que abre muda o tamanho do
//! nó, e portanto o layout de tudo que está abaixo dela na tela.
//!
//! Isso acrescenta uma quinta peça às quatro de `ANIMACOES.md`:
//! [`Shell::invalidate_layout`](iced::advanced::Shell::invalidate_layout) a
//! cada quadro da transição. Sem ela, o `iced` mede a árvore uma vez (quando a
//! view é reconstruída) e reusa essa medida em todos os quadros seguintes — a
//! animação correria no relógio interno e a tela ficaria parada. Com ela, o
//! `UserInterface::update` remede a árvore inteira antes de desenhar, no mesmo
//! quadro (é o mesmo gancho que um `text_input` usa quando o texto digitado
//! muda a largura dele).
//!
//! # Recorte, e por que ele é obrigatório
//!
//! O filho é medido **inteiro** e ancorado no topo; o nó devolvido é que tem a
//! altura reduzida. Sem recorte, o pedaço que sobra seria desenhado por cima do
//! que vier depois — então o `draw()` acontece dentro de um
//! `renderer.with_layer(bounds)`, o mesmo recurso que segura o conteúdo de um
//! `scrollable` dentro da moldura dele.
//!
//! O mesmo transbordo vale para o **ponteiro**: os limites do filho continuam
//! valendo a altura toda, e um clique abaixo da dobra cairia nele — em cima do
//! widget que o usuário realmente vê ali. Por isso, enquanto a seção não está
//! completamente aberta, o cursor entregue ao filho é mascarado para
//! [`mouse::Cursor::Unavailable`] fora da parte visível.
//!
//! # Custo
//!
//! O filho de uma seção **fechada** continua sendo montado e medido a cada
//! quadro (é o preço de poder animá-lo ao fechar: um nó que não existe não tem
//! altura de onde encolher). O que ele não faz é desenhar — `draw()` sai cedo
//! com altura zero — nem receber ponteiro. Para um corpo caro dentro de uma
//! seção fechada, a saída continua sendo a de sempre: `virtualize` na lista lá
//! dentro (ver `PRIMITIVAS.md`).

use std::time::{Duration, Instant};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay};
use iced::animation::{Animation, Easing};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse, window};

/// Duração default da abertura/fechamento. 180ms é o meio do caminho entre o
/// `quick()` (200ms) do `AnimatedToggler` e a sensação de "já era" — uma seção
/// de accordion move muito mais pixel que um knob, e o mesmo tempo lê como
/// lento.
pub const DEFAULT_DURATION: Duration = Duration::from_millis(180);

/// Ver o [módulo](self). Criado por [`reveal`].
pub struct Reveal<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
    open: bool,
    duration: Duration,
    easing: Easing,
}

/// Um [`Reveal`] envolvendo `content`, aberto ou fechado. A primeira aparição
/// **não anima**: nasce assentado no estado dado (o mesmo contrato do
/// `AnimatedToggler`) — o que anima é *receber um `open` diferente* depois.
pub fn reveal<'a, Message>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    open: bool,
) -> Reveal<'a, Message> {
    Reveal {
        content: content.into(),
        open,
        duration: DEFAULT_DURATION,
        easing: Easing::EaseOutCubic,
    }
}

impl<Message> Reveal<'_, Message> {
    /// Duração da transição. `Duration::ZERO` desliga a animação (a seção volta
    /// a abrir de estalo) — é o escape para quem não a quer.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Curva da transição. Default `EaseOutCubic`: sai rápido e assenta devagar,
    /// que é como uma gaveta bem amortecida se comporta.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

/// Estado vivo na árvore de widgets: a animação 0 ⇄ 1, o alvo que ela persegue
/// (para o `diff` detectar a mudança), o instante do último quadro — o `layout`
/// e o `draw` não recebem relógio — e a duração declarada, só para reconstruir
/// a animação quando o hot-reload muda o atributo.
struct State {
    animation: Animation<bool>,
    target: bool,
    now: Instant,
    duration: Duration,
}

impl State {
    /// Quanto da altura natural do filho mostrar agora: 0 fechado, 1 aberto.
    ///
    /// `duration="0"` (a saída de quem não quer a animação) curto-circuita
    /// aqui: uma duração zero faria a interpolação dividir por zero, e um
    /// `NaN` viraria altura `NaN` no layout — um nó que some sem erro nenhum.
    /// O `is_finite` é a mesma rede de segurança para qualquer outro extremo.
    fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return if self.target { 1.0 } else { 0.0 };
        }
        let p = self.animation.interpolate(0.0, 1.0, self.now);
        if p.is_finite() {
            p.clamp(0.0, 1.0)
        } else if self.target {
            1.0
        } else {
            0.0
        }
    }

    /// Há transição correndo agora? Com `duration="0"` nunca há — e é o que
    /// impede o laço de quadros de nascer.
    fn animando(&self, now: Instant) -> bool {
        !self.duration.is_zero() && self.animation.is_animating(now)
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Reveal<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            animation: Animation::new(self.open)
                .duration(self.duration)
                .easing(self.easing),
            target: self.open,
            now: Instant::now(),
            duration: self.duration,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Hot-reload trocou a duração no `.gv`: reconstrói a animação já
        // assentada no estado atual. Só fora de uma transição — no meio dela,
        // recomeçar do zero seria um pulo bem visível.
        if state.duration != self.duration && !state.animando(state.now) {
            state.duration = self.duration;
            state.animation = Animation::new(state.target)
                .duration(self.duration)
                .easing(self.easing);
        }

        if state.target != self.open {
            state.target = self.open;
            state.animation.go_mut(self.open, Instant::now());
        }

        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        // Largura: o que o filho for (um corpo de accordion é `fill`). Altura:
        // `Shrink` sempre — quem manda nela é a animação, não o pai, e declarar
        // `Fill` faria a seção fechada continuar ocupando a coluna inteira.
        Size {
            width: self.content.as_widget().size().width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.loose());

        let natural = child.size();
        let progress = tree.state.downcast_ref::<State>().progress();

        // O filho continua com a altura inteira, ancorado no topo (posição
        // (0,0) do nó pai): o que encolhe é só a janela por onde ele aparece.
        // É o `height` do CSS transicionando, não um `translate` — o conteúdo
        // fica parado e é descoberto de cima para baixo.
        layout::Node::with_children(
            Size::new(natural.width, (natural.height * progress).max(0.0)),
            vec![child],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        // Fechado, o corpo está fora de alcance: foco (Tab) não entra numa
        // seção que não se vê, e um `scroll_to` não deve mirar nela.
        if tree.state.downcast_ref::<State>().progress() <= 0.0 {
            return;
        }
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // A peça 3 de `ANIMACOES.md` (cada quadro agenda o seguinte) mais a
        // quinta, que só este widget precisa: remedir a árvore. `is_animating`
        // falso devolve o widget ao custo zero por quadro.
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<State>();
            state.now = *now;
            if state.animando(*now) {
                shell.invalidate_layout();
                shell.request_redraw();
            }
        }

        let progress = tree.state.downcast_ref::<State>().progress();
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            mask_cursor(cursor, layout.bounds(), progress),
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let progress = tree.state.downcast_ref::<State>().progress();
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            mask_cursor(cursor, layout.bounds(), progress),
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let progress = state.progress();
        let bounds = layout.bounds();
        if progress <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        // Fora da transição não há transbordo nenhum — desenhar direto poupa
        // uma camada do renderer no caso comum (a seção aberta e parada).
        let clipped = progress < 1.0;
        let Some(visible) = bounds.intersection(viewport) else {
            return;
        };

        let pintar = |renderer: &mut iced::Renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                defaults,
                layout.children().next().unwrap(),
                mask_cursor(cursor, bounds, progress),
                if clipped { &visible } else { viewport },
            );
        };

        if clipped {
            renderer.with_layer(visible, pintar);
        } else {
            pintar(renderer);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        // Um `<select>` dentro de uma seção só abre o menu dele quando a seção
        // está inteiramente aberta: um overlay não é recortado por
        // `with_layer`, então ele escaparia da dobra durante a transição.
        if tree.state.downcast_ref::<State>().progress() < 1.0 {
            return None;
        }
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

/// O cursor que o filho enxerga. Enquanto a seção não está completamente
/// aberta, os limites do filho valem a altura inteira e um clique **abaixo da
/// dobra** cairia nele — em cima do widget que o usuário de fato vê ali. Fora
/// da parte visível, portanto, o filho recebe [`mouse::Cursor::Unavailable`].
fn mask_cursor(cursor: mouse::Cursor, visible: Rectangle, progress: f32) -> mouse::Cursor {
    if progress >= 1.0 {
        return cursor;
    }
    match cursor.position() {
        Some(p) if visible.contains(p) => cursor,
        _ => mouse::Cursor::Unavailable,
    }
}

impl<'a, Message: 'a> From<Reveal<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(reveal: Reveal<'a, Message>) -> Self {
        Element::new(reveal)
    }
}
