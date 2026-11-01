-- Onda 11, habilitador C, em Luau — séries múltiplas do lado do script.
--
-- `series="carga"` é uma convenção de DADOS. O script mantém uma tabela de
-- séries e a escreve em `ctx.carga`; o `<linechart>` lê essa chave no render e
-- desenha uma linha por série, com a legenda e o ciclo de cores do motor.
-- Nenhuma função aqui toca em pixel.

local NOMES = { "API", "Web", "Fila" }

-- O estado do exemplo: uma lista de pontos por série. Mora no script porque é
-- dado do app; `ctx.carga` é só o espelho JSON que o gráfico consome.
local pontos = {
    API = { 120, 132, 101, 134, 90 },
    Web = { 220, 182, 191, 234, 290 },
    Fila = { 150, 232, 201, 154, 190 },
}

local function publicar()
    local series = {}
    for i, nome in ipairs(NOMES) do
        local s = { name = nome, points = pontos[nome] }
        -- A terceira série ganha cor fixa; as outras caem no ciclo do tema.
        if nome == "Fila" then
            s.color = "#f38ba8"
        end
        series[i] = s
    end
    ctx.carga = series
end

function init()
    publicar()
    ctx.status = "3 séries — a legenda e as cores são do motor"
end

-- Acrescenta um ponto a cada série (um passeio aleatório simples) e republica.
function amostrar()
    for _, nome in ipairs(NOMES) do
        local serie = pontos[nome]
        local ultimo = serie[#serie] or 100
        local delta = math.random(-40, 40)
        serie[#serie + 1] = math.max(0, ultimo + delta)
        -- Uma janela deslizante: nunca mais de 24 pontos.
        if #serie > 24 then
            table.remove(serie, 1)
        end
    end
    publicar()
    ctx.status = "amostra adicionada — o gráfico seguiu"
end

function zerar()
    for _, nome in ipairs(NOMES) do
        pontos[nome] = { 0 }
    end
    publicar()
    ctx.status = "zerado"
end
