--!strict
--!nolint FunctionUnused

require("handlers/validar")

function init(): ()
  ctx.f_nome, ctx.f_cpf, ctx.f_tel = "", "", ""
  ctx.f_idade, ctx.f_nasc, ctx.f_uf, ctx.f_aceite = "25", "", "SP", "false"
  ctx.f_ufs = json.encode(json.array({
    "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MT", "MS",
    "MG", "PA", "PB", "PR", "PE", "PI", "RJ", "RN", "RS", "RO", "RR", "SC",
    "SP", "SE", "TO",
  }))
  ctx.hoje = date.today()
  ctx.erro_nome, ctx.erro_cpf, ctx.erro_idade, ctx.erro_aceite = "", "", "", ""
  ctx.cls_nome, ctx.cls_cpf, ctx.cls_idade = "", "", ""
  ctx.f_status = ""
end
