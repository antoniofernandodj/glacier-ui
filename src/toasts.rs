//! Notificações "toast": info, sucesso, aviso e erro, empilhadas no canto
//! inferior direito da tela e dispensadas sozinhas depois de um tempo.
//!
//! Ao contrário de um [`crate::dialogs::DialogSpec`] — modal, único, bloqueia
//! interação com o resto da tela — um toast é não-modal e cumulativo: vários
//! podem estar em exibição ao mesmo tempo, cada um com seu próprio
//! cronômetro de expiração, e o resto da UI continua clicável por baixo dele.
//!
//! ```ignore
//! ctx.show_toast(ToastSpec::success("Deploy concluído"));
//! ctx.show_toast(ToastSpec::error("Falha ao conectar").with_title("Erro de rede"));
//! ```
//!
//! O motor mantém a lista de toasts ativos (com o instante em que cada um
//! apareceu) fora do [`crate::component::Context`] — assim como o diálogo,
//! isso é aplicado depois que `update()` retorna. A expiração automática
//! depende de um tique periódico (ver [`crate::GlacierUI::toast_subscription`]);
//! sem ele os toasts continuam sendo mostrados/fechados manualmente (clique no
//! "×"), só não somem sozinhos.

use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::widget::{Space, button, column, container, row, text};
use iced::{
    Alignment, Background, Border, Color, Element, Event, Length, Rectangle, Shadow, Size, Vector,
    window,
};
use std::time::{Duration, Instant};

use crate::widget::EngineMessage;

/// O tipo do toast, escolhendo o ícone e a cor de destaque — o mesmo papel de
/// [`crate::dialogs::DialogIcon`], mas sem a variante `Question` (não faz
/// sentido perguntar algo numa notificação que desaparece sozinha) e com
/// `Success` (que os diálogos não têm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    fn glyph(self) -> &'static str {
        match self {
            ToastKind::Info => "ℹ",
            ToastKind::Success => "✓",
            ToastKind::Warning => "⚠",
            ToastKind::Error => "✕",
        }
    }

    /// Cor de destaque (ícone + barra lateral), lida da paleta estendida do
    /// tema ativo — `Success` usa `palette.success`, que os diálogos não têm
    /// motivo para usar (nenhuma variante de `QMessageBox` é "deu certo").
    fn color(self, palette: &iced::theme::palette::Extended) -> Color {
        match self {
            ToastKind::Info => palette.primary.base.color,
            ToastKind::Success => palette.success.base.color,
            ToastKind::Warning => palette.warning.base.color,
            ToastKind::Error => palette.danger.base.color,
        }
    }
}

/// A especificação de um toast: tipo, título opcional (linha em negrito,
/// acima da mensagem), mensagem e por quanto tempo fica em exibição antes de
/// expirar sozinho (ver [`crate::GlacierUI::toast_subscription`]).
///
/// Construída com um dos atalhos [`ToastSpec::info`], [`ToastSpec::success`],
/// [`ToastSpec::warning`], [`ToastSpec::error`] (todos com `duration` padrão
/// de 4 segundos) e ajustada com os builders [`ToastSpec::with_title`] /
/// [`ToastSpec::with_duration`].
#[derive(Debug, Clone)]
pub struct ToastSpec {
    pub kind: ToastKind,
    pub title: Option<String>,
    pub message: String,
    pub duration: Duration,
}

impl ToastSpec {
    /// Tempo padrão em exibição antes de expirar, para os construtores de
    /// conveniência (`info`/`success`/`warning`/`error`).
    const DEFAULT_DURATION: Duration = Duration::from_secs(4);

    /// Um toast do zero, com o tipo, mensagem e duração padrão explícitos.
    pub fn new(kind: ToastKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            title: None,
            message: message.into(),
            duration: Self::DEFAULT_DURATION,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Info, message)
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Success, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Warning, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Error, message)
    }

    /// Anexa um título em negrito, mostrado acima da mensagem.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Substitui a duração padrão (4s) de exibição antes da expiração automática.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Quanto dura a entrada (o cartão sobe de baixo) e a saída (desliza para fora da tela).
