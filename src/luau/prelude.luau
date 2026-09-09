-- Declara e inicializa
_G.__logs = {}

function fetch(url, opts)
    return coroutine.yield({
        __glacier_fetch = true,
        url = url,
        opts = opts or {}
    })
end

-- Server-Sent Events. NÃO suspende: cede um pedido de abertura, o motor
-- registra o stream e retoma na hora devolvendo um handle com `:close()`.
-- `opts` = { headers?, on_open?, on_message?, on_error?, on_close? }, cada
-- callback uma FUNÇÃO (closure e upvalue funcionam). Um nome de função global
-- também é aceito, como atalho — ver `handler_key` em src/luau/mod.rs.
function sse(url, opts)
    local id = coroutine.yield({
        __glacier_stream_open = true,
        kind = "sse",
        url = url,
        opts = opts or {}
    })
    return {
        id = id,
        close = function(self)
            return coroutine.yield({
                __glacier_stream_cmd = true,
                id = self.id,
                cmd = "close"
            })
        end,
    }
end

-- Toast (notificação efêmera) a partir do Luau. NÃO suspende: cede um pedido, o
-- motor o exibe (via Context::show_toast) e retoma na hora. `opts` = string
-- (mensagem, kind "info") ou tabela `{ message, kind?, title? }` — kind é
-- "info"|"success"|"warning"|"error".
function toast(opts)
    if type(opts) == "string" then
        opts = { message = opts }
    end
    return coroutine.yield({
        __glacier_toast = true,
        message = opts.message,
        kind = opts.kind,
        title = opts.title,
    })
end

-- Notificação NATIVA DO SISTEMA OPERACIONAL (via Context::notify). Diferente do
-- `toast` (efêmero, desenhado dentro da própria janela), esta vai para a central
-- de notificações do SO (freedesktop/D-Bus no Linux, WinRT no Windows,
-- NSUserNotification no macOS) e aparece mesmo com o app minimizado. NÃO suspende:
-- cede um pedido e retoma na hora; o motor a entrega fora da thread de UI. `opts`
-- = string (vira o `body`, sem título) ou tabela `{ title?, body?, app_name?,
-- icon? }`. `app_name` = nome do app exibido/associado (default: nome do
-- executável); útil quando o desktop filtra notificações pela identidade do app.
-- `icon` = nome de ícone do tema ou caminho.
function notify(opts)
    if type(opts) == "string" then
        opts = { body = opts }
    end
    return coroutine.yield({
        __glacier_notify = true,
        title = opts.title,
        body = opts.body,
        app_name = opts.app_name,
        icon = opts.icon,
    })
end

-- Diálogo modal de confirmação. SUSPENDE (como `fetch`): cede um pedido, o motor
-- mostra o diálogo e só retoma quando o usuário escolhe um botão, devolvendo um
-- booleano — `true` se confirmou, `false` se cancelou/dispensou. Deixa o fluxo
-- síncrono, sem função de callback separada:
--   if confirm({ title = "Remover?", message = "...", confirm_label = "Remover",
--                destructive = true }) then
--       -- fazer a ação
--   end
-- `opts = { title, message, confirm_label?, cancel_label?, destructive? }`.
function confirm(opts)
    return coroutine.yield({
        __glacier_dialog = true,
        title = opts.title,
        message = opts.message,
        confirm_label = opts.confirm_label,
        cancel_label = opts.cancel_label,
        destructive = opts.destructive,
    })
end

-- Diálogo que PEDE UM VALOR — o `QInputDialog`. SUSPENDE como o `confirm`, e
-- devolve o que o usuário digitou, ou `nil` se ele desistiu:
--   local nome = prompt({ title = "Renomear", label = "Novo nome", value = atual })
--   if nome then ctx.servico = nome end
--
-- O `nil` da desistência é a mesma convenção do `open_file()` cancelado, e é o
-- que separa "não respondeu" de "respondeu vazio" (que chega como "").
--
-- `kind` escolhe o campo, e são as quatro variantes estáticas do QInputDialog:
--   "text"   (default) -> QInputDialog::getText,   campo de texto
--   "int"              -> QInputDialog::getInt,    spinbox sem casas decimais
--   "double"           -> QInputDialog::getDouble, spinbox com casas decimais
--   "item"             -> QInputDialog::getItem,   lista suspensa
--
-- `opts = { title, message?, label?, value?, placeholder?, kind?,
--           min?, max?, step?, decimals?, items?,
--           confirm_label?, cancel_label? }`.
--
-- O valor volta sempre como STRING, inclusive nos dois numéricos — quem quer o
-- número chama `tonumber()`. É a mesma regra do resto do motor: o contexto
-- guarda texto, e a conversão é de quem sabe o que o valor significa.
function prompt(opts)
    return coroutine.yield({
        __glacier_dialog = true,
        kind = opts.kind or "text",
        title = opts.title,
        message = opts.message,
        label = opts.label,
        value = opts.value,
        placeholder = opts.placeholder,
        min = opts.min,
        max = opts.max,
        step = opts.step,
        decimals = opts.decimals,
        items = opts.items,
        confirm_label = opts.confirm_label,
        cancel_label = opts.cancel_label,
    })
end

-- Diálogo de COR — o `QColorDialog`. SUSPENDE como o `prompt`, e devolve a cor
-- escolhida em `#rrggbb`, ou `nil` se o usuário desistiu:
--   local cor = pick_color({ title = "Cor do rótulo", value = ctx.cor })
--   if cor then ctx.cor = cor end
--
-- `opts = { title, message?, value?, size?, confirm_label?, cancel_label? }`.
--
-- É um `prompt` cujo campo é uma roda: mesma porta, mesmo retorno, mesma
-- convenção de desistência. `size` é o lado da roda em pixels (default 220).
function pick_color(opts)
    opts = opts or {}
    return coroutine.yield({
        __glacier_dialog = true,
        kind = "color",
        title = opts.title,
        message = opts.message,
        value = opts.value,
        size = opts.size,
        confirm_label = opts.confirm_label,
        cancel_label = opts.cancel_label,
    })
end

-- Diálogo de FONTE — o `QFontDialog`. SUSPENDE como o `pick_color`, e devolve a
-- família escolhida (uma string), ou `nil` se o usuário desistiu:
--   local f = pick_font({ title = "Fonte do editor", value = ctx.fonte, size = 15 })
--   if f then ctx.fonte = f end
--
-- `opts = { title, message?, value?, size?, confirm_label?, cancel_label? }`.
--
-- `value` é a família inicial; `size` semeia o campo de tamanho (default 16).
-- O tamanho e o estilo são PRÉ-VISUALIZAÇÃO — o retorno é só a família. A
-- lista é a mesma `__fonts` do `<fontselect>` (registro de `src/fonts.rs`;
-- com a feature `system-fonts`, inclui as do SO).
function pick_font(opts)
    opts = opts or {}
    return coroutine.yield({
        __glacier_dialog = true,
        kind = "font",
        title = opts.title,
        message = opts.message,
        value = opts.value,
        size = opts.size,
        confirm_label = opts.confirm_label,
        cancel_label = opts.cancel_label,
    })
end

