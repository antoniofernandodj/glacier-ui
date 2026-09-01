--!strict
-- Toda a lógica desta tela. O `main.rs` só registra o template.
--
-- O ponto do exemplo: os `<dateedit>`/`<timeedit>` estão com `onChange`, e isso
-- inverte quem manda. Sem `onChange` o widget grava a chave sozinho; com ele, o
-- widget só avisa — quem decide se o valor entra é este script. É o mesmo
-- contrato do `<TextInput>`, e é o que torna possível recusar um valor.
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

-- ── Comparação ──────────────────────────────────────────────────────────────
--
-- O widget grava SEMPRE em ISO (`YYYY-MM-DD`, `HH:MM`), independentemente de o
-- campo estar exibindo `DD/MM/YYYY`. E ISO ordena como texto: comparar duas
-- strings dessas É comparar as datas, sem parse e sem biblioteca. É a razão
-- prática de o widget nunca gravar no formato de exibição.
local function depois_de(a: string, b: string): boolean
    return a > b
end

-- ── Calendário ──────────────────────────────────────────────────────────────
--
-- Aritmética à mão de propósito: o motor não depende de `chrono`/`time` (a
-- decisão segue em aberto no PLANO_WIDGETS.md §4), e um exemplo não deveria ser
-- o que força essa escolha.

-- Dias desde a época Unix, pelo `days_from_civil` de Howard Hinnant: tratar
-- março como primeiro mês do ano joga o bissexto para o fim e elimina o caso
-- especial de fevereiro.
local function dias_civis(ano: number, mes: number, dia: number): number
    local y: number = if mes <= 2 then ano - 1 else ano
    local era: number = math.floor(y / 400)
    local yoe: number = y - era * 400
    local mp: number = (mes + 9) % 12
    local doy: number = math.floor((153 * mp + 2) / 5) + dia - 1
    local doe: number = yoe * 365 + math.floor(yoe / 4) - math.floor(yoe / 100) + doy
    return era * 146097 + doe - 719468
end

-- `nil` quando a string não é uma data ISO — o campo pode estar vazio antes do
-- `init`, e o resumo precisa sobreviver a isso.
local function para_dias(iso: string): number?
    local a, m, d = string.match(iso, "(%d+)-(%d+)-(%d+)")
    if a == nil or m == nil or d == nil then
        return nil
    end
    local ano, mes, dia = tonumber(a), tonumber(m), tonumber(d)
    if ano == nil or mes == nil or dia == nil then
        return nil
    end
    return dias_civis(ano, mes, dia)
end

-- A parte de data de um `YYYY-MM-DD HH:MM`. Existe porque comparar um valor de
-- `<datetimeedit>` com um de `<dateedit>` como texto puro dá errado:
-- `"2026-09-10 08:00" > "2026-09-10"` é verdadeiro (a string é mais longa),
-- ainda que os dois sejam o mesmo dia. ISO só ordena entre formatos **iguais** —
-- recortar antes de comparar é o que restabelece isso.
local function dia_de(iso: string): string
    return string.sub(iso, 1, 10)
end

-- ── Estado derivado ─────────────────────────────────────────────────────────

local function recalcular(): ()
    local entrada: number? = para_dias(ctx.checkin)
    local saida: number? = para_dias(ctx.checkout)
    -- Dois `if` separados, e não `if not entrada or not saida`: o refinamento
    -- de tipo do Luau estreita uma variável por vez, então a forma com `or`
    -- deixa as duas como `number?` depois do `return` e a subtração não compila.
    if entrada == nil then
        ctx.resumo = "—"
        return
    end
    if saida == nil then
        ctx.resumo = "—"
        return
    end

    local noites: number = saida - entrada
    local plural: string = if noites == 1 then "noite" else "noites"
    ctx.resumo = string.format(
        "%d %s · entrada %s às %s · saída %s · lembrete %s",
        noites,
        plural,
        ctx.checkin,
        ctx.chegada,
        ctx.checkout,
        ctx.lembrete
    )
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
    ctx.checkin = "2026-09-10"
    ctx.checkout = "2026-09-12"
    ctx.chegada = "14:00"
    ctx.lembrete = "2026-09-09 18:00"
    ctx.aviso = ""
    recalcular()
end

-- Só aceita a entrada se ela continuar antes da saída. As duas regras são a
-- mesma comparação vista dos dois lados, e por isso passam pelo mesmo helper.
function set_checkin(novo: string): ()
    if not depois_de(ctx.checkout, novo) then
        recusar("A entrada precisa ser antes da saída.")
        return
    end
    ctx.checkin = novo
    aceitar()
end

-- Mesma regra pelo outro lado.
function set_checkout(novo: string): ()
    if not depois_de(novo, ctx.checkin) then
        recusar("A saída precisa ser depois da entrada.")
        return
    end
    ctx.checkout = novo
    aceitar()
end

-- O lembrete tem de cair até o dia da entrada. Repare que a comparação é entre
-- os **dias**, não entre as strings inteiras: o lembrete carrega hora e o
-- check-in não.
function set_lembrete(novo: string): ()
    if depois_de(dia_de(novo), ctx.checkin) then
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
