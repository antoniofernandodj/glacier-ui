//! `Anchored`: um painel que **flutua sobre a tela**, ancorado a um widget —
//! o habilitador B da Onda 5 do `PLANO_WIDGETS.md`, e o motor por trás de
//! `<popover>`, `<popup>`, `<autocomplete>` e do `calendarPopup` do
//! `<dateedit>`.
//!
//! ```xml
//! <popover open="menu_usuario" placement="bottom" align="end">
//!     <button slot="anchor" text="Antônio ▾" on_click="app:toggle:menu_usuario" />
//!     <column class="painel"> … </column>
//! </popover>
//! ```
//!
//! # Por que este módulo existe, e não mais um `stack![]`
//!
//! `src/menu.rs` já sobrepõe painéis, e o `DIALOGS.md` já mapeou as armadilhas
//! — mas pela técnica pragmática dos diálogos: uma camada `stack![]` do tamanho
//! da janela, com o painel posicionado por um `padding` **calculado
//! analiticamente** a partir da posição do cursor no instante do clique. O
//! próprio `menu.rs` documenta o limite disso em duas frases: não há medição de
//! layout antes de posicionar, e a âncora é o **cursor**, não o widget.
//!
//! Para um menu de linhas de altura fixa aberta no ponto do clique, isso basta.
//! Para um popover, não: um painel ancorado a um botão precisa nascer alinhado
//! à borda **do botão** — e precisa saber o tamanho de si mesmo para decidir se
//! abre para baixo ou para cima. As duas coisas são exatamente o que o
//! `Widget::overlay()` do `iced` dá de graça:
//!
//! - **a âncora é o layout do gatilho** (`layout.bounds() + translation`), não
//!   o cursor — então o painel acompanha o botão quando a tela rola ou muda de
//!   tamanho, e acerta mesmo quando quem abriu foi o teclado;
//! - **o painel é medido antes de posicionado** (`Overlay::layout` recebe o
//!   tamanho da janela e devolve um nó já movido), então virar para o outro
//!   lado quando não cabe é uma conta, não um chute.
//!
//! É o "caminho nativo correto" que o cabeçalho de `menu.rs` descreve e adia
//! por falta de precedente. O precedente agora existe: `reveal.rs` e
//! `animated_toggler.rs` já são `iced::advanced::Widget`, e este é o primeiro a
//! ir até o `Overlay`.
//!
//! # A forma
//!
//! Um `Anchored` tem **dois** filhos e renderiza os dois em camadas
//! diferentes:
//!
//! | filho | onde vive | quando |
//! |---|---|---|
//! | `anchor` | no fluxo normal, onde o markup o pôs | sempre |
//! | `content` | numa camada por cima de tudo | só com `open` |
//!
//! O `anchor` é o que dá o **lugar**: mesmo num `<popup>` centrado na janela
//! (que não usa a âncora para posicionar) ele continua sendo o gatilho, e é
//! quem o `Tab` alcança. Um popover sem gatilho nenhum escreve um `<space/>`
//! ali — o motor faz isso sozinho quando o markup não marca `slot="anchor"`.
//!
//! # Quem fecha o painel
//!
//! O motor, não o app — e sempre pela mesma porta, `on_dismiss`, que é a
//! mensagem que zera a chave de `open`:
//!
//! - **clique fora** do painel: o overlay recebe o evento *antes* da árvore
//!   normal (é assim que o `iced` ordena as camadas), publica o `on_dismiss` e
//!   **consome** o clique. A consequência prática vale saber: o clique que
//!   fecha um popover não chega a mais nada — é o comportamento de um menu de
//!   SO, e é de propósito;
//! - **Esc**, pelo mesmo caminho;
//! - **clique no próprio gatilho**: cai no caso "fora do painel" acima, então
//!   um gatilho que alterna a chave fecha o painel e o clique não o reabre no
//!   mesmo quadro. Sem isso, alternar seria impossível.
//!
//! `dismiss="false"` desliga os dois primeiros, para o caso em que quem manda
//! no fechamento é o app (um wizard, um painel que só sai por um botão).
//!
//! # O que ele **não** faz
//!
//! Modalidade. Um `<dialog>` bloqueia a tela atrás dele (ver `DIALOGS.md`);
//! este painel não escurece nada nem impede o resto de existir — ele só está
//! por cima. É a diferença entre `QDialog` e `QMenu`, e é a razão de `<popup>`
//! não ser um diálogo centrado: um diálogo já existe.

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::tree::Tree;
use iced::advanced::{Clipboard, Shell, Widget};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, keyboard, mouse};

