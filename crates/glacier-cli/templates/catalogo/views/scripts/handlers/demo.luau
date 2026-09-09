--!strict
--!nolint FunctionUnused
-- Handlers curtos que os demos do catálogo compartilham. Nenhum guarda estado:
-- um catálogo é uma vitrine, não um app.

-- Sidebar: on_click="ir:botoes" → troca a chave que o roteador do app.gv lê.
function ir(rota: string): ()
  ctx.view = rota
end

-- Feedback genérico: on_click="dizer:algo" mostra um toast com o sufixo.
function dizer(msg: string): ()
  toast({ message = msg, kind = "info" })
end

-- Setter genérico: on_click="def:chave:valor" grava ctx[chave] = valor.
-- (O motor passa só o primeiro sufixo depois do primeiro ":", então o resto
--  vem junto e é separado aqui.)
function def(spec: string): ()
  local chave, valor = string.match(spec, "^([^:]+):(.*)$")
  if chave ~= nil then
    ctx[chave] = valor
  end
end

-- §2.10 toasts por tipo: on_click="dlg_toast:success"
function dlg_toast(kind: string): ()
  toast({ message = "toast do tipo " .. kind, kind = kind })
end

-- §2.6 remover uma tag do <chip for-each>
function di_tirar_tag(tag: string): ()
  local restantes = {}
  for _, t in ipairs(json.decode(ctx.di_tags)) do
    if t ~= tag then
      table.insert(restantes, t)
    end
  end
  ctx.di_tags = json.encode(json.array(restantes))
end

-- §2.12 alternar o ponto de notificação
function ov_alternar_notif(): ()
  ctx.ov_notif = if ctx.ov_notif == "true" then "false" else "true"
end

return {}
