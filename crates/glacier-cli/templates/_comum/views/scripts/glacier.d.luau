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

declare storage: {
    get: (key: string) -> any,
    set: (key: string, value: any) -> (),
    remove: (key: string) -> (),
}
