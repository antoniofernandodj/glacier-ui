//! Estilos visuais **embutidos** — o análogo dos `QStyle` do Qt (`Fusion`,
//! `windowsvista`, …): um pacote pronto de paleta + regras de tag que muda a
//! cara do app inteiro com uma linha, sem o app escrever `.gss` nenhum.
//!
//! ```no_run
//! use glacier_ui::{style, GlacierDaemon};
//!
//! GlacierDaemon::new()
//!     .style(style::FUSION_DARK)   // todas as janelas nascem com o estilo
//!     .main(|motor| { /* registra componentes como sempre */ })
//!     .run()
//!     .unwrap();
//! ```
//!
//! Num app de janela única (sem daemon), chame [`crate::GlacierUI::set_style`]
//! direto no `init`.
//!
//! # O que um estilo carrega
//!
//! Um [`Style`] é dois ingredientes, ambos declarativos:
//!
//! 1. **Paleta** (`background`/`text`/`primary`/…): vira o `iced::Theme` do
//!    app — o mesmo slot de um `<link rel="theme">`. É ela que pinta o fundo
//!    da janela, os widgets que seguem o tema (checkbox, toggle, slider,
//!    inputs) e os defaults de texto.
//! 2. **GSS embutido** ([`Style::gss`]): regras de **tag** (`Button { }`,
//!    `Select { }`, com `:hover`/`:active`/…) instaladas como **underlay** —
//!    a folha de menor prioridade, abaixo de qualquer `.gss` do app. Qualquer
//!    classe, id ou atributo inline do app vence o estilo, exatamente como um
//!    stylesheet de app vence o QStyle no Qt.
//!
//! O GSS de cada estilo também publica a paleta como variáveis (`var(--primary)`,
//! `var(--surface)`, …), então o `.gss` do app pode se ancorar nas cores do
//! estilo ativo em vez de repetir hex.
//!
//! # Trocando em runtime
//!
//! Duas ações builtin (nenhum componente envolvido), no espírito do combo
//! "Style:" do Widget Gallery do Qt:
//!
//! - `on_click="style:<nome>"` num botão troca para o estilo `<nome>`;
//! - um `<Select>` com `onChange="style:set"` troca para o valor escolhido.
//!
//! O nome do estilo ativo fica no contexto sob [`CONTEXT_KEY`] (`glacier_style`)
//! — é o que um `<Select value="glacier_style">` usa para exibir a seleção.

use crate::error::Result;
use crate::stylesheet::StyleSheet;

/// Chave de contexto onde o motor publica o nome do estilo ativo (atualizada a
/// cada [`crate::GlacierUI::set_style`]). Um `<Select value="glacier_style">`
/// exibe — e, com `onChange="style:set"`, troca — o estilo em runtime.
pub const CONTEXT_KEY: &str = "glacier_style";

/// Chave sintética sob a qual o GSS do estilo é instalado no conjunto global de
/// folhas — fixa, para que trocar de estilo **substitua** a folha anterior em
/// vez de empilhar uma nova.
pub(crate) const UNDERLAY_KEY: &str = "builtin:style";

/// Um estilo visual completo: paleta (vira o `iced::Theme`) + GSS de regras de
/// tag (instalado como underlay). Ver o [módulo](self).
///
/// Os quatro embutidos são [`FROST`], [`FUSION`], [`FUSION_DARK`] e
/// [`PHANTOM`]; um app pode declarar o seu próprio como `const` (os campos são
/// todos `&'static str`) e passá-lo aos mesmos [`crate::GlacierUI::set_style`]
/// / [`crate::GlacierDaemon::style`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// Nome do estilo, em kebab-case (`"fusion-dark"`). É o token aceito pela
    /// ação `style:<nome>` e o valor publicado em [`CONTEXT_KEY`].
    pub name: &'static str,
    /// Fundo da janela (hex `#RRGGBB`).
    pub background: &'static str,
    /// Cor padrão de texto.
    pub text: &'static str,
    /// Cor de destaque (seleção, foco, botões primários do tema).
    pub primary: &'static str,
    /// Cor de sucesso.
    pub success: &'static str,
    /// Cor de aviso.
    pub warning: &'static str,
    /// Cor de perigo/erro.
    pub danger: &'static str,
    /// Regras GSS do estilo (seletores de tag + pseudo-estados + `:root`).
    /// Compilado no binário; um parse inválido aqui é bug da lib (coberto por
    /// teste), nunca do app.
    pub gss: &'static str,
}

