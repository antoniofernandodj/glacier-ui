function init()
    -- Semeia os dados para os Selects do modal
    local horas = {}
    for i = 0, 23 do table.insert(horas, string.format("%02d", i)) end
    ctx.lista_horas = json.encode(horas)

    local minutos = {}
    for i = 0, 59, 5 do table.insert(minutos, string.format("%02d", i)) end
    ctx.lista_minutos = json.encode(minutos)

    ctx.inicio = "09:00" -- Valor default
    ctx.mostrar_modal_hora = "false"
end

-- Disparado quando o usuário digita direto no TimePicker
function formatar_tempo()
    -- Ex: remover caracteres não numéricos e garantir o formato HH:MM
    local raw = ctx.inicio:gsub("%D", "")
    if #raw >= 4 then
        ctx.inicio = raw:sub(1, 2) .. ":" .. raw:sub(3, 4)
    end
end

-- Disparado quando clica no botão ⏰
function abrir_seletor_hora()
    -- Quebra o tempo atual para popular o modal
    local h, m = ctx.inicio:match("(%d+):(%d+)")
    ctx.h_sel = h or "09"
    ctx.m_sel = m or "00"
    ctx.mostrar_modal_hora = "true"
end

function atualizar_h() ctx.h_sel = ctx.h_sel end
function atualizar_m() ctx.m_sel = ctx.m_sel end

function fechar_modal()
    ctx.mostrar_modal_hora = "false"
end

function confirmar_hora()
    ctx.inicio = ctx.h_sel .. ":" .. ctx.m_sel
    ctx.mostrar_modal_hora = "false"
end