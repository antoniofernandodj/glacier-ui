--!strict
--!nolint FunctionUnused

function init(): ()
    ctx.contador = "0"
    ctx.origem = ctx.origem or "ninguém"
end

function contar(): ()
    ctx.contador = tostring((tonumber(ctx.contador) or 0) + 1)
end

function avisar_e_fechar(): ()
    broadcast("detalhe_contou", { valor = ctx.contador })
    close_window()
end
