--!strict
--!nolint FunctionUnused
-- A vitrine da validação: cada `on_change` grava a chave e recalcula os
-- `erro_<campo>`. O markup só mostra `{erro_nome}` etc. — não decide nada.

-- on_change="campo:f_nome" → campo("f_nome", <texto novo>)
function campo(chave: string, valor: string?): ()
  ctx[chave] = valor or ""
  revalidar()
end

-- Devolve "" quando ok, ou a mensagem. `<maskedinput>` guarda o CPF SEM
-- pontuação — por isso o teste é "11 dígitos", não um formato.
local function erro_de(chave: string): string
  if chave == "f_nome" then
    return if #(ctx.f_nome or "") >= 3 then "" else "informe ao menos 3 letras"
  elseif chave == "f_cpf" then
    local so_digitos = string.gsub(ctx.f_cpf or "", "%D", "")
    return if #so_digitos == 11 then "" else "o CPF precisa de 11 dígitos"
  elseif chave == "f_idade" then
    local n = tonumber(ctx.f_idade) or 0
    return if n >= 18 then "" else "precisa ser maior de idade"
  elseif chave == "f_aceite" then
    return if ctx.f_aceite == "true" then "" else "é preciso aceitar os termos"
  end
  return ""
end

function revalidar(): ()
  ctx.erro_nome = erro_de("f_nome")
  ctx.erro_cpf = erro_de("f_cpf")
  ctx.erro_idade = erro_de("f_idade")
  ctx.erro_aceite = erro_de("f_aceite")
end

function enviar(): ()
  revalidar()
  if ctx.erro_nome == "" and ctx.erro_cpf == "" and ctx.erro_idade == "" and ctx.erro_aceite == "" then
    ctx.f_status = `salvo: {ctx.f_nome} · {ctx.f_uf}`
    toast({ message = "Cadastro salvo", kind = "success" })
  else
    ctx.f_status = ""
    toast({ message = "Corrija os campos destacados", kind = "warning" })
  end
end

function limpar(): ()
  ctx.f_nome, ctx.f_cpf, ctx.f_tel = "", "", ""
  ctx.f_idade, ctx.f_nasc, ctx.f_aceite = "25", "", "false"
  ctx.f_status = ""
  revalidar()
end

return {}
