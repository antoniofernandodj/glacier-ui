use std::collections::HashMap;
use roxmltree::Node;

#[derive(Debug, Clone, PartialEq)]
pub enum TipoNo {
    Container,
    Column,
    Row,
    Text {
        content: String,
        size: Option<f32>,
        bold: bool,
        color: Option<String>,
    },
    Button {
        text: String,
        on_click: Option<String>,
        color: Option<String>,
    },
    TextInput {
        placeholder: String,
        value_var: String,
        on_change: String,
    },
    Image {
        source: String,
        clip_circle: bool,
    },
    Include {
        src: String,
        props: HashMap<String, String>,
    },
    /// A reference to another registered component by its own tag name,
    /// e.g. `<PerfilCard nome="..." />`. Attributes become props.
    Component {
        name: String,
        props: HashMap<String, String>,
    },
    ForEach {
        items: String,
        var: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoUI {
    pub tipo: TipoNo,
    pub filhos: Vec<NoUI>,
    pub largura: Option<String>,
    pub altura: Option<String>,
    pub padding: Option<String>,
    pub align_x: Option<String>,
    pub align_y: Option<String>,
    pub spacing: Option<f32>,
    pub background: Option<String>,
    pub border_radius: Option<f32>,
    pub border_width: Option<f32>,
    pub border_color: Option<String>,
}

impl NoUI {
    /// Helper to find a specific attribute case-insensitively
    fn get_attr(node: &Node, keys: &[&str]) -> Option<String> {
        for key in keys {
            if let Some(val) = node.attribute(*key) {
                return Some(val.to_string());
            }
        }
        None
    }

    /// Helper to parse a float attribute
    fn get_attr_f32(node: &Node, keys: &[&str]) -> Option<f32> {
        Self::get_attr(node, keys).and_then(|s| s.parse::<f32>().ok())
    }

    /// Helper to parse a bool attribute
    fn get_attr_bool(node: &Node, keys: &[&str]) -> bool {
        Self::get_attr(node, keys)
            .map(|s| s.eq_ignore_ascii_case("true") || s == "1")
            .unwrap_or(false)
    }

    /// Recursively parse a roxmltree Node into NoUI
    pub fn from_node(node: Node) -> Option<Self> {
        if !node.is_element() {
            return None;
        }

        let tag = node.tag_name().name();
        
        // Parse standard layout/style attributes
        let largura = Self::get_attr(&node, &["width", "largura", "w"]);
        let altura = Self::get_attr(&node, &["height", "altura", "h"]);
        let padding = Self::get_attr(&node, &["padding", "espacamento_interno"]);
        let align_x = Self::get_attr(&node, &["alignX", "align_x", "align-x", "alinhamento_x"]);
        let align_y = Self::get_attr(&node, &["alignY", "align_y", "align-y", "alinhamento_y"]);
        let spacing = Self::get_attr_f32(&node, &["spacing", "espacamento"]);
        let background = Self::get_attr(&node, &["background", "bg", "fundo"]);
        let border_radius = Self::get_attr_f32(&node, &["borderRadius", "border_radius", "border-radius", "raio_borda"]);
        let border_width = Self::get_attr_f32(&node, &["borderWidth", "border_width", "border-width", "largura_borda"]);
        let border_color = Self::get_attr(&node, &["borderColor", "border_color", "border-color", "cor_borda"]);

        let tipo = match tag {
            "Container" | "container" => TipoNo::Container,
            "Column" | "column" => TipoNo::Column,
            "Row" | "row" => TipoNo::Row,
            "Text" | "text" => {
                let content = Self::get_attr(&node, &["content", "conteudo", "text", "texto"]).unwrap_or_default();
                let size = Self::get_attr_f32(&node, &["size", "tamanho"]);
                let bold = Self::get_attr_bool(&node, &["bold", "negrito"]);
                let color = Self::get_attr(&node, &["color", "cor"]);
                TipoNo::Text { content, size, bold, color }
            }
            "Button" | "button" | "Botao" | "botao" => {
                let text = Self::get_attr(&node, &["text", "texto", "content", "conteudo"]).unwrap_or_default();
                let on_click = Self::get_attr(&node, &["onClick", "on_click", "on-click", "aoClicar", "ao_clicar"]);
                let color = Self::get_attr(&node, &["color", "cor"]);
                TipoNo::Button { text, on_click, color }
            }
            "TextInput" | "textinput" | "Input" | "input" | "EntradaTexto" | "entrada_texto" => {
                let placeholder = Self::get_attr(&node, &["placeholder", "dica"]).unwrap_or_default();
                let value_var = Self::get_attr(&node, &["value", "valor"]).unwrap_or_default();
                let on_change = Self::get_attr(&node, &["onChange", "on_change", "on-change", "aoMudar", "ao_mudar"]).unwrap_or_default();
                TipoNo::TextInput { placeholder, value_var, on_change }
            }
            "Image" | "image" | "Imagem" | "imagem" => {
                let source = Self::get_attr(&node, &["source", "src", "origem", "caminho"]).unwrap_or_default();
                let clip = Self::get_attr(&node, &["clip", "corte"]);
                let clip_circle = clip.map(|s| s.eq_ignore_ascii_case("Circle") || s.eq_ignore_ascii_case("circle")).unwrap_or(false);
                TipoNo::Image { source, clip_circle }
            }
            "Include" | "include" | "Incluir" | "incluir" => {
                let src = Self::get_attr(&node, &["src", "fonte"]).unwrap_or_default();
                // Extract all other attributes as custom parameters
                let mut props = HashMap::new();
                for attr in node.attributes() {
                    let attr_name = attr.name();
                    if attr_name != "src" && attr_name != "fonte" {
                        props.insert(attr_name.to_string(), attr.value().to_string());
                    }
                }
                TipoNo::Include { src, props }
            }
            "ForEach" | "foreach" | "For" | "for" => {
                let items = Self::get_attr(&node, &["items", "itens", "source", "origem"]).unwrap_or_default();
                let var = Self::get_attr(&node, &["var", "variavel"]).unwrap_or_default();
                TipoNo::ForEach { items, var }
            }
            _ => {
                // Any unknown tag is treated as a reference to another component
                // by its own name (e.g. <PerfilCard nome="..." />).
                // All attributes are forwarded as props.
                let mut props = HashMap::new();
                for attr in node.attributes() {
                    props.insert(attr.name().to_string(), attr.value().to_string());
                }
                TipoNo::Component {
                    name: tag.to_string(),
                    props,
                }
            }
        };

        // Recursively parse children
        let mut filhos = Vec::new();
        for child in node.children() {
            if let Some(child_node) = Self::from_node(child) {
                filhos.push(child_node);
            }
        }

        Some(Self {
            tipo,
            filhos,
            largura,
            altura,
            padding,
            align_x,
            align_y,
            spacing,
            background,
            border_radius,
            border_width,
            border_color,
        })
    }

    /// Parse a full XML string into NoUI
    pub fn parse_xml(xml: &str) -> Result<Self, String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| e.to_string())?;
        Self::from_node(doc.root_element()).ok_or_else(|| "Failed to parse root element".to_string())
    }
}
