--!strict

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
