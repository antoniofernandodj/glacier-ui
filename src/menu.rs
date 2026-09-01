//! Menu bar dropdowns e menus de contexto (botão direito), com submenus
//! aninhados a profundidade arbitrária — a contraparte de [`crate::dialogs`]
//! para esta categoria de overlay: transiente, construído a partir da árvore
//! avaliada ([`MenuNode`]) e sobreposto à tela ativa em
//! [`crate::GlacierUI::render_current`].
//!
//! ## Por que não o overlay nativo do iced
//!
//! `iced` já traz internamente o mecanismo "certo" para popups ancorados —
//! `iced::overlay::menu::Menu` (usado por `pick_list`/`combo_box`), que
//! ancora via `Widget::overlay()` na própria posição de layout do widget
//! gatilho. Mas é genérico só sobre `T: ToString + Clone` (lista plana) —
//! não serve como está para itens ricos com submenu. Implementar o
//! equivalente próprio (um `iced::advanced::{Widget, Overlay}` custom, com
//! `Overlay` aninhado por nível de cascata) é o caminho "nativo" correto,
//! mas exige boilerplate substancial sem precedente neste código-base.
//!
//! Este módulo usa em vez disso a mesma técnica pragmática de
//! `dialogs.rs`/`toasts.rs`: uma camada `stack![]` cobrindo a janela inteira,
//! com cada painel posicionado via `container` + `padding` calculado
//! analiticamente a partir do ponto-âncora (posição do cursor no momento do
//! clique/clique-direito — ver [`crate::EngineMessage::CursorMoved`]) e da
//! altura determinística de cada painel (linhas de altura fixa). Não há
//! medição de layout real antes de posicionar (o que a forma nativa dá de
//! graça), mas como a altura é sempre computável de antemão isso não importa
//! na prática. Upgrade de v2: migrar para `advanced::Overlay` de verdade,
//! ganhando ancoragem pixel-perfeita ao widget-gatilho em vez de ao cursor.
//!
//! ## Estado
//!
//! Só um menu/cascata pode estar aberto por vez no app inteiro — abrir
//! qualquer menu substitui o anterior, como diálogos reais de SO desktop. Por
//! isso [`crate::GlacierUI::active_menu`] é um único `Option<ActiveMenuState>`
//! global, o mesmo padrão de `GlacierUI::dialog` — nenhum dos pré-requisitos
//! de "estado por instância" do `PLANO_WIDGETS.md` §3 (que bloqueia
//! `Tabs`/`Accordion`) se aplica aqui.

use crate::parser::{NodeType, UiNode};
use crate::widget::{EngineMessage, is_truthy};
use iced::widget::{Space, button, column, container, mouse_area, row, rule, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Shadow};
use std::collections::HashMap;
use std::sync::Arc;

/// Um nó já resolvido da árvore de menu — independente de ter vindo de
/// markup estático (`<Menu>`/`<MenuItem>`/`<MenuSeparator>`) ou de um array
/// JSON dinâmico (`items="chave_de_contexto"`, mesma convenção de
/// `Select::options` — ver [`build_tree`]/[`MenuNode::from_json`]).
#[derive(Debug, Clone)]
pub struct MenuNode {
    pub label: String,
    pub icon: Option<String>,
    /// Já namespaced (`Componente::acao`) quando veio de um `<MenuItem
    /// onClick="...">` estático — ver a regra em `eval.rs`. Quando veio de
    /// `items` JSON (Luau), é a string crua: `route_to_owner` cai de volta
    /// pra `current_screen` quando não tem prefixo, então na prática só
    /// importa escrever o prefixo à mão quando o menu não pertence à tela
    /// ativa (ver o comentário em [`MenuNode::from_json`]).
    pub on_click: Option<String>,
    pub checked: Option<bool>,
    pub disabled: bool,
    pub separator: bool,
    /// Não-vazio ⇒ esta linha abre um submenu ao passar o mouse.
    pub children: Vec<MenuNode>,
}

impl MenuNode {
    fn separator_node() -> Self {
        Self {
            label: String::new(),
            icon: None,
            on_click: None,
            checked: None,
            disabled: false,
            separator: true,
            children: Vec::new(),
        }
    }

