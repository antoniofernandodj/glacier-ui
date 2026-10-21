--!strict
-- Testa o que o painel publica no `ctx` sem abrir janela: `glacier test`.
-- Os valores são aleatórios, então o teste olha a FORMA: quantas séries, quantos
-- pontos, quais chaves existem. Uma chave que nenhum gráfico recebe aparece
-- vazia na tela, sem erro nenhum — é esse buraco que isto vigia.
--
-- Cada função `test_*` é um teste; o `---` acima dela é a documentação
-- (`glacier test -vv`, e sempre numa falha).

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

--- O init publica todas as chaves que a tela lê.
function test_o_init_publica_todas_as_chaves_que_a_tela_le()
    local tela = nova()
    each({
        { "kpi_total" },
        { "kpi_taxa" },
        { "kpi_lat" },
        { "g_cpu" },
        { "g_disco" },
        { "relogio" },
        { "barras" },
        { "fatias" },
    }, function(chave)
        assert_not_nil(tela:get(chave))
    end)
end

--- A série de requisições tem duas linhas de 24 pontos.
function test_a_serie_de_requisicoes_tem_duas_linhas_de_24_pontos()
    local series = json.decode(nova():get("serie_req") or "[]")
    assert_eq(#series, 2)
    assert_eq(#series[1].points, 24)
    assert_eq(series[1].name, "API")
end

--- As faixas do gauge são três e a última vai até 100.
function test_as_faixas_do_gauge_cobrem_ate_100()
    local faixas = json.decode(nova():get("g_bands") or "[]")
    assert_eq(#faixas, 3)
    assert_eq(faixas[#faixas].to, 100)
end

--- Uma chave escrita errada num dos lados passa em todo teste de handler e deixa
--- a tela vazia; `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…)
--- com o ctx depois do init e a nomeia.
function test_toda_chave_que_a_tela_le_e_publicada()
    assert_complete(nova())
end
