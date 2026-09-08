//! Os dois widgets da Onda 9 que arrastam sobre **filhos**: `<splitter>`
//! (`QSplitter`) e `<swipeview>` (`QML SwipeView`).
//!
//! # Por que estes dois estão juntos, e o `<dial>` não
//!
//! Um widget que desenha a si mesmo num `canvas` ganha do `iced` um
//! `Program::State` por instância e guarda ali o "estou arrastando" — foi assim
//! no `<dial>` da Onda 7 e é assim nos quatro do [`crate::pointer`]. Um widget
//! que tem **filhos** não tem esse luxo: ele monta `Element`s de verdade e o
//! arrasto acontece *entre* eles, num quadro em que o widget nem existe mais
//! como valor. Esse arrasto mora numa chave do motor, e a chave é o
//! [`crate::grip`].
//!
//! É a única diferença entre os dois grupos, e ela não é sobre estado por
//! instância — a marca `●` que o catálogo dava aos dois nunca valeu. O que o
//! arrasto move é sempre um valor que o app nomeia: as trilhas de um
//! `<splitter>`, o índice de um `<swipeview>`.

use iced::widget::{Space, column, container, mouse_area, row};
use iced::{Background, Element, Length};

use crate::ContextMap;
use crate::grid::Trilha;
use crate::grip::{Alvo, Arrasto, Eixo};
use crate::widget::EngineMessage;

/// O teto de um painel arrastado. Não é estética: sem teto, arrastar rápido
/// para fora da janela deixaria a chave com um número de cinco dígitos e o
/// painel vizinho invisível para sempre.
const MAXIMO_PAINEL: f32 = 4000.0;

/// Traduz uma trilha na `Length` que o `iced` entende. Uma trilha `Auto` num
/// painel é `Fill`, e **não** `Shrink`: um painel de `<splitter>` que encolhesse
/// até o conteúdo não repartiria nada, que é o que um splitter existe para
/// fazer.
fn largura_de(t: Trilha) -> Length {
    match t {
        Trilha::Fixa(px) => Length::Fixed(px),
        Trilha::Flexivel(p) => Length::FillPortion(p),
        Trilha::Auto => Length::Fill,
    }
}

/// A medida que um arrasto começa carregando. Uma trilha flexível não tem
/// pixels ainda — o `valor0` dela sai como zero e o primeiro movimento a
/// converte em fixa a partir dali, que é a mesma conversão que a alça de coluna
/// do `<tableheader>` faz desde a Onda 6.
fn medida_inicial(t: Trilha, padrao: f32) -> f32 {
    match t {
        Trilha::Fixa(px) => px,
        _ => padrao,
    }
}

/// `<splitter>` — ver [`crate::parser::NodeType::Splitter`].
///
/// Os painéis são os filhos; as alças vão **entre** eles, como irmãs numa
/// `Row`/`Column`, nunca por cima. Um overlay aqui roubaria o clique do
/// conteúdo — é a mesma decisão que a alça de coluna tomou na Onda 6, e pelo
/// mesmo motivo.
#[allow(clippy::too_many_arguments)]
pub fn render_splitter<'a>(
    node: &'a crate::parser::UiNode,
    context: &'a ContextMap,
    sizes_var: &'a str,
    vertical: bool,
    handle: f32,
    min: f32,
    filhos: Vec<Element<'a, EngineMessage>>,
) -> Element<'a, EngineMessage> {
    let n = filhos.len();
    if n == 0 {
        return Space::new().width(0).height(0).into();
    }

    // As trilhas declaradas na chave. Faltando (a tela que ainda não semeou
    // nada), todos os painéis repartem igual — que é o estado inicial correto
    // de um QSplitter e o que faz a primeira renderização não depender de um
    // `init`.
    let trilhas: Vec<Trilha> = {
        let mut t = context
            .get(sizes_var)
            .map(|s| Trilha::parse_faixas(s))
            .unwrap_or_default();
        t.resize(n, Trilha::Flexivel(1));
        t
    };

    let vao = node.spacing.unwrap_or(0.0);
    let arrastavel = !sizes_var.is_empty();

    let mut pecas: Vec<Element<'a, EngineMessage>> = Vec::with_capacity(n * 2 - 1);
    for (i, filho) in filhos.into_iter().enumerate() {
        let painel = if vertical {
            container(filho)
                .height(largura_de(trilhas[i]))
                .width(Length::Fill)
        } else {
            container(filho)
                .width(largura_de(trilhas[i]))
                .height(Length::Fill)
        };
        pecas.push(painel.into());

        // Uma alça a cada par — a última não tem depois de quem ficar.
        if arrastavel && i + 1 < n {
            pecas.push(alca(sizes_var, i, vertical, handle, min, trilhas[i]));
        }
    }

    if vertical {
        column(pecas).spacing(vao).width(Length::Fill).into()
    } else {
        row(pecas).spacing(vao).height(Length::Fill).into()
    }
}

