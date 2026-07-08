-- Demonstra as lacunas fechadas em PLANO_LUAU_ROBUSTEZ.md: timers (`after`),
-- persistência (`storage`), leitura de viewport, tabelas em `ctx` e erros
-- visíveis (`on_error`).

-- Handle do temporizador em andamento (variável local do módulo, sobrevive
-- entre chamadas porque o interpretador Luau do componente é o mesmo).
local temporizador = nil

-- Handle do cronômetro repetitivo (`every`), mesma ideia de `temporizador`.
local cronometro = nil

function init()
    -- `storage.get` lê do disco: se o app foi fechado e reaberto, o rascunho
    -- da última sessão volta aqui.
    ctx.rascunho = storage.get("rascunho") or ""
    ctx.status = "parado"
    ctx.largura = "?"
    ctx.altura = "?"
    ctx.prefs = ""
    ctx.ultimo_erro = "(nenhum ainda)"
    ctx.tiques = 0
end

function salvar_rascunho()
    storage.set("rascunho", ctx.rascunho)
    toast({ message = "Rascunho salvo em disco.", kind = "success" })
end

function iniciar_temporizador()
    ctx.status = "aguardando (3s)..."
    -- NÃO bloqueia: a corrotina de `iniciar_temporizador` termina no mesmo
    -- turno, e `tempo_esgotado` roda numa corrotina nova quando o prazo vence.
    temporizador = after(3000, "tempo_esgotado")
end

function cancelar_temporizador()
    if temporizador then
        temporizador:cancel()
        temporizador = nil
        ctx.status = "cancelado"
    end
end

function tempo_esgotado()
    ctx.status = "tempo esgotado!"
    toast({ message = "A contagem regressiva terminou.", kind = "info" })
end

-- every(): igual a `after`, mas se reagenda sozinho a cada disparo (equivalente
-- a setInterval). `:cancel()` no handle devolvido interrompe as próximas
-- repetições.
function iniciar_cronometro()
    if cronometro then
        return
    end
    ctx.tiques = 0
    cronometro = every(1000, "tique")
end

function parar_cronometro()
    if cronometro then
        cronometro:cancel()
        cronometro = nil
    end
end

function tique()
    ctx.tiques = ctx.tiques + 1
end

function ler_viewport()
    local v = viewport()
    ctx.largura = tostring(math.floor(v.width))
    ctx.altura = tostring(math.floor(v.height))
end

function gerar_prefs()
    -- Antes: uma tabela em `ctx` desaparecia silenciosamente. Agora é
    -- serializada via `json.encode` por baixo automaticamente.
    ctx.prefs = { tema = "escuro", notificacoes = true, volume = 7 }
end

function provocar_erro()
    -- Indexar campo de `nil` — erro de runtime comum e fácil de escrever sem
    -- querer (ex.: resposta de `fetch` que veio vazia).
    local resposta = nil
    return resposta.campo_que_nao_existe
end

-- Opcional: se não existisse, o motor promoveria a mensagem crua do erro a um
-- toast sozinho (ver PLANO_LUAU_ROBUSTEZ.md §3). Definindo `on_error`, o
-- script controla a mensagem exibida ao usuário final e ainda guarda o erro
-- técnico em `ctx` para depuração.
function on_error(msg)
    ctx.ultimo_erro = msg
    toast({
        title = "Erro no script",
        message = "Algo deu errado aqui dentro — detalhes no console.",
        kind = "error",
    })
end
