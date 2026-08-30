//! Conteúdo embutido no binário pelo `build.rs`: os presets de projeto e as
//! árvores das extensões de VS Code.

/// Uma extensão de VS Code embutida, com os arquivos que vão para dentro do
/// `.vsix` (caminhos relativos à raiz da extensão, sempre com `/`).
pub struct Extensao {
    /// Nome do diretório de onde ela veio (`vscode-gv`, `vscode`) — só para
    /// mensagens; a identidade real (`publisher.name`) sai do `package.json`.
    pub origem: &'static str,
    pub arquivos: &'static [(&'static str, &'static [u8])],
}

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

/// Preset implícito, mesclado em todos os outros: o que vale para qualquer
/// projeto glacier (o `.gitignore`, o `.luaurc` e os tipos do luau-lsp) e que
/// não deveria estar copiado quatro vezes dentro de `templates/`.
const COMUM: &str = "_comum";

/// Arquivos de um preset, com o caminho já relativo à raiz do projeto gerado
/// (`templates/completo/views/app.gv` → `views/app.gv`).
///
/// O preset vem por cima de [`COMUM`]: um arquivo homônimo nos dois lados é o
/// do preset que vale, para que um preset possa especializar o comum sem
/// precisar sair dele.
pub fn arquivos_do_preset(preset: &str) -> Vec<(&'static str, &'static [u8])> {
    if preset == COMUM {
        return Vec::new();
    }

    let mut arquivos = arquivos_sob(preset);
    for (caminho, bytes) in arquivos_sob(COMUM) {
        if !arquivos.iter().any(|(existente, _)| *existente == caminho) {
            arquivos.push((caminho, bytes));
        }
    }
    arquivos.sort_by_key(|(caminho, _)| *caminho);
    arquivos
}

fn arquivos_sob(dir: &str) -> Vec<(&'static str, &'static [u8])> {
    let prefixo = format!("{dir}/");
    TEMPLATES
        .iter()
        .filter_map(|(caminho, bytes)| Some((caminho.strip_prefix(&prefixo)?, *bytes)))
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    /// O `_comum` não é um preset — ele é o que os presets herdam.
    #[test]
    fn comum_nao_e_oferecido_como_preset() {
        assert!(arquivos_do_preset(COMUM).is_empty());
        assert!(crate::scaffold::PRESETS.iter().all(|p| p.id != COMUM));
    }

    /// Todo preset herda o `.gitignore`, o `.luaurc` e os tipos do luau-lsp.
    #[test]
    fn presets_herdam_os_arquivos_comuns() {
        for p in crate::scaffold::PRESETS {
            let arquivos = arquivos_do_preset(p.id);
            for esperado in ["gitignore", ".luaurc", "views/scripts/glacier.d.luau"] {
                assert!(
                    arquivos.iter().any(|(c, _)| *c == esperado),
                    "preset '{}' não herdou '{esperado}'",
                    p.id
                );
            }
        }
    }
}
