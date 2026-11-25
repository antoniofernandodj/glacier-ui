//! Fase 12 do `docs/PLANO_LINGUAGEM_GV_GSS.md`, num binário próprio porque o
//! modo estrito é uma chave global do processo.

use glacier_ui::stylesheet::{parse_gss, set_estrito};

/// Fora do modo estrito, propriedade desconhecida é aviso; dentro, erro com a
/// posição e a sugestão.
#[test]
fn modo_estrito_faz_propriedade_desconhecida_virar_erro() {
    set_estrito(false);
    assert!(parse_gss(".a { colr: #fff; }").is_ok());
    set_estrito(true);
    let e = parse_gss(".a {\n  colr: #fff;\n}").unwrap_err().to_string();
    set_estrito(false);
    assert!(e.contains(":2"), "devia citar a linha 2: {e}");
    assert!(e.contains("você quis dizer 'color'"), "{e}");
}
