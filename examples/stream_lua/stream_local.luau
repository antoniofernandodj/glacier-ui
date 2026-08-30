-- Aponta para a API Quart local (examples/../python/sse-test):
--   GET  http://127.0.0.1:8000/sse  -> um evento por segundo
--   WS   ws://127.0.0.1:8000/ws     -> saúda ao conectar e ecoa o que enviar

function init()
    ctx.sse_status = "fechado"
    ctx.sse_msg = "(abra o SSE)"
    ctx.ws_status = "desconectado"
    ctx.ws_msg = "(conecte)"
    -- Auto-conecta aos dois streams da API Python assim que o app sobe.
    abrir_sse()
    abrir_ws()
end

-- ---- SSE -----------------------------------------------------------------

function abrir_sse()
    if sse_conn then return end
    ctx.sse_status = "conectando..."
    sse_conn = sse("http://127.0.0.1:8000/sse", {
        on_open    = function() ctx.sse_status = "aberto" end,
        on_message = function(data) ctx.sse_msg = data end,
        on_error   = function(e) ctx.sse_status = "erro: " .. e end,
        on_close   = function()
            ctx.sse_status = "fechado"
            sse_conn = nil
        end,
    })
end

function fechar_sse()
    if sse_conn then sse_conn:close() end
    sse_conn = nil
    ctx.sse_status = "fechado"
end

-- ---- WebSocket -----------------------------------------------------------

function abrir_ws()
    if ws_conn then return end
    ctx.ws_status = "conectando..."
    ws_conn = websocket("ws://127.0.0.1:8000/ws", {
        on_open    = function() ctx.ws_status = "conectado" end,
        on_message = function(data) ctx.ws_msg = data end,
        on_error   = function(e) ctx.ws_status = "erro: " .. e end,
        on_close   = function()
            ctx.ws_status = "desconectado"
            ws_conn = nil
        end,
    })
end

function enviar_ws()
    if ws_conn then ws_conn:send("ping " .. os.time()) end
end

function fechar_ws()
    if ws_conn then ws_conn:close() end
end
