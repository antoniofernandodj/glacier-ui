--!strict
-- Testa a navegação e os handlers genéricos do catálogo sem abrir janela:
-- `glacier test`. O que mais importa aqui é o `init`: cada demo lê uma chave, e
-- uma chave sem semente faz o widget aparecer VAZIO, sem aviso.
--
-- Cada função `test_*` é um teste; o `---` acima dela é a documentação
-- (`glacier test -vv`, e sempre numa falha).

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

--- O catálogo abre na página de botões, com a gaveta aberta.
function test_abre_em_botoes_com_a_gaveta_aberta()
    local tela = nova()
    assert_eq(tela:get("view"), "botoes")
    assert_eq(tela:get("menu"), "true")
end

--- Cada demo lê uma chave; sem semente no init o widget aparece vazio.
function test_o_init_semeia_as_chaves_dos_demos()
    local tela = nova()
    each({ { "plano" }, { "cb_cache" }, { "ti_nome" }, { "sb_qtd" }, { "sl_volume" }, { "ga_bands" } }, function(chave)
        assert_not_nil(tela:get(chave))
    end)
end

--- A sidebar troca a tela com `ir`.
function test_a_sidebar_troca_a_tela()
    local tela = nova()
    tela:call("ir", "texto")
    assert_eq(tela:get("view"), "texto")
end

--- `def` grava qualquer chave, e o valor pode ter ':' dentro.
function test_def_grava_qualquer_chave_inclusive_com_dois_pontos()
    local tela = nova()
    tela:call("def", "plano:pro:anual")
    assert_eq(tela:get("plano"), "pro:anual")
end

--- `dizer` e `dlg_toast` viram toasts, na ordem em que foram chamados.
function test_dizer_e_dlg_toast_viram_toasts()
    local tela = nova()
    tela:call("dizer", "oi")
    tela:call("dlg_toast", "success")
    local avisos = tela:toasts()
    assert_eq(avisos[1].message, "oi")
    assert_eq(avisos[2].kind, "success")
end

--- Uma chave escrita errada num dos lados passa em todo teste de handler e deixa
--- a tela vazia; `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…)
--- com o ctx depois do init e a nomeia.
function test_toda_chave_que_a_tela_le_e_publicada()
    assert_complete(nova())
end
