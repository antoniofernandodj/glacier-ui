-- O cliente `http(...)` do prelúdio, por inteiro.
--
-- `http(base_url, opts)` devolve um objeto com base_url / headers / query /
-- timeout / retries de BASE, mais interceptors e error handlers. Todo método de
-- chamada (`:get`, `:post`, …) SUSPENDE a corrotina, como o `fetch` cru.

-- Um client para a API inteira: base_url, header base, timeout base e 1 retry
-- automático em falha de rede / 429 / 5xx.
local api = http("https://jsonplaceholder.typicode.com", {
    headers = { Accept = "application/json" },
    timeout = 8000, -- ms, por requisição
    retries = 1,
})

-- Uma variável de log, para a tela mostrar o que passou pelos interceptors.
local function log(linha: string)
    ctx.log = (ctx.log or "") .. linha .. "\n"
end

-- Interceptor de REQUISIÇÃO: roda antes de cada `fetch`. Pode mutar `req` ou
-- devolver uma tabela nova. Aqui: carimba um id de rastreio e registra.
api:on_request(function(req)
    req.headers["X-Trace-Id"] = string.format("t-%d", math.random(1000, 9999))
    log(("→ %s %s"):format(req.method, req.path))
    return req
end)

-- Interceptor de RESPOSTA: roda depois, já com `res.json` decodificado.
api:on_response(function(res)
    log(("← %d%s"):format(res.status, res.ok and "" or " (falha)"))
    return res
end)

-- Error handler: dispara quando o resultado final (depois dos retries) não é
-- ok. `err = { status, error, url, method, attempt, body }`.
api:on_error(function(err)
    toast({ message = ("%s %s → %s"):format(err.method, err.url, err.error), kind = "error" })
    log("‼ " .. err.error)
end)

-- Um client DERIVADO para um sub-recurso: herda base_url, headers e os
-- interceptors do pai, e acrescenta o que passar.
local posts = api:extend("/posts")

function init()
    ctx.status = "pronto"
    ctx.corpo = "(clique num botão)"
    ctx.log = ""
    math.randomseed(os.clock() * 1e6)
end

-- GET com um recurso só.
function buscar_um()
    ctx.status = "carregando…"
    local r = posts:get("/1")
    ctx.status = ("GET /posts/1 → %d"):format(r.status)
    ctx.corpo = r.json and r.json.title or r.body
end

-- GET com query params (viram ?userId=1 na URL, já url-encoded).
function buscar_lista()
    ctx.status = "carregando…"
    local r = posts:get("", { query = { userId = 1 } })
    if r.ok and type(r.json) == "table" then
        ctx.status = ("GET /posts?userId=1 → %d itens"):format(#r.json)
        ctx.corpo = r.json[1] and r.json[1].title or "(vazio)"
    else
        ctx.status = "falhou"
        ctx.corpo = r.error
    end
end

-- POST com uma TABELA de corpo → o client faz json.encode e põe o
-- Content-Type: application/json sozinho.
function criar()
    ctx.status = "enviando…"
    local r = posts:post("", { title = "de glacier", body = "corpo", userId = 7 })
    ctx.status = ("POST /posts → %d"):format(r.status)
    ctx.corpo = r.json and ("id gerado: " .. tostring(r.json.id)) or r.body
end

-- Uma rota que não existe: 404 → o error handler acima dispara o toast.
function forcar_erro()
    ctx.status = "carregando…"
    local r = api:get("/rota/que/nao/existe")
    ctx.status = ("→ %d"):format(r.status)
    ctx.corpo = r.error ~= "" and r.error or r.body
end