/// De que lado do gatilho o painel abre.
///
/// Todas as quatro **viram para o lado oposto** quando não cabem — e só quando
/// o outro lado cabe melhor, senão o painel acabaria pior do que começou. É a
/// mesma regra do `overlay::menu` do `iced` (que compara `space_below` com
/// `space_above`), generalizada para os dois eixos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Placement {
    /// Abaixo do gatilho — o default, e o que um menu suspenso faz.
    #[default]
    Bottom,
    /// Acima.
    Top,
    /// À direita, como um submenu.
    Right,
    /// À esquerda.
    Left,
    /// **Ignora a âncora** e centra o painel na janela. É o `<popup>`: a mesma
    /// primitiva sem ancoragem, pelo padrão `<dateedit>`/`<timeedit>`.
    Center,
}

impl Placement {
    /// `bottom` (default), `top`, `right`, `left`, `center` — com os sinônimos
    /// em pt-BR que o resto do markup aceita.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "top" | "acima" | "cima" => Self::Top,
            "right" | "direita" => Self::Right,
            "left" | "esquerda" => Self::Left,
            "center" | "centro" | "centro_janela" | "screen" => Self::Center,
            _ => Self::Bottom,
        }
    }

    fn vertical(self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }
}

/// Onde o painel encosta no **eixo transversal** ao do [`Placement`]: para um
/// painel que abre para baixo, é o alinhamento horizontal; para um que abre à
/// direita, o vertical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Bordas iniciais coincidem (esquerda com esquerda / topo com topo).
    #[default]
    Start,
    /// Centros coincidem.
    Center,
    /// Bordas finais coincidem — o alinhamento de um menu de usuário no canto
    /// direito da barra, que é o caso em que `start` jogaria o painel para fora
    /// da tela.
    End,
}

impl Align {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "center" | "centro" | "middle" | "meio" => Self::Center,
            "end" | "fim" | "final" | "right" | "direita" | "bottom" | "baixo" => Self::End,
            _ => Self::Start,
        }
    }

    /// A coordenada inicial do painel no eixo transversal.
    fn origem(self, anchor_inicio: f32, anchor_tamanho: f32, painel: f32) -> f32 {
        match self {
            Self::Start => anchor_inicio,
            Self::Center => anchor_inicio + (anchor_tamanho - painel) / 2.0,
            Self::End => anchor_inicio + anchor_tamanho - painel,
        }
    }
}

/// A largura do painel.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Largura {
    /// O que o conteúdo pedir. O default.
    #[default]
    Natural,
    /// Exatamente a do gatilho — o que um `<autocomplete>` quer, e o que faz a
    /// lista de sugestões parecer parte do campo em vez de um painel solto.
    Ancora,
    /// Um número de pixels, escrito no markup.
    Fixa(f32),
}

