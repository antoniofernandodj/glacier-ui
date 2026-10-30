-- Painel de referência. `fetch` lê a série de um servidor local (a Fase 0 mede a
-- aplicação, não a internet): python3 -m http.server na pasta de `dados.json`.
local servicos = {
    { nome = "api", estado = "ok" },
    { nome = "banco", estado = "ok" },
    { nome = "fila", estado = "lento" },
}

local function publicar()
    ctx.servicos = servicos
end

function init()
    ctx.novo_nome = ""
    ctx.status = "carregando…"
    publicar()
    local res = fetch(ctx.url_dados or "http://127.0.0.1:8098/dados.json")
    if res.ok then
        ctx.serie = json.decode(res.body)
        ctx.status = "ok (" .. res.status .. ")"
    else
        ctx.status = "falhou: " .. res.error
    end
end

function adicionar()
    local nome = ctx.novo_nome
    if nome == "" then
        return
    end
    table.insert(servicos, { nome = nome, estado = "novo" })
    ctx.novo_nome = ""
    publicar()
end
