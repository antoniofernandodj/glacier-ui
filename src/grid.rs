//! `Grid`: a **medição de colunas** — o habilitador da Onda 6 do
//! `PLANO_WIDGETS.md`, e o item de motor de que saem `<grid>`, `<tableheader>`,
//! `<tableview>` e `<columnview>`.
//!
//! ```xml
//! <grid columns="3" spacing="8">
//!     <text>Nome</text>   <text>Estado</text>  <text>Uso</text>
//!     <text>api</text>    <text>no ar</text>   <text>41%</text>
//! </grid>
//! ```
//!
//! # O que faltava, exatamente
//!
//! O documento catalogava dois itens separados como caros — o `Grid` ("o `iced`
//! não tem grade") e o `TableView` ("**grande**: cabeçalho, seleção, sort,
//! edição") — sem notar que a parte difícil dos dois é **a mesma**: descobrir a
//! largura de uma coluna a partir de **todas** as células que passam por ela,
//! antes de desenhar qualquer uma.
//!
//! Uma `Row` de `Column`s não resolve: cada `Column` mede os filhos dela e
//! ignora as vizinhas, então a segunda linha da tabela sai desalinhada da
//! primeira assim que um texto for mais longo. Um `Column` de `Row`s tem o
//! problema espelhado. **A medição precisa ser bidimensional**, e é isso que
//! este widget faz — em dois passos, no `layout()`:
//!
//! 1. mede cada célula solta (limites frouxos) e guarda o tamanho natural;
//! 2. a largura de uma coluna é o **máximo** das células dela, a altura de uma
//!    linha o máximo da linha — e só então cada célula é medida de novo, agora
//!    contra a largura definitiva da coluna dela.
//!
//! Dois passos são o preço, e ele é conhecido: é o mesmo que qualquer motor de
//! layout de tabela paga. Para uma grade grande dentro de um `<scrollable>`, a
//! saída continua sendo a de sempre — `virtualize` (ver `PRIMITIVAS.md`).
//!
//! # As trilhas
//!
//! `columns` aceita duas formas, e a segunda é a que o `<tableview>` usa:
//!
//! | escrito | quer dizer |
//! |---|---|
//! | `columns="3"` | três colunas medidas |
//! | `columns="140 fill 80"` | fixa, flexível, fixa |
//!
//! Uma trilha `fill` recebe o que sobrar da largura disponível, repartido entre
//! todas as `fill` (`fill 2` leva o dobro de `fill`). Uma coluna **medida**
//! cujas células declarem `Length::Fill` vira flexível sozinha — senão a célula
//! reportaria a largura da janela inteira no primeiro passo e comeria a grade.
//!
//! # O que ele **não** faz
//!
//! Célula que ocupa mais de uma coluna (`colspan`). O `QGridLayout` tem; esta
//! grade não, porque o consumidor que a motivou — a tabela — não usa, e
//! `colspan` muda a medição de "máximo por coluna" para uma distribuição com
//! restrições. Fica anotado como o que ela cresceria a seguir.
//!
//! # `Flow`/`Wrap` não está aqui, e é uma correção de rota
//!
//! O plano listava `Flow`/`Wrap` como "a mesma medição num eixo só", saindo
//! deste mesmo mecanismo. **O `iced` já tinha**: `Row::wrap()`
//! (`iced_widget::row::Wrapping`) quebra linha exatamente assim. `<flow>` é um
//! `Row` embrulhado, três linhas em `widget.rs`, e não passa por aqui — a sexta
//! vez que este projeto descobre que um item catalogado como caro já existia.

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::{Alignment, Element, Event, Length, Padding, Rectangle, Size, Vector, mouse};

/// A largura de uma coluna, como o markup a declarou.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trilha {
    /// Medida: o máximo das células desta coluna.
    Auto,
    /// Um número de pixels.
    Fixa(f32),
    /// Reparte o que sobrar, em proporção ao número.
    Flexivel(u16),
}

