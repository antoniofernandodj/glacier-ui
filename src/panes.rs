//! Os widgets que arrastam sobre **filhos**: `<splitter>` (`QSplitter`) e
//! `<swipeview>` (`QML SwipeView`) da Onda 9, mais `<mdiarea>`/`<mdisubwindow>`
//! (`QMdiArea`) da Onda 11.
//!
//! # Por que estes widgets estão juntos, e o `<dial>` não
//!
//! Um widget que desenha a si mesmo num `canvas` ganha do `iced` um
//! `Program::State` por instância e guarda ali o "estou arrastando" — foi assim
//! no `<dial>` da Onda 7 e é assim nos quatro do [`crate::pointer`]. Um widget
//! que tem **filhos** não tem esse luxo: ele monta `Element`s de verdade e o
//! arrasto acontece *entre* eles, num quadro em que o widget nem existe mais
//! como valor. Esse arrasto mora numa chave do motor, e a chave é o
//! [`crate::grip`].
//!
//! É a única diferença entre os grupos, e ela não é sobre estado por
//! instância — a marca `●` que o catálogo dava a todos eles nunca valeu. O que
//! o arrasto move é sempre um valor que o app nomeia: as trilhas de um
//! `<splitter>`, o índice de um `<swipeview>`, a posição/tamanho de um
//! `<mdisubwindow>`.
//!
//! # `<mdiarea>`: a mesma capacidade, em duas dimensões
//!
//! O `<splitter>`/`<swipeview>` arrastam em **um** eixo (`grip::Alvo::Trilha`/
//! `Indice`); uma janela interna arrasta nos **dois** ao mesmo tempo — o
//! habilitador B da Onda 11, `grip::Alvo::Ponto`. É a mesma extensão que a
//! Onda 9 fez ao tirar a conta de dentro do `__colgrip`, um nível acima: nem
//! a âncora no primeiro movimento, nem o listener condicional, precisaram
//! mudar — só o mapeamento de pixel para valor ganhou uma segunda dimensão.
//!
//! Cada `<mdisubwindow>` nomeia QUATRO chaves (`x`/`y`/`w`/`h`), a mesma forma
//! que o `<rangeslider>` usa para duas — e é por isso que o `●` do catálogo
//! nunca valeu aqui também (17ª correção de nível): mover é escrever em duas
//! chaves com `Alvo::Ponto`, redimensionar é escrever nas outras duas com o
//! MESMO `Alvo::Ponto`, só com outros limites.

use iced::widget::{Space, Stack, column, container, mouse_area, pin, row, text};
use iced::{Alignment, Background, Border, Element, Length};

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
        chave_y: None,
        chave_modo: None,
        indice,
        eixo: if vertical { Eixo::Y } else { Eixo::X },
        origem: None,
        origem_y: None,
        // Um painel ainda flexível não sabe quantos pixels tem; ele parte do
        // piso e o primeiro movimento o fixa a partir dali. É grosseiro no
        // primeiro arrasto de um painel `fill` e exato em todos os seguintes —
        // a alternativa seria medir o layout, que é o que a Onda 6 fez e
        // custou um `iced::advanced::Widget` inteiro.
        valor0: medida_inicial(trilha, min),
        valor0_y: 0.0,
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
            chave_y: None,
            chave_modo: None,
            indice: 0,
            eixo: Eixo::X,
            origem: None,
            origem_y: None,
            valor0: atual as f32,
            valor0_y: 0.0,
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

// ─────────────────────────────────────────────────────────────────────────────
// <mdiarea> / <mdisubwindow> — a Onda 11
// ─────────────────────────────────────────────────────────────────────────────

/// Faixa de tamanho de um `<mdisubwindow>`. Como o `MAXIMO_PAINEL` do
/// `<splitter>`: não é estética, é o que evita uma janela arrastada rápido
/// demais sumir com um número de cinco dígitos, ou encolher a zero e virar
/// impossível de agarrar de novo.
const LARGURA_MINIMA_JANELA: f32 = 160.0;
const LARGURA_MAXIMA_JANELA: f32 = 2400.0;
const ALTURA_MINIMA_JANELA: f32 = 100.0;
const ALTURA_MAXIMA_JANELA: f32 = 1600.0;

