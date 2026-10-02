function init()
    ctx.sse_status = "nunca aberto"
    ctx.ws_status = "nunca aberto"
    ctx.sse_msg = "-"
    ctx.ws_msg = "-"
    ctx.visitas = "0"
    ctx.ciclo = "(ainda não entrou no monitor)"
end

function ir_monitor()
    navigate("monitor")
end
