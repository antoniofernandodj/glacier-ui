-- Onda 10 em Luau — a fonte que o motor não sabia nomear, do lado do script.
--
-- O que este arquivo mostra: o `pick_font{}`. Todo o resto — o registro de
-- famílias, o `<fontselect>`, a chave `__fonts`, o `font="{fonte}"` na amostra
-- — é motor e markup, e não pede uma linha aqui.
--
-- `pick_font` SUSPENDE a corrotina (como `confirm`/`fetch`): cede um pedido, o
-- motor abre o diálogo, e o `coroutine.yield` só retorna quando o usuário
-- aceita (a família, uma string) ou desiste (`nil`). É a mesma forma do
-- `pick_color`.

function init()
    ctx.status = "Pronto"
    -- A família que o `<fontselect>` grava e a amostra lê. Sem seed a lista
    -- abre sem seleção — degradação aceitável.
    ctx.fonte = "DejaVu Serif"
    ctx.ultima = "—"
end

-- ── pick_font: o QFontDialog ────────────────────────────────────────────
--
-- `value` é a família inicial (aqui, a que já está selecionada na lista),
-- `size` semeia o campo de tamanho do corpo. O retorno é só a família; o
-- tamanho e o estilo do diálogo são pré-visualização (ver
-- `builtins::font_dialog`).
function abrir_dialogo()
    local escolhida = pick_font({
        title = "Fonte da amostra",
        value = ctx.fonte,
        size = 18,
    })

    if escolhida then
        ctx.fonte = escolhida
        ctx.ultima = escolhida
        ctx.status = "Escolhida: " .. escolhida
    else
        ctx.status = "Diálogo cancelado — a fonte não mudou"
    end
end