    /// Constrói um nó a partir de um elemento de array JSON de `items`
    /// (convenção: `label`/`text`, `action`/`onClick`/`on_click`, `icon`,
    /// `checked`, `disabled`, `separator`, `items` aninhado para submenu).
    ///
    /// `action` **não** passa por `namespace_action` — isso só acontece em
    /// `eval.rs`, em cima de um `on_click` de markup estático, num momento
    /// em que o `owner` ainda está em escopo; o JSON de `items` é opaco
    /// nesse estágio. Na prática funciona porque `GlacierUI::route_to_owner`
    /// já cai de volta para `current_screen` quando a ação não tem prefixo
    /// `Componente::` — o caso comum de um componente Luau que define seu
    /// próprio menu e reage a ele. Para um menu que não pertence à tela
    /// ativa, o Luau precisa escrever o prefixo manualmente
    /// (`action = "OutroComponente::acao"`), a mesma saída de escape que já
    /// existe hoje para ações escritas à mão.
    fn from_json(v: &serde_json::Value) -> Option<Self> {
        let o = v.as_object()?;
        let get_str = |k: &str| -> Option<String> {
            o.get(k).and_then(|x| x.as_str()).map(|s| s.to_string())
        };
        let get_bool = |k: &str| o.get(k).and_then(|x| x.as_bool()).unwrap_or(false);

        if get_bool("separator") || get_bool("isSeparator") {
            return Some(Self::separator_node());
        }

        let label = get_str("label")
            .or_else(|| get_str("text"))
            .unwrap_or_default();
        let icon = get_str("icon");
        let on_click = get_str("action")
            .or_else(|| get_str("onClick"))
            .or_else(|| get_str("on_click"));
        let checked = o.get("checked").and_then(|x| x.as_bool());
        let disabled = get_bool("disabled");
        let children = o
            .get("items")
            .and_then(|x| x.as_array())
            .map(|arr| arr.iter().filter_map(MenuNode::from_json).collect())
            .unwrap_or_default();

        Some(Self {
            label,
            icon,
            on_click,
            checked,
            disabled,
            separator: false,
            children,
        })
    }
}

/// Converte os filhos avaliados de um `<Menu>`/`<ContextMenu>` (markup
/// `<Menu>`/`<MenuItem>`/`<MenuSeparator>` estático, recursivo — um `<Menu>`
/// aninhado vira um `MenuNode` cujos `children` são a mesma chamada
/// recursiva) numa árvore [`MenuNode`], mesclando ao final as entradas de um
/// array JSON dinâmico se `items_key` apontar para uma chave de contexto
/// presente (mesma convenção de `Select::options`/`SelectOption::from_json`
/// — `items_key` é o NOME da chave, não o JSON em si; o JSON é buscado no
/// `context` aqui, na hora da renderização).
pub fn build_tree(
    children: &[UiNode],
    items_key: Option<&str>,
    context: &HashMap<String, String>,
) -> Vec<MenuNode> {
    let mut out = Vec::new();
    for child in children {
        match &child.kind {
            NodeType::MenuSeparator => out.push(MenuNode::separator_node()),
            NodeType::MenuItem {
                label,
                icon,
                on_click,
                checked_var,
                disabled,
            } => {
                let checked = checked_var
                    .as_ref()
                    .and_then(|k| context.get(k))
                    .map(|v| is_truthy(v));
                out.push(MenuNode {
                    label: label.clone(),
                    icon: icon.clone(),
                    on_click: on_click.clone(),
                    checked,
                    disabled: *disabled,
                    separator: false,
                    children: Vec::new(),
                });
            }
            NodeType::Menu {
                label,
                icon,
                disabled,
                items,
            } => out.push(MenuNode {
                label: label.clone(),
                icon: icon.clone(),
                on_click: None,
                checked: None,
                disabled: *disabled,
                separator: false,
                children: build_tree(&child.children, items.as_deref(), context),
            }),
            // Qualquer outra coisa dentro de um <Menu>/<ContextMenu> (texto
            // solto, um nó não-relacionado a menu) é ignorada silenciosamente
            // — só os três tipos de menu compõem a árvore.
            _ => {}
        }
    }
    if let Some(key) = items_key {
        let dynamic = context
            .get(key)
            .and_then(|j| serde_json::from_str::<serde_json::Value>(j).ok())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        out.extend(dynamic.iter().filter_map(MenuNode::from_json));
    }
    out
}

/// O menu/cascata atualmente aberto (ver o campo `GlacierUI::active_menu`,
/// singleton — só um pode estar aberto por vez no app inteiro).
pub struct ActiveMenuState {
    pub tree: Arc<Vec<MenuNode>>,
    /// Posição do cursor (espaço da janela) no momento em que o menu foi
    /// aberto — ver [`crate::EngineMessage::CursorMoved`]. O painel raiz
    /// nasce logo abaixo/à direita deste ponto.
    pub anchor: iced::Point,
    /// Índices do caminho de cascata atualmente aberto, raiz→folha. Vazio =
    /// só o painel raiz está visível.
    pub open_path: Vec<usize>,
}

const ROW_HEIGHT: f32 = 30.0;
const SEPARATOR_HEIGHT: f32 = 9.0;
const PANEL_WIDTH: f32 = 200.0;
const PANEL_PADDING: f32 = 4.0;

