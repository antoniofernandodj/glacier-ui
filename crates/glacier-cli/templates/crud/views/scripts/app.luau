--!strict
--!nolint FunctionUnused

local Itens = require("handlers/itens")

function init(): ()
  ctx.q = ""
  ctx.sel = ""
  ctx.ordem = ""
  ctx.larguras = ""
  ctx.colunas = json.encode(json.array({
    { key = "nome", label = "Serviço" },
    { key = "ambiente", label = "Ambiente", width = "140" },
    { key = "replicas", label = "Réplicas", width = "90", align = "right" },
  }))
  Itens.publicar()
end
