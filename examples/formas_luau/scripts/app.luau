-- Onda 13 em Luau — o vocabulário de formas com dado do script.
--
-- O <canvas> e as formas (<circle>, <arc>, <line>) são MARKUP. A geometria
-- dirigida por dado é chave de contexto: o script calcula a ponta da agulha e o
-- setor varrido e escreve `ponta_x`/`ponta_y`/`varrido`. O canvas relê no
-- render seguinte. Nenhuma linha aqui desenha.
--
-- `every(ms, fn)` é o temporizador repetitivo do prelúdio. NOTA: ele é ligado
-- por um CLIQUE (`alternar`), não no `init` — o motor só encaminha os efeitos
-- assíncronos (timers, fetch) de um HANDLER, não os do `init` (só streams).
-- Por isso o botão "girar / parar".

local CX, CY, RAIO = 110, 110, 78
local angulo = 0
local timer = nil

-- Recalcula a ponta da agulha e o setor varrido, e publica no contexto (texto).
local function redesenhar()
    -- Convenção do canvas: 0° à direita, horário. -90° começa no topo.
    local rad = math.rad(angulo - 90)
    ctx.ponta_x = string.format("%.1f", CX + RAIO * math.cos(rad))
    ctx.ponta_y = string.format("%.1f", CY + RAIO * math.sin(rad))
    ctx.varrido = tostring(angulo)
    ctx.angulo = tostring(angulo)
end

function girar()
    angulo = angulo + 6
    if angulo >= 360 then
        angulo = angulo - 360
    end
    redesenhar()
end

-- Liga/desliga o giro. `every` aqui (num handler) É encaminhado pelo motor.
function alternar()
    if timer then
        timer:cancel()
        timer = nil
        ctx.status = "parado"
    else
        timer = every(60, girar)
        ctx.status = "girando"
    end
end

function zerar()
    angulo = 0
    redesenhar()
    ctx.status = "zerado"
end

function init()
    redesenhar()
    ctx.status = "clique \"girar / parar\""
end
