--!strict
--!nolint FunctionUnused

local State = require("../state")

local Itens = {}

-- A VITRINE: projeta o depósito no ctx, já filtrado.
function Itens.publicar(): ()
  local busca = string.lower(ctx.q or "")
  local linhas = {}
  for _, s in State.itens do
    if busca == "" or string.find(string.lower(s.nome), busca, 1, true) then
      table.insert(linhas, {
        id = s.id,
        nome = s.nome,
        ambiente = s.ambiente,
        replicas = s.replicas,
      })
    end
  end
  ctx.linhas = json.encode(json.array(linhas))
  ctx.total = tostring(#State.itens)
end

local function achar(id: string): State.Servico?
  for _, s in State.itens do
    if s.id == id then
      return s
    end
  end
  return nil
end

-- on_change do <textinput> de filtro
function filtrar(texto: string): ()
  ctx.q = texto
  Itens.publicar()
end

-- Novo: três `prompt{}` em sequência; cada um suspende e volta com a resposta.
function novo(): ()
  local nome = prompt({ title = "Novo serviço", label = "Nome" })
  if nome == nil or nome == "" then
    return
  end
  local amb = prompt({
    title = "Novo serviço",
    label = "Ambiente",
    kind = "item",
    items = { "produção", "homolog", "dev" },
  })
  if amb == nil then
    return
  end
  local rep = prompt({
    title = "Novo serviço",
    label = "Réplicas",
    kind = "int",
    min = 1,
    max = 32,
    value = "1",
  })
  table.insert(State.itens, {
    id = tostring(State.seq),
    nome = nome,
    ambiente = amb,
    replicas = tonumber(rep) or 1,
  })
  State.seq += 1
  ctx.sel = tostring(State.seq - 1)
  Itens.publicar()
  toast({ message = `"{nome}" criado`, kind = "success" })
end

function editar(): ()
  local s = achar(ctx.sel or "")
  if s == nil then
    toast({ message = "Selecione uma linha primeiro", kind = "warning" })
    return
  end
  local nome = prompt({ title = "Editar serviço", label = "Nome", value = s.nome })
  if nome == nil or nome == "" then
    return
  end
  local rep = prompt({
    title = "Editar serviço",
    label = "Réplicas",
    kind = "int",
    min = 1,
    max = 32,
    value = tostring(s.replicas),
  })
  s.nome = nome
  s.replicas = tonumber(rep) or s.replicas
  Itens.publicar()
  toast({ message = "Alterações salvas", kind = "success" })
end

function excluir(): ()
  local id = ctx.sel or ""
  local s = achar(id)
  if s == nil then
    toast({ message = "Selecione uma linha primeiro", kind = "warning" })
    return
  end
  if not confirm({ title = `Excluir "{s.nome}"?`, message = "A ação não pode ser desfeita.", confirm_label = "Excluir", destructive = true }) then
    return
  end
  for i, item in State.itens do
    if item.id == id then
      table.remove(State.itens, i)
      break
    end
  end
  ctx.sel = ""
  Itens.publicar()
  toast({ message = "Serviço removido", kind = "info" })
end

return Itens
