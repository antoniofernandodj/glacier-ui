-- O `console.*` do prelúdio. Tudo o que estes handlers fazem sai no TERMINAL —
-- a janela só dispara as chamadas.

function init()
    console.config({ prefix = "[demo]" })
    console.info("pronto — clique nos botões e olhe o terminal")
    ctx.nivel = "info"
    ctx.cor = "on"
end

function log_tudo()
    console.debug("debug: passo interno (some no nível default)")
    console.log("log:", "um número", 42, "e uma tabela", { id = 7, ativo = true })
    console.info("info: igual ao log")
    console.warn("warn: cache velho")
    console.error("error:", "falha simulada", { codigo = 500 })
end

function mostrar_tabela()
    console.table({
        { nome = "Ana", papel = "admin", ativo = true },
        { nome = "Beto", papel = "editor", ativo = false },
        { nome = "Cauê", papel = "leitor", ativo = true },
    })
    -- Uma lista de escalares vira (índice) + valor.
    console.table({ "sa-east-1", "us-east-1", "eu-west-1" })
end

-- Cicla debug → info → warn → error → silent → debug.
function ciclar_nivel()
    local ordem = { "debug", "info", "warn", "error", "silent" }
    local atual = ctx.nivel or "info"
    local i = 1
    for k, v in ordem do
        if v == atual then
            i = k
        end
    end
    local prox = ordem[(i % #ordem) + 1]
    ctx.nivel = prox
    console.set_level(prox)
    -- Este info só aparece se o novo nível deixar.
    console.info("nível agora: " .. prox)
end

function alternar_cor()
    local liga = ctx.cor ~= "on"
    ctx.cor = liga and "on" or "off"
    console.config({ color = liga })
    console.info("cor: " .. ctx.cor)
end