fn row_h(n: &MenuNode) -> f32 {
    if n.separator {
        SEPARATOR_HEIGHT
    } else {
        ROW_HEIGHT
    }
}

fn panel_height(nodes: &[MenuNode]) -> f32 {
    PANEL_PADDING * 2.0 + nodes.iter().map(row_h).sum::<f32>()
}

/// Deslocamento vertical (dentro do painel, já contando o padding) do topo
/// da linha `index` — usado para posicionar o painel-filho alinhado com a
/// linha que o abriu.
fn row_offset(nodes: &[MenuNode], index: usize) -> f32 {
    let end = index.min(nodes.len());
    PANEL_PADDING + nodes[..end].iter().map(row_h).sum::<f32>()
}

fn position_root(anchor_x: f32, viewport_w: f32) -> f32 {
    anchor_x.clamp(0.0, (viewport_w - PANEL_WIDTH).max(0.0))
}

/// Posição X do painel-filho: à direita do painel-pai por padrão; se não
/// couber, abre à esquerda dele em vez de simplesmente encostar na borda da
/// tela (o que faria o submenu se sobrepor ao próprio pai).
fn position_child_x(parent_x: f32, viewport_w: f32) -> f32 {
    let right = parent_x + PANEL_WIDTH;
    if right + PANEL_WIDTH > viewport_w {
        (parent_x - PANEL_WIDTH).max(0.0)
    } else {
        right
    }
}

fn position_y(y: f32, height: f32, viewport_h: f32) -> f32 {
    y.clamp(0.0, (viewport_h - height).max(0.0))
}

/// Renderiza o menu/cascata aberto como uma camada de overlay completa:
/// fundo transparente cobrindo a janela toda (capturando clique-fora — ver a
/// nota de `Interaction::Idle`/`on_press` sempre presente, a mesma lição de
/// `dialogs.rs`/`DIALOGS.md`) mais um painel por nível de cascata aberto.
/// Chame de [`crate::GlacierUI::render_current`], que já empilha isto por
/// cima de tudo quando `active_menu.is_some()`.
pub fn overlay<'a>(
    state: &'a ActiveMenuState,
    theme: &iced::Theme,
    viewport: (f32, f32),
) -> Element<'a, EngineMessage> {
    let palette = theme.extended_palette();
    let (vw, vh) = viewport;

    let backdrop = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style::default());
    // Mesma lição de `dialogs.rs`: `Interaction::Idle` (não `None`) evita que
    // o hover vaze para a camada de baixo via `Stack::mouse_interaction()`;
    // `on_press` sempre presente evita que o clique atravesse o stack em vez
    // de ser capturado por este backdrop (`MouseArea::update` só chama
    // `shell.capture_event()` quando tem um handler de `on_press`).
    let backdrop_area = mouse_area(backdrop)
        .interaction(iced::mouse::Interaction::Idle)
        .on_press(EngineMessage::MenuDismiss);

    let mut layers: Vec<Element<'a, EngineMessage>> = vec![backdrop_area.into()];

    let mut path_prefix: Vec<usize> = Vec::new();
    let mut nodes: &'a [MenuNode] = &state.tree;
    let mut px = position_root(state.anchor.x, vw);
    let mut py = position_y(state.anchor.y + 4.0, panel_height(nodes), vh);

    loop {
        layers.push(positioned_panel(nodes, &path_prefix, px, py, palette));

        if path_prefix.len() == state.open_path.len() {
            break;
        }
        let idx = state.open_path[path_prefix.len()];
        let Some(item) = nodes.get(idx) else { break };
        if item.children.is_empty() {
            break;
        }
        let child_h = panel_height(&item.children);
        let child_x = position_child_x(px, vw);
        let child_y = position_y(py + row_offset(nodes, idx), child_h, vh);

        path_prefix.push(idx);
        nodes = &item.children;
        px = child_x;
        py = child_y;
    }

    iced::widget::Stack::with_children(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn positioned_panel<'a>(
    nodes: &'a [MenuNode],
    path_prefix: &[usize],
    x: f32,
    y: f32,
    palette: &iced::theme::palette::Extended,
) -> Element<'a, EngineMessage> {
    container(panel_box(nodes, path_prefix, palette))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: y,
            right: 0.0,
            bottom: 0.0,
            left: x,
        })
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .into()
}

