--!strict
-- Testa o DEPOIS da validação sem abrir janela: `glacier test`.
-- As regras (`rules="required|minlen:3"`) moram no <form> e o motor as aplica
-- ao enviar — isso só se vê rodando o app. Aqui ficam os três handlers.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

describe("cadastro", function()
    test("o init semeia os valores iniciais", function()
        local tela = nova()
        expect(tela:get("f_nome")):toBe("")
        expect(tela:get("f_idade")):toBe("25")
        expect(tela:get("f_uf")):toBe("SP")
    end)

    test("salvar registra o resultado e avisa", function()
        local tela = nova()
        tela:set("f_nome", "Ana")
        tela:call("salvar")
        expect(tela:get("f_status")):toBe("salvo: Ana · SP")
        expect(tela:toasts()[1].kind):toBe("success")
    end)

    test("apontar conta os campos com erro e limpa o status", function()
        local tela = nova()
        tela:call("apontar", '[{"campo":"f_nome","msg":"x"},{"campo":"f_cpf","msg":"y"}]')
        expect(tela:get("f_status")):toBe("")
        expect(tela:toasts()[1].message):toBe("Corrija 2 campo(s)")
    end)

    test("limpar devolve tudo ao estado inicial", function()
        local tela = nova()
        tela:set("f_nome", "Ana")
        tela:set("f_uf", "RJ")
        tela:call("limpar")
        expect(tela:get("f_nome")):toBe("")
        expect(tela:get("f_uf")):toBe("SP")
    end)

    test("toda chave que a tela lê é publicada", function()
        -- `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…) com o ctx
        -- depois do init. Uma chave escrita errada num dos lados passa em todo
        -- teste de handler e deixa a tela vazia; aqui ela aparece pelo nome.
        -- Os `erro_<campo>` do <form> não entram: o motor os publica ao enviar.
        expect(nova():missing()):toEqual({})
    end)
end)
