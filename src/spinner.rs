//! `QProgressBar` indeterminado (`setRange(0, 0)`) / `BusyIndicator` do QML:
//! um anel de pontos girando sem fim, para operações sem duração conhecida
//! (carregando, conectando, …). Ver [`crate::style`] para a "galeria de
//! widgets" e `ANIMACOES.md` para o padrão de animação por trás deste widget
//! (é a mesma receita do `crate::animated_toggler`).
//!
//! # Por que isto NÃO é bloqueado pelo "estado por instância"
//!
//! O `PLANO_WIDGETS.md` marca `Spinner`/`BusyIndicator` como `●` — a mesma
//! marca de `Tabs`/`SpinBox`/`Calendar`, hoje bloqueados porque `ctx.set`
//! grava num **único** contexto global e duas instâncias colidiriam. Um
//! indicador indeterminado não sofre disso: ele não guarda **valor** nenhum
//! (não há "qual instância tem o quê" para colidir) — só precisa de uma FASE
//! de rotação que avança sozinha com o tempo. Essa fase mora no `tree::State`
//! do próprio widget do iced, endereçado pela posição na árvore (o mesmo
//! mecanismo que dá a cada `text_input` seu próprio cursor) — então N
//! spinners na tela giram cada um com seu relógio, sem escrever nada no
//! contexto do app.
//!
//! # Desenho sem `canvas`
//!
//! Em vez de um arco desenhado via `canvas`/`Frame` (que puxaria o trait
//! `geometry::Renderer`, mais pesado), o anel é N pontos circulares
//! desenhados com o `fill_quad` de baixo nível — a mesma técnica do knob do
//! `AnimatedToggler`. Cada ponto tem sua opacidade calculada pela distância
//! angular até a "cabeça" da rotação: o de trás é quase opaco, o mais distante
//! quase invisível, dando o efeito de rastro giratório.

use std::f32::consts::TAU;
use std::time::{Duration, Instant};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::animation::{Animation, Easing};
use iced::{Border, Color, Element, Event, Length, Rectangle, Size, mouse, window};

/// Quantos pontos formam o anel.
const DOTS: usize = 8;
/// Uma volta completa a cada 900ms — perto do `BusyIndicator` do QML.
const REVOLUTION: Duration = Duration::from_millis(900);
/// Opacidade mínima do ponto mais "velho" do rastro (nunca some de todo).
const MIN_ALPHA: f32 = 0.12;

/// Ver o [módulo](self). Criado por [`spinner`].
pub struct Spinner {
    size: f32,
    color: Option<Color>,
}

/// Um [`Spinner`] no tamanho default ([`Spinner::DEFAULT_SIZE`]). Sem
/// [`Spinner::color`], o anel usa o `primary` do tema ativo — inclusive o de
/// um estilo builtin (`crate::style`), sem precisar de GSS nenhum.
pub fn spinner() -> Spinner {
    Spinner {
        size: Spinner::DEFAULT_SIZE,
        color: None,
    }
}

impl Spinner {
    /// O tamanho default (diâmetro) de um [`Spinner`].
    pub const DEFAULT_SIZE: f32 = 24.0;

    /// Diâmetro do anel em px.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Cor dos pontos. Sem chamar isto, cai no `primary` do tema.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Estado vivo na árvore de widgets: só o relógio da rotação (ver o [módulo](self)
/// sobre por que isto não colide entre instâncias).
struct State {
    /// Progresso 0→1 de uma volta, repetindo para sempre (ver
    /// [`Animation::repeat_forever`]) — `Easing::Linear` para velocidade
    /// angular constante (um spinner não "desacelera" a cada volta).
    animation: Animation<f32>,
    now: Instant,
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Spinner {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let now = Instant::now();
        let mut animation = Animation::new(0.0)
            .duration(REVOLUTION)
            .easing(Easing::Linear)
            .repeat_forever();
        animation.go_mut(1.0, now);
        tree::State::new(State { animation, now })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.size), Length::Fixed(self.size))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.size, self.size))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        // Gira para sempre enquanto estiver na tela: cada quadro desenhado
        // agenda o seguinte. Ao contrário do `AnimatedToggler` (que para
        // quando a transição termina), aqui `is_animating` nunca fica falso —
        // `repeat_forever` é justamente o sinal de "continue pedindo".
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<State>();
            state.now = *now;
            shell.request_redraw();
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
        let center = bounds.center();

        let color = self
            .color
            .unwrap_or_else(|| theme.extended_palette().primary.base.color);

        // Fase atual da "cabeça" do rastro, em radianos (0 = topo, sentido horário).
        // `Animation<f32>` não tem o `interpolate(start, end, at)` de açúcar que
        // só existe para `Animation<bool>` — projeta o progresso 0..1 cru
        // (`interpolate_with` com identidade) e escala para radianos aqui.
        let progress = state.animation.interpolate_with(|t| t, state.now);
        let phase = progress * TAU;

        let dot_diameter = self.size * 0.18;
        let dot_radius = dot_diameter / 2.0;
        let track_radius = self.size / 2.0 - dot_radius;

        for i in 0..DOTS {
            let dot_angle = i as f32 * TAU / DOTS as f32;
            // Distância angular (0..TAU) de "quão atrás da cabeça" este ponto
            // está — 0 é o ponto mais novo/brilhante do rastro.
            let behind = (phase - dot_angle).rem_euclid(TAU);
            let alpha = MIN_ALPHA + (1.0 - MIN_ALPHA) * (1.0 - behind / TAU);

            // Ângulo 0 no topo (12h), como um relógio, não à direita (3h).
            let theta = dot_angle - std::f32::consts::FRAC_PI_2;
            let dot_center_x = center.x + track_radius * theta.cos();
            let dot_center_y = center.y + track_radius * theta.sin();

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: dot_center_x - dot_radius,
                        y: dot_center_y - dot_radius,
                        width: dot_diameter,
                        height: dot_diameter,
                    },
                    border: Border {
                        radius: iced::border::Radius::new(dot_radius),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..renderer::Quad::default()
                },
                Color { a: alpha, ..color },
            );
        }
    }
}

impl<'a, Message: 'a> From<Spinner> for Element<'a, Message, iced::Theme, iced::Renderer> {
    fn from(spinner: Spinner) -> Self {
        Element::new(spinner)
    }
}
