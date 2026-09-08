//! A extensão do VS Code manda o Ctrl+Clique de uma tag para um cabeçalho do
//! `references/glacier-view.md`. Quando o cabeçalho não existe, ela cai na
//! linha 1 do arquivo — sem erro, sem aviso, e quem clicou acha que a tag não
//! tem documentação.
//!
//! Este teste é o que impede a documentação de ficar para trás: uma tag nova em
//! `NATIVE_TAGS` sem seção no doc falha aqui, e não depois, na mão de quem usa.

use std::collections::BTreeMap;

const EXT: &str = "editors/vscode-gv/extension.js";
const DOC: &str = "editors/vscode-gv/references/glacier-view.md";

/// As tags que a extensão conhece: `Canônico -> [grafias aceitas]`, lidas do
/// bloco `NATIVE_TAGS` do `extension.js`.
///
/// Ler o JS com um parser de verdade seria trazer uma dependência para
/// conferir um objeto literal; o bloco tem forma fixa (`  Nome: ["a", "b"],`) e
/// é gerado à mão, então um recorte por linha basta — e se a forma mudar, o
/// teste falha ruidosamente em vez de passar vazio (ver a asserção de tamanho).
fn tags_da_extensao(js: &str) -> BTreeMap<String, Vec<String>> {
    let ini = js
        .find("const NATIVE_TAGS = {")
        .expect("bloco NATIVE_TAGS sumiu do extension.js");
    let fim = js[ini..].find("\n};").expect("bloco NATIVE_TAGS sem fim") + ini;

    let mut fora = BTreeMap::new();
    for linha in js[ini..fim].lines() {
        // `  Nome: ["a", "b"],` — dois espaços de recuo, o nome, dois pontos.
        let Some(resto) = linha.strip_prefix("  ") else {
            continue;
        };
        let Some((nome, cauda)) = resto.split_once(':') else {
            continue;
        };
        let nome = nome.trim();
        if nome.is_empty() || !nome.chars().all(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let grafias: Vec<String> = cauda
            .split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect();
        fora.insert(nome.to_string(), grafias);
    }
    fora
}

/// O mesmo casamento que a `resolveNative` do `extension.js` faz: o cabeçalho
/// que nomeia a grafia, ou um cabeçalho que a mencione entre `<>` (o caso de
/// `<Else>`, documentado sob o título do `<If>`).
fn tem_secao(doc_min: &str, grafia: &str) -> bool {
    let g = grafia.to_lowercase();
    doc_min.lines().filter(|l| l.starts_with('#')).any(|l| {
        let titulo = l.trim_start_matches('#').trim_start();
        let nomeia = titulo
            .trim_start_matches('`')
            .trim_start_matches('<')
            .strip_prefix(&g)
            .is_some_and(|r| !r.starts_with(|c: char| c.is_alphanumeric() || c == '_'));
        nomeia || titulo.contains(&format!("<{g}>"))
    })
}

#[test]
fn toda_tag_da_extensao_tem_secao_no_doc() {
    let js = std::fs::read_to_string(EXT).expect(EXT);
    let doc = std::fs::read_to_string(DOC).expect(DOC);
    let doc_min = doc.to_lowercase();

    let tags = tags_da_extensao(&js);
    assert!(
        tags.len() > 80,
        "o recorte do NATIVE_TAGS achou só {} tags — a forma do bloco mudou, e \
         um teste que não lê nada passa por engano",
        tags.len()
    );

    let mut sem_doc = Vec::new();
    for (canonico, grafias) in &tags {
        // A extensão tenta o canônico e depois cada apelido — é o que faz
        // `<dialog>` (canônico `DialogDef`) achar a seção que o documenta pela
        // grafia que as pessoas escrevem.
        let achou = std::iter::once(canonico)
            .chain(grafias.iter())
            .any(|g| tem_secao(&doc_min, g));
        if !achou {
            sem_doc.push(canonico.clone());
        }
    }

    assert!(
        sem_doc.is_empty(),
        "estas tags não têm seção no {DOC}, então o Ctrl+Clique nelas cai na \
         linha 1: {sem_doc:?}"
    );
}
