-- Demonstra a função global `notify` (notificação nativa do SO). Duas formas:
--   notify({ title = ..., body = ... })  -- tabela com título e corpo
--   notify("mensagem")                   -- string vira o corpo, sem título
-- Diferente de `toast(...)`, que é efêmero e desenhado dentro da janela.

local total = 0

local function bump()
    total = total + 1
    ctx.total = tostring(total)
end

function notificar_completo()
    notify({
        title = "Deploy concluído",
        body = "O serviço api está no ar e recebendo tráfego.",
    })
    bump()
end

function notificar_simples()
    notify("Build da imagem finalizado.")
    bump()
end

function init()
    ctx.total = "0"
end