impl Trilha {
    /// Uma trilha só: `"140"`, `"fill"`, `"fill2"`, `"auto"`.
    pub fn parse_uma(spec: &str) -> Trilha {
        let baixo = spec.trim().to_ascii_lowercase();
        if let Some(resto) = baixo
            .strip_prefix("fill")
            .or_else(|| baixo.strip_prefix("flex"))
        {
            let porcao = resto.trim_matches(['-', '_']).parse::<u16>().unwrap_or(1);
            return Trilha::Flexivel(porcao.max(1));
        }
        if baixo == "auto" || baixo == "*" || baixo.is_empty() {
            return Trilha::Auto;
        }
        baixo
            .parse::<f32>()
            .ok()
            .filter(|w| *w > 0.0)
            .map_or(Trilha::Auto, Trilha::Fixa)
    }

    /// Uma trilha por palavra, **sem** o atalho do número solto.
    ///
    /// É a leitura que a chave de `widths` de um `<tableview>` precisa: ali
    /// `"160"` quer dizer *uma coluna de 160px*, não *160 colunas*. A diferença
    /// entre esta e a [`Self::parse_lista`] é exatamente essa, e ela custou um
    /// teste vermelho para aparecer.
    pub fn parse_faixas(spec: &str) -> Vec<Trilha> {
        spec.split_whitespace()
            .map(Self::parse_uma)
            .take(64)
            .collect()
    }

    /// Lê uma lista de trilhas de um `columns="…"`.
    ///
    /// `"3"` (um número inteiro só, sem outra palavra) quer dizer **três
    /// colunas medidas** — a forma curta, e a que 90% das grades usam. Qualquer
    /// outra coisa cai na [`Self::parse_faixas`].
    pub fn parse_lista(spec: &str) -> Vec<Trilha> {
        let campos: Vec<&str> = spec.split_whitespace().collect();
        if let [unico] = campos[..]
            && let Ok(n) = unico.parse::<usize>()
        {
            return vec![Trilha::Auto; n.clamp(1, 64)];
        }
        Self::parse_faixas(spec)
    }
}

/// Ver o [módulo](self). Criado por [`grid`].
pub struct Grid<'a, Message> {
    children: Vec<Element<'a, Message, iced::Theme, iced::Renderer>>,
    trilhas: Vec<Trilha>,
    spacing_x: f32,
    spacing_y: f32,
    padding: Padding,
    width: Length,
    height: Length,
    /// Alinhamento vertical de uma célula dentro da linha dela.
    align_y: Alignment,
}

/// Uma grade com as trilhas dadas. Uma lista de trilhas vazia vira uma coluna
/// só — uma grade sem colunas não existe, e um `Vec` vazio faria a divisão de
/// índice por zero logo abaixo.
pub fn grid<'a, Message>(
    children: Vec<Element<'a, Message, iced::Theme, iced::Renderer>>,
    trilhas: Vec<Trilha>,
) -> Grid<'a, Message> {
    Grid {
        children,
        trilhas: if trilhas.is_empty() {
            vec![Trilha::Auto]
        } else {
            trilhas
        },
        spacing_x: 0.0,
        spacing_y: 0.0,
        padding: Padding::ZERO,
        width: Length::Shrink,
        height: Length::Shrink,
        align_y: Alignment::Start,
    }
}

impl<Message> Grid<'_, Message> {
    pub fn spacing(mut self, x: f32, y: f32) -> Self {
        self.spacing_x = x;
        self.spacing_y = y;
        self
    }

    pub fn padding(mut self, p: Padding) -> Self {
        self.padding = p;
        self
    }

    pub fn width(mut self, w: Length) -> Self {
        self.width = w;
        self
    }

    pub fn height(mut self, h: Length) -> Self {
        self.height = h;
        self
    }

    pub fn align_y(mut self, a: Alignment) -> Self {
        self.align_y = a;
        self
    }

    fn colunas(&self) -> usize {
        self.trilhas.len()
    }

    fn linhas(&self) -> usize {
        self.children.len().div_ceil(self.colunas())
    }
}

