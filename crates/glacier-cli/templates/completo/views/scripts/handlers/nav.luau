--!strict
--!nolint FunctionUnused

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