impl Style {
    /// A paleta como um `iced::Theme` custom — o que o motor instala no slot de
    /// tema quando o estilo é aplicado. Erra apenas se algum hex da paleta for
    /// inválido (impossível nos builtin, testados; possível num `Style` do app).
    pub fn theme(&self) -> Result<iced::Theme> {
        let color = |field: &'static str, hex: &str| -> Result<iced::Color> {
            crate::widget::parse_hex_color(hex).ok_or_else(|| crate::error::GlacierError::Theme {
                path: format!("estilo '{}'", self.name),
                message: format!("'{hex}' não é um hex válido para '{field}'"),
            })
        };
        let palette = iced::theme::Palette {
            background: color("background", self.background)?,
            text: color("text", self.text)?,
            primary: color("primary", self.primary)?,
            success: color("success", self.success)?,
            warning: color("warning", self.warning)?,
            danger: color("danger", self.danger)?,
        };
        Ok(iced::Theme::custom(self.name.to_string(), palette))
    }

    /// O [`Style::gss`] parseado. Erros citam `builtin:<nome>` como arquivo.
    pub fn sheet(&self) -> Result<StyleSheet> {
        StyleSheet::parse_in(self.gss, Some(&format!("builtin:{}", self.name)), 1)
    }
}

/// Todos os estilos embutidos, na ordem de exibição sugerida (claros primeiro).
pub fn all() -> [&'static Style; 4] {
    [&FROST, &FUSION, &FUSION_DARK, &PHANTOM]
}

/// Busca um estilo embutido pelo [`Style::name`] (sem distinguir maiúsculas;
/// `_` e `-` são equivalentes). É a resolução usada pela ação `style:<nome>`.
pub fn by_name(name: &str) -> Option<&'static Style> {
    let wanted = name.trim().to_ascii_lowercase().replace('_', "-");
    all().into_iter().find(|s| s.name == wanted)
}

