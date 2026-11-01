--!strict
-- Testa o DEPOIS da validação sem abrir janela: `glacier test`.
-- As regras (`rules="required|minlen:3"`) moram no form e o motor as aplica
-- ao enviar — isso só se vê rodando o app. Aqui ficam os três handlers.
--
-- Cada função `test_*` é um teste; o `---` acima dela é a documentação
-- (`glacier test -vv`, e sempre numa falha).

local function nova()
    local tela = component("views/app.gv")
    tela:call("init")
    return tela
end

--- O init semeia os valores iniciais do formulário.
function test_o_init_semeia_os_valores_iniciais()
    local tela = nova()
    assert_eq(tela:get("f_nome"), "")
    assert_eq(tela:get("f_idade"), "25")
    assert_eq(tela:get("f_uf"), "SP")
end

--- Salvar registra o resultado no status e mostra um toast de sucesso.
function test_salvar_registra_o_resultado_e_avisa()
    local tela = nova()
    tela:set("f_nome", "Ana")
    tela:call("salvar")
    assert_eq(tela:get("f_status"), "salvo: Ana · SP")
    assert_eq(tela:toasts()[1].kind, "success")
end

--- Apontar conta os campos com erro e limpa o status.
function test_apontar_conta_os_campos_com_erro_e_limpa_o_status()
    local tela = nova()
    tela:call("apontar", '[{"campo":"f_nome","msg":"x"},{"campo":"f_cpf","msg":"y"}]')
    assert_eq(tela:get("f_status"), "")
    assert_eq(tela:toasts()[1].message, "Corrija 2 campo(s)")
end

--- Limpar devolve todos os campos ao estado inicial.
function test_limpar_devolve_tudo_ao_estado_inicial()
    local tela = nova()
    tela:set("f_nome", "Ana")
    tela:set("f_uf", "RJ")
    tela:call("limpar")
    assert_eq(tela:get("f_nome"), "")
    assert_eq(tela:get("f_uf"), "SP")
end

--- Uma chave escrita errada num dos lados passa em todo teste de handler e deixa
--- a tela vazia; `missing()` cruza o que o .gv lê ({x}, @x, :value, :items…)
--- com o ctx depois do init e a nomeia.
--- Os `erro_<campo>` do form não entram: o motor os publica ao enviar.
function test_toda_chave_que_a_tela_le_e_publicada()
    assert_complete(nova())
end
