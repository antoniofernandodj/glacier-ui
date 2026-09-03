--!strict
-- Toda a lógica da tela da Onda 3. O `main.rs` só registra o template.
--
-- O exemplo existe para mostrar as duas metades do contrato do calendário, que
-- é o mesmo do `<textinput>` e do `<dateedit>`:
--
--   * **sem `onChange`** o widget grava a chave sozinho — o `<calendar>` da
--     primeira caixa não tem uma linha de script atrás dele;
--   * **com `onChange`** ele só avisa, e quem decide se o valor entra é este
--     arquivo. É o que permite recusar um fim de semana, contar as noites de
--     uma reserva ou derivar um texto por extenso.
--
-- E existe uma segunda razão para ele ser em Luau: a prop `today`. O realce de
-- hoje é PROP, não relógio — o motor não lê a hora do sistema em lugar nenhum
-- (`PLANO_WIDGETS.md` §4), e `date.today()` do prelúdio é a linha que fecha
-- esse buraco sem uma crate de data no motor.
--
-- Nota sobre o `luau-lsp analyze`: ele acusa `FunctionUnused` em todo handler
-- daqui. É falso positivo estrutural — quem chama `init`, `set_entrega` e
-- afins é o motor, pelo nome, do lado Rust. O que não se tolera é `TypeError`.

-- 1 = domingo, a mesma base do `date.weekday` e do `os.date("*t").wday`.
local SEMANA: { string } = {
    "domingo",
    "segunda",
    "terça",
    "quarta",
    "quinta",
    "sexta",
    "sábado",
}

local MESES: { string } = {
    "janeiro",
    "fevereiro",
    "março",
    "abril",
    "maio",
    "junho",
    "julho",
    "agosto",
    "setembro",
    "outubro",
    "novembro",
    "dezembro",
}

-- `DD/MM/YYYY` mais o dia da semana. A chave continua ISO o tempo todo: isto é
-- só o que vai para a tela — a mesma separação entre valor e exibição que o
-- `format="br"` do `<dateedit>` faz.
local function por_extenso(iso: string): string
    local dia: number? = date.weekday(iso)
    if dia == nil then
        return "—"
    end
    return `{date.format(iso, "DD/MM/YYYY")} ({SEMANA[dia]})`
end

-- ── Handlers (chamados pelo motor, pelo nome) ───────────────────────────────

function init(): ()
    ctx.abas = '[{"id":"dia","label":"calendar"},' ..
        '{"id":"mes","label":"monthyearpicker"},' ..
        '{"id":"intervalo","label":"daterangepicker"}]'
    ctx.aba = "dia"

    -- A única linha que o `today` custa. Tudo abaixo sai dela.
    ctx.hoje = date.today()
    -- `min`/`max` recortam a grade: os dias fora saem inertes e apagados.
    ctx.limite = date.add(ctx.hoje, { months = 3 })

    ctx.dia = ctx.hoje
    ctx.entrega = ""
    ctx.entrega_txt = "— nenhuma —"
    ctx.aviso = ""

    ctx.competencia = date.format(ctx.hoje, "YYYY-MM")
    ctx.competencia_txt = MESES[tonumber(date.format(ctx.hoje, "MM")) :: number]
    ctx.exercicio = date.format(ctx.hoje, "YYYY")

    ctx.entrada = ""
    ctx.saida = ""
    ctx.entrada_txt = "—"
    ctx.saida_txt = "—"
    ctx.noites = "Escolha a entrada e a saída."

    -- A chave que dirige o mês visível do último calendário da terceira aba.
    -- O formato é `YYYY-MM|<nível>` — o mesmo que o motor guarda em
    -- `__cal_<chave>` quando o app NÃO passa `month=`.
    ctx.mes_visivel = date.format(ctx.hoje, "YYYY-MM") .. "|d"

    ctx.status = "Pronto"
end

-- A regra que só existe porque o widget delegou: entrega não sai no fim de
-- semana. Recusar é simplesmente não gravar — como o campo desenha a partir da
-- chave, ele volta sozinho ao valor anterior no quadro seguinte.
function set_entrega(novo: string): ()
    local dia: number? = date.weekday(novo)
    if dia == 1 or dia == 7 then
        ctx.aviso = "Não entregamos aos fins de semana."
        toast({ message = ctx.aviso, kind = "warning", title = "Entrega" })
        return
    end
    ctx.entrega = novo
    ctx.entrega_txt = por_extenso(novo)
    ctx.aviso = ""
    ctx.status = `Entrega marcada para {ctx.entrega_txt}`
end

-- O modo mês grava `YYYY-MM` — sem dia, porque não há dia para escolher. O
-- `date.parse` lê assim mesmo (o dia vira 1), então o mês por extenso sai de
-- uma chamada só.
function set_competencia(nova: string): ()
    ctx.competencia = nova
    local p = date.parse(nova)
    ctx.competencia_txt = if p == nil then "—" else `{MESES[p.month]} de {p.year}`
    ctx.status = `Competência: {ctx.competencia_txt}`
end

-- O intervalo delega com as DUAS pontas numa string só (`"<início> <fim>"`), e
-- o fim vem vazio no primeiro clique. Quem grava as duas chaves separadas é
-- este handler — que é justamente o ponto de as chaves serem separadas: o
-- `date.diff` lê as duas direto, sem recortar nada.
function set_periodo(par: string): ()
    local inicio: string = string.match(par, "^%S+") or ""
    local fim: string = string.match(par, "%S+$") or ""
    if fim == inicio then
        fim = ""
    end

    ctx.entrada = inicio
    ctx.saida = fim
    ctx.entrada_txt = if inicio == "" then "—" else por_extenso(inicio)
    ctx.saida_txt = if fim == "" then "—" else por_extenso(fim)

    local noites: number? = if fim == "" then nil else date.diff(fim, inicio)
    if noites == nil then
        ctx.noites = "Escolha a saída para fechar o intervalo."
        ctx.status = "Início marcado"
        return
    end
    local plural: string = if noites == 1 then "noite" else "noites"
    ctx.noites = `{noites} {plural}`
    ctx.status = `Reserva de {noites} {plural}`
end

function limpar(): ()
    ctx.entrada = ""
    ctx.saida = ""
    ctx.entrada_txt = "—"
    ctx.saida_txt = "—"
    ctx.noites = "Escolha a entrada e a saída."
    ctx.status = "Intervalo limpo"
end

-- Dirigir o mês visível de fora é escrever a chave que `month=` nomeia. O
-- sufixo `|d` é o degrau da escada; sem ele o calendário abre no nível que a
-- tag pedir.
function ir_janeiro(): ()
    ctx.mes_visivel = `{date.format(ctx.hoje, "YYYY")}-01|d`
    ctx.status = "Grade em janeiro"
end

function ir_dezembro(): ()
    ctx.mes_visivel = `{date.format(ctx.hoje, "YYYY")}-12|d`
    ctx.status = "Grade em dezembro"
end
