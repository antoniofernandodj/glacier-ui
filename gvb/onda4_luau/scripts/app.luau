--!strict
--
-- A Onda 4 inteira em Luau. O `main.rs` ao lado não tem `impl Component`: ele
-- registra o `.gv` e sai de cena.
--
-- O que o `examples/onda4` faz em Rust — `init`, `update` de cinco ações e o
-- recorte da página — mora aqui, e a comparação lado a lado é o ponto do
-- exemplo. Duas diferenças aparecem logo:
--
--  1. **Não há `self`.** O `Onda4` de Rust é um struct; aqui o estado que
--     sobrevive entre ações mora no `ctx` (o que a tela precisa ver) ou num
--     `local` do módulo (o que ela não precisa). `SERVICOS` abaixo é o segundo
--     caso: a lista completa nunca vai para o contexto, só a página recortada.
--  2. **`ctx` guarda texto.** Atribuir uma tabela a uma chave a codifica em
--     JSON — que é exatamente o formato que um `for-each` do motor lê, então
--     `ctx.servicos = { ... }` substitui o `serde_json::Value::Array` do lado
--     Rust sem cerimônia nenhuma.
--
-- O que NÃO muda entre as duas versões é o mais importante: nenhum dos sete
-- widgets pede uma linha de script. Paginação, seleção, seções, estrelas e
-- máscara gravam as chaves sozinhos. O que está aqui é só o que é do APP —
-- recortar a lista, e reagir aos botões da `<buttonbox>`.

-- A lista inteira, da qual a paginação recorta uma página. Num app de verdade
-- viria de um backend; aqui é literal para o exemplo não depender de rede.
local SERVICOS: { { id: string, label: string } } = {
    { id = "api", label = "API pública" },
    { id = "db", label = "Banco primário" },
    { id = "cache", label = "Redis" },
    { id = "fila", label = "Fila de eventos" },
    { id = "cdn", label = "CDN" },
    { id = "auth", label = "Autenticação" },
    { id = "mail", label = "Disparo de e-mail" },
    { id = "logs", label = "Coletor de logs" },
    { id = "cron", label = "Agendador" },
    { id = "busca", label = "Índice de busca" },
    { id = "relat", label = "Relatórios" },
    { id = "backup", label = "Backup noturno" },
}

-- Quantos itens cabem numa página. O `<pagination>` não sabe disto — ele conta
-- PÁGINAS, não itens; recortar a lista é do app, porque só o app sabe o que é
-- um item.
local POR_PAGINA = 4

-- Recorta a página visível e a grava na chave que o `<listview>` lê.
--
-- É a divisão de trabalho da paginação: o widget escreve o número da página, o
-- app decide o que aquele número significa.
local function recortar()
    local pagina = math.max(1, tonumber(ctx.pagina) or 1)
    local de = (pagina - 1) * POR_PAGINA

    local itens = {}
    for i = de + 1, math.min(de + POR_PAGINA, #SERVICOS) do
        local s = SERVICOS[i]
        table.insert(itens, { id = s.id, label = s.label, sub = "id: " .. s.id })
    end

    -- Uma tabela atribuída ao `ctx` vira JSON — o formato que o `for-each` do
    -- motor lê. É o `serde_json::Value::Array(...).to_string()` da versão Rust,
    -- sem a conversão explícita.
    ctx.servicos = itens
    ctx.faixa = string.format(
        "%d–%d de %d",
        de + 1,
        math.min(de + POR_PAGINA, #SERVICOS),
        #SERVICOS
    )
end

function init()
    ctx.abas = {
        { id = "listas", label = "Pagination + ListView" },
        { id = "secoes", label = "Accordion + ToolBox" },
        { id = "campos", label = "MaskedInput + Rating" },
    }
    ctx.aba = "listas"

    -- Paginação: o total de PÁGINAS, que é o que o widget conta.
    ctx.pagina = "1"
    ctx.total_paginas = tostring(math.ceil(#SERVICOS / POR_PAGINA))

    -- Seleção: uma chave para o modo simples, outra para o conjunto. O modo
    -- múltiplo guarda o conjunto inteiro numa string ("api,cache") — não um bit
    -- por item —, e é o `contains` do motor que testa o destaque.
    ctx.servico = "api"
    ctx.marcados = "api,cache"

    -- As seções abertas. Mesma ideia: um conjunto nomeado no accordion, um
    -- valor único no tool box.
    ctx.abertas = "rede"
    ctx.ferramenta = "medidas"

    -- Campos. O `<maskedinput>` guarda o valor CRU aqui e mostra o mascarado na
    -- tela, então começar vazio é o certo.
    ctx.cpf = ""
    ctx.telefone = ""
    ctx.placa = ""
    ctx.nota = "4"
    -- Chave própria: o rating de 10 estrelas tem outra escala, e uma nota
    -- de 7 numa régua de 5 não quer dizer nada.
    ctx.nota_ampla = "7"
    ctx.preco = "19.90"
    ctx.desconto = "0"

    -- As duas chaves das seções da aba 2, que os widgets escrevem sozinhos.
    ctx.usar_proxy = "false"
    ctx.avisar = "true"
    ctx.uso_disco = "41"

    ctx.status = "Pronto"
    recortar()
end

-- A paginação de cima tem `on_change`, então ela DELEGA: quem grava `pagina` é
-- esta função. É o mesmo contrato do `<textinput>` e do `<calendar>`, e aqui ele
-- paga o aluguel — o app precisa recortar a lista no mesmo passo em que a
-- página muda, e não há gancho melhor.
--
-- A paginação da barra de status não tem `on_change`: ela grava sozinha e nunca
-- chega aqui. Os dois modos ficam visíveis lado a lado na mesma tela.
function repaginar()
    if value ~= nil and value ~= "" then
        ctx.pagina = value
    end
    recortar()
    ctx.status = "Página " .. ctx.pagina
end

function limpar_marcados()
    ctx.marcados = ""
    ctx.status = "Seleção múltipla limpa"
end

function salvar()
    ctx.status = "Cadastro salvo"
end

function cancelar()
    ctx.status = "Cancelado"
end

function excluir()
    ctx.status = "Excluído — e o botão ficou longe dos outros"
end

function usar_proxy()
    ctx.usar_proxy = value
end

function avisar()
    ctx.avisar = value
end