/// Uma alça: a faixa fina entre dois painéis que começa o arrasto.
fn alca<'a>(
    sizes_var: &str,
    indice: usize,
    vertical: bool,
    espessura: f32,
    min: f32,
    trilha: Trilha,
) -> Element<'a, EngineMessage> {
    // **Nunca `Fill` no eixo transverso quando o pai pode ser infinito.** Um
    // `<splitter>` dentro de um `<scrollable>` recebe altura infinita, e um
    // `Fill` aqui empurraria o conteúdo inteiro para fora da tela sem erro
    // nenhum — a armadilha que o `PRIMITIVAS.md` registra e que a alça de
    // coluna já tinha pisado. `Fill` no eixo transverso de um Row/Column é
    // seguro; o risco é o eixo do scroll, e aí o container do pai limita.
    let corpo = if vertical {
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .height(espessura)
    } else {
        container(Space::new().width(1).height(Length::Fill))
            .height(Length::Fill)
            .width(espessura)
    };

    mouse_area(corpo.style(|theme: &iced::Theme| container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.strong.color,
        )),
        ..Default::default()
    }))
    .interaction(if vertical {
        iced::mouse::Interaction::ResizingVertically
    } else {
        iced::mouse::Interaction::ResizingHorizontally
    })
    .on_press(EngineMessage::GripStart(Arrasto {
        chave: sizes_var.to_string(),
        indice,
        eixo: if vertical { Eixo::Y } else { Eixo::X },
        origem: None,
        // Um painel ainda flexível não sabe quantos pixels tem; ele parte do
        // piso e o primeiro movimento o fixa a partir dali. É grosseiro no
        // primeiro arrasto de um painel `fill` e exato em todos os seguintes —
        // a alternativa seria medir o layout, que é o que a Onda 6 fez e
        // custou um `iced::advanced::Widget` inteiro.
        valor0: medida_inicial(trilha, min),
        alvo: Alvo::Trilha {
            min,
            max: MAXIMO_PAINEL,
        },
    }))
    .into()
}

/// `<swipeview>` — ver [`crate::parser::NodeType::SwipeView`].
///
/// Renderiza **só** a página ativa, embrulhada num `mouse_area` que começa o
/// arrasto. Ao contrário do `<tabs>`/`<stackview>` (que avaliam todas as
/// páginas e renderizam uma), aqui as páginas são filhos crus da árvore: a
/// avaliação já aconteceu, e o que se escolhe é qual `Element` montar.
pub fn render_swipeview<'a>(
    context: &'a ContextMap,
    value_var: &'a str,
    threshold: f32,
    mut filhos: Vec<Element<'a, EngineMessage>>,
) -> Element<'a, EngineMessage> {
    let n = filhos.len();
    if n == 0 {
        return Space::new().width(0).height(0).into();
    }
    let atual = context
        .get(value_var)
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(0)
        .min(n - 1);

    // `swap_remove` e não `remove`: tirar do meio de um `Vec` de `Element`
    // deslocaria o resto por nada — o resto vai ser descartado na linha
    // seguinte.
    let pagina = filhos.swap_remove(atual);

    if value_var.is_empty() {
        return pagina;
    }

    mouse_area(container(pagina).width(Length::Fill).height(Length::Fill))
        .interaction(iced::mouse::Interaction::Grab)
        .on_press(EngineMessage::GripStart(Arrasto {
            chave: value_var.to_string(),
            indice: 0,
            eixo: Eixo::X,
            origem: None,
            valor0: atual as f32,
            // Arrastar para a esquerda anda para trás porque o conteúdo segue o
            // dedo — é o que todo carrossel de telefone faz, e o contrário
            // parece quebrado mesmo estando "certo" pela seta.
            alvo: Alvo::Indice {
                passo: threshold,
                max: n - 1,
            },
        }))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trilha_auto_de_painel_e_fill_e_nao_shrink() {
        // Um painel que encolhesse até o conteúdo não reparte nada — e um
        // `<splitter>` existe para repartir.
        assert_eq!(largura_de(Trilha::Auto), Length::Fill);
        assert_eq!(largura_de(Trilha::Fixa(240.0)), Length::Fixed(240.0));
        assert_eq!(largura_de(Trilha::Flexivel(2)), Length::FillPortion(2));
    }

    #[test]
    fn um_painel_flexivel_parte_do_piso() {
        // Ele não sabe quantos pixels tem até o layout acontecer; o primeiro
        // movimento é que o fixa.
        assert_eq!(medida_inicial(Trilha::Flexivel(1), 60.0), 60.0);
        assert_eq!(medida_inicial(Trilha::Auto, 60.0), 60.0);
        assert_eq!(medida_inicial(Trilha::Fixa(300.0), 60.0), 300.0);
    }
}
