--!strict
--!nolint FunctionUnused
-- O `<form>` declara as regras nos campos (`rules="required|minlen:3"`, etc.) e
-- o motor faz o ciclo inteiro no envio: roda as regras, publica
-- `{erro_<campo>}`, acende o `:invalid` no `.gss` e chama UM destes:
--   • `on_submit="salvar"`            — tudo passou;
--   • `on_validation_error="apontar"` — algo falhou.
-- Estes handlers só cuidam do *depois*. Nada de `revalidar()` na mão.

-- Só roda quando o formulário inteiro passou.
function salvar(): ()
  ctx.f_status = `salvo: {ctx.f_nome} · {ctx.f_uf}`
  toast({ message = "Cadastro salvo", kind = "success" })
end

-- Recebe as falhas já prontas, na ordem do documento:
--   [{ "campo": "f_nome", "msg": "informe ao menos 3 letras" }, ...]
-- Os `{erro_<campo>}` e o `:invalid` já estão na tela — aqui é o lugar do
-- extra (um toast, focar o primeiro campo, telemetria).
function apontar(erros_json: string?): ()
  ctx.f_status = ""
  local erros = json.decode(erros_json or "[]")
  toast({
    message = `Corrija {#erros} campo(s)`,
    kind = "warning",
  })
end

-- Botão `type="reset"`: o motor já apagou os `{erro_<campo>}` (e com eles o
-- `:invalid`); só devolvemos os valores ao estado inicial.
function limpar(): ()
  ctx.f_nome, ctx.f_cpf, ctx.f_tel = "", "", ""
  ctx.f_idade, ctx.f_nasc, ctx.f_aceite = "25", "", "false"
  ctx.f_uf = "SP"
  ctx.f_status = ""
end

return {}
