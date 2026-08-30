--!strict
-- state.luau — o estado mutável compartilhado pelos handlers.
--
-- `require` é cacheado pelo CAMINHO resolvido do arquivo, então todos os
-- handlers recebem ESTA MESMA tabela: o que um deles muda, os outros veem. É a
-- contraparte do `ctx`, que só guarda strings (o que o markup lê) — aqui ficam
-- as estruturas de verdade.

export type Item = {
    id: string,
    nome: string,
    ativo: boolean,
}

export type State = {
    itens: { Item },
    carregando: boolean,
}

local State: State = {
    itens = {},
    carregando = false,
}

return State
