--!strict
-- Testa o contador da janela filha sem abrir janela: `glacier test`.
-- `open_window`, `broadcast` e a bandeja são do motor e precisam de janela de
-- verdade; o que se testa aqui é o script de `detalhe.luau`.

describe("janela de detalhe", function()
    test("herda a origem e conta a partir de zero", function()
        local tela = component("views/detalhe.gvb", { origem = "painel" })
        tela:call("init")
        expect(tela:get("contador")):toBe("0")
        -- o init não pisa no dado que quem abriu a janela passou
        expect(tela:get("origem")):toBe("painel")
    end)

    test("contar soma um a cada chamada", function()
        local tela = component("views/detalhe.gvb")
        tela:call("init")
        tela:call("contar")
        tela:call("contar")
        expect(tela:get("contador")):toBe("2")
        expect(tela:get("origem")):toBe("ninguém")
    end)

    test("toda chave que a tela lê é publicada", function()
        local tela = component("views/detalhe.gvb", { origem = "painel" })
        tela:call("init")
        expect(tela:missing()):toEqual({})
    end)
end)

describe("janela do painel", function()
    test("toda chave que a tela lê é publicada", function()
        local tela = component("views/painel.gvb")
        tela:call("init")
        expect(tela:missing()):toEqual({})
    end)
end)
