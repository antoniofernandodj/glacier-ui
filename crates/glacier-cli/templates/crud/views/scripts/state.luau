--!strict
-- O DEPÓSITO: a lista de verdade, tipada. O `ctx` só recebe a projeção JSON
-- (ver `handlers/itens.luau::publicar`).

export type Servico = {
  id: string,
  nome: string,
  ambiente: string,
  replicas: number,
}

export type State = {
  itens: { Servico },
  seq: number,   -- próximo id
}

local State: State = {
  itens = {
    { id = "1", nome = "api-gateway", ambiente = "produção", replicas = 3 },
    { id = "2", nome = "fila-jobs", ambiente = "produção", replicas = 2 },
    { id = "3", nome = "replica-fria", ambiente = "homolog", replicas = 1 },
  },
  seq = 4,
}

return State
