//! `.gva` — o markup de atributos: a grafia XML de uma tela do Glacier.
//!
//! É o formato de que o motor sempre falou, agora com nome. O `.gv` é o nome
//! **legado** dele: um arquivo `.gv` é tratado exatamente como um `.gva`, e é
//! por isso que nada aqui olha a extensão para decidir *como* ler — só para
//! decidir se há algo a fazer antes (ver [`crate::gvb`], que dessugara o seu
//! formato para o XML que este módulo consome).
//!
//! Este módulo é o ponto de entrada do que vem **depois** de a leitura entregar
//! XML: tirar o `<script>`, normalizar as diretivas nuas (`else`) e parsear. O
//! parser em si (`UiNode::parse_xml_with_source`) continua em [`crate::parser`].

use crate::error::Result;
use crate::eval;
use crate::parser::UiNode;

/// Extensões que são `.gva`, na ordem de preferência. `gv` é o nome legado.
pub const EXTENSOES: [&str; 2] = ["gva", "gv"];

/// Se `path` é um template `.gva` (ou `.gv`, o legado).
pub fn eh_gva(path: &str) -> bool {
    let ext = std::path::Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase());
    ext.is_some_and(|e| EXTENSOES.contains(&e.as_str()))
}

/// O markup de um template em `UiNode`, mais o `<script>` que ele carregava.
///
/// `content` já é XML — para um `.gvb`, a leitura (ver
/// [`crate::asset_source::read_markup`]) o dessugarou antes de chegar aqui.
pub(crate) fn parse_markup(path: Option<&str>, content: &str) -> Result<(UiNode, Option<String>)> {
    let (markup, script) = eval::strip_script(content);
    let markup = eval::normalize_bare_directives(&markup);
    // `content` (e não `markup`) como fonte dos trechos: o erro deve mostrar a
    // linha que o autor escreveu, não a que o pré-processamento produziu.
    Ok((
        UiNode::parse_xml_with_source(&markup, content, path)?,
        script,
    ))
}