/// O lado do quadrado de redimensionar, no canto inferior direito.
const GRIP_JANELA: f32 = 14.0;

/// `<mdiarea>` — ver [`crate::parser::NodeType::MdiArea`].
///
/// Só empilha e posiciona; a posição de cada `(x, y, Element)` já veio
/// resolvida de quem chamou (`widget.rs`, que é quem tem o `context` e o
/// `NodeType::MdiSubWindow` do filho à mão para ler `x_var`/`y_var` e aplicar
/// o cascade). Sem reordenação por clique — a ordem de empilhamento
/// (Z) é a ordem do markup, uma simplificação anotada por escrito na Onda 11.
pub fn render_mdi_area<'a>(
    posicionados: Vec<(f32, f32, Element<'a, EngineMessage>)>,
) -> Element<'a, EngineMessage> {
    if posicionados.is_empty() {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }
    let mut st = Stack::new();
    for (x, y, el) in posicionados {
        st = st.push(pin(el).x(x).y(y));
    }
    st.width(Length::Fill).height(Length::Fill).into()
}

/// Uma janela interna: barra de título (arrasta `x_var`/`y_var` juntos, via
/// [`Alvo::Ponto`]) + corpo + canto de redimensionar (arrasta `w_var`/`h_var`
/// juntos, o MESMO `Alvo::Ponto` com outros limites).
#[allow(clippy::too_many_arguments)]
pub fn render_mdi_subwindow<'a>(
    context: &'a ContextMap,
    title: &'a str,
    x_var: &'a str,
    y_var: &'a str,
    w_var: &'a str,
    h_var: &'a str,
    default_w: f32,
    default_h: f32,
    corpo: Element<'a, EngineMessage>,
) -> Element<'a, EngineMessage> {
    let ler = |chave: &str, padrao: f32| -> f32 {
        context
            .get(chave)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(padrao)
    };
    let w = ler(w_var, default_w).clamp(LARGURA_MINIMA_JANELA, LARGURA_MAXIMA_JANELA);
    let h = ler(h_var, default_h).clamp(ALTURA_MINIMA_JANELA, ALTURA_MAXIMA_JANELA);
    let x0 = ler(x_var, 0.0);
    let y0 = ler(y_var, 0.0);

    let fundo_titulo = |theme: &iced::Theme| container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.strong.color,
        )),
        ..Default::default()
    };

    let titlebar = mouse_area(
        container(text(title.to_string()).size(13))
            .padding([6, 10])
            .width(Length::Fill)
            .style(fundo_titulo),
    )
    .interaction(iced::mouse::Interaction::Grab)
    .on_press(EngineMessage::GripStart(Arrasto {
        chave: x_var.to_string(),
        chave_y: (!y_var.is_empty()).then(|| y_var.to_string()),
        chave_modo: None,
        indice: 0,
        eixo: Eixo::X, // ignorado por `Alvo::Ponto`
        origem: None,
        origem_y: None,
        valor0: x0,
        valor0_y: y0,
        alvo: Alvo::Ponto {
            min_x: 0.0,
            max_x: MAXIMO_PAINEL,
            min_y: 0.0,
            max_y: MAXIMO_PAINEL,
        },
    }));

    let grip = mouse_area(
        container(Space::new().width(GRIP_JANELA).height(GRIP_JANELA)).style(
            |theme: &iced::Theme| container::Style {
                background: Some(Background::Color(
                    theme.extended_palette().background.strong.color,
                )),
                border: Border {
                    radius: iced::border::Radius::new(3.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
    )
    .interaction(iced::mouse::Interaction::ResizingDiagonallyDown)
    .on_press(EngineMessage::GripStart(Arrasto {
        chave: w_var.to_string(),
        chave_y: (!h_var.is_empty()).then(|| h_var.to_string()),
        chave_modo: None,
        indice: 0,
        eixo: Eixo::X,
        origem: None,
        origem_y: None,
        valor0: w,
        valor0_y: h,
        // Arrastar o canto para a direita/baixo AUMENTA w/h — a mesma soma
        // (não subtração) que mover a janela usa para x/y. É por isso que
        // redimensionar não pediu nada novo do `grip.rs`: é o mesmo alvo,
        // só com limites de tamanho em vez de limites de posição.
        alvo: Alvo::Ponto {
            min_x: LARGURA_MINIMA_JANELA,
            max_x: LARGURA_MAXIMA_JANELA,
            min_y: ALTURA_MINIMA_JANELA,
            max_y: ALTURA_MAXIMA_JANELA,
        },
    }));

    // O grip fica POR CIMA do corpo, no canto — o mesmo truque do `%` sobre a
    // `<progressbar>`, com o `<stack>` da Onda 11 no lugar do `stack!` cru.
    let corpo_com_grip = Stack::new()
        .push(container(corpo).width(Length::Fill).height(Length::Fill))
        .push(
            container(grip)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::End)
                .align_y(Alignment::End)
                .padding(2),
        );

    container(column![titlebar, corpo_com_grip])
        .width(w)
        .height(h)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(Background::Color(
                theme.extended_palette().background.base.color,
            )),
            border: Border {
                width: 1.0,
                color: theme.extended_palette().background.strong.color,
                radius: iced::border::Radius::new(4.0),
            },
            ..Default::default()
        })
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// <dock> — o habilitador D da Onda 12
// ─────────────────────────────────────────────────────────────────────────────

/// Quão longe o cabeçalho precisa ser arrastado para reancorar. Abaixo disso
/// foi um clique, e a borda não muda (ver [`Arrasto::modo_no_release`]).
const LIMIAR_DOCK: f32 = 36.0;

/// O estilo da faixa de título e da aba de restaurar — o mesmo fundo forte do
/// `<mdisubwindow>`.
fn fundo_forte(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.strong.color,
        )),
        ..Default::default()
    }
}

