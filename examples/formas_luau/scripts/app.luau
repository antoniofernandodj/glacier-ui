-- Onda 13 em Luau — o vocabulário de formas com dado do script.
--
-- O <canvas> e as formas (<circle>, <arc>, <line>) são MARKUP. A geometria
-- dirigida por dado é chave de contexto: o script calcula a ponta da agulha e o
-- sweep do arco e escreve `ponta_x`/`ponta_y`/`varrido`. O canvas relê no
-- render seguinte. Nenhuma linha aqui desenha.
--
-- `every(ms, fn)` é o temporizador repetitivo do prelúdio (construído sobre o
-- tick do motor). É o que gira a agulha.

local CX, CY, RAIO = 110, 110, 78
local rodando = true

-- Recalcula a ponta da agulha e o setor varrido a partir do ângulo atual.
local function redesenhar()
    -- Convenção do canvas: 0° à direita, horário. -90° começa no topo.
    local rad = math.rad(ctx.angulo - 90)
    ctx.ponta_x = string.format("%.1f", CX + RAIO * math.cos(rad))
    ctx.ponta_y = string.format("%.1f", CY + RAIO * math.sin(rad))
    ctx.varrido = tostring(ctx.angulo)
end

function girar()
    if not rodando then
        return
    end
    ctx.angulo = (tonumber(ctx.angulo) or 0) + 6
    if ctx.angulo >= 360 then
        ctx.angulo = ctx.angulo - 360
    end
    redesenhar()
end

function alternar()
    rodando = not rodando
    ctx.status = rodando and "girando" or "parado"
end

function zerar()
    ctx.angulo = 0
    redesenhar()
    ctx.status = "zerado"
end

function init()
    ctx.angulo = 0
    redesenhar()
    ctx.status = "girando"
    -- 60ms ≈ 16 quadros/s de atualização da agulha — suave o bastante.
    every(60, girar)
end
