-- O handler do componente que embrulha o diálogo (Onda 8b, lado A).
--
-- Ele mora AQUI, no `<script>` do próprio componente — não na tela. É o que o
-- `::` de dono no `buttons=` (`Salvar:EditorRotulo::salvar:accept`) garante: a
-- ação do botão do diálogo volta para este componente, e `salvar` é uma função
-- global deste chunk.

function init()
    ctx.er_valor = "— sem rótulo —"
end

function salvar()
    local texto = ctx["__dialog.texto"] or ""
    console.log("texto:" .. texto)
    if texto == "" then texto = "(sem texto)" end
    local negrito = ctx["__dialog.negrito"] == "true"
    console.log("negrito:" .. tostring(negrito))
    ctx.er_valor = texto .. (if negrito then "  ·  negrito" else "")
end
