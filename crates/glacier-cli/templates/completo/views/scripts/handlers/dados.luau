--!strict
--!nolint FunctionUnused

local State = require("../state")

local SEMENTE: { State.Item } = {
    { id = "1", nome = "servidor de borda", ativo = true },
    { id = "2", nome = "fila de trabalhos", ativo = true },
    { id = "3", nome = "réplica fria", ativo = false },
}

local Dados = {}

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
            ativo = if item.ativo then "1" else "0",
        })
    end

    ctx.itens = json.encode(json.array(linhas))
    ctx.total = tostring(#State.itens)
    ctx.ativos = tostring(ativos)
end

function alternar_primeiro(): ()
    local primeiro = State.itens[1]
    if primeiro == nil then
        return
    end
    primeiro.ativo = not primeiro.ativo
    Dados.publicar()
    toast({ message = `{primeiro.nome}: {if primeiro.ativo then "ativo" else "parado"}`, kind = "info" })
end

function buscar_ip(): ()
    if State.carregando then
        return
    end
    State.carregando = true
    ctx.status = "buscando..."

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
