--!strict
-- Definições dos globais que o motor glacier-ui injeta no interpretador Luau em
-- runtime (não existem como arquivo). Consumido só pelo type-checker
-- (luau-lsp): o mlua ignora tipos e anotações em runtime. Analise com
--   luau-lsp analyze --definitions=views/scripts/glacier.d.luau views/scripts

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

-- Contexto global do motor: chave -> valor string (o motor coage tudo p/ string).
-- É o que os `{marcadores}` dos .gv leem.
declare ctx: { [string]: string }

-- Valor do on_change/on_toggle/ação-com-sufixo (nil quando a ação não traz valor).
declare value: string?

-- Tamanho atual da janela em px lógicos (o mesmo que alimenta o @media do .gss).
export type Viewport = { width: number, height: number }
declare function viewport(): Viewport

-- HTTP/HTTPS, ou leitura de arquivo local quando a URL começa com `file://`.
-- SUSPENDE a corrotina da ação e retoma com a resposta: a UI não trava e o
-- código fica linear, com cara de await.
declare function fetch(url: string, opts: any?): FetchResult
-- Contraparte de escrita do `fetch("file://…")`. `false, mensagem` na falha.
declare function write_file(path: string, contents: string): (boolean, string?)

-- Streams: cada evento chama o callback nomeado em `opts`.
declare function sse(url: string, opts: any?): StreamHandle
declare function websocket(url: string, opts: any?): StreamHandle

-- Notificação efêmera dentro da janela; `notify` é a nativa do SO.
declare function toast(opts: any): ()
declare function notify(opts: any): ()
-- Diálogo modal; o botão de confirmação despacha `opts.confirm_action`.
declare function confirm(opts: any): boolean

declare function navigate(tela: string): ()
declare function navigate_back(): ()

-- Multi-janela: cada janela é um motor isolado. `on_broadcast(event, payload)`
-- é definido por você na janela receptora, então não é declarado aqui.
declare function open_window(opts: any): ()
declare function broadcast(event: string, payload: any?): ()
declare function close_window(): ()

-- Anexa texto ao fim do <textarea> ligado a `binding` sem recriar o conteúdo
-- (preserva o scroll) — para logs que crescem.
declare function append_textarea(binding: string, text: string): ()

-- Timers. `after` dispara uma vez; `every` repete até o handle ser fechado.
declare function after(ms: number, callback: () -> ()): ()
declare function every(ms: number, callback: () -> ()): any

declare json: {
    decode: (s: string) -> any,
    encode: (v: any) -> string,
    array: <T>(t: { T }) -> { T },
}

-- Persistência local em JSON, análoga ao localStorage do browser: sobrevive a
-- um restart do processo (ao contrário do `ctx`, que é só memória). A raiz é
-- definida no Rust por `GlacierDaemon::storage_dir`.
declare storage: {
    get: (key: string) -> any,
    set: (key: string, value: any) -> (),
    remove: (key: string) -> (),
}