/// A saída acontece **depois** de `duration`, então o motor só remove o toast ao fim dela
/// (ver [`EXIT`] em `prune_expired_toasts`).
pub const ENTER: Duration = Duration::from_millis(280);
pub const EXIT: Duration = Duration::from_millis(240);

/// Quanto o cartão sobe na entrada, em pixels.
const ENTER_RISE: f32 = 28.0;

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

fn ease_in(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t
}

/// Embrulho que anima o cartão: **sobe** ao aparecer, **desliza para a direita** ao sair e
/// desenha uma barra lateral que começa cheia e vai esvaziando até o fim do tempo de exibição.
///
/// O relógio é do próprio widget (`base` + o tempo desde que a `view` foi montada), e não do
/// motor, porque o iced só remonta a `view` a cada mensagem: entre duas mensagens o `draw` roda
/// com o mesmo `Element`, e é ele quem tem de saber que o tempo passou. Sem estado na árvore
/// de propósito — a árvore do iced é posicional, e quando um toast da frente some os de trás
/// herdariam o estado errado.
struct Animado<'a> {
    filho: Element<'a, EngineMessage>,
    /// Há quanto tempo o toast estava em exibição quando a `view` foi montada.
    base: Duration,
    montado_em: Instant,
    duration: Duration,
    accent: Color,
}

impl Animado<'_> {
    fn decorrido(&self) -> Duration {
        self.base + self.montado_em.elapsed()
    }

    fn terminou(&self) -> bool {
        self.decorrido() >= self.duration + EXIT
    }
}

