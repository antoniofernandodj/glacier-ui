--!strict
--!nolint FunctionUnused
-- (o motor chama os handlers pelo nome; o lsp não vê a chamada)

-- A cozinha, em Luau. O markup é o mesmo do exemplo `foreach_fallback`: cada
-- etapa é um <foreach fallback="{vazio}">, e o script não sabe nada sobre os
-- fallbacks — ele só publica listas. Uma lista vazia vira `[]`, e é isso que
-- faz o componente de "vazio" entrar.
--
-- Ações com sufixo (`preparar:{p.id}`) chamam `preparar(id)`: o que vem depois
-- do `:` é o primeiro argumento.

type Pedido = { id: string, mesa: string, prato: string }

local CARDAPIO: { string } = {
  "Risoto de cogumelos",
  "Moqueca",
  "Feijoada",
  "Lasanha",
  "Salada caprese",
  "Picanha na chapa",
}

-- O DEPÓSITO: tipado. O `ctx` só recebe a projeção (ver `publicar`).
local fila: { Pedido } = {}
local preparo: { Pedido } = {}
local prontos: { Pedido } = {}
local entregues: { Pedido } = {}
local proximo = 0

-- A VITRINE: as quatro listas e os totais, já prontos para o markup.
local function publicar(): ()
  -- json.array: sem ele, a lista vazia vira `{}` e não `[]`.
  ctx.fila = json.encode(json.array(fila))
  ctx.preparo = json.encode(json.array(preparo))
  ctx.prontos = json.encode(json.array(prontos))
  ctx.entregues = json.encode(json.array(entregues))
  ctx.total_fila = tostring(#fila)
  ctx.total_preparo = tostring(#preparo)
  ctx.total_prontos = tostring(#prontos)
end

local function novo_pedido(): Pedido
  proximo += 1
  local p: Pedido = {
    id = tostring(proximo),
    mesa = tostring(proximo * 7 % 12 + 1),
    prato = CARDAPIO[proximo % #CARDAPIO + 1],
  }
  table.insert(fila, p)
  return p
end

-- Tira o pedido `id` de `de` e o põe no fim de `para`. Devolve o pedido, ou nil.
local function mover(de: { Pedido }, para: { Pedido }, id: string): Pedido?
  for i, p in de do
    if p.id == id then
      table.remove(de, i)
      table.insert(para, p)
      return p
    end
  end
  return nil
end

function init(): ()
  -- Começa com a fila e o fogo ocupados, e "Prontos"/"Entregues" vazios: os
  -- dois fallbacks já aparecem na primeira tela.
  novo_pedido()
  novo_pedido()
  local primeiro = novo_pedido()
  mover(fila, preparo, primeiro.id)
  ctx.status = "Pronto"
  publicar()
end

function novo(): ()
  local p = novo_pedido()
  ctx.status = `Pedido #{p.id} — mesa {p.mesa}`
  publicar()
end

function preparar(id: string): ()
  local p = mover(fila, preparo, id)
  if p == nil then
    return
  end
  ctx.status = `No fogo: {p.prato}`
  publicar()
end

function finalizar(id: string): ()
  local p = mover(preparo, prontos, id)
  if p == nil then
    return
  end
  ctx.status = `Pronto: {p.prato}`
  publicar()
end

function entregar(id: string): ()
  local p = mover(prontos, entregues, id)
  if p == nil then
    return
  end
  ctx.status = `Entregue na mesa {p.mesa}`
  publicar()
end

function limpar(): ()
  fila = {}
  preparo = {}
  prontos = {}
  entregues = {}
  ctx.status = "Tudo limpo — cada lista mostra o seu fallback"
  publicar()
end