/// Ver o [módulo](self). Criado por [`anchored`].
pub struct Anchored<'a, Message> {
    /// `[0]` é o gatilho (fica no fluxo), `[1]` é o painel (vai para o
    /// overlay). Um `Vec` em vez de dois campos só para que `diff_children`
    /// receba a fatia inteira de uma vez, como um `Stack` faz.
    children: Vec<Element<'a, Message, iced::Theme, iced::Renderer>>,
    open: bool,
    placement: Placement,
    align: Align,
    offset: f32,
    largura: Largura,
    /// A mensagem que fecha. `None` = o painel não se fecha sozinho.
    on_dismiss: Option<Message>,
    /// A mensagem que **abre**, publicada quando o ponteiro é pressionado sobre
    /// o gatilho e o painel está fechado. `None` = quem abre é o app.
    ///
    /// Ela não consome o evento de propósito: o botão que serve de gatilho
    /// continua disparando o `on_click` dele. Abrir um painel é o que o widget
    /// faz *além* do que o markup pediu, não em vez disso.
    on_open: Option<Message>,
    /// Navegação por teclado **enquanto o painel está aberto** — ▲, ▼ e Enter.
    /// É o que faz um `<autocomplete>` ser usável sem o mouse, e mora aqui (e
    /// não num listener global como o do `<datetimeedit>`) porque o overlay
    /// recebe o evento antes de qualquer outra coisa: nem o `text_input`
    /// focado logo abaixo chega a ver a tecla.
    on_prev: Option<Message>,
    on_next: Option<Message>,
    on_accept: Option<Message>,
}

/// Um [`Anchored`] com `anchor` no fluxo e `content` por cima quando `open`.
pub fn anchored<'a, Message>(
    anchor: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    open: bool,
) -> Anchored<'a, Message> {
    Anchored {
        children: vec![anchor.into(), content.into()],
        open,
        placement: Placement::default(),
        align: Align::default(),
        offset: 4.0,
        largura: Largura::default(),
        on_dismiss: None,
        on_open: None,
        on_prev: None,
        on_next: None,
        on_accept: None,
    }
}

impl<'a, Message> Anchored<'a, Message> {
    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }

    pub fn align(mut self, a: Align) -> Self {
        self.align = a;
        self
    }

    /// Folga entre o gatilho e o painel, em pixels. Default 4.
    pub fn offset(mut self, o: f32) -> Self {
        self.offset = o;
        self
    }

    pub fn largura(mut self, l: Largura) -> Self {
        self.largura = l;
        self
    }

    /// A mensagem publicada quando o painel se fecha sozinho (clique fora,
    /// Esc). Sem ela o painel só sai quando o app zerar a chave.
    pub fn on_dismiss(mut self, m: Option<Message>) -> Self {
        self.on_dismiss = m;
        self
    }

    /// A mensagem publicada quando o gatilho é pressionado com o painel
    /// fechado — é o que faz um `<popover>` abrir sem uma linha de app.
    pub fn on_open(mut self, m: Option<Message>) -> Self {
        self.on_open = m;
        self
    }

    /// ▲, ▼ e Enter enquanto o painel está aberto. Cada uma consome a tecla —
    /// declarar só as que fazem sentido é o que deixa Enter passar para o
    /// `<textinput>` de baixo quando o painel não é uma lista.
    pub fn on_keys(
        mut self,
        prev: Option<Message>,
        next: Option<Message>,
        accept: Option<Message>,
    ) -> Self {
        self.on_prev = prev;
        self.on_next = next;
        self.on_accept = accept;
        self
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Anchored<'_, Message>
where
    Message: Clone,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        // O nó ocupa exatamente o que o gatilho ocupa: o painel não empurra
        // nada, é essa a diferença entre flutuar e estar no fluxo.
        self.children[0].as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Só o gatilho entra no layout desta camada. O painel é medido em
        // `Overlay::layout`, contra a janela inteira — é lá que ele sabe
        // quanto espaço tem de verdade.
        let filho =
            self.children[0]
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits);
        let tamanho = filho.size();
        layout::Node::with_children(tamanho, vec![filho])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            self.children[0].as_widget_mut().operate(
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
        let layout_gatilho = layout.children().next().unwrap();

        // Abrir vem ANTES de o gatilho ver o evento, e por um motivo concreto:
        // um `<button>` **consome** o pressionar dentro dos limites dele, então
        // um `mouse_area` por fora nunca chegaria a disparar. Aqui, quem manda
        // na ordem somos nós.
        //
        // E, ao contrário do fechar, abrir **não** consome: o botão que serve
        // de gatilho continua disparando o `on_click` que o markup lhe deu.
        if !self.open
            && let Some(abrir) = &self.on_open
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(iced::touch::Event::FingerPressed { .. })
            )
            && cursor.is_over(layout_gatilho.bounds())
        {
            shell.publish(abrir.clone());
        }

        self.children[0].as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout_gatilho,
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
        self.children[0].as_widget().mouse_interaction(
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
        self.children[0].as_widget().draw(
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
        // `split_at_mut` porque as duas metades saem daqui emprestadas ao mesmo
        // tempo: o gatilho pode ter overlay dele (um `<select>` no botão), e o
        // painel é o nosso.
        let (gatilho_el, painel_el) = self.children.split_at_mut(1);
        let (gatilho_tree, painel_tree) = tree.children.split_at_mut(1);
        let layout_gatilho = layout.children().next().unwrap();

        let do_gatilho = gatilho_el[0].as_widget_mut().overlay(
            &mut gatilho_tree[0],
            layout_gatilho,
            renderer,
            viewport,
            translation,
        );

        if !self.open {
            return do_gatilho;
        }

        // A âncora em coordenadas de janela. `translation` é o que um
        // `scrollable` acima já deslocou — sem somá-lo, um popover dentro de
        // uma lista rolada nasceria onde o gatilho *estaria* sem rolagem.
        let ancora = layout_gatilho.bounds() + translation;

        let painel = Panel {
            content: &mut painel_el[0],
            tree: &mut painel_tree[0],
            ancora,
            placement: self.placement,
            align: self.align,
            offset: self.offset,
            largura: self.largura,
            on_dismiss: self.on_dismiss.clone(),
            on_prev: self.on_prev.clone(),
            on_next: self.on_next.clone(),
            on_accept: self.on_accept.clone(),
        };
        let nosso = overlay::Element::new(Box::new(painel));

        Some(match do_gatilho {
            Some(dele) => overlay::Group::with_children(vec![dele, nosso]).overlay(),
            None => nosso,
        })
    }
}