-- Diálogo de PROGRESSO — o `QProgressDialog`. NÃO suspende: ele acompanha um
-- trabalho que continua rodando, e parar a corrotina aqui pararia justamente o
-- que ele mostra.
--
--   progress({ title = "Baixando", label = "conectando…", max = #pacotes,
--              on_cancel = "cancelar_download" })
--   for i, p in ipairs(pacotes) do
--       if ctx.cancelado == "true" then break end
--       progress_set(i, "baixando " .. p.nome)
--       baixar(p)
--   end
--   progress_close()
--
-- `opts = { title, message?, label?, value?, max?, busy_label?,
--           on_cancel?, cancel_label? }`.
--
-- Sem `value`, a barra nasce INDETERMINADA (um spinner) — que é o estado certo
-- de quem acabou de abrir e ainda não sabe o primeiro número.
-- Sem `on_cancel`, o diálogo não tem botão: é a etapa que não dá para
-- interromper, e um "Cancelar" que não cancela seria pior do que botão nenhum.
function progress(opts)
    return coroutine.yield({
        __glacier_dialog_open = true,
        title = opts.title,
        message = opts.message,
        label = opts.label,
        value = opts.value,
        max = opts.max,
        busy_label = opts.busy_label,
        on_cancel = opts.on_cancel,
        cancel_label = opts.cancel_label,
    })
end

-- Atualiza o progresso aberto. `value` nil volta ao indeterminado (o spinner);
-- `label` é opcional e, quando dado, troca a linha de texto acima da barra.
--
-- Escreve nas mesmas chaves que o corpo lê — não há caminho especial: o número
-- anda pelo mesmo lugar de qualquer outro valor do motor, e é isso que impede
-- este diálogo de virar um caso à parte.
function progress_set(value, label)
    if value == nil then
        -- Volta ao indeterminado: as duas chaves em branco, e o corpo desenha
        -- o spinner.
        ctx["__dialog.progresso"] = ""
        ctx["__dialog.pct"] = ""
    else
        ctx["__dialog.progresso"] = tostring(value)
        -- A barra desenha na escala 0–100 porque o `min`/`max` de um
        -- `<progressbar>` é literal: ele não aceita `{interpolação}`. A conta
        -- vive aqui, onde o `max` declarado na abertura está à mão.
        local max = tonumber(ctx["__dialog.max"]) or 100
        local pct = if max > 0 then (tonumber(value) or 0) / max * 100 else 0
        ctx["__dialog.pct"] = tostring(math.clamp(pct, 0, 100))
    end
    if label ~= nil then
        ctx["__dialog.label"] = label
    end
end

-- Fecha o diálogo de progresso (ou qualquer diálogo em exibição).
function progress_close()
    return coroutine.yield({ __glacier_dialog_close = true })
end

-- Diálogo de arquivo/pasta NATIVO DO SO. SUSPENDE (como `confirm`): cede um
-- pedido, o motor mostra o diálogo do SO (fora da thread de UI) e só retoma
-- quando o usuário escolhe ou cancela. `opts` (opcional) = { title?,
-- filters? = {{ name = "Imagens", extensions = {"png","jpg"} }, ...},
-- starting_dir? }.
--
-- open_file    -> string do caminho escolhido, ou nil se cancelou.
-- open_files   -> tabela-array de strings (um ou mais caminhos), ou nil.
-- save_file    -> como open_file; `opts.default_name` sugere um nome inicial.
-- pick_folder  -> como open_file, escolhendo um diretório.
local function file_dialog(mode, opts)
    opts = opts or {}
    return coroutine.yield({
        __glacier_file_dialog = true,
        mode = mode,
        title = opts.title,
        filters = opts.filters,
        starting_dir = opts.starting_dir,
        default_name = opts.default_name,
    })
end

function open_file(opts)
    return file_dialog("open", opts)
end

function open_files(opts)
    return file_dialog("open_multiple", opts)
end

function save_file(opts)
    return file_dialog("save", opts)
end

function pick_folder(opts)
    return file_dialog("directory", opts)
end

-- Navegação. NÃO suspendem: cedem um pedido, o motor troca de tela (via
-- Context::navigate_to/navigate_back) e retoma na hora.
function navigate(screen)
    return coroutine.yield({
        __glacier_nav = true,
        screen = screen,
    })
end

function navigate_back()
    return coroutine.yield({
        __glacier_nav_back = true,
    })
end

-- Abre uma NOVA JANELA. NÃO suspende: cede um pedido, o daemon sobe um motor
-- independente para a janela e retoma na hora. `opts` é uma string (caminho do
-- arquivo) ou uma tabela:
--   open_window("telas/detalhe.gv")
--   open_window({ file = "telas/detalhe.gv", title = "Detalhe", width = 400, height = 300 })
--   open_window({ component = "perfil", title = "Perfil" })  -- componente já registrado
function open_window(opts)
    local t = type(opts) == "table" and opts or { file = opts }
    t.__glacier_window = true
    return coroutine.yield(t)
end

-- Envia uma mensagem para as OUTRAS janelas abertas (não para a própria). NÃO
-- suspende: cede um pedido, o motor o acumula e o daemon o entrega ao
-- `on_broadcast(event, payload)` de cada outra janela, retomando na hora.
-- `payload` é opcional; uma tabela é serializada em JSON automaticamente (a
-- janela receptora a recebe já decodificada de volta numa tabela).
--   broadcast("project_created", { id = "42", name = "api" })
function broadcast(event, payload)
    return coroutine.yield({
        __glacier_broadcast = true,
        event = event,
        payload = payload,
    })
end

-- Fecha a PRÓPRIA janela. NÃO suspende: cede um pedido, o daemon fecha a janela
-- dona deste motor após o turno. Útil logo após um `broadcast(...)` para
-- dispensar uma janela auxiliar (ex.: um formulário) ao concluir.
function close_window()
    return coroutine.yield({
        __glacier_window_close = true,
    })
end

-- Anexa `text` ao FIM do <textarea> ligado a `binding` (a chave usada no seu
-- `value`), de forma INCREMENTAL: o motor faz um insert no fim do buffer do
-- text_editor em vez de recriá-lo, então o scroll é preservado e o custo é
-- O(text), não O(conteúdo todo). Ideal para logs vivos — cada linha nova é
-- appendada no fim, sem o "pulo pro topo" de reescrever ctx[binding] inteiro.
-- O motor também sincroniza ctx[binding] com o texto completo, para `clipboard:`
-- copiar tudo e o conteúdo sobreviver a uma reavaliação. NÃO suspende.
--   append_textarea("meus_logs", "12:00:00 nova linha\n")
function append_textarea(binding, text)
    return coroutine.yield({
        __glacier_editor_append = true,
        binding = binding,
        text = text,
    })
end

-- Temporizador de disparo único (equivalente a setTimeout). NÃO suspende: cede
-- um pedido, o motor agenda `fn` (função ou nome de função global) para daqui
-- a `ms` milissegundos e retoma na hora devolvendo um handle cancelável.
function after(ms, fn)
    local id = coroutine.yield({
        __glacier_after = true,
        ms = ms,
        fn = fn,
    })
    return {
        id = id,
        cancel = function(self)
            return coroutine.yield({
                __glacier_after_cancel = true,
                id = self.id,
            })
        end,
    }
end