impl<'a> Widget<EngineMessage, iced::Theme, iced::Renderer> for Animado<'a> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.filho)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.filho));
    }

    fn size(&self) -> Size<Length> {
        self.filho.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.filho.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.filho
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let bounds = layout.bounds();
        let e = self.decorrido();

        // Entrada: sobe e assenta. Saída: desliza para fora pela direita.
        let t_in = e.as_secs_f32() / ENTER.as_secs_f32();
        let t_out = e.saturating_sub(self.duration).as_secs_f32() / EXIT.as_secs_f32();
        let deslocamento = Vector::new(
            ease_in(t_out) * (bounds.width + 40.0),
            (1.0 - ease_out(t_in)) * ENTER_RISE,
        );

        renderer.with_translation(deslocamento, |renderer| {
            self.filho.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );

            // A barra lateral: cheia no começo, vazia no fim; o topo desce.
            let restante = 1.0 - (e.as_secs_f32() / self.duration.as_secs_f32().max(0.001)).clamp(0.0, 1.0);
            let altura_total = (bounds.height - 18.0).max(0.0);
            let altura = altura_total * restante;
            if altura > 0.5 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + 5.0,
                            y: bounds.y + 9.0 + (altura_total - altura),
                            width: 3.0,
                            height: altura,
                        },
                        border: Border {
                            radius: iced::border::Radius::new(1.5),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    Background::Color(self.accent),
                );
            }
        });
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        op: &mut dyn Operation,
    ) {
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, EngineMessage>,
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
        // O iced redesenha sob demanda: enquanto o toast anima (entra, esvazia a barra, sai),
        // cada quadro pede o seguinte (~30 fps bastam para uma barra e um deslize).
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && !self.terminou()
        {
            shell.request_redraw_at(*now + Duration::from_millis(33));
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.filho
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, EngineMessage, iced::Theme, iced::Renderer>> {
        self.filho
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

impl<'a> From<Animado<'a>> for Element<'a, EngineMessage> {
    fn from(w: Animado<'a>) -> Self {
        Element::new(w)
    }
}

/// Renderiza a pilha de toasts ativos como um overlay não-modal, ancorado no
/// canto inferior direito da tela. Ao contrário de [`crate::dialogs::overlay`],
/// não cobre a tela inteira com um fundo capturando clique/hover — só os
/// próprios cartões ocupam espaço de interação; clicar fora deles (em
/// qualquer ponto sem um cartão) atravessa para a camada de baixo normalmente.
///
/// `active` é `(id, spec)` de cada toast em exibição, na ordem em que devem
/// aparecer (mais antigo no topo, mais recente embaixo — a ordem de
/// [`crate::GlacierUI`]'s lista interna), com há quanto tempo cada um está em exibição
/// (anima a entrada, a barra de tempo e a saída). Cada cartão tem um botão "×" que
/// despacha [`EngineMessage::ToastDismiss`] com o `id` correspondente.
pub fn overlay<'a>(
    active: impl IntoIterator<Item = (u64, &'a ToastSpec, Duration)>,
    theme: &iced::Theme,
) -> Element<'a, EngineMessage> {
    let palette = theme.extended_palette();

    let mut list = column![].spacing(10).align_x(Alignment::End);
    let montado_em = Instant::now();
    for (id, spec, decorrido) in active {
        list = list.push(Animado {
            filho: toast_card(id, spec, palette),
            base: decorrido,
            montado_em,
            duration: spec.duration,
            accent: spec.kind.color(palette),
        });
    }

    container(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .align_x(Alignment::End)
        .align_y(Alignment::End)
        .into()
}

/// Renderiza um único cartão de toast, colorido pelo seu [`ToastKind`].
fn toast_card<'a>(
    id: u64,
    spec: &'a ToastSpec,
    palette: &iced::theme::palette::Extended,
) -> Element<'a, EngineMessage> {
    let accent = spec.kind.color(palette);
    let text_color = palette.background.base.text;

    let mut header = row![text(spec.kind.glyph()).size(16).color(accent)]
        .spacing(8)
        .align_y(Alignment::Center);
    if let Some(title) = &spec.title {
        header = header.push(text(title.as_str()).size(14).color(text_color));
    }
    header = header.push(Space::new().width(Length::Fill));
    header = header.push(dismiss_button(id, text_color));

    let card = column![
        header,
        text(spec.message.as_str()).size(13).color(text_color)
    ]
    .spacing(6);

    // Elevação por fundo opaco, não por `shadow` — ver `TROUBLESHOOTING.md`.
    // Nos drivers Vulkan incompletos (Intel Ivy Bridge/Haswell na Mesa) o quad
    // de uma sombra translúcida se soma quadro após quadro em vez de ser
    // apagado com o corpo do container, e o toast escurece até ficar preto. A
    // borda de 2px na cor do tipo já destaca o cartão; um fundo um tom mais
    // claro que a tela completa o efeito sem nada translúcido.
    let card_bg = palette.background.weak.color;
    container(card)
        .width(Length::Fixed(320.0))
        .padding(iced::Padding {
            top: 14.0,
            right: 14.0,
            bottom: 14.0,
            left: 20.0,
        })
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(Background::Color(card_bg)),
            border: Border {
                radius: iced::border::Radius::new(8.0),
                width: 2.0,
                color: accent,
            },
            ..Default::default()
        })
        .into()
}

/// O botão "×" que fecha um toast antes da expiração automática.
fn dismiss_button<'a>(id: u64, text_color: Color) -> Element<'a, EngineMessage> {
    button(text("×").size(16).color(text_color))
        .padding([0, 6])
        .style(move |_theme: &iced::Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Color {
                    a: 0.12,
                    ..text_color
                },
                button::Status::Pressed => Color {
                    a: 0.2,
                    ..text_color
                },
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color,
                border: Border {
                    radius: iced::border::Radius::new(4.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .on_press(EngineMessage::ToastDismiss(id))
        .into()
}
