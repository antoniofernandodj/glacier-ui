--!strict
--!nolint FunctionUnused
-- (o motor chama estas funções pelo nome; o lsp não vê a chamada)

-- Fase gpui-kit em Luau: `alert`, `kbd`, `descriptionlist` e `command`.
-- O markup desenha; o script só escreve chaves.

type Servico = { id: string, nome: string, imagem: string, porta: number, replicas: number, ativo: boolean }
type Comando = { id: string, label: string, hint: string?, shortcut: string?, keywords: string? }

-- O DEPÓSITO: tipado, com a forma real do dado.
local servicos: { Servico } = {
    { id = "api",  nome = "api de borda",      imagem = "nginx:1.27",   porta = 8080, replicas = 3, ativo = true },
    { id = "fila", nome = "fila de trabalhos", imagem = "redis:7",      porta = 6379, replicas = 1, ativo = true },
    { id = "frio", nome = "réplica fria",      imagem = "postgres:17",  porta = 5432, replicas = 0, ativo = false },
}
local atual = 1

-- Os comandos da paleta: um por serviço (selecionar) mais as ações fixas.
local function comandos(): { Comando }
    local lista: { Comando } = {
        { id = "alternar", label = "Ligar/desligar o serviço", shortcut = "Ctrl+L", keywords = "parar iniciar" },
        { id = "escalar",  label = "Escalar +1 réplica",       hint = "réplicas", keywords = "scale" },
        { id = "erro",     label = "Simular uma falha",        keywords = "erro alerta" },
    }
    for i, s in servicos do
        table.insert(lista, { id = `ir_{i}`, label = `Abrir {s.nome}`, hint = s.imagem })
    end
    return lista
end

-- Um aviso que fica na tela até o × (ou até outro aviso substituí-lo).
local function avisar(tipo: string, titulo: string, detalhe: string): ()
    ctx.aviso_tipo = tipo
    ctx.aviso = titulo
    ctx.aviso_detalhe = detalhe
end

-- A VITRINE: projeta o depósito no ctx.
local function publicar(): ()
    local s = servicos[atual]
    ctx.ficha = json.encode(json.array({
        { label = "Nome",     value = s.nome },
        { label = "Imagem",   value = s.imagem },
        { label = "Porta",    value = tostring(s.porta) },
        { label = "Réplicas", value = tostring(s.replicas) },
        { label = "Estado",   value = if s.ativo then "ativo" else "parado" },
    }))
    ctx.comandos = json.encode(json.array(comandos()))
end

function init(): ()
    ctx.busca = ""
    ctx.paleta = false
    ctx.ultimo = "nenhum"
    ctx.atalhos = json.encode(json.array({
        { label = "Ctrl+K", value = "abre e fecha a paleta" },
        { label = "Esc",    value = "fecha a paleta" },
    }))
    avisar("info", "Bem-vindo", "Abra a paleta com Ctrl+K e digite parte do nome de um comando.")
    publicar()
end

function alternar_paleta(): ()
    ctx.paleta = not (ctx.paleta == true or ctx.paleta == "true")
    ctx.busca = ""
end

function fechar_paleta(): ()
    ctx.paleta = false
    ctx.busca = ""
end

function fechar_aviso(): ()
    ctx.aviso = ""
end

-- `on_select = executar` — o `id` do item chega como argumento.
function executar(id: string): ()
    ctx.ultimo = id
    fechar_paleta()

    local s = servicos[atual]
    if id == "alternar" then
        s.ativo = not s.ativo
        avisar("success", `{s.nome}: {if s.ativo then "ativo" else "parado"}`, "")
    elseif id == "escalar" then
        s.replicas += 1
        avisar("info", `{s.nome}: {s.replicas} réplicas`, "")
    elseif id == "erro" then
        avisar("error", "Falha no deploy", `{s.nome} não respondeu ao health check.`)
    elseif string.sub(id, 1, 3) == "ir_" then
        atual = tonumber(string.sub(id, 4)) or atual
        fechar_aviso()
    end
    publicar()
end
