--!strict
-- O motor chama `init` pelo nome; o luau-lsp não vê a chamada.
--!nolint FunctionUnused

-- O DEPÓSITO: janelas rolantes por série. O `ctx` é só a vitrine — a cada tique
-- as janelas são re-projetadas nele como JSON, e os widgets nem sabem que há um
-- relógio (ver AGENTS.md, "Onde mora o estado que não é texto").
local N = 24
local req_api: { number } = {}
local req_db: { number } = {}
local cpu: { number } = {}
local mem: { number } = {}

local function empurra(t: { number }, v: number): ()
  table.insert(t, v)
  if #t > N then
    table.remove(t, 1)
  end
end

local function publicar(): ()
  ctx.serie_req = json.encode(json.array({
    { name = "API", points = json.array(req_api) },
    { name = "DB", points = json.array(req_db) },
  }))
  ctx.spark_cpu = json.encode(json.array(cpu))
  ctx.spark_mem = json.encode(json.array(mem))
end

local function tick(): ()
  empurra(req_api, 40 + math.random(0, 30))
  empurra(req_db, 25 + math.random(0, 20))
  empurra(cpu, 30 + math.random(0, 50))
  empurra(mem, 45 + math.random(0, 25))
  ctx.kpi_total = tostring(1000 + math.random(0, 400))
  ctx.kpi_taxa = string.format("%.1f", math.random() * 4)
  ctx.kpi_lat = tostring(80 + math.random(0, 120))
  ctx.g_cpu = tostring(cpu[#cpu])
  ctx.g_disco = tostring(55 + math.random(0, 30))
  ctx.relogio = date.time()
  publicar()
end

function init(): ()
  for _ = 1, N do
    empurra(req_api, 40 + math.random(0, 30))
    empurra(req_db, 25 + math.random(0, 20))
    empurra(cpu, 30 + math.random(0, 50))
    empurra(mem, 45 + math.random(0, 25))
  end

  ctx.barras = json.encode(json.array({
    { label = "/login", value = 42 },
    { label = "/feed", value = 88 },
    { label = "/api", value = 61 },
    { label = "/static", value = 120 },
  }))
  ctx.fatias = json.encode(json.array({
    { label = "2xx", value = 82 },
    { label = "4xx", value = 12 },
    { label = "5xx", value = 6 },
  }))
  ctx.g_bands = json.encode(json.array({
    { to = 60, color = "#A6E3A1" },
    { to = 85, color = "#F9E2AF" },
    { to = 100, color = "#F38BA8" },
  }))

  tick()
  every(1000, tick)   -- o painel anda sozinho a partir daqui
end
