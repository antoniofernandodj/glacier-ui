--!strict
-- Toda a lógica desta tela. O `main.rs` só registra o template.
--
-- O ponto do exemplo: os `<dateedit>`/`<timeedit>` estão com `onChange`, e isso
-- inverte quem manda. Sem `onChange` o widget grava a chave sozinho; com ele, o
-- widget só avisa — quem decide se o valor entra é este script. É o mesmo
-- contrato do `<TextInput>`, e é o que torna possível recusar um valor.
--
-- O segundo ponto é o global `date` (prelúdio, `src/luau/prelude.luau`): as
-- regras aqui embaixo são de negócio, mas as contas que elas fazem — quantas
-- noites, que dia da semana, um dia antes — não são. Antes da 0.72 este exemplo
-- carregava um `days_from_civil` escrito à mão só para não puxar uma crate de
-- data para dentro do motor; ele não precisa mais, e o motor continua sem a
-- crate: o `date` é Luau puro sobre strings ISO, e o relógio sai do `os.date`
-- que o próprio Luau já tem.
--
-- O `.luaurc` da raiz põe o projeto em `languageMode: strict`, então vale
-- anotar: `ctx` é declarado como `{ [string]: string }`, e é dele que sai o
-- tipo de tudo que atravessa a fronteira com o motor.
--
-- Nota sobre o `luau-lsp analyze`: ele acusa `FunctionUnused` em todo handler
-- daqui. É um falso positivo estrutural desta arquitetura — quem chama `init`,
-- `set_checkin` e afins é o motor, **pelo nome**, do lado Rust; renomear com `_`
-- para silenciar quebraria a busca. Todos os scripts de `examples/` têm o mesmo
-- ruído. O que **não** se tolera é `TypeError`.

-- ── Exibição ────────────────────────────────────────────────────────────────
--
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

-- `DD/MM` mais o dia da semana. A chave continua ISO o tempo todo: isto é só o
-- que vai para a tela, a mesma separação que o widget faz entre o valor e o
-- `format="br"`.
local function por_extenso(iso: string): string
    local dia: number? = date.weekday(iso)
    if dia == nil then
        return "—"
    end
    return `{date.format(iso, "DD/MM")} ({SEMANA[dia]})`
end

-- ── Estado derivado ─────────────────────────────────────────────────────────

local function recalcular(): ()
    -- Uma chamada faz o que antes eram três funções e vinte linhas de
    -- aritmética civil. `diff` conta dias de CALENDÁRIO — a hora não entra,
    -- que é exatamente como uma diária é contada.
    local noites: number? = date.diff(ctx.checkout, ctx.checkin)
    if noites == nil then
        ctx.resumo = "—"
        ctx.periodo = ""
        return
    end

    local plural: string = if noites == 1 then "noite" else "noites"
    ctx.resumo = `{noites} {plural} · chegada às {ctx.chegada} · lembrete {date.format(ctx.lembrete, "DD/MM HHhmm")}`
    ctx.periodo = `{por_extenso(ctx.checkin)} → {por_extenso(ctx.checkout)}`
end

local function recusar(motivo: string): ()
    ctx.aviso = motivo
    -- O valor recusado simplesmente não é gravado; como o campo desenha a
    -- partir da chave, ele volta sozinho ao valor anterior no próximo quadro.
    toast({ message = motivo, kind = "warning", title = "Reserva" })
end

local function aceitar(): ()
    ctx.aviso = ""
    recalcular()
end

-- ── Handlers (chamados pelo motor, pelo nome) ───────────────────────────────

function init(): ()
    -- Semeado a partir do relógio do sistema, não de datas fixas: `today` e
    -- `add` já devolvem no formato que o `<dateedit>` espera, então o valor sai
    -- daqui e entra no widget sem conversão nenhuma no meio.
    ctx.checkin = date.today()
    ctx.checkout = date.add(ctx.checkin, { days = 2 })
    ctx.chegada = "14:00"
    -- Véspera da entrada, às 18h — a forma da entrada manda: como a string tem
    -- data e hora, a saída também tem.
    ctx.lembrete = date.add(`{ctx.checkin} 18:00`, { days = -1 })
    ctx.aviso = ""
    recalcular()
end

-- Duas regras, e é aqui que o `date` paga o aluguel: `is_before` compara pelo
-- INSTANTE, não pelo texto. Comparar as strings direto funcionaria entre dois
-- `<dateedit>` (ISO ordena), mas quebra assim que uma das pontas tem hora —
-- `"2026-09-10 08:00" > "2026-09-10"` é verdadeiro só porque a string é mais
-- longa. Uma comparação só, sem recortar nada antes.
function set_checkin(novo: string): ()
    if date.is_before(novo, date.today()) then
        recusar("A entrada não pode ser no passado.")
        return
    end
    if not date.is_before(novo, ctx.checkout) then
        recusar("A entrada precisa ser antes da saída.")
        return
    end
    ctx.checkin = novo
    aceitar()
end

-- Mesma regra pelo outro lado.
function set_checkout(novo: string): ()
    if not date.is_after(novo, ctx.checkin) then
        recusar("A saída precisa ser depois da entrada.")
        return
    end
    ctx.checkout = novo
    aceitar()
end

-- O lembrete carrega hora e o check-in não, e o `is_after` resolve os dois
-- formatos sem o recorte manual que este exemplo fazia antes: uma data pura
-- vale a meia-noite dela, então "até o dia da entrada" cai naturalmente.
function set_lembrete(novo: string): ()
    if date.is_after(novo, `{ctx.checkin} 23:59`) then
        recusar("O lembrete tem de ser até o dia da entrada.")
        return
    end
    ctx.lembrete = novo
    aceitar()
end

-- A hora não tem regra de ordem: entra sempre. Mesmo assim passa por aqui,
-- porque é o script que grava — o widget delegou.
function set_chegada(nova: string): ()
    ctx.chegada = nova
    aceitar()
end

-- Presets escrevem a chave direto, sem passar pelo widget: para o motor não há
-- diferença entre a escrita de um handler e a de qualquer outro.
function turno_manha(): ()
    ctx.chegada = "08:00"
    recalcular()
end

function turno_tarde(): ()
    ctx.chegada = "14:00"
    recalcular()
end

function turno_noite(): ()
    ctx.chegada = "20:00"
    recalcular()
end

-- Atalhos de período: `add` sobre o valor corrente, na mesma forma dele. É o
-- caso em que a aritmética não é do widget nem do negócio — é só data.
function mais_uma_noite(): ()
    ctx.checkout = date.add(ctx.checkout, { days = 1 })
    aceitar()
end

-- Meses andam pelo calendário e grudam no fim: 31/01 + 1 mês é 28/02. Vale
-- experimentar com uma entrada em 31 de janeiro.
function daqui_um_mes(): ()
    local entrada: string = date.add(ctx.checkin, { months = 1 })
    ctx.checkout = date.add(entrada, { days = date.diff(ctx.checkout, ctx.checkin) })
    ctx.checkin = entrada
    ctx.lembrete = date.add(`{date.date_of(ctx.checkin)} 18:00`, { days = -1 })
    aceitar()
end
