--!strict
--
-- O texto do artigo mora em ARQUIVOS (examples/conteudo_prose): o `fetch("file://…")` os
-- lê e as chaves do contexto alimentam os `prose(content = "{chave}")` da tela. É o
-- mesmo caminho de um texto que viesse de uma API ou de um banco — o `prose` só vê a
-- chave. Três sintaxes para o MESMO artigo.

local function ler(caminho: string): string
    local res = fetch("file://" .. caminho)
    if res.ok then
        return res.body
    end
    return "erro ao ler " .. caminho .. ": " .. res.error
end

function init()
    ctx.artigo_html = ler("examples/conteudo_prose/artigo.html")
    ctx.artigo_gv = ler("examples/conteudo_prose/artigo.gv.txt")
    ctx.artigo_md = ler("examples/conteudo_prose/artigo.md")
    -- Conteúdo hostil: um link que tenta disparar uma ação do app. É RECUSADO.
    ctx.artigo_ruim = 'p { span("Veja ") a("apagar tudo", on_click = "app::apagar") }'
end
