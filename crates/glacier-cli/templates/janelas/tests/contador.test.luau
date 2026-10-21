--!strict
-- Testa o contador da janela filha sem abrir janela: `glacier test`.
-- `open_window`, `broadcast` e a bandeja são do motor e precisam de janela de
-- verdade; o que se testa aqui é o script de `detalhe.luau`.
--
-- Cada função `test_*` é um teste; o `---` acima dela é a documentação
-- (`glacier test -vv`, e sempre numa falha).

--- A janela de detalhe herda a origem e conta a partir de zero.
--- O init não pisa no dado que quem abriu a janela passou.
function test_detalhe__herda_a_origem_e_conta_a_partir_de_zero()
    local tela = component("views/detalhe.gvb", { origem = "painel" })
    tela:call("init")
    assert_eq(tela:get("contador"), "0")
    assert_eq(tela:get("origem"), "painel")
end

--- `contar` soma um a cada chamada; sem origem, vale "ninguém".
function test_detalhe__contar_soma_um_a_cada_chamada()
    local tela = component("views/detalhe.gvb")
    tela:call("init")
    tela:call("contar")
    tela:call("contar")
    assert_eq(tela:get("contador"), "2")
    assert_eq(tela:get("origem"), "ninguém")
end

--- Toda chave que a tela de detalhe lê é publicada.
function test_detalhe__toda_chave_que_a_tela_le_e_publicada()
    local tela = component("views/detalhe.gvb", { origem = "painel" })
    tela:call("init")
    assert_complete(tela)
end

--- Toda chave que a tela do painel lê é publicada.
function test_painel__toda_chave_que_a_tela_le_e_publicada()
    local tela = component("views/painel.gvb")
    tela:call("init")
    assert_complete(tela)
end
