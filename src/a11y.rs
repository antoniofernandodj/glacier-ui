//! Acessibilidade: a árvore que um leitor de tela (Orca, NVDA, VoiceOver) lê, e o contraste.
//!
//! # O que existe
//!
//! - **No markup**, três atributos universais: `role="button"` (o papel, quando o padrão da
//!   tag não serve), `aria_label="Fechar"` (o nome lido, quando o texto do nó não serve — um
//!   botão só com `✕`) e `aria_hidden="true"` (decorativo, fora da árvore).
//! - **A árvore** ([`arvore`], atrás da feature `a11y`): construída da árvore **avaliada** a
//!   cada reavaliação — papel pela tag, nome pelo texto, estado pelo contexto (marcado, valor
//!   de um slider…) — e entregue ao AT-SPI pelo `accesskit_unix`, **sem passar pelo `winit`**.
//! - **O contraste** ([`contraste`], sempre ligado): o `.gss` avisa quando o par
//!   `color`/`background` de uma regra falha no WCAG AA (4,5:1; 3:1 para texto grande).
//!
//! # Por que o AT-SPI direto e não o `accesskit_winit`
//!
//! O `iced` 0.14 e o `winit` 0.30 não expõem o AccessKit, e o `iced` é dono do laço de
//! eventos: o `accesskit_winit` precisa do `ActiveEventLoop` e dos `WindowEvent`, que o
//! `iced` não entrega. O `accesskit_unix::Adapter`, ao contrário, não depende do `winit`:
//! registra o app no barramento de acessibilidade (D-Bus) por conta própria. Por isso a
//! ponte é **só Linux**, e é **por processo** (uma janela). Ver `docs/ACESSIBILIDADE.md`
//! para o que isto NÃO entrega (foco do teclado, geometria, texto selecionado).

use crate::parser::UiNode;

// ───────────────────────────── contraste (WCAG) ─────────────────────────────

/// Lê `#RGB`, `#RGBA`, `#RRGGBB` ou `#RRGGBBAA` em (r, g, b, a) de 0 a 255.
fn le_cor(s: &str) -> Option<(u8, u8, u8, u8)> {
    let h = s.trim().strip_prefix('#')?;
    let dobra = |c: char| c.to_digit(16).map(|d| (d * 17) as u8);
    let par = |a: &str| u8::from_str_radix(a, 16).ok();
    match h.len() {
        3 | 4 => {
            let mut it = h.chars();
            let (r, g, b) = (dobra(it.next()?)?, dobra(it.next()?)?, dobra(it.next()?)?);
            let a = it.next().map(dobra).unwrap_or(Some(255))?;
            Some((r, g, b, a))
        }
        6 | 8 => Some((par(&h[0..2])?, par(&h[2..4])?, par(&h[4..6])?, if h.len() == 8 { par(&h[6..8])? } else { 255 })),
        _ => None,
    }
}

