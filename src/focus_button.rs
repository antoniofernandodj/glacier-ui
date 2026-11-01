//! Foco de teclado para o `button`.
//!
//! O `button` do `iced` não é *focável*: Tab passa reto por ele, e Enter/Espaço não o
//! acionam. Para quem só usa o teclado (ou um leitor de tela) um app inteiro de botões
//! fica inalcançável. [`FocusButton`] o embrulha e acrescenta o que falta — o mesmo desenho
//! do [`crate::link_focus`]:
//!
//! - entra na ordem de Tab (implementa [`Focusable`], então a operação `focus_next` /
//!   `focus_previous` do motor o enxerga como um campo qualquer);
//! - **Enter** ou **Espaço**, com o foco, publicam a MESMA mensagem do clique (a que o
//!   `on_press` do botão emitiria);
//! - o foco é desenhado como um anel de 2 px na cor de destaque do tema;
//! - clicar fora tira o foco (sem isso o anel ficaria preso).
//!
//! Um botão `disabled` não tem mensagem, e portanto não entra na ordem de Tab.

use iced::advanced::widget::operation::{Focusable, Operation};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::keyboard::{self, Key, key::Named};
use iced::{Border, Color, Element, Event, Length, Rectangle, Size, Vector};

pub struct FocusButton<'a, Message> {
    botao: Element<'a, Message, iced::Theme, iced::Renderer>,
    /// A mensagem do clique; `None` = desabilitado (fora da ordem de Tab).
    mensagem: Option<Message>,
}

impl<'a, Message> FocusButton<'a, Message> {
    pub fn new(
        botao: Element<'a, Message, iced::Theme, iced::Renderer>,
        mensagem: Option<Message>,
    ) -> Self {
        Self { botao, mensagem }
    }
}

#[derive(Debug, Default)]
struct Estado {
    focado: bool,
}

impl Focusable for Estado {
    fn is_focused(&self) -> bool {
        self.focado
    }
    fn focus(&mut self) {
        self.focado = true;
    }
    fn unfocus(&mut self) {
        self.focado = false;
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, iced::Theme, iced::Renderer> for FocusButton<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Estado>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Estado::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.botao)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.botao));
    }

    fn size(&self) -> Size<Length> {
        self.botao.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.botao.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        self.botao.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
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
        self.botao
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
        if tree.state.downcast_ref::<Estado>().focado {
            use iced::advanced::Renderer as _;
            let b = layout.bounds();
            let cor = theme.extended_palette().primary.strong.color;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: b.x - 2.0,
                        y: b.y - 2.0,
                        width: b.width + 4.0,
                        height: b.height + 4.0,
                    },
                    border: Border { color: cor, width: 2.0, radius: 6.0.into() },
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        // Desabilitado: não aparece na ordem de Tab.
        if self.mensagem.is_some() {
            let estado = tree.state.downcast_mut::<Estado>();
            operation.focusable(None, layout.bounds(), estado);
        }
        self.botao
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        let estado = tree.state.downcast_mut::<Estado>();
        if estado.focado {
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    if let (Key::Named(Named::Enter | Named::Space), Some(m)) =
                        (key.as_ref(), &self.mensagem)
                    {
                        shell.publish(m.clone());
                        shell.capture_event();
                        return;
                    }
                }
                // Clicar fora tira o foco: sem isso o anel ficaria preso.
                Event::Mouse(mouse::Event::ButtonPressed(_)) if !cursor.is_over(layout.bounds()) => {
                    estado.focado = false;
                    shell.request_redraw();
                }
                _ => {}
            }
        }
        self.botao.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        self.botao
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
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.botao
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

impl<'a, Message: Clone + 'a> From<FocusButton<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(w: FocusButton<'a, Message>) -> Self {
        Element::new(w)
    }
}
