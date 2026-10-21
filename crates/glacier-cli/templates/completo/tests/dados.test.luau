--!strict
-- Testa o estado e a navegação sem abrir janela: `glacier test`.
-- `buscar_ip` não entra: ele chama `fetch`, que suspende o handler.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

describe("dados", function()
    test("o init publica os serviços semeados", function()
        local tela = nova()
        expect(tela:get("total")):toBe("3")
        expect(tela:get("ativos")):toBe("2")
        expect(#json.decode(tela:get("itens") or "[]")):toBe(3)
    end)

    test("alternar o primeiro serviço muda a contagem de ativos", function()
        local tela = nova()
        tela:call("alternar_primeiro")
        expect(tela:get("ativos")):toBe("1")
        tela:call("alternar_primeiro")
        expect(tela:get("ativos")):toBe("2")
        expect(#tela:toasts()):toBe(2)
    end)
end)

describe("navegação", function()
    test("começa em home e vai e volta de sobre", function()
        local tela = nova()
        expect(tela:get("view")):toBe("home")
        tela:call("ir_sobre")
        expect(tela:get("view")):toBe("sobre")
        tela:call("ir_home")
        expect(tela:get("view")):toBe("home")
    end)
end)
