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

    -- Item 1: o resultado do `<dialog>` e as opções do `<select>` que vive
    -- DENTRO dele. `cores_rotulo` é uma chave comum do app — o corpo do modal é
    -- avaliado no contexto do app, então enxerga tudo o que a tela enxerga.
    ctx.rotulo = "— nada ainda —"
    ctx.cores_rotulo = [[
        [{"label":"Azul","value":"azul"},
         {"label":"Verde","value":"verde"},
         {"label":"Âmbar","value":"ambar"},
         {"label":"Vermelho","value":"vermelho"}]
    ]]
end

-- ── Item 1: o botão "Salvar" do `<dialog name="editar_rotulo">` ─────────
--
-- Chega aqui como uma AÇÃO comum — a mesma rota de um `<button on_click>`. O
-- motor já fechou o diálogo antes de despachar, e as chaves `__dialog.*` ainda
-- estão no contexto neste instante (são apagadas logo depois, no fechamento).
-- É o contraste com os suspensivos abaixo: nada de corrotina, nada de retorno —
-- o valor digitado mora numa chave, como em qualquer campo de texto.
function salvar_rotulo()
    local texto = ctx["__dialog.texto"] or ""
    local cor = ctx["__dialog.cor"] or ""
    local tamanho = ctx["__dialog.tamanho"] or ""
    local negrito = ctx["__dialog.negrito"] == "true"

    if texto == "" then texto = "(sem texto)" end
    if cor == "" then cor = "azul" end
    if tamanho == "" then tamanho = "14" end

    ctx.rotulo = texto
        .. " · " .. cor
        .. " · " .. tamanho .. "px"
        .. (if negrito then " · negrito" else "")
    ctx.status = "Rótulo salvo"
end

-- O botão "Descartar" da caixa de mensagem encadeada. O botão "Voltar" dela não
-- passa por aqui: `dialog:editar_rotulo` reabre o primeiro diálogo direto no
-- motor.
function descartou_rotulo()
    ctx.status = "Alterações descartadas"
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
-- `progress` NÃO suspende — ele acompanha um trabalho que continua rodando.
--
-- E aqui está a armadilha que isso cria, que vale mais do que o widget: um
-- laço SÍNCRONO não mostra progresso nenhum. Todo o `update` roda num turno
-- só, e a UI não pinta um quadro no meio dele — o diálogo abriria e fecharia
-- entre dois quadros, e o usuário não veria nada.
--
-- Progresso é sobre trabalho que **cede a vez**. Aqui cada passo é um
-- `after(0, …)`, que devolve o controle ao motor entre um e outro; num app de
-- verdade quem cede a vez é o `fetch`, um stream, ou o próprio relógio.

local PACOTES = 8

function baixar()
    ctx.cancelado = "false"
    progress({
        title = "Baixando pacotes",
        label = "conectando…",
        max = PACOTES,
        on_cancel = "abortar_download",
        cancel_label = "Abortar",
    })
    baixar_passo(1)
end

function baixar_passo(i)
    i = tonumber(i) or 1
    if ctx.cancelado == "true" then
        ctx.progresso_fim = "abortado no pacote " .. i
        progress_close()
        return
    end
    if i > PACOTES then
        progress_close()
        ctx.progresso_fim = PACOTES .. " pacotes baixados"
        return
    end
    progress_set(i, "pacote " .. i .. " de " .. PACOTES)
    -- 220 ms por pacote: tempo de um download curto, e o bastante para o olho
    -- acompanhar a barra em vez de ver um salto.
    after(220, function() baixar_passo(i + 1) end)
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
    -- Pela mesma razão do `baixar`: fechar na linha seguinte fecharia antes de
    -- qualquer quadro ser pintado.
    after(1400, function()
        progress_close()
        ctx.progresso_fim = "índice reconstruído"
    end)
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