/// O painel enquanto está no ar. Vive um quadro: é reconstruído a cada
/// `Widget::overlay`, como o `Overlay` do `pick_list` do próprio `iced`.
struct Panel<'a, 'b, Message> {
    content: &'b mut Element<'a, Message, iced::Theme, iced::Renderer>,
    tree: &'b mut Tree,
    ancora: Rectangle,
    placement: Placement,
    align: Align,
    offset: f32,
    largura: Largura,
    on_dismiss: Option<Message>,
    on_prev: Option<Message>,
    on_next: Option<Message>,
    on_accept: Option<Message>,
}

impl<Message> overlay::Overlay<Message, iced::Theme, iced::Renderer> for Panel<'_, '_, Message>
where
    Message: Clone,
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        // O teto é a janela inteira; um painel maior que ela vira um painel do
        // tamanho dela (e o `<scrollable>` de dentro, se houver, cuida do
        // resto).
        let mut limites = layout::Limits::new(Size::ZERO, bounds);
        limites = match self.largura {
            Largura::Natural => limites,
            Largura::Ancora => limites.width(self.ancora.width),
            Largura::Fixa(w) => limites.width(w),
        };

        let no = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limites);
        let tamanho = no.size();
        no.move_to(self.posicao(tamanho, bounds))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let limites = layout.bounds();

        // O conteúdo primeiro: um clique DENTRO do painel é dele, e o teste de
        // "fora" abaixo nunca chega a rodar para esse caso.
        self.content.as_widget_mut().update(
            self.tree, event, layout, cursor, renderer, clipboard, shell, &limites,
        );

        if shell.is_event_captured() {
            return;
        }

        // ▲ ▼ Enter, antes de qualquer coisa lá embaixo: é aqui que a lista de
        // sugestões do `<autocomplete>` ganha do `text_input` focado — ele
        // nem chega a ver a tecla.
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(nomeada),
            ..
        }) = event
        {
            use keyboard::key::Named;
            let msg = match nomeada {
                Named::ArrowUp => self.on_prev.clone(),
                Named::ArrowDown => self.on_next.clone(),
                Named::Enter => self.on_accept.clone(),
                _ => None,
            };
            if let Some(msg) = msg {
                shell.publish(msg);
                shell.capture_event();
                return;
            }
        }

        let Some(fechar) = self.on_dismiss.clone() else {
            return;
        };

        let dispensa = match event {
            // Só o **pressionar**. `FingerMoved`/`FingerLifted` entrariam num
            // arrasto que começou dentro do painel e saiu dele — o cursor de um
            // `<slider>` no meio do caminho —, e o painel se fecharia debaixo do
            // dedo.
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(iced::touch::Event::FingerPressed { .. }) => !cursor.is_over(limites),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => true,
            _ => false,
        };

        if dispensa {
            shell.publish(fechar);
            // Consumir é o ponto: sem isto o mesmo clique seguiria para a
            // árvore de baixo e um gatilho que alterna a chave reabriria o
            // painel no mesmo quadro.
            shell.capture_event();
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let limites = layout.bounds();
        self.content
            .as_widget()
            .draw(self.tree, renderer, theme, style, layout, cursor, &limites);
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'c, Message, iced::Theme, iced::Renderer>> {
        // Um `<select>` dentro do painel abre o menu dele por aqui — é a
        // cascata que o `menu.rs` faz à mão, e que o `iced` já sabe aninhar.
        self.content.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
}

