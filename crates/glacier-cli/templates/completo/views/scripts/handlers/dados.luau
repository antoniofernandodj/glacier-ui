--!strict
--!nolint FunctionUnused
-- handlers/dados.luau — os dados da tela inicial.
--
-- Mostra os três caminhos que um app costuma precisar: estado local publicado
-- no contexto, uma mutação disparada por clique e uma chamada de rede que não
-- trava a UI.

local State = require("../state")

-- Semente local: o app abre com conteúdo mesmo sem rede.
local SEMENTE: { State.Item } = {
    { id = "1", nome = "servidor de borda", ativo = true },
    { id = "2", nome = "fila de trabalhos", ativo = true },
    { id = "3", nome = "réplica fria", ativo = false },
}

local Dados = {}

-- Copia o estado para o `ctx`, que é o que o markup lê. Um array vai como JSON:
-- é a forma que o `for-each` do template entende.
function Dados.publicar(): ()
    if #State.itens == 0 then
        State.itens = SEMENTE
    end

    local linhas = {}
    local ativos = 0
    for _, item in State.itens do
        if item.ativo then
            ativos += 1
        end
        table.insert(linhas, {
            id = item.id,
            nome = item.nome,
            -- O contexto coage tudo para string, e o markup compara com "1":
            -- converter aqui deixa a comparação explícita no template.
            ativo = if item.ativo then "1" else "0",
        })
    end

    ctx.itens = json.encode(json.array(linhas))
    ctx.total = tostring(#State.itens)
    ctx.ativos = tostring(ativos)
end

-- Mutação simples vinda de um clique: muda o estado e republica.
function alternar_primeiro(): ()
    local primeiro = State.itens[1]
    if primeiro == nil then
        return
    end
    primeiro.ativo = not primeiro.ativo
    Dados.publicar()
    toast({ message = `{primeiro.nome}: {if primeiro.ativo then "ativo" else "parado"}`, kind = "info" })
end

-- Rede. `fetch` SUSPENDE esta corrotina e retoma quando a resposta chega: a
-- janela continua respondendo, e o código segue linear, com cara de await.
function buscar_ip(): ()
    if State.carregando then
        return
    end
    State.carregando = true
    ctx.status = "buscando..."   -- já aparece na tela, antes da suspensão

    local res = fetch("https://api.ipify.org?format=json")
    State.carregando = false

    if not res.ok then
        ctx.status = "sem resposta"
        toast({ message = `Falhou: {res.error}`, kind = "error" })
        return
    end

    local corpo = json.decode(res.body)
    ctx.ip = tostring(corpo.ip)
    ctx.status = `ok ({res.status})`
    toast({ message = "IP atualizado", kind = "success" })
end

return Dados
