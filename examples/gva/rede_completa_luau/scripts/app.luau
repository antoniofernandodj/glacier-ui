--!strict
--
-- Toda a rede do Glacier. Precisa do servidor de teste:  python3 tools/servidor_rede.py
-- Cada `s<N>` abaixo é uma seção da tela; o que aparece é o que o servidor realmente recebeu.

local BASE = "http://127.0.0.1:8096"
local AMOSTRA = "examples/comum/rede_amostra.txt"

-- Resume o eco JSON do servidor numa linha.
local function eco(r: any): string
    local j = r.json or (r.body ~= "" and json.decode(r.body)) or {}
    local q = {}
    for k, v in j.query or {} do
        table.insert(q, `{k}={v}`)
    end
    table.sort(q)
    local h = {}
    for k, v in j.headers or {} do
        table.insert(h, `{k}={v}`)
    end
    table.sort(h)
    return `{j.method} {j.path}  query[{table.concat(q, ",")}]  headers[{table.concat(h, ", ")}]  corpo={j.body}`
end

function init()
    for i = 1, 21 do
        ctx["s" .. i] = "…"
    end

    -- ── fetch ────────────────────────────────────────────────────────────
    local r = fetch(BASE .. "/json?busca=glacier&pagina=2")
    ctx.s1 = `status {r.status}: ` .. eco({ body = r.body })

    local r2 = fetch(BASE .. "/json", {
        method = "POST",
        headers = { ["Content-Type"] = "application/json", ["X-Trace"] = "abc" },
        body = json.encode({ nome = "ana", idade = 30 }),
    })
    ctx.s2 = eco({ body = r2.body })

    local lento = fetch(BASE .. "/lento", { timeout = 300 })
    ctx.s3 = `ok={lento.ok} status={lento.status} error="{lento.error}"`

    local arq = fetch("file://" .. AMOSTRA)
    ctx.s4 = `ok={arq.ok}, {#arq.body} caracteres lidos de {AMOSTRA}`

    local dl = download_file(BASE .. "/download", "/tmp/glacier_rede_download.bin")
    local volta = fetch("file:///tmp/glacier_rede_download.bin", { response = "base64" })
    ctx.s5 = `ok={dl.ok}; lido de volta: {#volta.body} caracteres em base64 (2000 bytes ≈ 2668)`

    -- ── cookies ──────────────────────────────────────────────────────────
    clear_cookies()
    fetch(BASE .. "/set")
    ctx.s6 = fetch(BASE .. "/eco").body
    ctx.s7 = fetch(BASE .. "/eco", { cookies = false }).body
    fetch(BASE .. "/apaga")
    ctx.s8 = fetch(BASE .. "/eco").body
    clear_cookies()
    ctx.s9 = fetch(BASE .. "/eco").body

    -- ── multipart ────────────────────────────────────────────────────────
    local up = fetch(BASE .. "/upload", {
        method = "POST",
        multipart = {
            fields = { titulo = "olá", versao = 2 },
            files = { { name = "anexo", path = AMOSTRA, content_type = "text/plain" } },
        },
    })
    ctx.s10 = up.ok and up.body or ("erro: " .. up.error)

    -- ── abort ────────────────────────────────────────────────────────────
    local ctl = abort_controller()
    after(300, function()
        ctl:abort()
    end)
    local cancelado = fetch(BASE .. "/lento", { signal = ctl })
    ctx.s11 = `ok={cancelado.ok} error="{cancelado.error}" aborted={ctl:aborted()}`

    -- ── cliente http() ───────────────────────────────────────────────────
    local api = http(BASE, {
        headers = { ["X-Api-Key"] = "segredo", Accept = "application/json" },
        query = { versao = 1 },
        timeout = 5000,
    })

    local a = api:get("/json", { query = { busca = "x" } })
    ctx.s12 = eco(a)

    local b = api:post("/json", { nome = "bia", tags = { "a", "b" } })
    ctx.s13 = eco(b)

    -- interceptors: a requisição ganha um cabeçalho; a resposta ganha um campo.
    local api2 = http(BASE)
    api2:on_request(function(req)
        req.headers["X-Trace"] = "trace-42"
        return req
    end)
    api2:on_response(function(res)
        res.marcado = true
        return res
    end)
    local c = api2:get("/json")
    ctx.s14 = eco(c) .. `  marcado={c.marcado}`

    -- retries: o servidor falha 2 vezes (500) e responde na 3ª.
    local api3 = http(BASE, { retries = 3 })
    local d = api3:get("/falha", { query = { ate = 2 } })
    ctx.s15 = `ok={d.ok} depois de {d.json and d.json.tentativa or "?"} tentativa(s)`

    -- on_error: roda quando a chamada falha de vez.
    local erros = {}
    local api4 = http(BASE)
    api4:on_error(function(err)
        table.insert(erros, `{err.method} {err.url} → {err.status}`)
    end)
    api4:get("/status/404")
    ctx.s16 = `on_error recebeu: {erros[1] or "(nada)"}`

    -- extend: herda cabeçalhos e acrescenta o prefixo /v1.
    local v1 = api:extend("/v1")
    local e = v1:get("/json")
    ctx.s17 = eco(e)

    -- cookies no cliente
    fetch(BASE .. "/set")
    local com = api2:get("/eco") -- /eco não é JSON: o corpo é o texto
    local sem = http(BASE, { cookies = false }):get("/eco")
    api2:clear_cookies()
    local apos = api2:get("/eco")
    ctx.s18 = `jar ligado: [{com.body}]  cookies=false: [{sem.body}]  após clear_cookies: [{apos.body}]`

    -- multipart pelo cliente: usa a base_url e os cabeçalhos do cliente
    local upc = api:post("/upload", nil, {
        multipart = {
            fields = { origem = "cliente" },
            files = { { name = "anexo", path = AMOSTRA, filename = "cliente.txt" } },
        },
    })
    ctx.s19 = upc.ok and upc.body or ("erro: " .. upc.error)

    -- abort pelo cliente
    local ctl2 = abort_controller()
    after(300, function()
        ctl2:abort()
    end)
    local f = api:get("/lento", { signal = ctl2 })
    ctx.s20 = `ok={f.ok} error="{f.error}" (o cliente não repete uma chamada cancelada)`

    -- QUERY (RFC 10008): seguro e idempotente, com a consulta no corpo
    local q = api:query("/json", { termo = "glacier", limite = 10 })
    ctx.s21 = `status {q.status}: ` .. eco(q)
    -- sem corpo, o cliente nem envia (RFC 10008 §2)
    local ruim = api:query("/json")
    ctx.s21 ..= `\nsem corpo: ok={ruim.ok} status={ruim.status} error="{ruim.error}"`
end
