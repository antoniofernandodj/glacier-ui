//! Um toggler com a bolinha **animada** — o `iced::widget::toggler` de fábrica
//! desenha o knob teleportando de um lado para o outro; este widget é o mesmo
//! desenho (e o mesmo `toggler::Style`/`Catalog` de tema, então os estilos
//! builtin de [`crate::style`] valem sem mudança), mas com a posição do knob e
//! as cores interpoladas por uma [`iced::animation::Animation`] a cada quadro.
//!
//! Diferenças deliberadas em relação ao widget do iced:
//!
//! - **Sem rótulo embutido**: o texto ao lado é responsabilidade do chamador
//!   (`widget.rs` compõe uma `Row[toggler, text]`), o que dispensa replicar
//!   toda a máquina de layout/draw de parágrafo do iced aqui.
//! - **Estado de animação na árvore de widgets** (`tree::State`), transicionado
//!   no `diff()` — o ponto onde o iced reconcilia a árvore nova (rebuild pós
//!   `update` do app) com a antiga: é lá que "o `is_toggled` mudou" é visível.
//!   Enquanto a animação corre, cada `RedrawRequested` agenda o próximo quadro
//!   via `shell.request_redraw()`; parada a animação, o widget volta a ser
//!   estático (custo zero por quadro).
//!
//! O padrão completo — e o checklist para animar outros widgets — está
//! documentado em `ANIMACOES.md`, na raiz do repositório.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use std::time::Instant;

use iced::animation::{Animation, Easing};
use iced::widget::toggler;
use iced::{
    Background, Border, Color, Element, Event, Length, Rectangle, Size, mouse, touch, window,
};

/// Ver o [módulo](self). Criado por [`animated_toggler`].
pub struct AnimatedToggler<'a, Message> {
    is_toggled: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    size: f32,
}

/// Um [`AnimatedToggler`] no estado dado — mesmo desenho do `toggler` do iced,
/// com o knob animado.
pub fn animated_toggler<'a, Message>(is_toggled: bool) -> AnimatedToggler<'a, Message> {
    AnimatedToggler {
        is_toggled,
        on_toggle: None,
        size: 16.0,
    }
}

impl<'a, Message> AnimatedToggler<'a, Message> {
    /// Mensagem emitida ao alternar (com o novo estado). Sem isto o toggler é
    /// desenhado como desabilitado, igual ao widget do iced.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Altura do trilho em px (largura = 2×; default 16, como no iced).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    fn status(&self, cursor: mouse::Cursor, bounds: Rectangle) -> toggler::Status {
        if self.on_toggle.is_none() {
            toggler::Status::Disabled {
                is_toggled: self.is_toggled,
            }
        } else if cursor.is_over(bounds) {
            toggler::Status::Hovered {
                is_toggled: self.is_toggled,
            }
        } else {
            toggler::Status::Active {
                is_toggled: self.is_toggled,
            }
        }
    }
}

/// Estado vivo na árvore de widgets: a animação do knob (0 ⇄ 1), o alvo que ela
/// persegue (para o `diff` detectar a mudança) e o instante do último quadro
/// (o `draw` não recebe relógio; usa o do `RedrawRequested` mais recente).
struct State {
    animation: Animation<bool>,
    target: bool,
    now: Instant,
    last_status: Option<toggler::Status>,
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for AnimatedToggler<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            // Nasce já assentada no estado atual (sem animar na primeira
            // aparição). `quick` (200ms) + easeOutCubic ≈ o QStyle animado.
            animation: Animation::new(self.is_toggled)
                .quick()
                .easing(Easing::EaseOutCubic),
            target: self.is_toggled,
            now: Instant::now(),
            last_status: None,
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        if state.target != self.is_toggled {
            state.target = self.is_toggled;
            state.animation.go_mut(self.is_toggled, Instant::now());
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(2.0 * self.size, self.size))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if let Some(on_toggle) = &self.on_toggle
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. })
            )
            && cursor.is_over(layout.bounds())
        {
            shell.publish(on_toggle(!self.is_toggled));
            shell.capture_event();
        }

        let state = tree.state.downcast_mut::<State>();
        let current_status = self.status(cursor, layout.bounds());
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = *now;
            state.last_status = Some(current_status);
            // Enquanto a animação corre, cada quadro agenda o seguinte.
            if state.animation.is_animating(*now) {
                shell.request_redraw();
            }
        } else if state
            .last_status
            .is_some_and(|status| status != current_status)
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            if self.on_toggle.is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::NotAllowed
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        // Progresso 0 (desligado) → 1 (ligado) no instante do quadro atual.
        let progress = state.animation.interpolate(0.0_f32, 1.0, state.now);

        // As cores vêm do MESMO catálogo de tema do toggler do iced, avaliado
        // nos dois extremos e misturado pelo progresso — assim o fundo escorrega
        // de cinza para `primary` junto com o knob, e qualquer paleta (inclusive
        // as dos estilos builtin) funciona sem código novo.
        let styled = |is_toggled: bool| {
            toggler::default(
                theme,
                match state.last_status.unwrap_or(toggler::Status::Active {
                    is_toggled: self.is_toggled,
                }) {
                    toggler::Status::Active { .. } => toggler::Status::Active { is_toggled },
                    toggler::Status::Hovered { .. } => toggler::Status::Hovered { is_toggled },
                    toggler::Status::Disabled { .. } => toggler::Status::Disabled { is_toggled },
                },
            )
        };
        let off = styled(false);
        let on = styled(true);
        let style = if self.is_toggled { on } else { off };

        let border_radius = style
            .border_radius
            .unwrap_or_else(|| iced::border::Radius::new(bounds.height / 2.0));

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: border_radius,
                    width: style.background_border_width,
                    color: style.background_border_color,
                },
                ..renderer::Quad::default()
            },
            mix_background(off.background, on.background, progress),
        );

        let padding = (style.padding_ratio * bounds.height).round();
        let off_x = padding;
        let on_x = bounds.width - bounds.height + padding;
        let knob = Rectangle {
            x: bounds.x + off_x + (on_x - off_x) * progress,
            y: bounds.y + padding,
            width: bounds.height - (2.0 * padding),
            height: bounds.height - (2.0 * padding),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: knob,
                border: Border {
                    radius: border_radius,
                    width: style.foreground_border_width,
                    color: style.foreground_border_color,
                },
                ..renderer::Quad::default()
            },
            mix_background(off.foreground, on.foreground, progress),
        );
    }
}

/// Mistura linear entre as cores de dois `Background` sólidos (o catálogo do
/// toggler só produz sólidos). Um gradiente, se algum dia aparecer, cai no
/// extremo mais próximo em vez de interpolar.
fn mix_background(off: Background, on: Background, progress: f32) -> Background {
    match (off, on) {
        (Background::Color(a), Background::Color(b)) => {
            let t = progress.clamp(0.0, 1.0);
            Background::Color(Color {
                r: a.r + (b.r - a.r) * t,
                g: a.g + (b.g - a.g) * t,
                b: a.b + (b.b - a.b) * t,
                a: a.a + (b.a - a.a) * t,
            })
        }
        (a, b) => {
            if progress < 0.5 {
                a
            } else {
                b
            }
        }
    }
}

impl<'a, Message: 'a> From<AnimatedToggler<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(toggler: AnimatedToggler<'a, Message>) -> Self {
        Element::new(toggler)
    }
}