fn panel_box<'a>(
    nodes: &'a [MenuNode],
    path_prefix: &[usize],
    palette: &iced::theme::palette::Extended,
) -> Element<'a, EngineMessage> {
    let mut col = column![].width(Length::Fixed(PANEL_WIDTH));
    for (i, node) in nodes.iter().enumerate() {
        let mut path = path_prefix.to_vec();
        path.push(i);
        col = col.push(render_row(node, path, palette));
    }
    let bg = palette.background.base.color;
    let border_color = palette.background.strong.color;
    container(col)
        .padding(PANEL_PADDING)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                radius: iced::border::Radius::new(6.0),
                width: 1.0,
                color: border_color,
            },
            // Sem sombra (blur): a árvore inteira do overlay é reconstruída
            // a cada troca de hover (`MenuHoverSubmenu`), e uma sombra
            // desfocada — alpha parcial recomposto a cada linha sobre a qual
            // o mouse passa — deixava rastro/escurecia progressivamente sob
            // o renderer por software deste ambiente (sem GPU compatível).
            // Um `border` sólido já basta pra separar o painel do fundo.
            shadow: Shadow::default(),
            ..Default::default()
        })
        .into()
}

/// `path` já é o caminho completo (raiz→esta linha, inclusive). Para uma
/// linha-folha, o `MenuHoverSubmenu` que ela emite no hover é o caminho SEM
/// si mesma (fecha qualquer submenu mais fundo que estivesse aberto);para
/// uma linha com filhos, é o caminho COM si mesma (abre o próprio submenu).
fn render_row<'a>(
    node: &'a MenuNode,
    path: Vec<usize>,
    palette: &iced::theme::palette::Extended,
) -> Element<'a, EngineMessage> {
    if node.separator {
        return container(rule::horizontal(1))
            .padding([4, 0])
            .width(Length::Fill)
            .height(Length::Fixed(SEPARATOR_HEIGHT))
            .align_y(Alignment::Center)
            .into();
    }

    let has_children = !node.children.is_empty();
    let text_color = if node.disabled {
        palette.background.strong.color
    } else {
        palette.background.base.text
    };
    let check_glyph = if node.checked == Some(true) {
        "✓"
    } else {
        ""
    };

    let mut r = row![
        text(check_glyph)
            .size(13)
            .width(Length::Fixed(16.0))
            .color(text_color),
        text(node.label.as_str())
            .size(13)
            .width(Length::Fill)
            .color(text_color),
    ]
    .spacing(6)
    .height(Length::Fill)
    .align_y(Alignment::Center);
    if has_children {
        r = r.push(text("▸").size(12).color(text_color));
    }

    // Um `container` comum não tem noção de hover — só widgets com
    // `Status` (como `button`) recebem um estado de "sendo apontado" do
    // iced para se restylizar sozinhos. Por isso cada linha é um `button`
    // (mesmo truque já usado no gatilho da MenuBar/`<Menu>` avulso), não um
    // `container`: o brilho ao mirar vem de graça do próprio widget, sem o
    // motor precisar rastrear "qual linha está sob o cursor" como estado.
    //
    // `iced_widget::button` só reporta `Status::Hovered` quando tem
    // `on_press` anexado — sem ele, cai sempre em `Status::Disabled`
    // (nenhum destaque), então toda linha habilitada recebe um `on_press`:
    // a linha-folha dispara sua ação; a linha com submenu reabre o mesmo
    // submenu do hover (clicar um item de submenu é um gesto válido demais
    // em menus reais pra deixar sem resposta).
    let hovered_path = path.clone();
    let mut btn = button(r)
        .padding([0, 10])
        .width(Length::Fill)
        .height(Length::Fixed(ROW_HEIGHT))
        .style(move |theme: &iced::Theme, status: button::Status| {
            let pal = theme.extended_palette();
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(pal.primary.weak.color))
                }
                _ => None,
            };
            button::Style {
                background: bg,
                // Só um fallback — cada `Text` da linha já traz sua própria
                // `.color()` (ver acima), calculada a partir de
                // `node.disabled`, então isto nunca aparece de fato.
                text_color: pal.background.base.text,
                border: Border {
                    radius: iced::border::Radius::new(0.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        });
    if !node.disabled {
        if has_children {
            btn = btn.on_press(EngineMessage::MenuHoverSubmenu { path: path.clone() });
        } else {
            let mut leaf_path = path.clone();
            leaf_path.pop();
            if let Some(action) = node.on_click.clone() {
                btn = btn.on_press(EngineMessage::MenuItemClick(action));
            } else {
                btn = btn.on_press(EngineMessage::MenuHoverSubmenu { path: leaf_path });
            }
        }
    }

    if node.disabled {
        return btn.into();
    }
    let mut ma = mouse_area(btn);
    if has_children {
        ma = ma.on_enter(EngineMessage::MenuHoverSubmenu { path: hovered_path });
    } else {
        let mut leaf_path = hovered_path;
        leaf_path.pop();
        ma = ma.on_enter(EngineMessage::MenuHoverSubmenu { path: leaf_path });
    }
    ma.into()
}
