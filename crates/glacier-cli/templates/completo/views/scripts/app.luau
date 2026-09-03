--!strict

local Dados = require("handlers/dados")
require("handlers/nav")

function init(): ()
    ctx.view = "home"
    ctx.status = ""
    ctx.ip = "-"
    Dados.publicar()
end
