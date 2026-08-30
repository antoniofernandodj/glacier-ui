--!strict
--!nolint FunctionUnused
-- handlers/nav.luau — navegação da sidebar.
--
-- Cada função global aqui é o destino de um `on_click` no markup. Elas parecem
-- não usadas para o linter (nada as chama de dentro do Luau) — daí o nolint.

local function ir(rota: string): ()
    ctx.view = rota
end

function ir_home(): ()
    ir("home")
end

function ir_sobre(): ()
    ir("sobre")
end

return {}