/// A moldura do painel: borda de 1px na cor forte do tema, cantos suaves.
fn moldura_painel(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.base.color,
        )),
        border: Border {
            width: 1.0,
            color: theme.extended_palette().background.strong.color,
            radius: iced::border::Radius::new(4.0),
        },
        ..Default::default()
    }
}

/// O cabeçalho do painel: título arrastável (o gesto) + botões de modo.
///
/// `grip` é o arrasto que o título dispara — [`Alvo::Zona`] quando acoplado
/// (arrastar para uma borda reancora) ou [`Alvo::Ponto`] quando flutuante
/// (arrastar move, como uma janela do `<mdiarea>`). `None` quando o `<dock>`
/// não tem `mode=` e portanto não muda de estado.
#[allow(clippy::too_many_arguments)]
fn cabecalho_dock<'a>(
    title: &str,
    mode_var: &str,
    modo: &str,
    voltar_para: &str,
    on_change: &str,
    grip: Option<Arrasto>,
) -> Element<'a, EngineMessage> {
    let rotulo = container(text(title.to_string()).size(12))
        .padding([4, 8])
        .width(Length::Fill);
    let rotulo: Element<'a, EngineMessage> = match grip {
        Some(a) => mouse_area(rotulo)
            .interaction(iced::mouse::Interaction::Grab)
            .on_press(EngineMessage::GripStart(a))
            .into(),
        None => rotulo.into(),
    };

    let mut linha = row![rotulo].align_y(Alignment::Center).width(Length::Fill);

    if !mode_var.is_empty() {
        // Um botão troca `mode` e, opcionalmente, guarda o modo atual em
        // `<mode>__prev` (para o "voltar" saber onde estava). `on_change`, se
        // houver, é disparado depois da escrita — o gancho de persistência.
        let prev_key = format!("{mode_var}__prev");
        let acao = |pares: Vec<(String, String)>| -> EngineMessage {
            if on_change.is_empty() {
                EngineMessage::ContextPatch(pares)
            } else {
                EngineMessage::PatchThen {
                    patch: pares,
                    inner: Box::new(EngineMessage::UiClick(on_change.to_string())),
                }
            }
        };
        let btn = |glifo: &'static str, pares: Vec<(String, String)>| -> Element<'a, EngineMessage> {
            mouse_area(container(text(glifo).size(12)).padding([2, 6]))
                .interaction(iced::mouse::Interaction::Pointer)
                .on_press(acao(pares))
                .into()
        };

        // `❒` guarda o modo atual e flutua; `▣` volta para onde estava (ou a
        // borda default). É o "dock back" do `QDockWidget`.
        if modo == "float" {
            linha = linha.push(btn(
                "▣",
                vec![(mode_var.to_string(), voltar_para.to_string())],
            ));
        } else {
            linha = linha.push(btn(
                "❒",
                vec![
                    (mode_var.to_string(), "float".to_string()),
                    (prev_key.clone(), modo.to_string()),
                ],
            ));
        }
        // `✕` esconde e guarda o modo atual — a aba de restaurar o traz de volta.
        linha = linha.push(btn(
            "✕",
            vec![
                (mode_var.to_string(), "hidden".to_string()),
                (prev_key, modo.to_string()),
            ],
        ));
    }

    container(linha)
        .width(Length::Fill)
        .style(fundo_forte)
        .into()
}