-- Temporizador repetitivo (equivalente a setInterval), construído em cima de
-- `after`: a cada disparo, reagenda a si mesmo, até que `:cancel()` seja
-- chamado no handle devolvido (interrompe só as próximas repetições — o
-- disparo em curso, se já cedido, ainda roda).
function every(ms, fn)
    local invoke = fn
    if type(fn) == "string" then
        invoke = function() _G[fn]() end
    end

    local handle = { cancelled = false }
    local function tick()
        if handle.cancelled then
            return
        end
        invoke()
        if not handle.cancelled then
            handle.timer = after(ms, tick)
        end
    end
    handle.timer = after(ms, tick)
    handle.cancel = function(self)
        self.cancelled = true
        if self.timer then
            self.timer:cancel()
        end
    end
    return handle
end

-- Tamanho atual da janela em px lógicos: `{ width, height }`.
function viewport()
    return { width = __glacier_viewport.width, height = __glacier_viewport.height }
end

-- WebSocket. Igual ao `sse` na abertura e nos callbacks; o handle devolvido
-- também tem `:send(texto)`, que escreve na conexão viva.
function websocket(url, opts)
    local id = coroutine.yield({
        __glacier_stream_open = true,
        kind = "ws",
        url = url,
        opts = opts or {}
    })
    return {
        id = id,
        send = function(self, text)
            return coroutine.yield({
                __glacier_stream_cmd = true,
                id = self.id,
                cmd = "send",
                text = text
            })
        end,
        close = function(self)
            return coroutine.yield({
                __glacier_stream_cmd = true,
                id = self.id,
                cmd = "close"
            })
        end,
    }
end

-- ── `date`: aritmética de data e hora sobre strings ISO ─────────────────────
--
-- O motor guarda data e hora como TEXTO ISO — `YYYY-MM-DD`, `HH:MM[:SS]`, ou os
-- dois separados por espaço. É o que os `<dateedit>`/`<timeedit>`/
-- `<datetimeedit>` escrevem na chave de contexto, e é o único formato que
-- atravessa a fronteira com o motor, onde `ctx` é `{[string]: string}`. Este
-- módulo, portanto, recebe strings e devolve strings: não existe um "objeto
-- data" para vazar daqui para uma chave.
--
-- Nada disto é dependência nova. `today`/`now` saem do `os.date` do próprio
-- Luau (hora LOCAL do sistema, via `localtime_r`); o resto é aritmética —
-- `days_from_civil`, de Howard Hinnant, que trata março como primeiro mês do
-- ano e com isso empurra o bissexto para o fim, eliminando o caso especial de
-- fevereiro.
--
-- # Fuso: naive por padrão, explícito quando precisa
--
-- Um valor SEM fuso é hora local — é o que `today`/`now` devolvem e o que os
-- campos de edição guardam. Um valor COM fuso (`...Z`, `...-03:00`) é aceito em
-- toda entrada, e o offset viaja junto: `add` o preserva, `format` desenha as
-- componentes como estão escritas. Converter é explícito, com `to_local` e
-- `to_utc` — o módulo nunca muda um instante por conta própria.
--
-- Vale saber que o `os.time` do Luau NÃO é o do Lua: ele usa `timegm`, e não
-- `mktime`, então uma tabela de componentes é lida como UTC. É por isso que o
-- truque clássico de achar o offset local (`difftime(t, os.time(os.date("!*t",
-- t)))`) devolve **zero** aqui, silenciosamente. O que funciona é o contrário:
-- `os.time(os.date("*t", t)) - t`.
--
-- # Estrito aqui, tolerante lá
--
-- O `Instante` do lado Rust (`src/widget.rs`) faz o parse TOLERANTE de
-- propósito: um campo não pode renderizar quebrado enquanto a pessoa digita,
-- nem engolir a digitação de um app que ainda não semeou a chave. Aqui é o
-- contrário — entrada inválida devolve `nil`, e quem chamou decide o que
-- fazer com isso. Um script tem essa escolha; um widget, no meio de um
-- quadro, não tem.

date = {}

local function e_bissexto(ano: number): boolean
    return ano % 4 == 0 and (ano % 100 ~= 0 or ano % 400 == 0)
end

-- Quantos dias tem o mês. Irmã da `dias_no_mes` de `src/widget.rs` — a mesma
-- regra de bissexto completa (múltiplo de 4, exceto séculos que não são
-- múltiplos de 400).
function date.days_in_month(ano: number, mes: number): number
    if mes == 2 then
        return if e_bissexto(ano) then 29 else 28
    elseif mes == 4 or mes == 6 or mes == 9 or mes == 11 then
        return 30
    end
    return 31
end

-- Dia absoluto desde a época Unix. É o que permite somar dias e medir
-- intervalos sem laço nenhum: vira número, mexe, volta.
local function para_dias_civis(ano: number, mes: number, dia: number): number
    local y: number = if mes <= 2 then ano - 1 else ano
    local era: number = math.floor(y / 400)
    local yoe: number = y - era * 400
    local mp: number = (mes + 9) % 12
    local doy: number = math.floor((153 * mp + 2) / 5) + dia - 1
    local doe: number = yoe * 365 + math.floor(yoe / 4) - math.floor(yoe / 100) + doy
    return era * 146097 + doe - 719468
end

-- A volta: dia absoluto -> ano, mês, dia.
local function de_dias_civis(z: number): (number, number, number)
    z += 719468
    local era: number = math.floor(z / 146097)
    local doe: number = z - era * 146097
    local yoe: number = math.floor(
        (doe - math.floor(doe / 1460) + math.floor(doe / 36524) - math.floor(doe / 146096)) / 365
    )
    local y: number = yoe + era * 400
    local doy: number = doe - (365 * yoe + math.floor(yoe / 4) - math.floor(yoe / 100))
    local mp: number = math.floor((5 * doy + 2) / 153)
    local d: number = doy - math.floor((153 * mp + 2) / 5) + 1
    local m: number = if mp < 10 then mp + 3 else mp - 9
    return (if m <= 2 then y + 1 else y), m, d
end

-- Componentes <-> segundos, tratando as componentes como se fossem UTC. É a
-- ponte entre a aritmética civil e o epoch: quem sabe o offset é quem chama.
local function segundos_civis(p: { [string]: number }): number
    return para_dias_civis(p.year, p.month, p.day) * 86400
        + p.hour * 3600
        + p.min * 60
        + p.sec
end

local function partes_civis(s: number): { [string]: number }
    local dias: number = math.floor(s / 86400)
    local resto: number = s - dias * 86400
    local y, m, d = de_dias_civis(dias)
    return {
        year = y,
        month = m,
        day = d,
        hour = math.floor(resto / 3600),
        min = math.floor(resto % 3600 / 60),
        sec = resto % 60,
    }
end

-- Offset local em segundos a leste de UTC (-10800 em São Paulo). Lido do
-- sistema, não configurado: é o mesmo fuso que o `os.date` do Luau usa para
-- formatar, então tudo aqui concorda com ele.
local function offset_em(t: number): number?
    local tab = os.date("*t", t)
    if type(tab) ~= "table" then
        return nil
    end
    local ok, off = pcall(function()
        return (os.time(tab :: any) :: number) - t
    end)
    if ok and type(off) == "number" then
        return off
    end
    return nil
