--!strict
--!nolint FunctionUnused

require("handlers/validar")

-- `init` só semeia valores. Não há `erro_<campo>` nem classe de destaque para
-- semear: as regras moram no `<form>` (`rules="required|minlen:3"` etc.) e o
-- motor publica os `{erro_<campo>}` e acende o `:invalid` sozinho ao enviar.
function init(): ()
  ctx.f_nome, ctx.f_cpf, ctx.f_tel = "", "", ""
  ctx.f_idade, ctx.f_nasc, ctx.f_uf, ctx.f_aceite = "25", "", "SP", "false"
  ctx.f_ufs = json.encode(json.array({
    "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MT", "MS",
    "MG", "PA", "PB", "PR", "PE", "PI", "RJ", "RN", "RS", "RO", "RR", "SC",
    "SP", "SE", "TO",
  }))
  ctx.hoje = date.today()
  ctx.f_status = ""
end