/// A aba fina que traz um painel `hidden` de volta. `alvo` é o modo para onde
/// voltar (o `<mode>__prev` guardado, ou a borda default), e `edge` decide de
/// que lado ela encosta e se é vertical.
fn aba_restaurar<'a>(
    mode_var: &str,
    edge: &str,
    alvo: &str,
    on_change: &str,
    title: &str,
) -> Element<'a, EngineMessage> {
    let vertical = edge == "left" || edge == "right";
    // Horizontal cabe o título; vertical, só a seta.
    let conteudo = if vertical {
        text("▸").size(11)
    } else {
        text(format!("▸  {title}")).size(11)
    };
    let corpo = container(conteudo).padding(if vertical { [8, 2] } else { [3, 8] });
    let corpo = if vertical {
        corpo.width(18).height(Length::Fill)
    } else {
        corpo.width(Length::Fill).height(20)
    };
    let msg = if on_change.is_empty() {
        EngineMessage::UiInputChanged {
            action: mode_var.to_string(),
            value: alvo.to_string(),
        }
    } else {
        EngineMessage::PatchThen {
            patch: vec![(mode_var.to_string(), alvo.to_string())],
            inner: Box::new(EngineMessage::UiClick(on_change.to_string())),
        }
    };
    mouse_area(corpo.style(fundo_forte))
        .interaction(iced::mouse::Interaction::Pointer)
        .on_press(msg)
        .into()
}