/// Luminância relativa (WCAG 2.x).
fn luminancia((r, g, b, _): (u8, u8, u8, u8)) -> f64 {
    let lin = |c: u8| {
        let c = c as f64 / 255.0;
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// A razão de contraste (1 a 21) entre duas cores `#hex`. `None` se alguma não for uma cor
/// opaca que se saiba ler (gradiente, `var()` não resolvida, transparência: não dá para
/// saber o que está por baixo).
pub fn contraste(texto: &str, fundo: &str) -> Option<f64> {
    let (t, f) = (le_cor(texto)?, le_cor(fundo)?);
    if t.3 != 255 || f.3 != 255 {
        return None;
    }
    let (a, b) = (luminancia(t), luminancia(f));
    let (claro, escuro) = if a > b { (a, b) } else { (b, a) };
    Some((claro + 0.05) / (escuro + 0.05))
}

/// O mínimo do WCAG AA para o texto: 4,5 (normal) ou 3 (grande: 24 px ou mais, ou 18,66 px
/// em negrito).
pub fn minimo_aa(tamanho: Option<f32>, negrito: bool) -> f64 {
    match tamanho {
        Some(s) if s >= 24.0 || (negrito && s >= 18.66) => 3.0,
        _ => 4.5,
    }
}

/// O aviso de uma regra cujo par `color`/`background` falha no AA, ou `None` se passa ou se
/// não dá para medir.
pub fn aviso_de_contraste(
    seletor: &str,
    texto: &str,
    fundo: &str,
    tamanho: Option<f32>,
    negrito: bool,
) -> Option<String> {
    let r = contraste(texto, fundo)?;
    let min = minimo_aa(tamanho, negrito);
    (r < min).then(|| {
        format!(
            "contraste {r:.2}:1 em '{seletor}' ({texto} sobre {fundo}) está abaixo do mínimo WCAG AA de {min}:1"
        )
    })
}

// ───────────────────────────── papéis ─────────────────────────────

/// Os valores aceitos em `role="…"` (ARIA, em minúsculas). Fora desta lista o atributo é
/// ignorado com aviso.
pub const PAPEIS: &[&str] = &[
    "button", "link", "checkbox", "radio", "switch", "textbox", "searchbox", "slider", "progressbar",
    "img", "heading", "list", "listitem", "table", "row", "cell", "columnheader", "tab", "tablist",
    "tabpanel", "menu", "menubar", "menuitem", "dialog", "alert", "status", "group", "region",
    "navigation", "main", "separator", "article", "document", "form", "combobox", "spinbutton",
    "tree", "treeitem", "tooltip", "toolbar", "paragraph", "blockquote", "none",
];

/// O que o leitor de tela diz de um nó, independente do `accesskit` (testável sem a feature).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Descricao {
    /// O papel ARIA (`button`, `heading`…), ou `None` para um contêiner sem significado.
    pub papel: Option<&'static str>,
    pub nome: Option<String>,
    pub valor: Option<String>,
    /// Marcado/ligado, para checkbox/switch/radio.
    pub marcado: Option<bool>,
    pub nivel: Option<u8>,
    pub oculto: bool,
}

/// Descreve um nó avaliado. `ler` consulta o contexto (o valor de um slider, o marcado de uma
/// checkbox).
pub fn descreve(no: &UiNode, ler: &dyn Fn(&str) -> Option<String>) -> Descricao {
    use crate::parser::NodeType as N;
    let mut d = Descricao { oculto: no.aria_hidden(), ..Default::default() };
    let verdadeiro = |k: &str| ler(k).is_some_and(|v| crate::widget::is_truthy(&v));
    match &no.kind {
        N::Text { content, .. } => d.nome = Some(content.clone()),
        N::Markdown { content, .. } => {
            d.papel = Some("document");
            d.nome = Some(content.clone());
        }
        N::Button { text, .. } => {
            d.papel = Some("button");
            d.nome = Some(text.clone());
        }
        N::TextInput { placeholder, value_var, secure, .. } => {
            d.papel = Some("textbox");
            d.nome = Some(placeholder.clone()).filter(|p| !p.is_empty());
            if !*secure {
                d.valor = ler(value_var);
            }
        }
        N::TextArea { value_var, .. } => {
            d.papel = Some("textbox");
            d.valor = ler(value_var);
        }
        N::Checkbox { label, checked_var, .. } => {
            d.papel = Some("checkbox");
            d.nome = Some(label.clone());
            d.marcado = Some(verdadeiro(checked_var));
        }
        N::Toggle { label, checked_var, .. } => {
            d.papel = Some("switch");
            d.nome = Some(label.clone());
            d.marcado = Some(verdadeiro(checked_var));
        }
        N::Radio { label, value, group_var, .. } => {
            d.papel = Some("radio");
            d.nome = Some(label.clone());
            d.marcado = Some(ler(group_var).is_some_and(|g| &g == value));
        }
        N::Slider { value_var, .. } | N::Rating { value_var, .. } => {
            d.papel = Some("slider");
            d.valor = ler(value_var);
        }
        N::ProgressBar { value_var, .. } => {
            d.papel = Some("progressbar");
            d.valor = ler(value_var);
        }
        N::Select { value_var, placeholder, .. } => {
            d.papel = Some("combobox");
            d.nome = Some(placeholder.clone()).filter(|p| !p.is_empty());
            d.valor = ler(value_var);
        }
        N::Image { source, .. } | N::Svg { source, .. } => {
            d.papel = Some("img");
            d.nome = Some(source.clone());
        }
        N::Rule { .. } => d.papel = Some("separator"),
        N::Form { name, .. } => {
            d.papel = Some("form");
            d.nome = name.clone().filter(|n| !n.is_empty());
        }
        N::Prose { .. } => d.papel = Some("document"),
        N::Audio { name, .. } => {
            d.papel = Some("group");
            d.nome = Some(format!("áudio {name}"));
        }
        N::TableView { .. } => d.papel = Some("table"),
        N::TreeView { .. } => d.papel = Some("tree"),
        N::Menu { .. } | N::MenuBar { .. } => d.papel = Some("menu"),
        N::MenuItem { .. } => d.papel = Some("menuitem"),
        N::Scrollable { .. } => d.papel = Some("region"),
        N::Spinner { .. } => {
            d.papel = Some("progressbar");
            d.nome = Some("carregando".into());
        }
        _ => {}
    }
    // `role=` vence o padrão da tag, e `aria_label=` vence o texto.
    if let Some(r) = no.role() {
        let r = r.trim().to_ascii_lowercase();
        match PAPEIS.iter().find(|p| **p == r) {
            Some(p) if *p == "none" => d.oculto = true,
            Some(p) => d.papel = Some(p),
            None => eprintln!(
                "glacier-ui: role=\"{r}\" desconhecido (ignorado); os aceitos: {}",
                PAPEIS.join(", ")
            ),
        }
    }
    if let Some(l) = no.aria_label() {
        d.nome = Some(l.to_string());
    }
    d
}

// ───────────────────────────── a árvore e o AT-SPI ─────────────────────────────

#[cfg(all(feature = "a11y", target_os = "linux"))]
pub mod arvore {
    use super::*;
    use accesskit::{
        Action, ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node, NodeId,
        Role, Toggled, TreeId, TreeInfo, TreeUpdate,
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};

    const RAIZ: NodeId = NodeId(1);

    fn role_de(papel: &str) -> Role {
        match papel {
            "button" => Role::Button,
            "link" => Role::Link,
            "checkbox" => Role::CheckBox,
            "radio" => Role::RadioButton,
            "switch" => Role::Switch,
            "textbox" => Role::TextInput,
            "searchbox" => Role::SearchInput,
            "slider" => Role::Slider,
            "progressbar" => Role::ProgressIndicator,
            "img" => Role::Image,
            "heading" => Role::Heading,
            "list" => Role::List,
            "listitem" => Role::ListItem,
            "table" => Role::Table,
            "row" => Role::Row,
            "cell" => Role::Cell,
            "columnheader" => Role::ColumnHeader,
            "tab" => Role::Tab,
            "tablist" => Role::TabList,
            "tabpanel" => Role::TabPanel,
            "menu" => Role::Menu,
            "menubar" => Role::MenuBar,
            "menuitem" => Role::MenuItem,
            "dialog" => Role::Dialog,
            "alert" => Role::Alert,
            "status" => Role::Status,
            "region" => Role::Region,
            "navigation" => Role::Navigation,
            "main" => Role::Main,
            "separator" => Role::Splitter,
            "article" => Role::Article,
            "document" => Role::Document,
            "form" => Role::Form,
            "combobox" => Role::ComboBox,
            "spinbutton" => Role::SpinButton,
            "tree" => Role::Tree,
            "treeitem" => Role::TreeItem,
            "tooltip" => Role::Tooltip,
            "toolbar" => Role::Toolbar,
            "paragraph" => Role::Paragraph,
            "blockquote" => Role::Blockquote,
            _ => Role::Group,
        }
    }

    /// A tabela de ações do AT: nó → ação do markup (o `on_click` do botão).
    type Acoes = Arc<Mutex<HashMap<NodeId, String>>>;

    struct Construtor<'a> {
        nos: Vec<(NodeId, Node)>,
        acoes: HashMap<NodeId, String>,
        usados: std::collections::HashSet<u64>,
        ler: &'a dyn Fn(&str) -> Option<String>,
    }

    impl Construtor<'_> {
        fn id(&mut self, caminho: &[usize]) -> NodeId {
            // FNV-1a do caminho, e desempate linear: o id é estável entre reavaliações.
            let mut h: u64 = 0xcbf29ce484222325;
            for i in caminho {
                h ^= *i as u64 + 1;
                h = h.wrapping_mul(0x100000001b3);
            }
            let mut h = h.max(2);
            while !self.usados.insert(h) {
                h += 1;
            }
            NodeId(h)
        }

        /// Constrói o nó de `no` e devolve o id (ou `None` se é oculto/sem significado e sem filhos).
        fn no(&mut self, no: &UiNode, caminho: &mut Vec<usize>) -> Vec<NodeId> {
            let d = descreve(no, self.ler);
            if d.oculto {
                return Vec::new();
            }
            // Filhos (e os blocos do `<prose>`).
            let mut filhos = Vec::new();
            for (i, f) in no.children.iter().enumerate() {
                caminho.push(i);
                filhos.extend(self.no(f, caminho));
                caminho.pop();
            }
            if let crate::parser::NodeType::Prose { blocos, .. } = &no.kind {
                for (i, b) in blocos.iter().enumerate() {
                    caminho.push(10_000 + i);
                    if let Some(id) = self.bloco(b, caminho) {
                        filhos.push(id);
                    }
                    caminho.pop();
                }
            }
            // Um nó sem papel nem nome é só agrupamento visual: seus filhos sobem um nível.
            if d.papel.is_none() && d.nome.is_none() && d.valor.is_none() {
                return filhos;
            }
            let id = self.id(caminho);
            let role = match d.papel {
                Some(p) => role_de(p),
                None => Role::Label,
            };
            let mut n = Node::new(role);
            if let Some(nome) = &d.nome {
                if role == Role::Label {
                    n.set_value(nome.clone());
                } else {
                    n.set_label(nome.clone());
                }
            }
            if let Some(v) = &d.valor {
                match v.trim().parse::<f64>() {
                    Ok(x) if matches!(role, Role::Slider | Role::ProgressIndicator) => {
                        n.set_numeric_value(x)
                    }
                    _ => n.set_value(v.clone()),
                }
            }
            if let Some(m) = d.marcado {
                n.set_toggled(if m { Toggled::True } else { Toggled::False });
            }
            if no.disabled.unwrap_or(false) {
                n.set_disabled();
            }
            if matches!(role, Role::Button | Role::Link | Role::MenuItem) {
                n.add_action(Action::Click);
                let acao = match &no.kind {
                    crate::parser::NodeType::Button { on_click: Some(a), .. } if !a.is_empty() => {
                        Some(a.clone())
                    }
                    _ => no.on_press().map(str::to_string),
                };
                if let Some(a) = acao {
                    self.acoes.insert(id, a);
                }
            }
            n.set_children(filhos);
            self.nos.push((id, n));
            vec![id]
        }

        fn bloco(&mut self, b: &crate::prose::Bloco, caminho: &[usize]) -> Option<NodeId> {
            use crate::prose::Bloco;
            let texto = |spans: &[crate::parser::TextSpan]| spans.iter().map(|s| s.content.as_str()).collect::<String>();
            let (role, nome, nivel) = match b {
                Bloco::Paragrafo(s) => (Role::Paragraph, texto(s), None),
                Bloco::Titulo { nivel, spans } => (Role::Heading, texto(spans), Some(*nivel)),
                Bloco::Citacao { paragrafos, autoria } => {
                    let mut t: Vec<String> = paragrafos.iter().map(|p| texto(p)).collect();
                    if !autoria.is_empty() {
                        t.push(format!("— {}", texto(autoria)));
                    }
                    (Role::Blockquote, t.join(" "), None)
                }
                Bloco::Lista { itens, .. } => {
                    let id = self.id(caminho);
                    let mut filhos = Vec::new();
                    for (i, item) in itens.iter().enumerate() {
                        let mut c = caminho.to_vec();
                        c.push(i);
                        let iid = self.id(&c);
                        let mut n = Node::new(Role::ListItem);
                        n.set_label(texto(item));
                        self.nos.push((iid, n));
                        filhos.push(iid);
                    }
                    let mut n = Node::new(Role::List);
                    n.set_children(filhos);
                    self.nos.push((id, n));
                    return Some(id);
                }
                _ => return None,
            };
            let id = self.id(caminho);
            let mut n = Node::new(role);
            n.set_label(nome);
            if let Some(l) = nivel {
                n.set_level(l as usize);
            }
            self.nos.push((id, n));
            Some(id)
        }
    }

    /// A árvore de acessibilidade de uma tela avaliada, mais a tabela de ações.
    pub fn constroi(
        raiz: &UiNode,
        titulo: &str,
        ler: &dyn Fn(&str) -> Option<String>,
    ) -> (TreeUpdate, HashMap<NodeId, String>) {
        let mut c = Construtor { nos: Vec::new(), acoes: HashMap::new(), usados: Default::default(), ler };
        c.usados.insert(RAIZ.0);
        let filhos = c.no(raiz, &mut vec![0]);
        let mut janela = Node::new(Role::Window);
        janela.set_label(titulo.to_string());
        janela.set_children(filhos);
        c.nos.push((RAIZ, janela));
        let tree = TreeUpdate {
            nodes: c.nos,
            tree: Some(TreeInfo {
                root: RAIZ,
                toolkit_name: Some("glacier-ui".into()),
                toolkit_version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            tree_id: TreeId::ROOT,
            focus: RAIZ,
        };
        (tree, c.acoes)
    }

    // ── o adaptador ──

    struct Ativacao(Arc<Mutex<Option<TreeUpdate>>>);
    impl ActivationHandler for Ativacao {
        fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
            self.0.lock().ok()?.clone()
        }
    }
    struct Acao(Acoes);
    impl ActionHandler for Acao {
        fn do_action(&mut self, r: ActionRequest) {
            if r.action != Action::Click {
                return;
            }
            let a = self.0.lock().ok().and_then(|m| m.get(&r.target_node).cloned());
            if let Some(a) = a {
                crate::external::sender().click(a);
            }
        }
    }
    struct Desativacao;
    impl DeactivationHandler for Desativacao {
        fn deactivate_accessibility(&mut self) {}
    }

    struct Ponte {
        adaptador: Mutex<accesskit_unix::Adapter>,
        ultima: Arc<Mutex<Option<TreeUpdate>>>,
        acoes: Acoes,
    }

    fn ponte() -> &'static Option<Ponte> {
        static P: OnceLock<Option<Ponte>> = OnceLock::new();
        P.get_or_init(|| {
            if std::env::var("GLACIER_A11Y").is_ok_and(|v| v == "0") {
                return None;
            }
            let ultima = Arc::new(Mutex::new(None));
            let acoes: Acoes = Default::default();
            // O canal externo é por onde uma ação do leitor de tela volta ao app.
            let _ = crate::external::sender();
            let mut adaptador = accesskit_unix::Adapter::new(
                Ativacao(ultima.clone()),
                Acao(acoes.clone()),
                Desativacao,
            );
            adaptador.update_window_focus_state(true);
            Some(Ponte { adaptador: Mutex::new(adaptador), ultima, acoes })
        })
    }

    /// Publica a árvore da tela avaliada. Barato quando nenhum leitor de tela está ligado: o
    /// adaptador só pede a árvore (e só chama a fábrica) quando há um cliente AT-SPI.
    pub fn publica(raiz: &UiNode, titulo: &str, ler: &dyn Fn(&str) -> Option<String>) {
        let Some(p) = ponte() else { return };
        let (tree, acoes) = constroi(raiz, titulo, ler);
        *p.acoes.lock().unwrap_or_else(|e| e.into_inner()) = acoes;
        *p.ultima.lock().unwrap_or_else(|e| e.into_inner()) = Some(tree.clone());
        p.adaptador.lock().unwrap_or_else(|e| e.into_inner()).update_if_active(|| tree);
    }
}

/// Sem a feature `a11y` (ou fora do Linux) a publicação é um no-op.
#[cfg(not(all(feature = "a11y", target_os = "linux")))]
pub mod arvore {
    use super::*;
    pub fn publica(_: &UiNode, _: &str, _: &dyn Fn(&str) -> Option<String>) {}
}