impl<Message> Panel<'_, '_, Message> {
    /// Onde o canto superior esquerdo do painel cai, já virado para o outro
    /// lado se preciso e já preso dentro da janela.
    ///
    /// A ordem das três decisões importa: **virar** (o lado escolhido não cabe
    /// e o oposto cabe melhor), depois **alinhar** o eixo transversal, e só
    /// então **prender** nas bordas. Prender antes de virar produziria um
    /// painel colado no rodapé cobrindo o gatilho, que é o pior dos dois
    /// mundos.
    fn posicao(&self, painel: Size, janela: Size) -> Point {
        if self.placement == Placement::Center {
            return Point::new(
                ((janela.width - painel.width) / 2.0).max(0.0),
                ((janela.height - painel.height) / 2.0).max(0.0),
            );
        }

        let a = self.ancora;
        let prende = |v: f32, tamanho: f32, teto: f32| v.clamp(0.0, (teto - tamanho).max(0.0));

        if self.placement.vertical() {
            let abaixo = a.y + a.height + self.offset;
            let acima = a.y - self.offset - painel.height;
            let cabe_abaixo = abaixo + painel.height <= janela.height;
            let cabe_acima = acima >= 0.0;
            let y = match self.placement {
                Placement::Bottom if cabe_abaixo || !cabe_acima => abaixo,
                Placement::Bottom => acima,
                _ if cabe_acima || !cabe_abaixo => acima,
                _ => abaixo,
            };
            let x = self.align.origem(a.x, a.width, painel.width);
            Point::new(
                prende(x, painel.width, janela.width),
                prende(y, painel.height, janela.height),
            )
        } else {
            let direita = a.x + a.width + self.offset;
            let esquerda = a.x - self.offset - painel.width;
            let cabe_direita = direita + painel.width <= janela.width;
            let cabe_esquerda = esquerda >= 0.0;
            let x = match self.placement {
                Placement::Right if cabe_direita || !cabe_esquerda => direita,
                Placement::Right => esquerda,
                _ if cabe_esquerda || !cabe_direita => esquerda,
                _ => direita,
            };
            let y = self.align.origem(a.y, a.height, painel.height);
            Point::new(
                prende(x, painel.width, janela.width),
                prende(y, painel.height, janela.height),
            )
        }
    }
}

