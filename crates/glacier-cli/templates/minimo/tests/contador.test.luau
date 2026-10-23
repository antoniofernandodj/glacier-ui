--!strict
-- Testa o comportamento do contador sem abrir janela: `glacier test`.
-- Cobre o SCRIPT (o que cada botão faz com o `ctx`), não o layout: um widget
-- ligado à chave errada passaria aqui. Para isso, `make run` e olhar a tela.
--
-- Cada função `test_*` é um teste. O comentário `---` acima dela é a
-- documentação: aparece com `glacier test -vv` e sempre que o teste falha.

local function nova()
    local tela = component("views/app.gvb")
    tela:call("init")
    return tela
end

--- O contador nasce em zero.
function test_comeca_em_zero()
    assert_eq(nova():get("contador"), "0")
end

--- Somar duas vezes e subtrair uma deixa 1.
function test_soma_e_subtrai()
    local tela = nova()
    tela:call("somar")
    tela:call("somar")
    tela:call("subtrair")
    assert_eq(tela:get("contador"), "1")
end

--- Zerar volta a zero e mostra um toast.
function test_zerar_volta_a_zero_e_avisa()
    local tela = nova()
    tela:call("somar")
    tela:call("zerar")
    assert_eq(tela:get("contador"), "0")
    assert_eq(tela:toasts()[1].message, "Contador zerado")
end

--- Uma chave escrita errada num dos lados passa em todo teste de handler e deixa
--- a tela vazia; `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…)
--- com o ctx depois do init e a nomeia.
function test_toda_chave_que_a_tela_le_e_publicada()
    assert_complete(nova())
end

--- O contador é um `number` para o script, de um turno para o outro; o markup
--- lê o texto. (`tela:get` devolve o texto, `tela:value` o tipo.)
function test_o_contador_e_um_number()
    local tela = nova()
    tela:call("somar")
    assert_eq(tela:value("contador"), 1)
    assert_eq(tela:get("contador"), "1")
end