/// A largura definitiva de cada coluna.
///
/// Separada do `layout` porque é a peça que se testa: dá para conferir a
/// repartição sem um `Renderer`, e é onde os erros moram (uma coluna que come a
/// grade, uma sobra negativa que vira largura `NaN`).
fn larguras(
    trilhas: &[Trilha],
    naturais: &[f32],
    flexivel_por_conteudo: &[bool],
    disponivel: f32,
    spacing_x: f32,
) -> Vec<f32> {
    let n = trilhas.len();
    let vaos = spacing_x * (n.saturating_sub(1)) as f32;

    // O que cada coluna ocupa antes de repartir a sobra, e quanto de `fill`
    // cada uma pede.
    let mut base = vec![0.0f32; n];
    let mut porcoes = vec![0u16; n];
    for i in 0..n {
        match trilhas[i] {
            Trilha::Fixa(w) => base[i] = w,
            Trilha::Flexivel(p) => porcoes[i] = p,
            // Uma coluna medida cujas células se declaram `Fill` não tem
            // largura natural que signifique alguma coisa: elas reportariam a
            // janela inteira. Ela vira flexível, que é o que o markup quis
            // dizer ao escrever `width="fill"` na célula.
            Trilha::Auto if flexivel_por_conteudo[i] => porcoes[i] = 1,
            Trilha::Auto => base[i] = naturais[i],
        }
    }

    let total_porcoes: u32 = porcoes.iter().map(|p| *p as u32).sum();
    if total_porcoes == 0 {
        return base;
    }

    // `disponivel` infinito acontece dentro de um `<scrollable>` horizontal:
    // ali não há sobra para repartir, e a coluna flexível cai na largura
    // natural dela. Sem esta guarda, `INFINITY - x` viraria uma largura
    // infinita e o layout inteiro sairia `NaN`.
    let sobra = if disponivel.is_finite() {
        (disponivel - vaos - base.iter().sum::<f32>()).max(0.0)
    } else {
        0.0
    };
    for i in 0..n {
        if porcoes[i] > 0 {
            let fatia = sobra * (porcoes[i] as f32) / (total_porcoes as f32);
            // O piso é a largura natural: uma coluna flexível pode CRESCER com
            // a sobra, nunca encolher abaixo do que o conteúdo dela precisa.
            //
            // Exceto quando o que a tornou flexível foi a própria célula se
            // declarar `Fill`: aí a "natural" medida no passo 1 é a janela
            // inteira, e usá-la como piso devolveria justamente a coluna que
            // come a grade — o problema que a regra existe para evitar.
            let piso = if flexivel_por_conteudo[i] {
                0.0
            } else {
                naturais[i]
            };
            base[i] = fatia.max(piso);
        }
    }
    base
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Grid<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limites = limits
            .width(self.width)
            .height(self.height)
            .shrink(self.padding);
        let colunas = self.colunas();
        let linhas = self.linhas();
        if self.children.is_empty() {
            return layout::Node::new(Size::ZERO);
        }

        // ── Passo 1: medir cada célula solta ─────────────────────────────
        //
        // Limites frouxos: queremos o tamanho NATURAL, não o esticado. É a
        // medida que responde "de que largura esta coluna precisa?".
        let frouxo = limites.loose();
        let mut naturais = vec![0.0f32; colunas];
        let mut flexivel = vec![false; colunas];
        let mut medidas: Vec<Size> = Vec::with_capacity(self.children.len());
        for (i, filho) in self.children.iter_mut().enumerate() {
            let coluna = i % colunas;
            if matches!(
                filho.as_widget().size().width,
                Length::Fill | Length::FillPortion(_)
            ) {
                flexivel[coluna] = true;
            }
            let no = filho
                .as_widget_mut()
                .layout(&mut tree.children[i], renderer, &frouxo);
            let tamanho = no.size();
            naturais[coluna] = naturais[coluna].max(tamanho.width);
            medidas.push(tamanho);
        }

        let larguras = larguras(
            &self.trilhas,
            &naturais,
            &flexivel,
            limites.max().width,
            self.spacing_x,
        );

        // ── Passo 2: medir de novo, agora contra a coluna definitiva ─────
        //
        // Só agora uma célula `width="fill"` sabe o que "fill" significa, e só
        // agora um texto longo sabe onde quebrar. É por isso que são dois
        // passos e não um.
        let mut nos: Vec<layout::Node> = Vec::with_capacity(self.children.len());
        let mut alturas = vec![0.0f32; linhas];
        for (i, filho) in self.children.iter_mut().enumerate() {
            let coluna = i % colunas;
            let celula = layout::Limits::new(
                Size::ZERO,
                Size::new(larguras[coluna], limites.max().height),
            );
            let no = filho
                .as_widget_mut()
                .layout(&mut tree.children[i], renderer, &celula);
            // O `max` ignora altura não-finita. Dentro de um `<scrollable>` o
            // teto vertical é infinito, e uma célula que se declare
            // `height="fill"` mede infinito — o que faria a linha inteira
            // (e a grade, e a tela) sumir sem erro nenhum. Uma célula assim
            // simplesmente não vota na altura da linha.
            let h = no.size().height;
            if h.is_finite() {
                alturas[i / colunas] = alturas[i / colunas].max(h);
            }
            nos.push(no);
        }

        // ── Posicionar ───────────────────────────────────────────────────
        let x_de = |c: usize| -> f32 {
            self.padding.left + larguras[..c].iter().sum::<f32>() + self.spacing_x * c as f32
        };
        let mut y = self.padding.top;
        for (linha, altura_linha) in alturas.iter().enumerate() {
            for coluna in 0..colunas {
                let i = linha * colunas + coluna;
                let Some(no) = nos.get_mut(i) else {
                    break;
                };
                // O alinhamento vertical dentro da célula: `start` (o default,
                // e o que uma tabela quer), `center` ou `end`. Horizontal não
                // existe aqui de propósito — quem alinha o conteúdo na largura
                // da coluna é a própria célula, que já a recebeu no passo 2.
                let folga = (altura_linha - no.size().height).max(0.0);
                let dy = match self.align_y {
                    Alignment::Center => folga / 2.0,
                    Alignment::End => folga,
                    Alignment::Start => 0.0,
                };
                let ponto = iced::Point::new(x_de(coluna), y + dy);
                let velho = std::mem::replace(no, layout::Node::new(Size::ZERO));
                *no = velho.move_to(ponto);
            }
            y += altura_linha + self.spacing_y;
        }

        let total = Size::new(
            larguras.iter().sum::<f32>()
                + self.spacing_x * colunas.saturating_sub(1) as f32
                + self.padding.left
                + self.padding.right,
            alturas.iter().sum::<f32>()
                + self.spacing_y * linhas.saturating_sub(1) as f32
                + self.padding.top
                + self.padding.bottom,
        );
        layout::Node::with_children(limites.resolve(self.width, self.height, total), nos)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            for ((filho, estado), layout) in self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                filho
                    .as_widget_mut()
                    .operate(estado, layout, renderer, operation);
            }
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
        for ((filho, estado), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            filho.as_widget_mut().update(
                estado, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
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
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((filho, estado), layout)| {
                filho
                    .as_widget()
                    .mouse_interaction(estado, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
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
        // Só as células que intersectam o viewport, como a `Row` do iced faz:
        // numa tabela longa dentro de um `<scrollable>` isso já poupa o
        // desenho das linhas fora da tela (medir, não — para isso é o
        // `virtualize`).
        for ((filho, estado), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
        {
            filho
                .as_widget()
                .draw(estado, renderer, theme, style, layout, cursor, viewport);
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
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<Grid<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(g: Grid<'a, Message>) -> Self {
        Element::new(g)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn numero_sozinho_vira_n_colunas_medidas() {
        assert_eq!(Trilha::parse_lista("3"), vec![Trilha::Auto; 3]);
    }

    /// A mesma string, lida como faixas, é UMA coluna de 3px. É a diferença
    /// que a chave de `widths` de um `<tableview>` precisa: ali `"160"` quer
    /// dizer uma coluna de 160, não 160 colunas.
    #[test]
    fn parse_faixas_nao_tem_o_atalho_do_numero_solto() {
        assert_eq!(Trilha::parse_faixas("3"), vec![Trilha::Fixa(3.0)]);
        assert_eq!(Trilha::parse_uma("80"), Trilha::Fixa(80.0));
        assert_eq!(Trilha::parse_uma("fill 2"), Trilha::Flexivel(1));
        assert_eq!(Trilha::parse_uma(""), Trilha::Auto);
    }

    #[test]
    fn lista_mista_le_uma_trilha_por_palavra() {
        assert_eq!(
            Trilha::parse_lista("140 fill 80 fill2"),
            vec![
                Trilha::Fixa(140.0),
                Trilha::Flexivel(1),
                Trilha::Fixa(80.0),
                Trilha::Flexivel(2),
            ]
        );
    }

    #[test]
    fn coluna_medida_recebe_o_maximo_das_celulas() {
        // Sem trilha declarada, a largura é a maior célula da coluna — que é o
        // que uma `Row` de `Column`s NÃO consegue fazer, e a razão de este
        // widget existir.
        let w = larguras(
            &[Trilha::Auto, Trilha::Auto],
            &[120.0, 40.0],
            &[false, false],
            600.0,
            8.0,
        );
        assert_eq!(w, vec![120.0, 40.0]);
    }

    #[test]
    fn a_sobra_vai_para_as_flexiveis_em_proporcao() {
        // 600 de largura, 8 de vão, 100 fixos ⇒ 492 para repartir entre 1 e 2.
        let w = larguras(
            &[
                Trilha::Fixa(100.0),
                Trilha::Flexivel(1),
                Trilha::Flexivel(2),
            ],
            &[0.0, 0.0, 0.0],
            &[false; 3],
            600.0,
            8.0,
        );
        assert_eq!(w[0], 100.0);
        assert!((w[1] - 484.0 / 3.0).abs() < 0.01, "{:?}", w);
        assert!((w[2] - 2.0 * 484.0 / 3.0).abs() < 0.01, "{:?}", w);
    }

    #[test]
    fn flexivel_nunca_encolhe_abaixo_do_conteudo() {
        // Janela apertada: a sobra é zero, mas a coluna continua com a largura
        // que o conteúdo dela precisa — melhor transbordar do que sumir.
        let w = larguras(
            &[Trilha::Fixa(500.0), Trilha::Flexivel(1)],
            &[0.0, 90.0],
            &[false, false],
            520.0,
            0.0,
        );
        assert_eq!(w, vec![500.0, 90.0]);
    }

    #[test]
    fn celula_fill_torna_a_coluna_medida_flexivel() {
        // Sem esta regra, a célula reportaria a largura da janela no passo 1 e
        // a coluna comeria a grade inteira.
        let w = larguras(
            &[Trilha::Auto, Trilha::Auto],
            &[80.0, 9999.0],
            &[false, true],
            600.0,
            10.0,
        );
        assert_eq!(w[0], 80.0);
        assert!((w[1] - 510.0).abs() < 0.01, "{:?}", w);
    }

    #[test]
    fn largura_infinita_nao_produz_nan() {
        // Dentro de um `<scrollable>` horizontal o teto é infinito; sem a
        // guarda, `INFINITY - x` viraria uma largura infinita e o layout
        // inteiro sairia NaN — um nó que some sem erro nenhum.
        let w = larguras(
            &[Trilha::Flexivel(1), Trilha::Auto],
            &[70.0, 30.0],
            &[false, false],
            f32::INFINITY,
            8.0,
        );
        assert!(w.iter().all(|x| x.is_finite()), "{:?}", w);
        assert_eq!(w, vec![70.0, 30.0]);
    }
}
