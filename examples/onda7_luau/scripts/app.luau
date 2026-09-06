--!strict
--
-- A Onda 7 inteira em Luau. O `main.rs` ao lado não tem `impl Component`: ele
-- registra o `.gv` e sai de cena.
--
-- O que o `examples/onda7` faz em Rust — `init` mais um braço genérico de
-- `update` — mora aqui. Três diferenças aparecem, e a terceira é a que só esta
-- onda torna visível:
--
--  1. **As séries são tabelas.** Do lado Rust, `[38, 41, 39]` e
--     `[{label=…, value=…}]` exigem um `serde_json::json!` construído à mão;
--     aqui são tabelas literais, e o `ctx` as codifica em JSON ao atribuí-las.
--  2. **Não há `self`.** O estado que a tela precisa ver mora no `ctx`; o que
--     ela não precisa, num `local` do módulo.
--  3. **O painel anda sozinho.** A versão Rust desenha um instante congelado;
--     esta liga um `every` e o relógio, a CPU e a latência passam a se mover.
--     Não é enfeite: é a prova de que os sete widgets redesenham a partir da
--     CHAVE, sem nenhum deles saber que existe um temporizador.
--
-- O que NÃO muda entre as duas versões é o mais importante: nenhum dos sete
-- widgets desta onda pede uma linha de script para funcionar. Girar o
-- `<dial>`, ler um `<gauge>`, desenhar as séries — tudo isso já acontece com o
-- `init` sozinho. O `every` abaixo existe para mostrar outra coisa.

-- A latência de cada serviço nas últimas 24 h. Números crus: a forma curta da
-- convenção `items`, e a que uma `<sparkline>` quer (ela não desenha rótulo).
local LAT_API = {
    38, 41, 39, 44, 52, 47, 43, 40, 38, 37, 39, 45,
    61, 58, 49, 44, 41, 40, 42, 46, 51, 48, 43, 39,
}
local LAT_DB = {
    12, 11, 13, 12, 14, 19, 24, 21, 17, 15, 14, 13,
    12, 12, 13, 16, 22, 28, 31, 26, 19, 15, 13, 12,
}
local LAT_FILA = {
    88, 92, 85, 79, 81, 96, 140, 165, 132, 104, 91, 86,
    83, 80, 84, 93, 118, 151, 129, 102, 90, 86, 82, 79,
}

-- As vendas do semestre, na forma longa: rótulo e valor. É o que o eixo X de um
-- `<linechart>` e de um `<barchart>` desenha.
local VENDAS = {
    { label = "jan", value = 128 },
    { label = "fev", value = 141 },
    { label = "mar", value = 119 },
    { label = "abr", value = 173 },
    { label = "mai", value = 186 },
    { label = "jun", value = 164 },
}

local FATIAS = {
    { label = "API", value = 42 },
    { label = "Banco", value = 27 },
    { label = "Fila", value = 18 },
    { label = "CDN", value = 13 },
}

-- As faixas de um `<gauge>` também são uma tabela — e esta vai para uma CHAVE,
-- não para o atributo: é o caso em que os limites são do backend, não do
-- desenho.
local LIMITES_DISCO = {
    { to = 70, color = "#A6E3A1" },
    { to = 90, color = "#F9E2AF" },
    { to = 100, color = "#F38BA8" },
}

-- O p95 da série. Mora no app, e é o ponto: o widget desenha a série, o app
-- decide o que ela significa. Um gráfico que calculasse percentil sozinho
-- estaria adivinhando.
local function p95(vs: { number }): number
    local ord = table.clone(vs)
    table.sort(ord)
    local i = math.max(1, math.ceil(#ord * 0.95))
    return ord[i]
end

-- A série viva das sparklines: empurra um valor no fim e descarta o primeiro,
-- mantendo a janela de 24 pontos.
local function anda(vs: { number }, novo: number): { number }
    local out = table.clone(vs)
    table.remove(out, 1)
    table.insert(out, novo)
    return out
end

function init()
    ctx.abas = {
        { id = "medidores", label = "Dial + Gauge + LCD" },
        { id = "graficos", label = "Sparkline + Line + Bar + Pie" },
        { id = "fecho", label = "O que fechou" },
    }
    ctx.aba = "medidores"

    -- Uma chave por medidor. Nenhuma delas é escrita por este arquivo depois
    -- daqui: quem escreve `volume` e `brilho` é o próprio `<dial>`, porque o
    -- markup não lhe deu `onChange`.
    ctx.volume = 7
    ctx.brilho = 60
    ctx.cpu = 72
    ctx.memoria = 38.4
    ctx.disco = 91
    ctx.rede = 420
    ctx.limites_disco = LIMITES_DISCO

    -- O `<lcdnumber>` lê o valor como TEXTO — é o que deixa um relógio passar
    -- inteiro pelos dois-pontos. E o relógio é do APP, não do motor: a mesma
    -- lição da Onda 3, agora num display.
    ctx.relogio = date.time()

    ctx.lat_api = LAT_API
    ctx.lat_db = LAT_DB
    ctx.lat_fila = LAT_FILA
    ctx.p95_api = string.format("%.0f", p95(LAT_API))
    ctx.p95_db = string.format("%.0f", p95(LAT_DB))
    ctx.p95_fila = string.format("%.0f", p95(LAT_FILA))

    ctx.vendas = VENDAS
    ctx.fatias = FATIAS

    -- De propósito: os quatro gráficos escrevem "sem dados" em vez de desenhar
    -- nada. Um gráfico que desenha nada é indistinguível de um quebrado.
    ctx.serie_vazia = {}

    ctx.status = "Pronto"

    -- O painel vivo. Repare no que NÃO está aqui: nenhuma referência a widget,
    -- nenhum redesenho pedido à mão. Escrever a chave basta — é o contrato de
    -- todo widget deste motor, e os sete desta onda não são exceção.
    every(1000, function()
        ctx.relogio = date.time()

        -- Um passeio aleatório curto, preso na faixa. Serve para os medidores
        -- se mexerem sem fingir que o exemplo tem um backend.
        local cpu = math.clamp(tonumber(ctx.cpu) :: number + math.random(-4, 4), 3, 99)
        ctx.cpu = cpu

        LAT_API = anda(LAT_API, math.clamp(LAT_API[#LAT_API] + math.random(-5, 5), 20, 120))
        ctx.lat_api = LAT_API
        ctx.p95_api = string.format("%.0f", p95(LAT_API))

        ctx.rede = math.clamp(tonumber(ctx.rede) :: number + math.random(-40, 40), 0, 1000)
    end)
end
