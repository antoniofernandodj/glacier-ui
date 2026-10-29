--!strict
-- Um "service": lógica pura, sem `ctx` e sem markup. Por isso se testa sem abrir
-- janela (ver `tests/saudacao.test.luau`).
--
-- Esta pasta está declarada em `test.luau_path` do `glacier.jsonc`, então um
-- `require("saudacao")` num teste a encontra. Num script do app, o `require` é
-- relativo ao arquivo: `require("../../services/saudacao")`.

local Saudacao = {}

--- "Olá, Ana!" — e um nome vazio não vira "Olá, !".
function Saudacao.para(nome: string): string
    local limpo = string.gsub(nome, "^%s+", "")
    limpo = string.gsub(limpo, "%s+$", "")
    if limpo == "" then
        return "Olá!"
    end
    return `Olá, {limpo}!`
end

return Saudacao
