//! Rolagem suave da roda do mouse.
//!
//! O `scrollable` do iced converte cada "linha" da roda em 60 px fixos e soma
//! o resultado no deslocamento **na hora** (há até um `TODO: Configurable
//! speed/friction`). Cada clique da bolinha vira um salto de 60 px: a rolagem
//! anda em degraus. O touchpad chega como `ScrollDelta::Pixels` e já é fluido.
//!
//! Este widget envolve o `scrollable` e, em vez de deixar o salto passar,
//! **guarda** a distância pedida pela roda e a entrega ao filho aos poucos, um
//! pedaço por quadro (decaimento exponencial). O filho recebe um
//! `ScrollDelta::Pixels` sintético, então toda a lógica de limites, barra e
//! `on_scroll` do iced continua valendo.
//!
//! Só a roda com `Lines` é suavizada. `Pixels` (touchpad) passa direto.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay};
use iced::time::Instant;
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse, window};

/// Pixels por "linha" da roda — o mesmo valor do `scrollable` do iced, para a
/// distância total por clique não mudar, só ficar fluida.
const PIXELS_POR_LINHA: f32 = 60.0;

/// Constante de tempo do decaimento, em segundos: em ~3τ (~0,1 s) já foi
/// entregue 95% do clique.
const TAU: f32 = 0.035;

/// Abaixo disto (px) o resto é entregue de uma vez e a animação termina.
const RESTO_MINIMO: f32 = 0.5;

/// Ver o [módulo](self). Criado por [`smooth_scroll`].
pub struct SmoothScroll<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
}

/// Envolve `content` (um `scrollable`) com a rolagem suave da roda.
pub fn smooth_scroll<'a, Message>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
) -> SmoothScroll<'a, Message> {
    SmoothScroll {
        content: content.into(),
    }
}

#[derive(Default)]
struct State {
    /// Distância ainda por entregar ao filho, nos dois eixos (px, no sinal de
    /// `ScrollDelta::Pixels`).
    resto: Vector,
    /// Instante do último quadro processado, para medir o `dt`.
    ultimo: Option<Instant>,
    /// Shift está segurado (visto por `ModifiersChanged`).
    shift: bool,
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for SmoothScroll<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
        let filho = layout.children().next().unwrap();

        match event {
            // Roda com passos: não passa para o filho; vira distância pendente.
            // Só quando o cursor está em cima e ninguém (um scrollable mais
            // interno) já tratou o evento.
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x, y },
            }) if cursor.is_over(layout.bounds()) && !shell.is_event_captured() => {
                let state = tree.state.downcast_mut::<State>();
                // Shift troca os eixos, como no scrollable do iced.
                let (dx, dy) = (*x * PIXELS_POR_LINHA, *y * PIXELS_POR_LINHA);
                state.resto = if state.shift {
                    Vector::new(state.resto.x + dy, state.resto.y + dx)
                } else {
                    Vector::new(state.resto.x + dx, state.resto.y + dy)
                };
                if state.ultimo.is_none() {
                    state.ultimo = Some(Instant::now());
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let state = tree.state.downcast_mut::<State>();
                let pendente = state.resto.x != 0.0 || state.resto.y != 0.0;
                if pendente {
                    let dt = state
                        .ultimo
                        .map(|t| now.saturating_duration_since(t).as_secs_f32())
                        .unwrap_or(0.0)
                        // Quadro perdido (janela parada): não entrega tudo de uma vez.
                        .clamp(0.0, 0.05);
                    state.ultimo = Some(*now);

                    let fracao = 1.0 - (-dt / TAU).exp();
                    let mut passo = Vector::new(state.resto.x * fracao, state.resto.y * fracao);
                    if state.resto.x.abs() < RESTO_MINIMO {
                        passo.x = state.resto.x;
                    }
                    if state.resto.y.abs() < RESTO_MINIMO {
                        passo.y = state.resto.y;
                    }
                    state.resto = Vector::new(state.resto.x - passo.x, state.resto.y - passo.y);
                    if state.resto.x.abs() < RESTO_MINIMO && state.resto.y.abs() < RESTO_MINIMO {
                        state.resto = Vector::ZERO;
                    }
                    let continua = state.resto != Vector::ZERO;
                    if !continua {
                        state.ultimo = None;
                    }

                    // Entrega ao `scrollable` como se fosse touchpad. O cursor
                    // real vale: o filho só rola se o mouse ainda está em cima.
                    let sintetico = Event::Mouse(mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels {
                            x: passo.x,
                            y: passo.y,
                        },
                    });
                    self.content.as_widget_mut().update(
                        &mut tree.children[0],
                        &sintetico,
                        filho,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                    if continua {
                        shell.request_redraw();
                    }
                }
            }
            Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m)) => {
                tree.state.downcast_mut::<State>().shift = m.shift();
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            filho,
            cursor,
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            defaults,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<SmoothScroll<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(w: SmoothScroll<'a, Message>) -> Self {
        Element::new(w)
    }
}