end

-- Fallback para quando o offset do instante pedido não pode ser lido — o
-- `os.date` do Luau recusa `t < 0`, então qualquer coisa antes de 1970 cai
-- aqui. Vale o offset de hoje, que é melhor do que um `nil` derrubando a tela.
local OFFSET_AGORA: number = offset_em(os.time()) or 0

local function offset_local(t: number): number
    return offset_em(t) or OFFSET_AGORA
end

-- A FORMA de uma string ISO: quais seções ela trazia, com que separador e com
-- que fuso. Existe porque `add` tem de devolver no mesmo formato que recebeu —
-- somar um dia a um `YYYY-MM-DD` não pode inventar um `00:00` no fim, e somar
-- um dia a um `...T12:00:00Z` não pode perder o `Z`.
type Forma = {
    data: boolean,
    hora: boolean,
    seg: boolean,
    t_sep: boolean?,
    offset: number?,
}

-- Núcleo compartilhado de tudo aqui: valida e devolve as partes junto com a
-- forma. `nil` quando a string não é ISO válida — inclusive quando é uma data
-- que não existe (31/02 não passa, ainda que case com o padrão).
local function analisar(iso: any): ({ [string]: number }?, Forma?)
    if type(iso) ~= "string" then
        return nil, nil
    end
    local texto: string = string.match(iso, "^%s*(.-)%s*$") :: string
    local p = { year = 2000, month = 1, day = 1, hour = 0, min = 0, sec = 0 }
    local forma: Forma = { data = false, hora = false, seg = false }

    local parte_data: string?, parte_hora: string? = nil, nil
    -- O `T` do RFC 3339 e o espaço do ISO 8601 estendido são o mesmo
    -- separador; qual deles veio fica na forma, para `add` devolver igual.
    local d, h = string.match(texto, "^(%S+)[Tt](%S+)$")
    if d ~= nil then
        parte_data, parte_hora = d, h
        forma.t_sep = true
    else
        d, h = string.match(texto, "^(%S+) (%S+)$")
        if d ~= nil then
            parte_data, parte_hora = d, h
        elseif string.find(texto, ":") ~= nil then
            parte_hora = texto
        else
            parte_data = texto
        end
    end

    if parte_data ~= nil then
        local a, m, dd = string.match(parte_data, "^(%d+)%-(%d+)%-(%d+)$")
        if a == nil then
            return nil, nil
        end
        p.year, p.month, p.day = tonumber(a) :: number, tonumber(m) :: number, tonumber(dd) :: number
        if p.year < 1 or p.year > 9999 or p.month < 1 or p.month > 12 then
            return nil, nil
        end
        if p.day < 1 or p.day > date.days_in_month(p.year, p.month) then
            return nil, nil
        end
        forma.data = true
    end

    if parte_hora ~= nil then
        local corpo: string = parte_hora
        -- Fuso no fim: `Z`, `±HH:MM`, `±HHMM` ou `±HH`. Sai do corpo e vira
        -- offset em segundos a leste de UTC.
        if string.match(corpo, "[Zz]$") ~= nil then
            forma.offset = 0
            corpo = string.sub(corpo, 1, #corpo - 1)
        else
            local sinal, zh, zm = string.match(corpo, "([+%-])(%d%d):?(%d?%d?)$")
            if sinal ~= nil then
                local minutos: number = if zm == "" then 0 else tonumber(zm) :: number
                local total: number = (tonumber(zh) :: number) * 3600 + minutos * 60
                if total > 86400 then
                    return nil, nil
                end
                forma.offset = if sinal == "-" then -total else total
                corpo = (string.gsub(corpo, "([+%-])(%d%d):?(%d?%d?)$", ""))
            end
        end
        -- Fração de segundo é aceita e DESCARTADA: o motor guarda data e hora
        -- com resolução de segundo, e devolver o milissegundo obrigaria toda
        -- saída daqui a carregá-lo.
        corpo = (string.gsub(corpo, "%.%d+$", ""))

        local hh, mm, ss = string.match(corpo, "^(%d+):(%d+):(%d+)$")
        if hh == nil then
            hh, mm = string.match(corpo, "^(%d+):(%d+)$")
        else
            forma.seg = true
        end
        if hh == nil then
            return nil, nil
        end
        p.hour, p.min = tonumber(hh) :: number, tonumber(mm) :: number
        p.sec = if forma.seg then tonumber(ss) :: number else 0
        if p.hour > 23 or p.min > 59 or p.sec > 59 then
            return nil, nil
        end
        forma.hora = true
    end

    return p, forma
end

-- O caminho de volta para texto, sempre ISO e sempre com zero à esquerda —
-- é o que faz duas saídas deste módulo comparáveis como string.
local function montar(p: { [string]: number }, forma: Forma): string
    local pedacos: { string } = {}
    if forma.data then
        table.insert(pedacos, string.format("%04d-%02d-%02d", p.year, p.month, p.day))
    end
    if forma.hora then
        if forma.seg then
            table.insert(pedacos, string.format("%02d:%02d:%02d", p.hour, p.min, p.sec))
        else
            table.insert(pedacos, string.format("%02d:%02d", p.hour, p.min))
        end
    end
    local saida: string = table.concat(pedacos, if forma.t_sep then "T" else " ")
    -- O fuso só faz sentido grudado numa hora; num `YYYY-MM-DD` puro não há o
    -- que deslocar.
    local offset: number? = forma.offset
    if offset ~= nil and forma.hora then
        if offset == 0 then
            saida ..= "Z"
        else
            local sinal: string = if offset < 0 then "-" else "+"
            local abs: number = math.abs(offset)
            saida ..= string.format(
                "%s%02d:%02d",
                sinal,
                math.floor(abs / 3600),
                math.floor(abs % 3600 / 60)
            )
        end
    end
    return saida
end

-- ── Relógio ────────────────────────────────────────────────────────────────
--
-- Hora LOCAL do sistema, lida NO MOMENTO DA CHAMADA. Não há assinatura de
-- relógio aqui: uma tela que precisa andar sozinha combina isto com `every`
-- (`every(1000, function() ctx.agora = date.now() end)`).

-- Hoje, em `YYYY-MM-DD` — pronto para ir direto a um `<dateedit>`.
function date.today(): string
    return os.date("%Y-%m-%d") :: string
end

-- Agora, em `YYYY-MM-DD HH:MM` (`:SS` com `segundos = true`) — o formato do
-- `<datetimeedit>`, com e sem a seção de segundos.
function date.now(segundos: boolean?): string
    return os.date(if segundos then "%Y-%m-%d %H:%M:%S" else "%Y-%m-%d %H:%M") :: string
end

-- A hora do relógio, em `HH:MM` — o formato do `<timeedit>`.
function date.time(segundos: boolean?): string
    return os.date(if segundos then "%H:%M:%S" else "%H:%M") :: string
end

-- ── Leitura ────────────────────────────────────────────────────────────────

-- Partes numéricas de uma string ISO, ou `nil` se ela não for válida. As
-- seções ausentes vêm no default (um `HH:MM` puro devolve `2000-01-01`), então
-- só leia o que a string realmente trazia.
function date.parse(iso: any): { [string]: number }?
    local p = analisar(iso)
    return p
end

function date.valid(iso: any): boolean
    return (analisar(iso)) ~= nil
end

-- Dia da semana, 1 = domingo … 7 = sábado. É a MESMA convenção do
-- `os.date("*t").wday` do Luau, de propósito: misturar as duas numa tela é
-- inevitável, e duas bases diferentes seria uma armadilha silenciosa.
function date.weekday(iso: any): number?
    local p = analisar(iso)
    if p == nil then
        return nil
    end
    -- 1970-01-01 foi uma quinta-feira (wday 5).
    return (para_dias_civis(p.year, p.month, p.day) + 4) % 7 + 1
end

-- A parte de data (`YYYY-MM-DD`) e a de hora (`HH:MM[:SS]`) de um valor. `nil`
-- quando a string não é válida OU não tem aquela seção.
function date.date_of(iso: any): string?
    local p, forma = analisar(iso)
    if p == nil or forma == nil or not forma.data then
        return nil
    end
    return montar(p, { data = true, hora = false, seg = false })
end

function date.time_of(iso: any): string?
    local p, forma = analisar(iso)
    if p == nil or forma == nil or not forma.hora then
        return nil
    end
    return montar(p, { data = false, hora = true, seg = forma.seg })
end

-- ── Fuso e epoch ───────────────────────────────────────────────────────────
--
-- A fronteira com o mundo de fora. Um backend fala RFC 3339 em UTC
-- (`2026-07-06T12:34:56Z`); uma tela fala hora local. Estas quatro funções são
-- a tradução, e são EXPLÍCITAS de propósito: nenhuma outra função do módulo
-- desloca um instante sozinha.

-- Segundos desde a época Unix. Um valor COM fuso usa o fuso dele; um valor SEM
-- fuso é lido como hora LOCAL — é o que `today`/`now` devolvem, então
-- `date.epoch(date.now())` bate com `os.time()`.
function date.epoch(iso: any): number?
    local p, forma = analisar(iso)
    if p == nil or forma == nil then
        return nil
    end
    local s: number = segundos_civis(p)
    local offset: number? = forma.offset
    if offset ~= nil then
        return s - offset
    end
    -- Sem fuso: acha o epoch cuja hora LOCAL bate com as componentes. Duas
    -- passadas porque o offset depende do instante (horário de verão), e o
    -- instante é justamente o que se está procurando.
    local t: number = s - offset_local(s)
    return s - offset_local(t)
end

-- A volta: epoch -> `YYYY-MM-DD HH:MM:SS` na hora de parede local (ou em UTC
-- com `utc = true`). Sempre com segundos, e sempre sem sufixo de fuso — o
-- resultado é um valor naive, pronto para `format`.
function date.from_epoch(segundos: any, utc: boolean?): string?
    local t = tonumber(segundos)
    if t == nil then
        return nil
    end
    t = math.floor(t)
    local parede: number = if utc then t else t + offset_local(t)
    return montar(partes_civis(parede), { data = true, hora = true, seg = true })
end

-- Converte para a hora de parede local / UTC, devolvendo um valor NAIVE (sem
-- `Z`, sem `T`) na mesma forma da entrada — sem segundos se a entrada não
-- tinha. É o par que uma tela usa para exibir o que veio do backend:
--
--   date.format(date.to_local(dep.started_at), "DD/MM HH:mm")
--
-- Um valor já naive é devolvido como está: ele JÁ é hora local.
local function converter(iso: any, para_utc: boolean): string?
    local p, forma = analisar(iso)
    if p == nil or forma == nil then
        return nil
    end
    local limpa: Forma = { data = forma.data, hora = forma.hora, seg = forma.seg }
    if forma.offset == nil and not para_utc then
        return montar(p, limpa)
    end
    local t: number? = date.epoch(iso)
    if t == nil then
        return nil
    end
    local parede: number = if para_utc then t else t + offset_local(t)
    return montar(partes_civis(parede), limpa)
end

function date.to_local(iso: any): string?
    return converter(iso, false)
end

function date.to_utc(iso: any): string?
    return converter(iso, true)
end

-- As componentes de um valor já na hora local — o que toda comparação usa, para
-- que um `...Z` vindo de um backend e um `date.today()` da tela sejam
-- comparáveis sem que quem chama precise converter antes.
local function partes_locais(iso: any): { [string]: number }?
    local p, forma = analisar(iso)
    if p == nil or forma == nil then
        return nil
    end
    if forma.offset == nil then
        return p
    end
    local t: number? = date.epoch(iso)
    if t == nil then
        return nil
    end
    return partes_civis(t + offset_local(t))
end

-- ── Comparação ─────────────────────────────────────────────────────────────
--
-- ISO ordena como texto, e por isso `a > b` funciona — mas só entre strings da
-- MESMA forma. `"2026-09-10 08:00" > "2026-09-10"` é verdadeiro só porque a
-- string é mais longa, ainda que os dois sejam o mesmo dia; é a pegadinha que
-- estas três funções existem para tirar da frente. Uma seção que falta de um
-- lado vale zero (uma data pura é a meia-noite dela).
--
-- Fuso também: um valor com offset é trazido para a hora local antes da
-- comparação, então um `...Z` vindo de um backend e um `date.today()` da tela
-- são comparáveis direto. É a única exceção à regra de que só `to_local`/
-- `to_utc` deslocam um instante — e ela não muda nenhum valor, só a leitura.

-- -1, 0 ou 1 (`a` antes, igual, depois de `b`); `nil` se alguma não for válida.
function date.compare(a: any, b: any): number?
    local pa = partes_locais(a)
    local pb = partes_locais(b)
    if pa == nil or pb == nil then
        return nil
    end
    local ca = segundos_civis(pa)
    local cb = segundos_civis(pb)
    if ca < cb then
        return -1
    end
    return if ca > cb then 1 else 0
end

-- `is_before`/`is_after`, e não `before`/`after`: o prelúdio já tem um global
-- `after(ms, fn)`, que é o temporizador, e ler os dois na mesma tela sem o
-- prefixo seria confuso mesmo com o `date.` na frente.
function date.is_before(a: any, b: any): boolean
    return date.compare(a, b) == -1
end

function date.is_after(a: any, b: any): boolean
    return date.compare(a, b) == 1
end

-- ── Aritmética ─────────────────────────────────────────────────────────────

-- Soma um intervalo e devolve o resultado NA MESMA FORMA da entrada: somar um
-- dia a um `YYYY-MM-DD` não inventa `00:00` no fim, e somar uma hora a um
-- `HH:MM` vira dentro do dia, sem data para onde transbordar.
--
-- `delta = { years?, months?, days?, hours?, minutes?, seconds? }`.
--
-- Meses e anos andam pelo CALENDÁRIO, e por isso podem grudar no fim do mês:
-- 31/01 + 1 mês é 28/02 (ou 29), como todo mundo espera e como o `QDateEdit`
-- faz ao trocar a seção do mês. Os outros campos são duração pura e
-- transbordam entre si — num valor só de hora, o que transbordaria do dia se
-- perde, porque um `HH:MM` não carrega que dia é.
function date.add(iso: any, delta: { [string]: number }?): string?
    local p, forma = analisar(iso)
    if p == nil or forma == nil then
        return nil
    end
    local d: { [string]: number } = delta or {}

    -- Calendário primeiro: ano e mês viram um total de meses, e o dia é
    -- prendido no fim do mês novo antes de qualquer duração entrar.
    local total_meses = (p.year * 12 + (p.month - 1))
        + (d.years or 0) * 12
        + (d.months or 0)
    local ano = math.floor(total_meses / 12)
    local mes = total_meses % 12 + 1
    local dia = math.min(p.day, date.days_in_month(ano, mes))

    -- Duração depois, em segundos absolutos, com o excedente virando dias.
    local segundos = p.hour * 3600
        + p.min * 60
        + p.sec
        + (d.hours or 0) * 3600
        + (d.minutes or 0) * 60
        + (d.seconds or 0)
    local dias_extras = math.floor(segundos / 86400)
    segundos -= dias_extras * 86400

    local novo = {
        year = ano,
        month = mes,
        day = dia,
        hour = math.floor(segundos / 3600),
        min = math.floor(segundos % 3600 / 60),
        sec = segundos % 60,
    }

    if forma.data then
        -- Só há para onde transbordar quando a string tem data. Sem ela, a hora
        -- vira dentro do dia e `days` não tem efeito — o `dias_extras` e o
        -- `d.days` são descartados de propósito, porque um `HH:MM` não carrega
        -- que dia é.
        local z = para_dias_civis(ano, mes, dia) + (d.days or 0) + dias_extras
        novo.year, novo.month, novo.day = de_dias_civis(z)
        if novo.year < 1 or novo.year > 9999 then
            return nil
        end
    end

    return montar(novo, forma)
end

-- Dias inteiros de `b` até `a` (positivo quando `a` é o mais recente). A hora
-- é IGNORADA: são dias de calendário, que é o que uma diária de hotel, um
-- prazo ou "faltam N dias" contam. Para o intervalo exato, `diff_seconds`.
function date.diff(a: any, b: any): number?
    local pa = partes_locais(a)
    local pb = partes_locais(b)
    if pa == nil or pb == nil then
        return nil
    end
    return para_dias_civis(pa.year, pa.month, pa.day) - para_dias_civis(pb.year, pb.month, pb.day)
end

function date.diff_seconds(a: any, b: any): number?
    local pa = partes_locais(a)
    local pb = partes_locais(b)
    if pa == nil or pb == nil then
        return nil
    end
    return segundos_civis(pa) - segundos_civis(pb)
end

-- ── Exibição ───────────────────────────────────────────────────────────────

-- A mesma separação que o widget faz entre o valor e o `displayFormat`: a
-- chave continua ISO, isto é só o que vai para a tela.
--
--   date.format("2026-09-10", "DD/MM/YYYY")  -->  "10/09/2026"
--   date.format(ctx.lembrete, "DD/MM HHhmm") -->  "09/09 18h00"
--
-- Tokens: `YYYY` `YY` `MM` `DD` `HH` `mm` `SS`. Qualquer outro caractere passa
-- intacto — inclusive letras, o que permite `HHhmm` sem escape.
local TOKENS: { string } = { "YYYY", "YY", "MM", "DD", "HH", "mm", "SS" }

function date.format(iso: any, fmt: string): string?
    local p = analisar(iso)
    if p == nil then
        return nil
    end
    local mapa: { [string]: string } = {
        YYYY = string.format("%04d", p.year),
        YY = string.format("%02d", p.year % 100),
        MM = string.format("%02d", p.month),
        DD = string.format("%02d", p.day),
        HH = string.format("%02d", p.hour),
        mm = string.format("%02d", p.min),
        SS = string.format("%02d", p.sec),
    }
    -- Varredura com o token mais longo primeiro (`YYYY` antes de `YY`), em vez
    -- de um `gsub` por token: substituir em série faria a saída de um token
    -- virar entrada do seguinte.
    local saida: { string } = {}
    local i = 1
    while i <= #fmt do
        local achou = false
        for _, tok in TOKENS do
            if string.sub(fmt, i, i + #tok - 1) == tok then
                table.insert(saida, mapa[tok])
                i += #tok
                achou = true
                break
            end
        end
        if not achou then
            table.insert(saida, string.sub(fmt, i, i))
            i += 1
        end
    end
    return table.concat(saida)
end

-- ── http: cliente HTTP reutilizável sobre `fetch` ──────────────────────────
--
-- `http(base_url?, opts?)` devolve um objeto com `base_url`, `headers`,
-- `query`, `timeout` e `retries` de base, mais `interceptors` de requisição e
-- de resposta e `error handlers`. Todo método de chamada (`:get`, `:post`, …)
-- SUSPENDE a corrotina, como o `fetch` cru — é feito para rodar dentro de um
-- handler.
--
--   local api = http("https://api.exemplo.com", {
--       headers = { Accept = "application/json" },
--       timeout = 8000,          -- ms, por requisição
--       retries = 2,             -- tentativas extras em falha de rede / 5xx
--   })
--
--   api:set_header("Authorization", "Bearer " .. token)   -- encadeável
--     :on_request(function(req) req.headers["X-Trace"] = trace_id(); return req end)
--     :on_response(function(res) return res end)
--     :on_error(function(err) toast({ message = err.error, kind = "error" }) end)
--
--   local r = api:get("/itens", { query = { page = 1 } })
--   local r = api:post("/itens", { nome = "x" })   -- body tabela → JSON + Content-Type
--   if r.ok then usar(r.json) end                  -- r.json = corpo já decodificado
--
-- A resposta é a do `fetch` (`{ ok, status, body, error }`) mais:
--   `r.json`     — o corpo decodificado, ou `nil` se não era JSON
--   `r.request`  — `{ method, url }` da chamada que a produziu

local function _http_escape(s: string): string
    return (string.gsub(s, "[^%w%-_%.~]", function(c)
        return string.format("%%%02X", string.byte(c))
    end))
end

local function _http_query(params: any): string
    if type(params) ~= "table" then
        return ""
    end
    local parts = {}
    for k, v in params do
        if v ~= nil then
            table.insert(parts, _http_escape(tostring(k)) .. "=" .. _http_escape(tostring(v)))
        end
    end
    if #parts == 0 then
        return ""
    end
    return "?" .. table.concat(parts, "&")
end

local function _http_join(base: string, path: string): string
    if string.match(path, "^https?://") then
        return path
    end
    if base ~= "" and string.sub(base, -1) == "/" then
        base = string.sub(base, 1, -2)
    end
    if path ~= "" and string.sub(path, 1, 1) ~= "/" then
        path = "/" .. path
    end
    return base .. path
end

-- Cópia rasa; `b` vence `a`. `nil` em `b` não apaga (pairs pula nil).
local function _http_merge(a: any, b: any): any
    local out = {}
    if type(a) == "table" then
        for k, v in a do
            out[k] = v
        end
    end
    if type(b) == "table" then
        for k, v in b do
            out[k] = v
        end
    end
    return out
end

local Http = {}
Http.__index = Http

function http(base_url: string?, opts: any?): any
    opts = opts or {}
    return setmetatable({
        base_url = base_url or "",
        headers = _http_merge(nil, opts.headers),
        query = _http_merge(nil, opts.query),
        timeout = opts.timeout,
        retries = opts.retries or 0,
        retry_on = opts.retry_on, -- function(res) -> boolean; nil = padrão (0, 429, 5xx)
        _req = {}, -- interceptors de requisição
        _res = {}, -- interceptors de resposta
        _err = {}, -- error handlers
    }, Http)
end

-- ── Config, encadeável (cada setter devolve self) ─────────────────────────
function Http:set_base_url(u: string)
    self.base_url = u or ""
    return self
end
function Http:set_header(k: string, v: any)
    self.headers[k] = v
    return self
end
function Http:set_headers(t: any)
    self.headers = _http_merge(self.headers, t)
    return self
end
function Http:remove_header(k: string)
    self.headers[k] = nil
    return self
end
function Http:set_query(k: string, v: any)
    self.query[k] = v
    return self
end
function Http:set_timeout(ms: number?)
    self.timeout = ms
    return self
end
function Http:set_retries(n: number)
    self.retries = n or 0
    return self
end
function Http:set_retry_on(fn: any)
    self.retry_on = fn
    return self
end
function Http:on_request(fn: any)
    table.insert(self._req, fn)
    return self
end
function Http:on_response(fn: any)
    table.insert(self._res, fn)
    return self
end
function Http:on_error(fn: any)
    table.insert(self._err, fn)
    return self
end

-- Client derivado: herda tudo, sobrescreve o que `opts` trouxer. `base_path`
-- é acrescentado à `base_url` do pai. Os interceptors do pai são copiados (o
-- filho pode acrescentar sem afetar o pai).
function Http:extend(base_path: string?, opts: any?): any
    opts = opts or {}
    local child = http(_http_join(self.base_url, base_path or ""), {
        headers = _http_merge(self.headers, opts.headers),
        query = _http_merge(self.query, opts.query),
        timeout = if opts.timeout ~= nil then opts.timeout else self.timeout,
        retries = if opts.retries ~= nil then opts.retries else self.retries,
        retry_on = opts.retry_on or self.retry_on,
    })
    for _, h in self._req do
        table.insert(child._req, h)
    end
    for _, h in self._res do
        table.insert(child._res, h)
    end
    for _, h in self._err do
        table.insert(child._err, h)
    end
    return child
end

-- ── O núcleo ──────────────────────────────────────────────────────────────
function Http:request(spec: any): any
    spec = spec or {}
    local req = {
        method = string.upper(spec.method or "GET"),
        path = spec.path or spec.url or "",
        headers = _http_merge(self.headers, spec.headers),
        query = _http_merge(self.query, spec.query),
        body = spec.body,
        timeout = if spec.timeout ~= nil then spec.timeout else self.timeout,
    }

    -- interceptors de requisição: podem mutar `req` OU devolver uma tabela nova.
    for _, hook in self._req do
        local changed = hook(req)
        if type(changed) == "table" then
            req = changed
        end
    end

    -- corpo tabela → JSON, com Content-Type se ninguém pôs um.
    local body = req.body
    if type(body) == "table" then
        body = json.encode(body)
        local tem_ct = false
        for k in req.headers do
            if string.lower(k) == "content-type" then
                tem_ct = true
                break
            end
        end
        if not tem_ct then
            req.headers["Content-Type"] = "application/json"
        end
    end

    local url = _http_join(self.base_url, req.path) .. _http_query(req.query)

    -- envia, repetindo enquanto `retry_on` disser que sim e ainda houver folga.
    local attempt = 0
    local res
    while true do
        attempt += 1
        res = fetch(url, {
            method = req.method,
            headers = req.headers,
            body = body,
            timeout = req.timeout,
        })
        local repetir
        if self.retry_on then
            repetir = self.retry_on(res) == true
        else
            repetir = res.status == 0 or res.status == 429 or res.status >= 500
        end
        if res.ok or attempt > self.retries or not repetir then
            break
        end
    end

    -- enriquece a resposta.
    res.request = { method = req.method, url = url }
    res.json = nil
    if type(res.body) == "string" and res.body ~= "" then
        local ok, decoded = pcall(json.decode, res.body)
        if ok then
            res.json = decoded
        end
    end

    -- interceptors de resposta.
    for _, hook in self._res do
        local changed = hook(res)
        if type(changed) == "table" then
            res = changed
        end
    end

    -- error handlers, depois de esgotados os retries.
    if not res.ok then
        local err = {
            status = res.status,
            error = if res.error ~= "" then res.error else "HTTP " .. tostring(res.status),
            url = url,
            method = req.method,
            attempt = attempt,
            body = res.body,
        }
        for _, hook in self._err do
            hook(err)
        end
    end

    return res
end

-- ── Açúcar por método ─────────────────────────────────────────────────────
function Http:get(path: string, spec: any?): any
    return self:request(_http_merge(spec, { method = "GET", path = path }))
end
function Http:delete(path: string, spec: any?): any
    return self:request(_http_merge(spec, { method = "DELETE", path = path }))
end
function Http:head(path: string, spec: any?): any
    return self:request(_http_merge(spec, { method = "HEAD", path = path }))
end
function Http:post(path: string, body: any?, spec: any?): any
    return self:request(_http_merge(spec, { method = "POST", path = path, body = body }))
end
function Http:put(path: string, body: any?, spec: any?): any
    return self:request(_http_merge(spec, { method = "PUT", path = path, body = body }))
end
function Http:patch(path: string, body: any?, spec: any?): any
    return self:request(_http_merge(spec, { method = "PATCH", path = path, body = body }))
end

-- ── console: logs coloridos e decorados para o TERMINAL ────────────────────
--
-- `console.log` / `.info` / `.warn` / `.error` / `.debug` / `.table`, com
-- rótulo, glifo, cor ANSI e hora. Filtrável por nível e configurável:
--
--   console.set_level("warn")          -- só warn e error a partir daqui
--   console.config({ prefix = "[api]", color = false, timestamp = false })
--   console.log("carregou", 42, { id = 7 })   -- vários argumentos, tabela inclusa
--   console.table(res.json)                    -- lista de objetos → grade ASCII
--
-- Tudo vai para o `stdout` do processo (o mesmo lugar do `print`). Os ERROS de
-- runtime do motor continuam saindo por conta própria no `stderr` — este
-- `console` é para o SEU log, não para eles.

local _console = {
    level = 1, -- 0 debug · 1 log/info · 2 warn · 3 error · 4 silent
    color = true,
    timestamp = true,
    label = true,
    prefix = "",
}

local _CONSOLE_LVL = { debug = 0, log = 1, info = 1, warn = 2, error = 3, silent = 4 }

local _CONSOLE_STYLE = {
    debug = { code = "90", glyph = "·", tag = "DEBUG" },
    info = { code = "36", glyph = "ℹ", tag = "INFO " },
    warn = { code = "33", glyph = "⚠", tag = "WARN " },
    error = { code = "31;1", glyph = "✖", tag = "ERROR" },
}

local function _console_level_num(v: any): number
    if type(v) == "number" then
        return math.clamp(math.floor(v), 0, 4)
    end
    return _CONSOLE_LVL[string.lower(tostring(v))] or 1
end

local function _console_paint(code: string, s: string): string
    if not _console.color then
        return s
    end
    return "\27[" .. code .. "m" .. s .. "\27[0m"
end

-- Renderiza um valor. No topo, uma string sai crua (como o console do browser);
-- aninhada, sai entre aspas. Tabelas viram `{ k = v, … }` ou `[a, b, …]`, até
-- 4 níveis; ciclos e profundidade viram `<ciclo>` / `{…}`.
local function _console_val(v: any, depth: number, seen: any, nested: boolean): string
    local t = type(v)
    if t == "string" then
        return if nested then string.format("%q", v) else v
    end
    if t ~= "table" then
        return tostring(v)
    end
    if seen[v] then
        return "<ciclo>"
    end
    if depth >= 4 then
        return "{…}"
    end
    seen[v] = true

    local n = #v
    local is_array = n > 0
    for k in v do
        if type(k) ~= "number" then
            is_array = false
            break
        end
    end

    local parts = {}
    if is_array then
        for i = 1, n do
            table.insert(parts, _console_val(v[i], depth + 1, seen, true))
        end
        seen[v] = nil
        return "[" .. table.concat(parts, ", ") .. "]"
    end

    local keys = {}
    for k in v do
        table.insert(keys, k)
    end
    table.sort(keys, function(a, b)
        return tostring(a) < tostring(b)
    end)
    for _, k in keys do
        table.insert(parts, tostring(k) .. " = " .. _console_val(v[k], depth + 1, seen, true))
    end
    seen[v] = nil
    return "{ " .. table.concat(parts, ", ") .. " }"
end

local function _console_join(...): string
    local args = table.pack(...)
    local out = {}
    for i = 1, args.n do
        table.insert(out, _console_val(args[i], 0, {}, false))
    end
    return table.concat(out, " ")
end

local function _console_emit(kind: string, ...)
    if _console_level_num(kind) < _console.level then
        return
    end
    local style = _CONSOLE_STYLE[kind]
    local segs = {}
    if _console.prefix ~= "" then
        table.insert(segs, _console_paint("35", _console.prefix))
    end
    if _console.timestamp then
        table.insert(segs, _console_paint("90", os.date("%H:%M:%S")))
    end
    if _console.label then
        table.insert(segs, _console_paint(style.code, style.glyph .. " " .. style.tag))
    end

    local body = _console_join(...)
    if kind == "error" then
        body = _console_paint("31", body)
    elseif kind == "warn" then
        body = _console_paint("33", body)
    end

    local head = table.concat(segs, " ")
    if head ~= "" then
        print(head .. "  " .. body)
    else
        print(body)
    end
end

console = {}

function console.debug(...)
    _console_emit("debug", ...)
end
function console.log(...)
    _console_emit("info", ...)
end
function console.info(...)
    _console_emit("info", ...)
end
function console.warn(...)
    _console_emit("warn", ...)
end
function console.error(...)
    _console_emit("error", ...)
end

-- `"debug"` | `"info"`/`"log"` | `"warn"` | `"error"` | `"silent"`, ou 0–4.
function console.set_level(level: any)
    _console.level = _console_level_num(level)
    return console
end

-- `{ level?, color?, timestamp?, label?, prefix? }` — só o que vier é mexido.
function console.config(opts: any?)
    opts = opts or {}
    if opts.level ~= nil then
        _console.level = _console_level_num(opts.level)
    end
    if opts.color ~= nil then
        _console.color = not not opts.color
    end
    if opts.timestamp ~= nil then
        _console.timestamp = not not opts.timestamp
    end
    if opts.label ~= nil then
        _console.label = not not opts.label
    end
    if opts.prefix ~= nil then
        _console.prefix = tostring(opts.prefix)
    end
    return console
end

-- Largura de exibição de uma string (conta caracteres, não bytes — importa para
-- acento e para as bordas). Cai para `#s` se o `utf8` não estiver disponível.
local function _console_width(s: string): number
    local ok, len = pcall(utf8.len, s)
    if ok and type(len) == "number" then
        return len
    end
    return #s
end

local function _console_cell(s: string, w: number): string
    local pad = w - _console_width(s)
    if pad <= 0 then
        return s
    end
    return s .. string.rep(" ", pad)
end

-- Grade ASCII de uma lista de linhas. `rows` = lista de tabelas (uma linha
-- cada) OU de escalares (viram a coluna `valor`). `columns` opcional escolhe e
-- ordena as chaves; sem ele, a união das chaves na ordem em que aparecem.
function console.table(rows: any, columns: any?)
    if _console.level > 1 then
        return
    end
    if type(rows) ~= "table" then
        _console_emit("info", rows)
        return
    end

    -- normaliza para uma lista de { _key = <índice/chave>, [col] = valor }
    local linhas = {}
    local ordem_cols = {}
    local vistas = {}
    local tem_valor_escalar = false

    local function registra_col(c: any)
        local k = tostring(c)
        if not vistas[k] then
            vistas[k] = true
            table.insert(ordem_cols, k)
        end
    end

    if columns ~= nil then
        for _, c in columns do
            registra_col(c)
        end
    end

    for k, v in rows do
        local linha = { _key = tostring(k) }
        if type(v) == "table" then
            for ck, cv in v do
                linha[tostring(ck)] = _console_val(cv, 1, {}, true)
                if columns == nil then
                    registra_col(ck)
                end
            end
        else
            linha["valor"] = _console_val(v, 1, {}, true)
            tem_valor_escalar = true
            if columns == nil then
                registra_col("valor")
            end
        end
        table.insert(linhas, linha)
    end

    if tem_valor_escalar and columns == nil and not vistas["valor"] then
        registra_col("valor")
    end

    -- cabeçalho: (índice) + as colunas
    local headers = { "(índice)" }
    for _, c in ordem_cols do
        table.insert(headers, c)
    end

    -- larguras
    local larg = {}
    for i, h in headers do
        larg[i] = _console_width(h)
    end
    for _, linha in linhas do
        larg[1] = math.max(larg[1], _console_width(linha._key))
        for ci, c in ordem_cols do
            local cell = linha[c] or ""
            larg[ci + 1] = math.max(larg[ci + 1], _console_width(cell))
        end
    end

    local function regua(esq: string, meio: string, dir: string): string
        local segs = {}
        for _, w in larg do
            table.insert(segs, string.rep("─", w + 2))
        end
        return _console_paint("90", esq .. table.concat(segs, meio) .. dir)
    end

    -- Pinta DEPOIS de preencher: os códigos ANSI não contam na largura.
    local function linha_de(cels: { string }, code: string?): string
        local bar = _console_paint("90", "│")
        local segs = {}
        for i, w in larg do
            local c = _console_cell(cels[i] or "", w)
            if code then
                c = _console_paint(code, c)
            end
            table.insert(segs, " " .. c .. " ")
        end
        return bar .. table.concat(segs, bar) .. bar
    end

    print(regua("┌", "┬", "┐"))
    print(linha_de(headers, "1"))
    print(regua("├", "┼", "┤"))
    for _, linha in linhas do
        local cels = { linha._key }
        for _, c in ordem_cols do
            table.insert(cels, linha[c] or "")
        end
        print(linha_de(cels))
    end
    print(regua("└", "┴", "┘"))
end

