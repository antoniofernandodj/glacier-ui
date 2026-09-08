--!strict
-- `init` e os handlers deste arquivo são chamados pelo MOTOR, pelo nome — o
-- luau-lsp não tem como ver essa chamada e os reporta como código morto. É o
-- único aviso falso que um projeto novo produz, e ele fica desligado aqui para
-- que a saída do `make luau` seja só o que importa.
--!nolint FunctionUnused

local Dados = require("handlers/dados")
require("handlers/nav")

function init(): ()
    ctx.view = "home"
    ctx.status = ""
    ctx.ip = "-"
    Dados.publicar()
end
