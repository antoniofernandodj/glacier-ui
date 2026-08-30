--!strict
--!nolint FunctionUnused
-- painel.luau — comportamento da janela principal.

function init(): ()
    ctx.recebidos = "0"
    ctx.ultimo = "nada ainda"
end

-- Abre uma janela nova a partir de um template. Os pares de `data` são gravados
-- no contexto do motor DELA antes do init — é como passar parâmetros.
-- Título e tamanho não precisam ser repetidos aqui: o <screen> do arquivo os
-- declara, e esta chamada só os sobreporia se soubesse algo que ele não sabe.
function abrir_detalhe(): ()
    open_window({
        file = "views/detalhe.gv",
        data = { origem = "painel" },
    })
end

-- Notificação NATIVA do SO (some da tela pelo caminho do sistema), diferente do
-- `toast`, que é efêmero e vive dentro da janela.
function avisar(): ()
    notify({ title = "{{titulo}}", body = "Uma notificação do sistema." })
end

-- Recebe o que as OUTRAS janelas mandam por `broadcast`. O payload chega já
-- decodificado numa tabela. Uma janela nunca recebe o próprio broadcast.
function on_broadcast(event: string, payload: any): ()
    if event ~= "detalhe_contou" then
        return
    end
    ctx.recebidos = tostring((tonumber(ctx.recebidos) or 0) + 1)
    ctx.ultimo = `a filha parou em {payload.valor}`
    toast({ message = "Uma janela filha reportou", kind = "info" })
end
