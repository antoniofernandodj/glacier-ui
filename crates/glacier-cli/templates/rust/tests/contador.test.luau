--!strict
-- O contador deste preset é um `Component` em Rust (src/contador.rs): a lógica
-- não está em Luau, então `glacier test` não a alcança. O que ele alcança é o
-- template — e um `.gvb` com erro de sintaxe é o erro mais comum de quem edita
-- a tela. Para testar a lógica, use `#[test]` em src/contador.rs.

describe("template do contador", function()
    test("carrega sem erro", function()
        local tela = component("views/contador.gvb")
        expect(tela:data()).never:toBeNil()
    end)
end)