/// `<dock>` (`QDockWidget`) — ver [`crate::parser::NodeType::Dock`].
///
/// Dois filhos: o **painel** (0) e o **centro** (1). Uma chave (`mode=`) diz
/// onde o painel está — `left`/`right`/`top`/`bottom` (acoplado, num
/// `<splitter>` reutilizado; arrastável se `size=` foi dado), `float` (solto num
/// `<stack>`, movido como uma janela do `<mdiarea>`) ou `hidden` (só a aba de
/// restaurar).
///
/// # A troca de pai acontece ENTRE quadros
///
/// É o que a Onda 11 cortou: acoplar↔soltar trocaria `<splitter>` por `<stack>`
/// no meio do arrasto. Aqui o cabeçalho dispara um [`Alvo::Zona`], que **não
/// escreve nada durante o gesto** e comete a borda na soltura
/// ([`Arrasto::modo_no_release`], via `DragEnd`); o `render_dock` seguinte lê a
/// chave e monta o pai certo. É o mesmo princípio de todo widget com estado
/// deste motor — uma chave nomeada, escrita por uma ação, lida no render.
///
/// N painéis = `<dock>` aninhados, um por painel — a saída do
/// `<accordion>`/`<accordionitem>`.
#[allow(clippy::too_many_arguments)]
pub fn render_dock<'a>(
    node: &'a crate::parser::UiNode,
    context: &'a ContextMap,
    mode_var: &'a str,
    edge_default: &'a str,
    on_change: &'a str,
    size_var: &'a str,
    float_x_var: &'a str,
    float_y_var: &'a str,
    title: &'a str,
    min: f32,
    handle: f32,
    float_w: f32,
    float_h: f32,
    panel: Element<'a, EngineMessage>,
    center: Element<'a, EngineMessage>,
) -> Element<'a, EngineMessage> {
    let modo = context
        .get(mode_var)
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(edge_default);
    let modo = match modo {
        "left" | "right" | "top" | "bottom" | "float" | "hidden" => modo,
        _ => "left",
    };

    // Para onde `▣` e a aba de restaurar voltam: o `<mode>__prev` guardado
    // pelo `❒`/`✕` (se for uma borda válida), senão a borda default.
    let voltar_para: &str = context
        .get(&format!("{mode_var}__prev"))
        .map(String::as_str)
        .filter(|s| matches!(*s, "left" | "right" | "top" | "bottom"))
        .unwrap_or(edge_default);

    if modo == "hidden" {
        let aba = aba_restaurar(mode_var, edge_default, voltar_para, on_change, title);
        let dentro = container(center).width(Length::Fill).height(Length::Fill);
        return match edge_default {
            "right" => row![dentro, aba].into(),
            "top" => column![aba, dentro].into(),
            "bottom" => column![dentro, aba].into(),
            _ => row![aba, dentro].into(),
        };
    }

    if modo == "float" {
        let fx = context
            .get(float_x_var)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(48.0);
        let fy = context
            .get(float_y_var)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(48.0);
        let grip = Arrasto {
            chave: float_x_var.to_string(),
            chave_y: (!float_y_var.is_empty()).then(|| float_y_var.to_string()),
            chave_modo: None,
            indice: 0,
            eixo: Eixo::X,
            origem: None,
            origem_y: None,
            valor0: fx,
            valor0_y: fy,
            alvo: Alvo::Ponto {
                min_x: 0.0,
                max_x: MAXIMO_PAINEL,
                min_y: 0.0,
                max_y: MAXIMO_PAINEL,
            },
        };
        let flutuante = container(column![
            cabecalho_dock(title, mode_var, modo, voltar_para, on_change, Some(grip)),
            container(panel).width(Length::Fill).height(Length::Fill),
        ])
        .width(float_w)
        .height(float_h)
        .style(moldura_painel);

        return Stack::new()
            .push(container(center).width(Length::Fill).height(Length::Fill))
            .push(pin(flutuante).x(fx).y(fy))
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    // Acoplado: o cabeçalho começa um arrasto de reancoragem (`Alvo::Zona`),
    // que só comete a borda na soltura — sem trocar de pai no meio do gesto.
    let grip = (!mode_var.is_empty()).then(|| Arrasto {
        chave: String::new(),
        chave_y: None,
        chave_modo: Some(mode_var.to_string()),
        indice: 0,
        eixo: Eixo::X,
        origem: None,
        origem_y: None,
        valor0: 0.0,
        valor0_y: 0.0,
        alvo: Alvo::Zona { limiar: LIMIAR_DOCK },
    });
    let painel_col: Element<'a, EngineMessage> = container(column![
        cabecalho_dock(title, mode_var, modo, voltar_para, on_change, grip),
        container(panel).width(Length::Fill).height(Length::Fill),
    ])
    .style(moldura_painel)
    .into();

    let vertical = modo == "top" || modo == "bottom";
    let painel_primeiro = modo == "left" || modo == "top";
    let filhos = if painel_primeiro {
        vec![painel_col, center]
    } else {
        vec![center, painel_col]
    };
    render_splitter(node, context, size_var, vertical, handle, min, filhos)
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
