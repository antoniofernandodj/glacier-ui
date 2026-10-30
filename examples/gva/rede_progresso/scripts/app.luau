--!strict
local BASE = "http://127.0.0.1:8096"
local MB = 1024 * 1024
local ctl: any = nil

function init()
    ctx.pct = 0
    ctx.estado = "Escolha uma operação."
end

local function mostra(p: any)
    ctx.pct = (p.fraction or 0) * 100
    local total = if p.total then `{math.floor(p.total / MB)}` else "?"
    ctx.estado = `{p.phase}: {math.floor(p.done / MB)} / {total} MB`
end

function enviar()
    ctx.pct = 0
    ctl = abort_controller()
    local corpo = string.rep("x", 50 * MB)
    local r = fetch(BASE .. "/enviar", {
        method = "POST",
        body = corpo,
        signal = ctl,
        on_progress = mostra,
    })
    ctx.estado = if r.ok then `pronto: {r.body}` else `falhou: {r.error}`
end

function baixar()
    ctx.pct = 0
    ctl = abort_controller()
    local r = fetch(BASE .. "/grande?mb=50", { signal = ctl, on_progress = mostra })
    ctx.estado = if r.ok then `pronto: {math.floor(#r.body / MB)} MB recebidos` else `falhou: {r.error}`
end

function cancelar()
    if ctl then
        ctl:abort()
    end
end
