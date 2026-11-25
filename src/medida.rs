//! Layout **sem janela**: monta a tela com o renderizador de software, roda o
//! layout do `iced` num tamanho de janela dado e devolve os nós que ficaram com
//! largura ou altura zero tendo conteúdo — o defeito que o `glacier test` não
//! pegava, porque ele avalia a árvore mas não a mede (S8/S9 do
//! `docs/PLANO_LINGUAGEM_GV_GSS.md`; a família de casos está no `PRIMITIVAS.md`,
//! "A armadilha do `Length::Fill`").
//!
//! Como funciona: com a medição ligada, o `render_node` embrulha cada nó num
//! [`Medidor`], que repassa tudo ao filho e, no `layout`, anota o tamanho que o
//! filho recebeu, pelo `node_id`. Depois o relatório anda a árvore avaliada,
//! junta os tamanhos e descreve cada nó colapsado pelo caminho até ele
//! (`column.form-panel > row.env__add > button "Salvar"`), já que o `UiNode` não
//! guarda a linha do arquivo.
//!
//! Não mede o desenho (cores, fontes de verdade): o texto é medido com as fontes
//! que o renderizador de software conhecer, o que basta para dizer "zero" ou
//! "não zero".

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse, overlay, renderer};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use crate::parser::{NodeType, UiNode};

thread_local! {
    static ATIVO: Cell<bool> = const { Cell::new(false) };
    static MEDIDAS: RefCell<HashMap<u64, Size>> = RefCell::new(HashMap::new());
}

/// A medição está ligada nesta thread?
pub(crate) fn ativo() -> bool {
    ATIVO.with(Cell::get)
}

/// Um nó que o layout deixou sem área.
#[derive(Debug, Clone, PartialEq)]
pub struct Colapso {
    /// O caminho até o nó: tag, `.classe` e o texto, do mais externo para dentro.
    pub caminho: String,
    pub largura: f32,
    pub altura: f32,
}

impl std::fmt::Display for Colapso {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let eixo = match (self.largura <= 0.0, self.altura <= 0.0) {
            (true, true) => "largura e altura zero",
            (true, false) => "largura zero",
            _ => "altura zero",
        };
        write!(f, "{}: {eixo} ({}×{})", self.caminho, self.largura, self.altura)
    }
}

/// Faz o layout de `elemento` numa janela de `tamanho` e devolve os nós de
/// `arvore` (a árvore avaliada que gerou o elemento) que ficaram sem área.
///
/// `montar` recebe o controle da medição ligada: o elemento TEM de ser montado
/// dentro dele, para os nós saírem embrulhados.
pub fn colapsos<'a, M>(
    arvore: &UiNode,
    tamanho: Size,
    montar: impl FnOnce() -> Element<'a, M>,
) -> Vec<Colapso>
where
    M: 'a,
{
    MEDIDAS.with(|m| m.borrow_mut().clear());
    ATIVO.with(|a| a.set(true));
    let elemento = montar();
    ATIVO.with(|a| a.set(false));

    let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(16.0),
    ));
    let ui = iced_runtime::UserInterface::build(
        elemento,
        tamanho,
        iced_runtime::user_interface::Cache::default(),
        &mut renderer,
    );
    drop(ui);

    let medidas = MEDIDAS.with(|m| std::mem::take(&mut *m.borrow_mut()));
    let mut out = Vec::new();
    anda(arvore, &medidas, &mut Vec::new(), &mut out);
    out
}

/// Desce a árvore; reporta o nó colapsado mais EXTERNO de cada ramo (os filhos de
/// um nó sem largura também não têm — dizer isso de cada um seria ruído).
fn anda(no: &UiNode, medidas: &HashMap<u64, Size>, caminho: &mut Vec<String>, out: &mut Vec<Colapso>) {
    if no.hidden == Some(true) {
        return;
    }
    caminho.push(rotulo(no));
    if let Some(sz) = medidas.get(&no.node_id)
        && (sz.width <= 0.0 || sz.height <= 0.0)
        && tem_conteudo(no)
        && !zero_de_proposito(no)
    {
        out.push(Colapso {
            caminho: caminho.join(" > "),
            largura: sz.width,
            altura: sz.height,
        });
        caminho.pop();
        return;
    }
    for c in no.children.iter() {
        anda(c, medidas, caminho, out);
    }
    caminho.pop();
}

/// O nó mostra alguma coisa? Um texto vazio ou um contêiner sem filhos pode ter
/// área zero sem que isso seja defeito.
fn tem_conteudo(no: &UiNode) -> bool {
    match &no.kind {
        NodeType::Text { content, .. } => !content.trim().is_empty(),
        NodeType::Column | NodeType::Row | NodeType::Container | NodeType::Stack => {
            no.children.iter().any(|c| c.hidden != Some(true) && tem_conteudo(c))
        }
        NodeType::Fragment => no.children.iter().any(tem_conteudo),
        _ => true,
    }
}

/// `width = 0`/`height = 0` declarados: zero de propósito.
fn zero_de_proposito(no: &UiNode) -> bool {
    let zero = |v: &Option<String>| v.as_deref().is_some_and(|s| s.trim().parse::<f32>() == Ok(0.0));
    zero(&no.width) || zero(&no.height)
}

fn rotulo(no: &UiNode) -> String {
    let mut s = no.kind.tag_name().unwrap_or("?").to_string();
    if let Some(c) = no.class.as_deref() {
        for parte in c.split_whitespace() {
            s.push('.');
            s.push_str(parte);
        }
    }
    if let NodeType::Text { content, .. } = &no.kind {
        let curto: String = content.chars().take(24).collect();
        s.push_str(&format!(" {curto:?}"));
    }
    s
}

/// Repassa tudo ao filho; no `layout`, anota o tamanho que ele recebeu.
pub struct Medidor<'a, Message, Theme, Renderer> {
    filho: Element<'a, Message, Theme, Renderer>,
    id: u64,
}

impl<'a, Message, Theme, Renderer> Medidor<'a, Message, Theme, Renderer> {
    pub fn new(filho: Element<'a, Message, Theme, Renderer>, id: u64) -> Self {
        Self { filho, id }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Medidor<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.filho.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.filho.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.filho.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.filho.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.filho.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.filho.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self.filho.as_widget_mut().layout(tree, renderer, limits);
        let sz = node.size();
        MEDIDAS.with(|m| {
            m.borrow_mut().insert(self.id, sz);
        });
        node
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
        self.filho.as_widget().draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, op: &mut dyn Operation) {
        self.filho.as_widget_mut().operate(tree, layout, renderer, op);
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
        self.filho
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
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
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Medidor<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(w: Medidor<'a, Message, Theme, Renderer>) -> Self {
        Element::new(w)
    }
}
