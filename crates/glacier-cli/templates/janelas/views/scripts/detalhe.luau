--!strict
--!nolint FunctionUnused
-- detalhe.luau — comportamento da janela filha.
--
-- Este script roda num motor SEPARADO do painel: `ctx` aqui é outro contexto, e
-- o contador abaixo pertence só a esta janela. Abrir duas filhas dá dois
-- contadores independentes.

function init(): ()
    ctx.contador = "0"
    -- `origem` já veio semeado pelo `data` do open_window; o `or` cobre o caso
    -- de alguém abrir este template sem passar nada.
    ctx.origem = ctx.origem or "ninguém"
end

function contar(): ()
    ctx.contador = tostring((tonumber(ctx.contador) or 0) + 1)
end

-- O padrão "janela auxiliar devolve um resultado e some": manda o valor para as
-- outras janelas e fecha a própria. `close_window` existe porque o motor
-- isolado não conhece o próprio id de janela — quem fecha é o daemon.
function avisar_e_fechar(): ()
    broadcast("detalhe_contou", { valor = ctx.contador })
    close_window()
end
