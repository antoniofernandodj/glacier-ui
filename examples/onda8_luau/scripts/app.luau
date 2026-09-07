-- Onda 8 em Luau — os quatro diálogos que a onda entrega à camada de script.
--
-- O que este arquivo mostra e a versão Rust não mostra: os itens 2, 3 e 4 são
-- API de script, e a razão é o **suspensivo**. `prompt` e `pick_color` param a
-- corrotina no diálogo e voltam com a resposta, o que deixa o fluxo linear:
--
--     local nome = prompt({ ... })
--     if nome then ... end
--
-- Do lado Rust o mesmo pedido é um `show_dialog` mais um braço no `update`
-- para o botão — correto, e três lugares em vez de um. É a mesma diferença que
-- o `fetch` tem desde sempre.

function init()
    ctx.status = "Pronto"
    ctx.resposta = "— nada ainda —"
    ctx.confirmacao = "—"
    ctx.progresso_fim = "—"
    ctx.cor = "#89B4FA"
end

-- ── Item 2: as quatro variantes do QInputDialog ─────────────────────────
--
-- Repare que as quatro chamam a MESMA função. O que muda é o `kind`, e o que
-- ele escolhe é o widget do campo — a moldura, os botões e o retorno são
-- idênticos nas quatro.

function pedir_texto()
    local nome = prompt({
        title = "Renomear serviço",
        label = "Novo nome",
        value = "api-gateway",
        placeholder = "sem espaços",
    })
    -- `nil` é a desistência, e é diferente de `""` (respondeu vazio). É por
    -- isso que o cancelamento devolve nil e não false.
    ctx.resposta = if nome then ("texto: " .. nome) else "cancelou"
end

function pedir_inteiro()
    local n = prompt({
        title = "Réplicas",
        label = "Quantas?",
        kind = "int",
        value = "3",
        min = 1,
        max = 32,
    })
    ctx.resposta = if n then ("inteiro: " .. n) else "cancelou"
end

function pedir_decimal()
    local v = prompt({
        title = "Orçamento",
        label = "Limite mensal (R$)",
        kind = "double",
        value = "250.00",
        min = 0,
        max = 10000,
        step = 10,
        decimals = 2,
    })
    ctx.resposta = if v then ("decimal: " .. v) else "cancelou"
end

function pedir_item()
    local amb = prompt({
        title = "Ambiente",
        label = "Onde publicar?",
        kind = "item",
        value = "prod",
        items = {
            { label = "Produção", value = "prod" },
            { label = "Homologação", value = "homolog" },
            { label = "Desenvolvimento", value = "dev" },
        },
    })
    ctx.resposta = if amb then ("item: " .. amb) else "cancelou"
end

-- ── Item 3: o progresso ─────────────────────────────────────────────────
--
-- Note a ausência de `local x = progress(...)`: ele NÃO suspende. O laço logo
-- abaixo continua rodando, e é justamente isso que o diálogo mostra.

function baixar()
    ctx.cancelado = "false"
    progress({
        title = "Baixando pacotes",
        label = "conectando…",
        max = 8,
        on_cancel = "abortar_download",
        cancel_label = "Abortar",
    })
    for i = 1, 8 do
        if ctx.cancelado == "true" then
            ctx.progresso_fim = "abortado no pacote " .. i
            progress_close()
            return
        end
        progress_set(i, "pacote " .. i .. " de 8")
    end
    progress_close()
    ctx.progresso_fim = "8 pacotes baixados"
end

-- O cancelamento é uma AÇÃO comum, não um mecanismo novo: o botão despacha
-- `abortar_download` como qualquer `<button on_click>`, e o handler faz o que
-- o app quiser. O `QProgressDialog::wasCanceled()` vira uma chave que o laço
-- consulta.
function abortar_download()
    ctx.cancelado = "true"
    ctx.status = "Download abortado"
end

function indexar()
    -- Sem `value`, a barra nasce indeterminada — um spinner. É o estado certo
    -- de quem acabou de abrir e ainda não sabe o primeiro número.
    progress({ title = "Reindexando", busy_label = "lendo o catálogo…" })
    progress_close()
    ctx.progresso_fim = "índice reconstruído"
end

-- ── Item 4: a cor ───────────────────────────────────────────────────────

function escolher_cor()
    local nova = pick_color({ title = "Cor do rótulo", value = ctx.cor })
    if nova then
        ctx.cor = nova
        ctx.status = "Cor: " .. nova
    end
end

-- ── O que não mudou ─────────────────────────────────────────────────────

function remover()
    local ok = confirm({
        title = "Remover tudo?",
        message = "A ação não pode ser desfeita.",
        confirm_label = "Remover",
        destructive = true,
    })
    -- Booleano, não string: `confirm` continua sendo o que sempre foi.
    ctx.confirmacao = if ok then "confirmou (boolean)" else "cancelou (boolean)"
end
