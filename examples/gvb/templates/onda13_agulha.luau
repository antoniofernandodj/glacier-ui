local x = 10
local t = nil
local function passo() x = x + 5 ctx.ponta_x = tostring(x) end
function alternar() if t then t:cancel() t = nil else t = every(50, passo) end end
function init() ctx.ponta_x = tostring(x) end
