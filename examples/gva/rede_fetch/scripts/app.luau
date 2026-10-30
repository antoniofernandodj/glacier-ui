--!strict
--
-- A rede do `fetch`: cookies, `cookies = false`, multipart e `abort`. Precisa do servidor
-- de teste:  python3 tools/servidor_rede.py   (porta 8096)

local BASE = "http://127.0.0.1:8096"

function init()
    ctx.r1 = "…"
    ctx.r2 = "…"
    ctx.r3 = "…"
    ctx.r4 = "…"
    ctx.r5 = "…"
    ctx.r6 = "…"

    -- Cookies: o servidor define `sid` e `tema`; o jar os devolve sozinho.
    fetch(BASE .. "/set")
    ctx.r1 = fetch(BASE .. "/eco").body

    -- `cookies = false`: nada é enviado.
    ctx.r2 = fetch(BASE .. "/eco", { cookies = false }).body

    -- Max-Age=0 apaga o `sid`; o `tema` continua.
    fetch(BASE .. "/apaga")
    ctx.r3 = fetch(BASE .. "/eco").body

    -- Multipart: dois campos e um arquivo do disco.
    local up = fetch(BASE .. "/upload", {
        method = "POST",
        multipart = {
            fields = { titulo = "olá", versao = 2 },
            files = { { name = "anexo", path = "examples/gva/rede_fetch/amostra.txt", content_type = "text/plain" } },
        },
    })
    ctx.r4 = up.ok and up.body or ("erro: " .. up.error)

    -- Abort: o /lento demora 4 s; cancelo com 300 ms.
    local ctl = abort_controller()
    after(300, function()
        ctl:abort()
    end)
    local t0 = os.clock()
    local lento = fetch(BASE .. "/lento", { signal = ctl })
    ctx.r5 = `ok={lento.ok} error="{lento.error}" (voltou em {math.floor((os.clock() - t0) * 1000)} ms de CPU; aborted={ctl:aborted()})`

    -- Sem cancelar, o mesmo pedido responde.
    ctx.r6 = "(o /lento sem abort leva 4 s; não é repetido aqui)"
end
