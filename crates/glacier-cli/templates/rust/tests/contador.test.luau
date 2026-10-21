--!strict
-- O contador deste preset é um `Component` em Rust (src/contador.rs): a lógica
-- não está em Luau, então `glacier test` não a alcança. O que ele alcança é o
-- template: que carrega, e que lê só chaves que o `Component` publica. Para
-- testar a lógica, use `#[test]` em src/contador.rs.

describe("template do contador", function()
    test("toda chave que a tela lê é publicada pelo Component", function()
        -- O ctx inicial espelha o que `Contador::init` publica (ver `publicar`
        -- em src/contador.rs): um Luau não roda o Rust, então o espelho é manual.
        -- Mexeu em um dos lados, mexa no outro — este teste é o que avisa.
        local tela = component("views/contador.gvb", { contador = "0", passo = "1" })

        -- `passo_texto` é o campo de texto do passo: o Component só o publica
        -- quando o usuário digita (`mudar_passo`), e ele começa vazio de
        -- propósito. É a ÚNICA ausência esperada; qualquer outra é um erro.
        expect(tela:missing()):toEqual({ "passo_texto" })
    end)
end)
