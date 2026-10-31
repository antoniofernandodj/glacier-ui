-- Painel de referência. `fetch` lê a série de um servidor local (a Fase 0 mede a
-- aplicação, não a internet): python3 -m http.server na pasta de `dados.json`.
-- `estado` e `status_estado` são CHAVES do catálogo (locales/*.json); o markup as traduz
-- com `{t(…)}`, então trocar o idioma reescreve a tela sem este script fazer nada.
local servicos = {
    { nome = "api", estado = "estado.ok" },
    { nome = "banco", estado = "estado.ok" },
    { nome = "fila", estado = "estado.lento" },
}

local function publicar()
    ctx.servicos = servicos
end

function init()
    ctx.novo_nome = ""
    i18n.set_locale("pt-BR") -- o idioma de antes; o do sistema não decide numa medição
    ctx.status_estado = "status.carregando"
    ctx.status_sufixo = ""
    publicar()
    local res = fetch(ctx.url_dados or "http://127.0.0.1:8098/dados.json")
    if res.ok then
        ctx.serie = json.decode(res.body)
        ctx.status_estado = "status.ok"
        ctx.status_sufixo = " (" .. res.status .. ")"
    else
        ctx.status_estado = "status.falhou"
        ctx.status_sufixo = ": " .. res.error
    end
end

function adicionar()
    local nome = ctx.novo_nome
    if nome == "" then
        return
    end
    table.insert(servicos, { nome = nome, estado = "estado.novo" })
    ctx.novo_nome = ""
    publicar()
end

function idioma_pt()
    i18n.set_locale("pt-BR")
end

function idioma_en()
    i18n.set_locale("en")
end