/// Claro nativo, no espírito do `windowsvista` do Qt: superfícies brancas,
/// bordas cinza discretas e azul de sistema no hover/foco.
pub const FROST: Style = Style {
    name: "frost",
    background: "#fbfbfb",
    text: "#1a1a1a",
    primary: "#0078d7",
    success: "#107c10",
    warning: "#9d5d00",
    danger: "#c42b1c",
    gss: r#"
:root {
    --background: #fbfbfb;
    --surface: #fdfdfd;
    --text: #1a1a1a;
    --primary: #0078d7;
    --border: #d0d0d0;
}
Button {
    color: #fdfdfd;
    text-color: #1a1a1a;
    border-width: 1;
    border-color: #d0d0d0;
    border-radius: 4;
    padding: 6 14;
}
Button:hover { background: #e0eef9; border-color: #0078d7; }
Button:active { background: #cce4f7; border-color: #005fb8; }
Button:disabled { background: #f5f5f5; text-color: #a3a3a3; border-color: #e0e0e0; }
TextInput:hover { border-color: #b8b8b8; }
TextInput:focus { border-color: #0078d7; }
Select {
    background: #fdfdfd;
    border-width: 1;
    border-color: #d0d0d0;
    border-radius: 4;
    padding: 6 10;
}
Select:hover { border-color: #0078d7; }
"#,
};

/// O Fusion claro do Qt: cinza neutro, botões quase chapados com borda visível
/// e raio pequeno, azul discreto como destaque.
pub const FUSION: Style = Style {
    name: "fusion",
    background: "#efefef",
    text: "#252525",
    primary: "#308cc6",
    success: "#2d7d46",
    warning: "#b07d00",
    danger: "#b4443c",
    gss: r#"
:root {
    --background: #efefef;
    --surface: #e8e8e8;
    --text: #252525;
    --primary: #308cc6;
    --border: #b4b4b4;
}
Button {
    color: #e8e8e8;
    text-color: #252525;
    border-width: 1;
    border-color: #b4b4b4;
    border-radius: 2;
    padding: 5 14;
}
Button:hover { background: #f2f2f2; }
Button:active { background: #d4d4d4; border-color: #9a9a9a; }
Button:disabled { background: #ececec; text-color: #9a9a9a; border-color: #cfcfcf; }
TextInput:hover { border-color: #9a9a9a; }
TextInput:focus { border-color: #308cc6; }
Select {
    background: #e8e8e8;
    border-width: 1;
    border-color: #b4b4b4;
    border-radius: 2;
    padding: 6 10;
}
Select:hover { border-color: #308cc6; }
"#,
};

/// O Fusion escuro (a paleta dark clássica do Qt): grafite frio, botões
/// levemente elevados e o azul `#2a82da` como destaque.
pub const FUSION_DARK: Style = Style {
    name: "fusion-dark",
    background: "#2b2b2b",
    text: "#d8d8d8",
    primary: "#2a82da",
    success: "#5cb85c",
    warning: "#d29922",
    danger: "#e06c75",
    gss: r#"
:root {
    --background: #2b2b2b;
    --surface: #3c3f41;
    --text: #d8d8d8;
    --primary: #2a82da;
    --border: #555555;
}
Button {
    color: #3c3f41;
    text-color: #d8d8d8;
    border-width: 1;
    border-color: #555555;
    border-radius: 2;
    padding: 5 14;
}
Button:hover { background: #46494c; border-color: #2a82da; }
Button:active { background: #2a82da; text-color: #ffffff; }
Button:disabled { background: #333537; text-color: #777777; border-color: #444444; }
TextInput:hover { border-color: #666666; }
TextInput:focus { border-color: #2a82da; }
Select {
    background: #3c3f41;
    border-width: 1;
    border-color: #555555;
    border-radius: 2;
    padding: 6 10;
}
Select:hover { border-color: #2a82da; }
"#,
};

/// Escuro grafite morno, no espírito do QtCurve/Phantom: superfícies quase
/// chapadas, contraste suave e um azul-aço dessaturado como destaque.
pub const PHANTOM: Style = Style {
    name: "phantom",
    background: "#3b3e40",
    text: "#ced2d6",
    primary: "#7d9fc4",
    success: "#8aac8b",
    warning: "#c7a35c",
    danger: "#c3626c",
    gss: r#"
:root {
    --background: #3b3e40;
    --surface: #46494c;
    --text: #ced2d6;
    --primary: #7d9fc4;
    --border: #2f3234;
}
Button {
    color: #46494c;
    text-color: #ced2d6;
    border-width: 1;
    border-color: #2f3234;
    border-radius: 3;
    padding: 5 14;
}
Button:hover { background: #505356; }
Button:active { background: #3a3d3f; border-color: #7d9fc4; }
Button:disabled { background: #404346; text-color: #7d8288; border-color: #35383a; }
TextInput:hover { border-color: #26282a; }
TextInput:focus { border-color: #7d9fc4; }
Select {
    background: #2e3133;
    border-width: 1;
    border-color: #26282a;
    border-radius: 3;
    padding: 6 10;
}
Select:hover { border-color: #7d9fc4; }
"#,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// O GSS de cada estilo embutido parseia — a garantia que permite ao motor
    /// tratar falha de parse como bug da lib (e nunca do app).
    #[test]
    fn gss_de_todos_os_builtin_parseia() {
        for style in all() {
            let sheet = style.sheet().unwrap_or_else(|e| {
                panic!("GSS do estilo '{}' não parseia: {e}", style.name)
            });
            // Todo estilo estiliza ao menos o Button (regra de tag).
            assert!(
                sheet.tags.contains_key("button"),
                "estilo '{}' não tem regra de tag para Button",
                style.name
            );
            // E publica a paleta como variáveis.
            assert!(
                sheet.variables.contains_key("--primary"),
                "estilo '{}' não publica --primary",
                style.name
            );
        }
    }

    /// A paleta de cada estilo embutido vira um `iced::Theme` sem erro.
    #[test]
    fn paleta_de_todos_os_builtin_e_valida() {
        for style in all() {
            style
                .theme()
                .unwrap_or_else(|e| panic!("paleta do estilo '{}': {e}", style.name));
        }
    }

    #[test]
    fn by_name_resolve_com_normalizacao() {
        assert_eq!(by_name("fusion").unwrap().name, "fusion");
        assert_eq!(by_name("FUSION-DARK").unwrap().name, "fusion-dark");
        assert_eq!(by_name("fusion_dark").unwrap().name, "fusion-dark");
        assert_eq!(by_name("  phantom  ").unwrap().name, "phantom");
        assert!(by_name("inexistente").is_none());
    }
}
