-- Onda 8b (Luau) — o lado B: um `<dialog>` cujo corpo usa `<CampoForm>`.
--
-- Nada de novo no handler: o diálogo tem corpo em markup, o corpo usa um
-- componente, e mesmo assim o que o usuário digita mora numa chave `__dialog.*`
-- comum. `salvar_perfil` a lê no aceite, como qualquer `<dialog>` com campo.
--
-- O lado A (o componente que embrulha o próprio diálogo) tem o handler dele em
-- `views/editor_rotulo.luau`, não aqui — é o ponto do exemplo.

function init()
    ctx.status = "Pronto"
    ctx.perfil = "— vazio —"
    ctx.cargos = [[
        [{"label":"Dev","value":"dev"},
         {"label":"Design","value":"design"},
         {"label":"PM","value":"pm"}]
    ]]
end

function salvar_perfil()
    local nome = ctx["__dialog.nome"] or ""
    local cargo = ctx["__dialog.cargo"] or ""
    console.log("Nome: " .. nome)
    console.log("Cargo: " .. cargo)
    if nome == "" then nome = "(sem nome)" end
    if cargo == "" then cargo = "—" end
    ctx.perfil = nome .. " · " .. cargo
    ctx.status = "Perfil salvo"
end
