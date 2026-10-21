--!strict
-- Testa a navegação e os handlers genéricos do catálogo sem abrir janela:
-- `glacier test`. O que mais importa aqui é o `init`: cada demo lê uma chave, e
-- uma chave sem semente faz o widget aparecer VAZIO, sem aviso.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

describe("catálogo", function()
    test("abre em botões, com a gaveta aberta", function()
        local tela = nova()
        expect(tela:get("view")):toBe("botoes")
        expect(tela:get("menu")):toBe("true")
    end)

    test("o init semeia as chaves dos demos", function()
        local tela = nova()
        for _, chave in { "plano", "cb_cache", "ti_nome", "sb_qtd", "sl_volume", "ga_bands" } do
            expect(tela:get(chave)).never:toBeNil()
        end
    end)

    test("a sidebar troca a tela", function()
        local tela = nova()
        tela:call("ir", "texto")
        expect(tela:get("view")):toBe("texto")
    end)

    test("def grava qualquer chave, inclusive com ':' no valor", function()
        local tela = nova()
        tela:call("def", "plano:pro:anual")
        expect(tela:get("plano")):toBe("pro:anual")
    end)

    test("dizer e dlg_toast viram toasts", function()
        local tela = nova()
        tela:call("dizer", "oi")
        tela:call("dlg_toast", "success")
        local avisos = tela:toasts()
        expect(avisos[1].message):toBe("oi")
        expect(avisos[2].kind):toBe("success")
    end)

    test("toda chave que a tela lê é publicada", function()
        -- `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…) com o ctx
        -- depois do init. Uma chave escrita errada num dos lados passa em todo
        -- teste de handler e deixa a tela vazia; aqui ela aparece pelo nome.
        expect(nova():missing()):toEqual({})
    end)
end)
