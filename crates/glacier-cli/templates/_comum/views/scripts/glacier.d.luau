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

declare ctx: { [string]: string }
declare value: string?

declare function viewport(): Viewport

declare function fetch(url: string, opts: any?): FetchResult
declare function write_file(path: string, contents: string): (boolean, string?)
declare function sse(url: string, opts: StreamOptions?): StreamHandle
declare function websocket(url: string, opts: StreamOptions?): StreamHandle

declare function toast(opts: any): ()
declare function notify(opts: any): ()
declare function confirm(opts: any): boolean

declare function navigate(tela: string): ()
declare function navigate_back(): ()

declare function open_window(opts: any): ()
declare function broadcast(event: string, payload: any?): ()
declare function close_window(): ()

declare function append_textarea(binding: string, text: string): ()

declare function after(ms: number, callback: () -> ()): ()
declare function every(ms: number, callback: () -> ()): any

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
}

declare storage: {
    get: (key: string) -> any,
    set: (key: string, value: any) -> (),
    remove: (key: string) -> (),
}
