--!strict
-- Tipos dos globais que o motor glacier-ui injeta em runtime. Só o luau-lsp lê
-- este arquivo; o mlua ignora tipos e anotações. Ver o README do projeto.

export type FetchResult = {
    ok: boolean,
    status: number,
    body: string,
    error: string,
}

export type StreamHandle = {
    id: number,
    close: (self: StreamHandle) -> any,
    send: (self: StreamHandle, text: string) -> any,
}

export type StreamOptions = {
    headers: { [string]: string }?,
    on_open: (() -> ())?,
    on_message: ((data: string) -> ())?,
    on_error: ((message: string) -> ())?,
    on_close: (() -> ())?,
}

export type Viewport = { width: number, height: number }

-- Na LEITURA o contexto é sempre texto (é o que o motor guarda); na ESCRITA
-- ele aceita qualquer valor serializável — tabela e número passam por
-- `json.encode`/`tostring` por baixo. Daí `any`: declarar `string` mentia do
-- lado da escrita e transformava `ctx.tiques = 0` num erro falso.
declare ctx: { [string]: any }
declare value: string?

declare function viewport(): Viewport

declare function fetch(url: string, opts: any?): FetchResult
declare function write_file(path: string, contents: string): (boolean, string?)
declare function sse(url: string, opts: StreamOptions?): StreamHandle
declare function websocket(url: string, opts: StreamOptions?): StreamHandle

declare function toast(opts: any): ()
declare function notify(opts: any): ()
declare function confirm(opts: any): boolean

-- Diálogos suspensivos (Onda 8). `prompt` e `pick_color` param a corrotina e
-- voltam com a resposta; `nil` é a desistência — a mesma convenção do
-- `open_file()` cancelado, e o que separa "não respondeu" de "respondeu vazio".
declare function prompt(opts: any): string?
declare function pick_color(opts: any): string?

-- O progresso NÃO suspende: ele acompanha um trabalho que continua rodando.
-- `progress_set(nil)` volta ao indeterminado.
declare function progress(opts: any): ()
declare function progress_set(value: (number | string)?, label: string?): ()
declare function progress_close(): ()

declare function navigate(tela: string): ()
declare function navigate_back(): ()

declare function open_window(opts: any): ()
declare function broadcast(event: string, payload: any?): ()
declare function close_window(): ()

declare function append_textarea(binding: string, text: string): ()

-- O segundo argumento é uma função OU o nome de uma função global — as duas
-- formas que o prelúdio aceita. E os dois devolvem um handle: `after` para
-- poder cancelar antes de vencer, `every` para interromper as repetições.
export type Timer = {
    id: number,
    cancel: (self: Timer) -> any,
}

declare function after(ms: number, callback: (() -> ()) | string): Timer
declare function every(ms: number, callback: (() -> ()) | string): Timer

declare json: {
    decode: (s: string) -> any,
    encode: (v: any) -> string,
    array: <T>(t: { T }) -> { T },
}

-- Aritmética de data/hora sobre strings ISO (`YYYY-MM-DD`, `HH:MM[:SS]`, ou as
-- duas separadas por espaço) — o formato que os <dateedit>/<timeedit>/
-- <datetimeedit> gravam na chave. Devolve `nil` quando a entrada não é ISO
-- válida, inclusive para uma data que não existe (31/02).
export type DateParts = {
    year: number,
    month: number,
    day: number,
    hour: number,
    min: number,
    sec: number,
}

export type DateDelta = {
    years: number?,
    months: number?,
    days: number?,
    hours: number?,
    minutes: number?,
    seconds: number?,
}

declare date: {
    -- Relógio local do sistema, no formato de cada campo.
    today: () -> string,
    now: (segundos: boolean?) -> string,
    time: (segundos: boolean?) -> string,
    -- Leitura.
    parse: (iso: string) -> DateParts?,
    valid: (iso: string) -> boolean,
    weekday: (iso: string) -> number?, -- 1 = domingo, como os.date("*t").wday
    date_of: (iso: string) -> string?,
    time_of: (iso: string) -> string?,
    days_in_month: (ano: number, mes: number) -> number,
    -- Comparação pelo INSTANTE, e não pelo texto: funciona entre formas
    -- diferentes ("2026-09-10 08:00" vs. "2026-09-10").
    compare: (a: string, b: string) -> number?,
    is_before: (a: string, b: string) -> boolean,
    is_after: (a: string, b: string) -> boolean,
    -- Aritmética. `add` devolve na MESMA forma da entrada; meses e anos andam
    -- pelo calendário (31/01 + 1 mês = 28/02).
    add: (iso: string, delta: DateDelta?) -> string?,
    diff: (a: string, b: string) -> number?, -- dias de calendário
    diff_seconds: (a: string, b: string) -> number?,
    -- Exibição: tokens YYYY YY MM DD HH mm SS.
    format: (iso: string, fmt: string) -> string?,
    -- Fuso e epoch. Um valor sem fuso é hora LOCAL; um com fuso (`...Z`,
    -- `...-03:00`, fração de segundo opcional) usa o dele. `to_local`/`to_utc`
    -- devolvem um valor naive, e são as únicas que deslocam um instante.
    epoch: (iso: string) -> number?,
    from_epoch: (segundos: number, utc: boolean?) -> string?,
    to_local: (iso: string) -> string?,
    to_utc: (iso: string) -> string?,
}

declare storage: {
    get: (key: string) -> any,
    set: (key: string, value: any) -> (),
    remove: (key: string) -> (),
}