impl<'a, Message> From<Anchored<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer>
where
    Message: Clone + 'a,
{
    fn from(a: Anchored<'a, Message>) -> Self {
        Element::new(a)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn painel<'a>(placement: Placement, align: Align, ancora: Rectangle) -> Panel<'a, 'a, ()> {
        // O `Panel` só precisa dos campos geométricos para `posicao`; os dois
        // emprestados nunca são tocados por ela.
        Panel {
            content: Box::leak(Box::new(Element::new(iced::widget::Space::new()))),
            tree: Box::leak(Box::new(Tree::empty())),
            ancora,
            placement,
            align,
            offset: 4.0,
            largura: Largura::Natural,
            on_dismiss: None,
            on_prev: None,
            on_next: None,
            on_accept: None,
        }
    }

    const JANELA: Size = Size {
        width: 800.0,
        height: 600.0,
    };

    #[test]
    fn abre_abaixo_quando_cabe() {
        let a = Rectangle::new(Point::new(100.0, 100.0), Size::new(120.0, 30.0));
        let p = painel(Placement::Bottom, Align::Start, a).posicao(Size::new(200.0, 150.0), JANELA);
        assert_eq!(p, Point::new(100.0, 134.0));
    }

    #[test]
    fn vira_para_cima_quando_o_rodape_corta() {
        // Gatilho quase no fim da janela: abaixo não cabe, acima cabe.
        let a = Rectangle::new(Point::new(100.0, 520.0), Size::new(120.0, 30.0));
        let p = painel(Placement::Bottom, Align::Start, a).posicao(Size::new(200.0, 150.0), JANELA);
        assert_eq!(p, Point::new(100.0, 366.0));
    }

    #[test]
    fn nao_vira_quando_nenhum_dos_dois_cabe() {
        // Painel mais alto que a janela: virar não melhora nada, então fica no
        // lado pedido e o clamp cuida do resto.
        let a = Rectangle::new(Point::new(100.0, 300.0), Size::new(120.0, 30.0));
        let p = painel(Placement::Bottom, Align::Start, a).posicao(Size::new(200.0, 700.0), JANELA);
        assert_eq!(p, Point::new(100.0, 0.0));
    }

    #[test]
    fn align_end_encosta_a_borda_direita_do_gatilho() {
        let a = Rectangle::new(Point::new(600.0, 40.0), Size::new(120.0, 30.0));
        let p = painel(Placement::Bottom, Align::End, a).posicao(Size::new(200.0, 100.0), JANELA);
        assert_eq!(p.x, 520.0);
    }

    #[test]
    fn align_center_centra_no_gatilho() {
        let a = Rectangle::new(Point::new(300.0, 40.0), Size::new(120.0, 30.0));
        let p =
            painel(Placement::Bottom, Align::Center, a).posicao(Size::new(200.0, 100.0), JANELA);
        assert_eq!(p.x, 260.0);
    }

    #[test]
    fn preso_dentro_da_janela_pela_esquerda() {
        let a = Rectangle::new(Point::new(10.0, 40.0), Size::new(40.0, 30.0));
        let p = painel(Placement::Bottom, Align::End, a).posicao(Size::new(200.0, 100.0), JANELA);
        assert_eq!(p.x, 0.0);
    }

    #[test]
    fn submenu_vira_para_a_esquerda_na_borda_direita() {
        let a = Rectangle::new(Point::new(700.0, 100.0), Size::new(80.0, 30.0));
        let p = painel(Placement::Right, Align::Start, a).posicao(Size::new(200.0, 100.0), JANELA);
        assert_eq!(p.x, 496.0);
    }

    #[test]
    fn center_ignora_a_ancora() {
        let a = Rectangle::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0));
        let p = painel(Placement::Center, Align::Start, a).posicao(Size::new(200.0, 100.0), JANELA);
        assert_eq!(p, Point::new(300.0, 250.0));
    }
}
