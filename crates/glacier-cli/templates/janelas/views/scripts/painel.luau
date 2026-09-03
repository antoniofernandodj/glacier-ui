--!strict
--!nolint FunctionUnused

function init(): ()
    ctx.recebidos = "0"
    ctx.ultimo = "nada ainda"
end

function abrir_detalhe(): ()
    open_window({
        file = "views/detalhe.gv",
        data = { origem = "painel" },
    })
end

function avisar(): ()
    notify({ title = "{{titulo}}", body = "Uma notificação do sistema." })
end

function on_broadcast(event: string, payload: any): ()
    if event ~= "detalhe_contou" then
        return
    end
    ctx.recebidos = tostring((tonumber(ctx.recebidos) or 0) + 1)
    ctx.ultimo = `a filha parou em {payload.valor}`
    toast({ message = "Uma janela filha reportou", kind = "info" })
end
