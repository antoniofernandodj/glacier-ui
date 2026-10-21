--!strict
-- Testa a vitrine e os handlers da lista sem abrir janela: `glacier test`.
-- `novo`, `editar` e `excluir` usam prompt{}/confirm{}, que suspendem o
-- handler; aqui só se testa o que roda até a primeira suspensão.
--
-- Cada função `test_*` é um teste; o `---` acima dela é a documentação
-- (`glacier test -vv`, e sempre numa falha).

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

--- O init publica os três serviços semeados.
function test_o_init_publica_os_tres_servicos_semeados()
    local tela = nova()
    assert_eq(tela:get("total"), "3")
    assert_eq(#nomes(tela), 3)
end

--- Filtrar por "API" deixa só o serviço que contém o trecho.
--- O total continua 3: o filtro esconde linhas, não apaga serviços.
function test_filtrar_deixa_so_os_nomes_que_contem_o_trecho()
    local tela = nova()
    tela:call("filtrar", "API")
    assert_eq(nomes(tela), { "api-gateway" })
    assert_eq(tela:get("total"), "3")

    tela:call("filtrar", "")
    assert_eq(#nomes(tela), 3)
end

--- Sem linha selecionada, editar e excluir só mostram um toast de aviso.
function test_editar_e_excluir_sem_selecao_so_avisam()
    local tela = nova()
    tela:call("editar")
    tela:call("excluir")
    local avisos = tela:toasts()
    assert_eq(#avisos, 2)
    assert_eq(avisos[1].kind, "warning")
end

--- Uma chave escrita errada num dos lados passa em todo teste de handler e deixa
--- a tela vazia; `missing()` cruza o que o .gvb lê ({x}, @x, :value, :items…)
--- com o ctx depois do init e a nomeia.
function test_toda_chave_que_a_tela_le_e_publicada()
    assert_complete(nova())
end
