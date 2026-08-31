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

-- Disparado quando o usuário digita direto no TimePicker.
-- O texto digitado chega como ARGUMENTO (o motor não escreve a chave de um
-- `<TextInput>` sozinho) — quem grava em `ctx.inicio` é este handler.
function formatar_tempo(digitado)
    local raw = digitado:gsub("%D", "")
    if #raw >= 4 then
        ctx.inicio = raw:sub(1, 2) .. ":" .. raw:sub(3, 4)
    else
        ctx.inicio = digitado
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

-- Mesma história dos Selects: a opção escolhida vem como argumento.
function atualizar_h(escolhido) ctx.h_sel = escolhido end
function atualizar_m(escolhido) ctx.m_sel = escolhido end

function fechar_modal()
    ctx.mostrar_modal_hora = "false"
end

function confirmar_hora()
    ctx.inicio = ctx.h_sel .. ":" .. ctx.m_sel
    ctx.mostrar_modal_hora = "false"
end