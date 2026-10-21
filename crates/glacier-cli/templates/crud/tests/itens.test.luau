--!strict
-- Testa a vitrine e os handlers da lista sem abrir janela: `glacier test`.
-- `novo`, `editar` e `excluir` usam prompt{}/confirm{}, que suspendem o
-- handler; aqui só se testa o que roda até a primeira suspensão.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

local function nomes(tela): { string }
    local lista = {}
    for _, linha in json.decode(tela:get("linhas") or "[]") do
        table.insert(lista, linha.nome)
    end
    return lista
end

describe("lista de serviços", function()
    test("o init publica os três serviços semeados", function()
        local tela = nova()
        expect(tela:get("total")):toBe("3")
        expect(#nomes(tela)):toBe(3)
    end)

    test("filtrar deixa só os nomes que contêm o trecho", function()
        local tela = nova()
        tela:call("filtrar", "API")
        expect(nomes(tela)):toEqual({ "api-gateway" })
        -- o filtro esconde linhas, não apaga serviços
        expect(tela:get("total")):toBe("3")

        tela:call("filtrar", "")
        expect(#nomes(tela)):toBe(3)
    end)

    test("editar e excluir sem seleção só avisam", function()
        local tela = nova()
        tela:call("editar")
        tela:call("excluir")
        local avisos = tela:toasts()
        expect(#avisos):toBe(2)
        expect(avisos[1].kind):toBe("warning")
    end)

    test("toda chave que a tela lê é publicada", function()
        -- `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…) com o ctx
        -- depois do init. Uma chave escrita errada num dos lados passa em todo
        -- teste de handler e deixa a tela vazia; aqui ela aparece pelo nome.
        expect(nova():missing()):toEqual({})
    end)
end)
