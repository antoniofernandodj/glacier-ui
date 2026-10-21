--!strict
-- Testa o estado e a navegação sem abrir janela: `glacier test`.
-- `buscar_ip` não entra: ele chama `fetch`, que suspende o handler.
--
-- Cada função `test_*` é um teste; o `---` acima dela é a documentação
-- (`glacier test -vv`, e sempre numa falha). `grupo__caso` agrupa por prefixo.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

--- O init publica os três serviços semeados, dois deles ativos.
function test_dados__o_init_publica_os_servicos_semeados()
    local tela = nova()
    assert_eq(tela:get("total"), "3")
    assert_eq(tela:get("ativos"), "2")
    assert_eq(#json.decode(tela:get("itens") or "[]"), 3)
end

--- Alternar o primeiro serviço muda a contagem de ativos, e cada alternância
--- mostra um toast.
function test_dados__alternar_o_primeiro_muda_a_contagem_de_ativos()
    local tela = nova()
    tela:call("alternar_primeiro")
    assert_eq(tela:get("ativos"), "1")
    tela:call("alternar_primeiro")
    assert_eq(tela:get("ativos"), "2")
    assert_eq(#tela:toasts(), 2)
end

--- A tela começa em home e vai e volta de sobre.
function test_navegacao__vai_e_volta_de_sobre()
    local tela = nova()
    assert_eq(tela:get("view"), "home")
    tela:call("ir_sobre")
    assert_eq(tela:get("view"), "sobre")
    tela:call("ir_home")
    assert_eq(tela:get("view"), "home")
end

--- Uma chave escrita errada num dos lados passa em todo teste de handler e deixa
--- a tela vazia; `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…)
--- com o ctx depois do init e a nomeia.
function test_navegacao__toda_chave_que_a_tela_le_e_publicada()
    assert_complete(nova())
end
