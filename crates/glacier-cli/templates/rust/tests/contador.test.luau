--!strict
-- O contador deste preset é um `Component` em Rust (src/contador.rs): a lógica
-- não está em Luau, então `glacier test` não a alcança. O que ele alcança é o
-- template: que carrega, e que lê só chaves que o `Component` publica. Para
-- testar a lógica, use `#[test]` em src/contador.rs.
--
-- Cada função `test_*` é um teste; o `---` acima dela é a documentação
-- (`glacier test -vv`, e sempre numa falha).

--- Toda chave que a tela lê é publicada pelo Component.
---
--- O ctx inicial espelha o que `Contador::init` publica (ver `publicar` em
--- src/contador.rs): um Luau não roda o Rust, então o espelho é manual. Mexeu
--- em um dos lados, mexa no outro — este teste é o que avisa.
---
--- `passo_texto` é o campo de texto do passo: o Component só o publica quando o
--- usuário digita (`mudar_passo`), e ele começa vazio de propósito. É a ÚNICA
--- ausência esperada; qualquer outra é um erro.
function test_toda_chave_que_a_tela_le_e_publicada_pelo_component()
    local tela = component("views/contador.gvb", { contador = "0", passo = "1" })
    assert_complete(tela, { "passo_texto" })
end
