--!strict
-- Testa o comportamento do contador sem abrir janela: `glacier test`.
-- Cobre o SCRIPT (o que cada botão faz com o `ctx`), não o layout: um widget
-- ligado à chave errada passaria aqui. Para isso, `make run` e olhar a tela.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

describe("contador", function()
    test("começa em zero", function()
        expect(nova():get("contador")):toBe("0")
    end)

    test("soma e subtrai", function()
        local tela = nova()
        tela:call("somar")
        tela:call("somar")
        tela:call("subtrair")
        expect(tela:get("contador")):toBe("1")
    end)

    test("zerar volta a zero e avisa", function()
        local tela = nova()
        tela:call("somar")
        tela:call("zerar")
        expect(tela:get("contador")):toBe("0")
        expect(tela:toasts()[1].message):toBe("Contador zerado")
    end)
end)
