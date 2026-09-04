--!strict
--
-- A Onda 6 inteira em Luau. O `main.rs` ao lado não tem `impl Component`: ele
-- registra o `.gv` e sai de cena.
--
-- O que o `examples/onda6` faz em Rust — `init` e duas ações — mora aqui, e a
-- comparação lado a lado é o ponto do exemplo. Duas diferenças aparecem logo:
--
--  1. **Não há `self`.** O `Onda6` de Rust é um struct; aqui o estado que
--     sobrevive entre ações mora no `ctx` (o que a tela precisa ver) ou num
--     `local` do módulo (o que ela não precisa).
--  2. **`ctx` guarda texto.** Atribuir uma tabela a uma chave a codifica em
--     JSON — e aqui isso rende mais do que nas ondas anteriores, porque esta
--     tela passa DUAS estruturas para os widgets: as linhas da tabela e a
--     árvore aninhada. Do lado Rust as duas exigem um `serde_json::json!`
--     construído à mão; aqui são tabelas literais.
--
-- O que NÃO muda entre as duas versões é o mais importante: nenhum dos seis
-- widgets desta onda pede uma linha de script. Ordenar uma coluna, inverter a
-- ordem, escolher uma linha, marcar várias, arrastar a alça de uma coluna,
-- abrir um nó da árvore, descer um nível no Miller — tudo isso o widget faz
-- escrevendo nas chaves que este arquivo nomeou. O que está aqui é só o que é
-- do APP: o que "limpar" e "fechar tudo" significam nesta tela.

-- As linhas da tabela. Num app de verdade viriam de um backend; aqui são
-- literais para o exemplo não depender de rede.
--
-- A coluna `replicas` é a que prova a ordenação numérica: ordenada como texto,
-- "10" viria antes de "9".
local SERVICOS = {
    { id = "api", nome = "API pública", ambiente = "produção", replicas = "12", uso = "41" },
    { id = "db", nome = "Banco primário", ambiente = "produção", replicas = "3", uso = "78" },
    { id = "cache", nome = "Redis", ambiente = "produção", replicas = "9", uso = "23" },
    { id = "fila", nome = "Fila de eventos", ambiente = "produção", replicas = "4", uso = "55" },
    { id = "cdn", nome = "CDN", ambiente = "borda", replicas = "24", uso = "12" },
    { id = "auth", nome = "Autenticação", ambiente = "produção", replicas = "6", uso = "62" },
    { id = "mail", nome = "Disparo de e-mail", ambiente = "homologação", replicas = "2", uso = "8" },
    { id = "logs", nome = "Coletor de logs", ambiente = "produção", replicas = "10", uso = "91" },
    { id = "cron", nome = "Agendador", ambiente = "homologação", replicas = "1", uso = "5" },
    { id = "busca", nome = "Índice de busca", ambiente = "produção", replicas = "8", uso = "70" },
}

-- As colunas: a mesma convenção `items="chave"` que o `<menu>` e o `<tabbar>` já
-- usavam — um array numa chave. O que a Onda 6 acrescentou não foi a ligação,
-- foi a MEDIÇÃO.
--
-- `align = "right"` numa coluna de número não é enfeite: é o que faz os
-- algarismos alinharem pela unidade, que é como se compara uma coluna de
-- contagem a olho.
local COLUNAS = {
    { key = "nome", label = "Serviço", width = "fill" },
    { key = "ambiente", label = "Ambiente", width = "140" },
    { key = "replicas", label = "Réplicas", width = "90", align = "right" },
    { key = "uso", label = "Uso %", width = "90", align = "right" },
}

-- A árvore: `{id, label, items}` aninhado, a mesma forma que o `<menu items>` já
-- usava para submenus. É aqui que a versão Luau fica visivelmente mais curta —
-- do lado Rust isto é um `serde_json::json!` de trinta linhas.
local ARVORE = {
    {
        id = "projeto",
        label = "projeto",
        items = {
            {
                id = "src",
                label = "src",
                items = {
                    { id = "main.rs", label = "main.rs" },
                    { id = "widget.rs", label = "widget.rs" },
                    { id = "grid.rs", label = "grid.rs" },
                    {
                        id = "builtins",
                        label = "builtins",
                        items = {
                            { id = "tabs.rs", label = "tabs.rs" },
                            { id = "drawer.rs", label = "drawer.rs" },
                        },
                    },
                },
            },
            {
                id = "examples",
                label = "examples",
                items = {
                    { id = "onda5", label = "onda5" },
                    { id = "onda6", label = "onda6" },
                },
            },
            { id = "Cargo.toml", label = "Cargo.toml" },
            { id = "README.md", label = "README.md" },
        },
    },
}

function init()
    ctx.abas = {
        { id = "layout", label = "Grid + Flow" },
        { id = "tabela", label = "TableView + TableHeader" },
        { id = "arvore", label = "TreeView + ColumnView" },
    }
    ctx.aba = "layout"

    ctx.colunas = COLUNAS
    ctx.servicos = SERVICOS

    -- As chaves que a tabela escreve sozinha. Todas nomeadas pelo app — é o
    -- padrão do `SpinBox`, e é o que permite duas tabelas na mesma tela.
    ctx.servico = "api"
    ctx.marcados = "api,cache"
    ctx.ordem = "nome asc"
    -- As larguras arrastadas. Semeada com o mesmo formato de `columns`, o que
    -- permite ao app salvar e restaurar o layout do usuário sem conhecer nada
    -- da estrutura interna do widget.
    ctx.larguras = "fill 140 90 90"

    ctx.arvore = ARVORE
    -- O conjunto de nós abertos numa string, e os dois caminhos escolhidos.
    -- `projeto/src` é um CAMINHO, não um id: é assim que um `id` repetido em
    -- ramos diferentes não colide.
    ctx.abertos = "projeto,projeto/src"
    ctx.no = "projeto/src"
    ctx.caminho = "projeto/src"

    ctx.status = "Pronto"
end

function limpar_marcados()
    ctx.marcados = ""
    ctx.status = "Seleção múltipla limpa"
end

-- Fechar tudo é zerar o conjunto — e é isso que torna o conjunto nomeado tão
-- prático: uma string vazia é "nenhum nó aberto", sem um bit por nó para
-- percorrer.
function fechar_tudo()
    ctx.abertos = ""
    ctx.status = "Árvore recolhida"
end
