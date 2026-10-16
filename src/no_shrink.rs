//! Botão que não encolhe abaixo do próprio texto.
//!
//! O layout de linha do iced entrega a cada filho `Shrink` o que **sobrou** da
//! largura depois dos irmãos anteriores. Quando a janela estreita, os últimos
//! botões da linha recebem menos que o rótulo precisa: o botão fica menor que o
//! texto e o texto vaza para fora dele, e tudo ao redor embaralha.
//!
//! Este widget mede o filho com largura **ilimitada** e devolve o tamanho
//! natural dele: o botão nunca fica menor que o conteúdo. Se a linha não cabe,
//! o excesso transborda (e a janela corta) em vez de espremer o texto; quem
//! quer que a linha se acomode usa `<flow>`, que quebra em várias linhas.
//!
//! Só se aplica a filho de largura `Shrink`: `fill` e larguras fixas seguem
//! mandando, como o autor da tela pediu.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::tree::Tree;
use iced::advanced::{Clipboard, Shell, Widget, overlay};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// Ver o [módulo](self). Criado por [`no_shrink`].
pub struct NoShrink<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
}

/// Envolve `content` para que ele não encolha abaixo do tamanho natural.
pub fn no_shrink<'a, Message>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
) -> NoShrink<'a, Message> {
    NoShrink {
        content: content.into(),
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for NoShrink<'_, Message> {
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
        let max = limits.max();
        let livre = layout::Limits::new(
            limits.min(),
            Size::new(f32::INFINITY, max.height),
        );
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &livre);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
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

impl<'a, Message: 'a> From<NoShrink<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(w: NoShrink<'a, Message>) -> Self {
        Element::new(w)
    }
}
