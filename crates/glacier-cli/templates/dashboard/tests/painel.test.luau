--!strict
-- Testa o que o painel publica no `ctx` sem abrir janela: `glacier test`.
-- Os valores são aleatórios, então o teste olha a FORMA: quantas séries, quantos
-- pontos, quais chaves existem. Uma chave que nenhum gráfico recebe aparece
-- vazia na tela, sem erro nenhum — é esse buraco que isto vigia.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

describe("painel", function()
    test("o init publica todas as chaves que a tela lê", function()
        local tela = nova()
        for _, chave in { "kpi_total", "kpi_taxa", "kpi_lat", "g_cpu", "g_disco", "relogio", "barras", "fatias" } do
            expect(tela:get(chave)).never:toBeNil()
        end
    end)

    test("a série de requisições tem duas linhas de 24 pontos", function()
        local series = json.decode(nova():get("serie_req") or "[]")
        expect(#series):toBe(2)
        expect(#series[1].points):toBe(24)
        expect(series[1].name):toBe("API")
    end)

    test("as faixas do gauge cobrem até 100", function()
        local faixas = json.decode(nova():get("g_bands") or "[]")
        expect(#faixas):toBe(3)
        expect(faixas[#faixas].to):toBe(100)
    end)
end)
