//! Foco de teclado para os links de um `prose`.
//!
//! O `rich_text` do `iced` desenha e clica os links, mas não é *focável*: Tab passa
//! reto por ele. [`FocusLinks`] o embrulha e acrescenta o que falta:
//!
//! - entra na ordem de Tab (implementa [`Focusable`], então a operação
//!   `focus_next`/`focus_previous` do motor o enxerga como um campo qualquer);
//! - com o foco, **Tab / Shift+Tab andam de link em link** dentro do parágrafo e só
//!   saem dele depois do último / antes do primeiro;
//! - **Enter** ou **Espaço** ativam o link em foco (a mesma mensagem do clique);
//! - o link em foco é desenhado em destaque (fundo, sublinhado).
//!
//! # Como o destaque é desenhado
//!
//! O `Rich` do `iced` guarda a geometria dos trechos num estado privado, então o
//! embrulho não sabe *onde* está o link para pintar um retângulo. Em vez disso, ele
//! recebe **uma variante pronta por link** — o mesmo texto com aquele link em
//! destaque — e desenha a que corresponde ao link em foco. O destaque (fundo e
//! sublinhado) não muda a geometria, então o `layout` da primeira vale para todas.
//! Custo: N variantes para N links, e N costuma ser pequeno.
//!
//! O estado (qual link está em foco) mora na árvore de widgets, não no `view`: o
//! `view` reconstrói o parágrafo a cada quadro e não sabe disso — e não precisa.

use iced::advanced::widget::operation::{Focusable, Operation};
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::keyboard::{self, Key, key::Named};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

/// O embrulho. `elementos[0]` é o parágrafo sem destaque; `elementos[1 + i]` é o
/// mesmo parágrafo com o link `i` em destaque. `links[i]` é a mensagem do link `i`.
pub struct FocusLinks<'a, Message, Theme, Renderer> {
    elementos: Vec<Element<'a, Message, Theme, Renderer>>,
    links: Vec<Message>,
    id: Option<widget::Id>,
}

impl<'a, Message, Theme, Renderer> FocusLinks<'a, Message, Theme, Renderer> {
    pub fn new(
        base: Element<'a, Message, Theme, Renderer>,
        destaques: Vec<Element<'a, Message, Theme, Renderer>>,
        links: Vec<Message>,
    ) -> Self {
        debug_assert_eq!(destaques.len(), links.len());
        let mut elementos = vec![base];
        elementos.extend(destaques);
        Self {
            elementos,
            links,
            id: None,
        }
    }

    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
}

#[derive(Debug, Default)]
struct Estado {
    focado: bool,
    /// Índice do link em foco; só vale com `focado`.
    atual: usize,
}

impl Focusable for Estado {
    fn is_focused(&self) -> bool {
        self.focado
    }

    fn focus(&mut self) {
        self.focado = true;
        self.atual = 0;
    }

    fn unfocus(&mut self) {
        self.focado = false;
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FocusLinks<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Estado>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Estado::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.elementos.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.elementos);
    }

    fn size(&self) -> Size<Length> {
        self.elementos[0].as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.elementos[0].as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Todas as variantes precisam do próprio `layout`: é ele que preenche o
        // estado (o parágrafo moldado) que o `draw` de cada uma lê.
        let mut primeiro = None;
        for (i, el) in self.elementos.iter_mut().enumerate() {
            let no = el
                .as_widget_mut()
                .layout(&mut tree.children[i], renderer, limits);
            primeiro.get_or_insert(no);
        }
        primeiro.expect("FocusLinks sempre tem o parágrafo base")
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
        let estado = tree.state.downcast_ref::<Estado>();
        let i = if estado.focado && !self.links.is_empty() {
            1 + estado.atual.min(self.links.len() - 1)
        } else {
            0
        };
        self.elementos[i].as_widget().draw(
            &tree.children[i],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let estado = tree.state.downcast_mut::<Estado>();
        operation.focusable(self.id.as_ref(), layout.bounds(), estado);
        self.elementos[0].as_widget_mut().operate(
            &mut tree.children[0],
            layout,
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let estado = tree.state.downcast_mut::<Estado>();
        let n = self.links.len();

        if estado.focado && n > 0 {
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key.as_ref() {
                        Key::Named(Named::Tab) => {
                            // Dentro do parágrafo o Tab anda de link em link; no
                            // último (ou primeiro, com Shift) ele segue adiante e
                            // a operação global de foco move para o vizinho.
                            if modifiers.shift() && estado.atual > 0 {
                                estado.atual -= 1;
                                shell.capture_event();
                                shell.request_redraw();
                                return;
                            }
                            if !modifiers.shift() && estado.atual + 1 < n {
                                estado.atual += 1;
                                shell.capture_event();
                                shell.request_redraw();
                                return;
                            }
                        }
                        Key::Named(Named::Enter | Named::Space) => {
                            shell.publish(self.links[estado.atual.min(n - 1)].clone());
                            shell.capture_event();
                            return;
                        }
                        _ => {}
                    }
                }
                // Clicar fora tira o foco: sem isso o destaque ficaria preso.
                Event::Mouse(mouse::Event::ButtonPressed(_)) if !cursor.is_over(layout.bounds()) => {
                    estado.focado = false;
                    shell.request_redraw();
                }
                _ => {}
            }
        }

        self.elementos[0].as_widget_mut().update(
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
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.elementos[0].as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.elementos[0].as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<FocusLinks<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(w: FocusLinks<'a, Message, Theme, Renderer>) -> Self {
        Element::new(w)
    }
}
