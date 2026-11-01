--!strict
-- Ações deste exemplo. Nenhuma delas sabe (nem precisa saber) se quem disparou
-- foi um clique do usuário ou o canal `external` de outra thread — é o mesmo
-- caminho de despacho.

function init(): ()
    ctx.nome = ctx.nome or "—"
    ctx.busca = ctx.busca or "—"
    ctx.estado = ctx.estado or "esperando a outra thread…"
end

-- Disparada de fora com `sender.click("conectar")`.
function conectar(): ()
    ctx.estado = "conectado por controle externo ✓"
end

-- Disparada de fora com `sender.action("buscar", "nginx")` — `value` é o
-- segundo argumento do `action`, igual ao `on_change` de um campo de texto.
function buscar(): ()
    ctx.busca = value or ""
    ctx.estado = "busca aplicada de fora: " .. tostring(value)
end

function reiniciar(): ()
    ctx.nome = "—"
    ctx.busca = "—"
    ctx.estado = "reiniciado por clique humano"
end
