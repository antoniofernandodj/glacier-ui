-- Tela `monitor`. Nada aqui roda no `init` (a tela inicial é `inicio`): o ciclo
-- de vida é todo por navegação.
--   on_enter : chega à tela -> abre os streams
--   on_leave : sai da tela  -> o motor FECHA os streams scope="screen" logo
--              depois deste gancho, e cada `on_close` roda

local sse_conn: StreamHandle? = nil
local ws_conn: StreamHandle? = nil

function on_enter()
    ctx.visitas = tostring((tonumber(ctx.visitas) or 0) + 1)
    ctx.ciclo = "on_enter #" .. ctx.visitas
    ctx.sse_status = "conectando..."
    ctx.ws_status = "conectando..."

    -- scope = "screen": vive só enquanto esta tela estiver ativa.
    sse_conn = sse(ctx.sse_url, {
        scope = "screen",
        on_open = function() ctx.sse_status = "aberto" end,
        on_message = function(data) ctx.sse_msg = data end,
        on_error = function(e) ctx.sse_status = "erro: " .. e end,
        on_close = function()
            ctx.sse_status = "fechado pelo motor"
            sse_conn = nil
        end,
    })

    ws_conn = websocket(ctx.ws_url, {
        scope = "screen",
        on_open = function() ctx.ws_status = "aberto" end,
        on_message = function(data) ctx.ws_msg = data end,
        on_error = function(e) ctx.ws_status = "erro: " .. e end,
        on_close = function()
            ctx.ws_status = "fechado pelo motor"
            ws_conn = nil
        end,
    })
end

function on_leave()
    ctx.ciclo = "on_leave #" .. ctx.visitas
end

function enviar_ws()
    if ws_conn then
        ws_conn:send("ping " .. os.time())
    end
end

function voltar()
    navigate_back()
end
