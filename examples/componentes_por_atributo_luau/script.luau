--!strict
--!nolint FunctionUnused
-- (o motor chama os handlers pelo nome; o lsp não vê a chamada)

-- Comportamento da tela de contatos.
--
-- O ponto do exemplo: o NOME de um componente é um dado como outro qualquer.
-- `ctx.cabecalho` e `ctx.vazio` guardam qual componente a <Vitrine> desenha em
-- cada papel, e trocar a string troca o componente na tela — sem uma linha de
-- markup condicional.
--
-- Ações com sufixo (`favoritar:{id}`) chamam `favoritar(id)`: o que vem depois
-- do `:` é o primeiro argumento.

type Contato = { id: string, nome: string, email: string }
type Sugestao = { nome: string, email: string }

local SUGESTOES: { Sugestao } = {
  { nome = "Ada Lovelace", email = "ada@analitica.org" },
  { nome = "Alan Turing", email = "alan@bletchley.uk" },
  { nome = "Grace Hopper", email = "grace@cobol.mil" },
  { nome = "Edsger Dijkstra", email = "edsger@caminho.nl" },
  { nome = "Barbara Liskov", email = "barbara@substituicao.edu" },
}

-- O DEPÓSITO: tipado, com a forma real do dado. O `ctx` só recebe a projeção.
local contatos: { Contato } = {}
local favoritos: { Contato } = {}
local proximo = 1

-- A posição do contato `id` em `lista`, ou nil.
local function posicao(lista: { Contato }, id: string): number?
  for i, c in lista do
    if c.id == id then
      return i
    end
  end
  return nil
end

-- A VITRINE: projeta o depósito no ctx, já filtrado e com os derivados prontos.
local function publicar(): ()
  local busca = string.lower(ctx.busca or "")
  local visiveis: { Contato } = {}
  for _, c in contatos do
    if busca == "" or string.find(string.lower(c.nome), busca, 1, true) then
      table.insert(visiveis, c)
    end
  end

  -- json.array: sem ele, a lista vazia vira `{}` e não `[]`.
  ctx.visiveis = json.encode(json.array(visiveis))
  ctx.favoritos = json.encode(json.array(favoritos))
  ctx.total = tostring(#visiveis)

  -- O fallback acompanha o MOTIVO de a lista estar vazia: sem contato nenhum,
  -- ou uma busca que não casou com ninguém.
  ctx.vazio = if busca ~= "" and #contatos > 0 then "BuscaSemResultado" else "NenhumContato"
end

local function inserir(): Contato
  local s = SUGESTOES[(proximo - 1) % #SUGESTOES + 1]
  local c: Contato = { id = tostring(proximo), nome = s.nome, email = s.email }
  table.insert(contatos, c)
  proximo += 1
  return c
end

function init(): ()
  ctx.cabecalho = "TituloGrande"
  ctx.busca = ""
  ctx.status = "Pronto"
  inserir()
  inserir()
  publicar()
end

function adicionar(): ()
  local c = inserir()
  ctx.status = `Adicionado: {c.nome}`
  publicar()
end

function remover_todos(): ()
  contatos = {}
  favoritos = {}
  ctx.status = "Lista esvaziada — os fallbacks entram no lugar"
  publicar()
end

function trocar_cabecalho(): ()
  local novo = if ctx.cabecalho == "TituloGrande" then "TituloCompacto" else "TituloGrande"
  ctx.cabecalho = novo
  ctx.status = `Cabeçalho: {novo}`
end

-- `on_change="filtrar"` — o texto novo chega como argumento.
function filtrar(texto: string): ()
  ctx.busca = texto
  publicar()
end

function favoritar(id: string): ()
  local i = posicao(contatos, id)
  if i == nil or posicao(favoritos, id) ~= nil then
    return
  end
  local c = contatos[i]
  table.insert(favoritos, c)
  ctx.status = `Favoritado: {c.nome}`
  publicar()
end

function desfavoritar(id: string): ()
  local i = posicao(favoritos, id)
  if i == nil then
    return
  end
  local nome = favoritos[i].nome
  table.remove(favoritos, i)
  ctx.status = `Fora dos favoritos: {nome}`
  publicar()
end
