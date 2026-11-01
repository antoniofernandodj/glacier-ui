--!strict
--
-- O comportamento do exemplo de seletores, em Luau. A graça da tela está no
-- `app.gss`; aqui só mora o estado: a lista de serviços (num `local`, porque a
-- tela vê só o que vai para o `ctx`) e a densidade.

type Servico = { id: string, nome: string, ativo: boolean }

local SERVICOS: { Servico } = {
  { id = "api", nome = "API pública", ativo = true },
  { id = "db", nome = "Banco primário", ativo = true },
  { id = "cache", nome = "Redis", ativo = false },
  { id = "fila", nome = "Fila de eventos", ativo = true },
  { id = "cron", nome = "Agendador", ativo = false },
}

local compacta = false

-- Grava o estado nas chaves que o markup lê. Uma tabela atribuída ao `ctx`
-- vira JSON, que é o que o `foreach` percorre.
local function publicar(): ()
  local itens = {}
  for _, s in SERVICOS do
    table.insert(itens, {
      id = s.id,
      nome = s.nome,
      estado = if s.ativo then "ativo" else "parado",
      acao = if s.ativo then "Parar" else "Ligar",
    })
  end
  ctx.servicos = itens
  ctx.densidade = if compacta then "compacta" else "confortavel"
end

function init(): ()
  publicar()
end

function densidade(): ()
  compacta = not compacta
  publicar()
end

function parar_todos(): ()
  for _, s in SERVICOS do
    s.ativo = false
  end
  publicar()
end

function ligar_todos(): ()
  for _, s in SERVICOS do
    s.ativo = true
  end
  publicar()
end

-- `on_click="alternar:{s.id}"` chega aqui como `alternar("api")`.
function alternar(id: string): ()
  for _, s in SERVICOS do
    if s.id == id then
      s.ativo = not s.ativo
    end
  end
  publicar()
end
