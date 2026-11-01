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
    -- "window" (padrão): vive até a janela fechar. "screen": o motor fecha ao SAIR
    -- da tela dona (depois do `on_leave`), com `on_close`. Abra em `on_enter`.
    scope: ("screen" | "window")?,
    on_open: (() -> ())?,
    on_message: ((data: string) -> ())?,
    on_error: ((message: string) -> ())?,
    on_close: (() -> ())?,
}

export type Viewport = { width: number, height: number }

-- O markup lê TEXTO, mas o script lê o tipo que gravou: `boolean`, `number`
-- ou `table` (uma lista/objeto de dados, com a marca de `json.array`/`json.object`);
-- o que veio do markup, de um widget ou de uma string escrita à mão volta
-- texto. Daí `any`: o tipo depende de quem escreveu a chave.
declare ctx: { [string]: any }
declare value: string?

declare function viewport(): Viewport

-- `opts`: method, headers, body, body_base64, body_file (caminho: o corpo é o conteúdo cru do arquivo), timeout (ms), response ("base64"),
-- `max_redirects` (segue até N redirecionamentos; padrão 0), `max_bytes` (teto do corpo da resposta),
-- `cookies` (false = ignora o jar), `signal` (um `abort_controller()`),
-- `on_progress` (function(p: FetchProgress): envio e recebimento, ver abaixo),
-- `multipart` ({ fields = {…}, files = { { name, path, filename?, content_type? } } }).
-- `phase` é "upload" (corpo do pedido) ou "download" (corpo da resposta); `total` é o
-- tamanho do corpo enviado / o Content-Length da resposta (nil se não houver).
export type FetchProgress = { phase: string, done: number, total: number?, fraction: number? }

declare function fetch(url: string, opts: any?): FetchResult

-- Cancelamento: `fetch(url, { signal = ctl })` termina com `ok = false`,
-- `error = "cancelado"` quando `ctl:abort()` é chamado.
export type AbortController = {
    abort: (self: AbortController) -> (),
    aborted: (self: AbortController) -> boolean,
}
declare function abort_controller(): AbortController

-- Esvazia o jar de cookies (do host dado, ou de todos sem argumento).
declare function clear_cookies(host: string?): ()

-- Baixa um recurso remoto direto para um arquivo local, em bytes crus (sem
-- passar pelo texto/UTF-8 do `fetch`, que corromperia um binário). `body` do
-- retorno vem sempre vazio — o conteúdo foi para `path`, não para a resposta.
-- É o caminho certo para cachear uma imagem remota e exibi-la com
-- `image(source = @path)`.
declare function download_file(url: string, path: string, opts: any?): FetchResult

-- ── http: cliente HTTP reutilizável sobre `fetch` ─────────────────────────
-- `http(base_url?, opts?)` devolve um objeto com base_url / headers / query /
-- timeout / retries de base, interceptors de requisição e resposta, e error
-- handlers. Todo método de chamada SUSPENDE, como o `fetch` cru.

export type HttpResponse = {
    ok: boolean,
    status: number,
    body: string,
    error: string,
    -- Corpo já decodificado, ou `nil` se não era JSON.
    json: any,
    -- `{ method, url }` da chamada que produziu esta resposta.
    request: { method: string, url: string },
}

export type HttpRequest = {
    method: string,
    path: string,
    headers: { [string]: any },
    query: { [string]: any },
    body: any,
    timeout: number?,
}

export type HttpError = {
    status: number,
    error: string,
    url: string,
    method: string,
    attempt: number,
    body: string,
}

export type HttpOptions = {
    headers: { [string]: any }?,
    query: { [string]: any }?,
    timeout: number?, -- ms, por requisição
    retries: number?, -- tentativas EXTRAS em falha de rede / 429 / 5xx
    retry_on: ((res: HttpResponse) -> boolean)?, -- sobrescreve o critério padrão
    cookies: boolean?, -- false = este cliente ignora o jar de cookies
}

export type HttpRequestSpec = {
    method: string?,
    path: string?,
    url: string?, -- apelido de `path`; uma URL absoluta ignora a base_url
    headers: { [string]: any }?,
    query: { [string]: any }?,
    body: any,
    body_file: string?, -- caminho: o corpo é o conteúdo cru do arquivo (sem base64)
    timeout: number?,
    cookies: boolean?, -- false = esta chamada ignora o jar
    signal: AbortController?, -- cancela a chamada (e ela não é repetida)
    on_progress: ((p: FetchProgress) -> ())?, -- envio/recebimento (~20x/s)
    -- multipart/form-data; o Content-Type (com o boundary) é do motor
    multipart: { fields: { [string]: any }?, files: { { name: string, path: string, filename: string?, content_type: string? } }? }?,
}

export type HttpClient = {
    base_url: string,
    headers: { [string]: any },
    query: { [string]: any },
    timeout: number?,
    retries: number,

    set_cookies: (self: HttpClient, on: boolean) -> HttpClient,
    clear_cookies: (self: HttpClient) -> HttpClient,
    set_base_url: (self: HttpClient, u: string) -> HttpClient,
    set_header: (self: HttpClient, k: string, v: any) -> HttpClient,
    set_headers: (self: HttpClient, t: { [string]: any }) -> HttpClient,
    remove_header: (self: HttpClient, k: string) -> HttpClient,
    set_query: (self: HttpClient, k: string, v: any) -> HttpClient,
    set_timeout: (self: HttpClient, ms: number?) -> HttpClient,
    set_retries: (self: HttpClient, n: number) -> HttpClient,
    set_retry_on: (self: HttpClient, fn: (res: HttpResponse) -> boolean) -> HttpClient,

    on_request: (self: HttpClient, fn: (req: HttpRequest) -> HttpRequest?) -> HttpClient,
    on_response: (self: HttpClient, fn: (res: HttpResponse) -> HttpResponse?) -> HttpClient,
    on_error: (self: HttpClient, fn: (err: HttpError) -> ()) -> HttpClient,

    extend: (self: HttpClient, base_path: string?, opts: HttpOptions?) -> HttpClient,

    request: (self: HttpClient, spec: HttpRequestSpec) -> HttpResponse,
    get: (self: HttpClient, path: string, spec: HttpRequestSpec?) -> HttpResponse,
    delete: (self: HttpClient, path: string, spec: HttpRequestSpec?) -> HttpResponse,
    head: (self: HttpClient, path: string, spec: HttpRequestSpec?) -> HttpResponse,
    options: (self: HttpClient, path: string, spec: HttpRequestSpec?) -> HttpResponse,
    -- O método HTTP QUERY (RFC 10008): seguro e idempotente, com a consulta no corpo.
    query: (self: HttpClient, path: string, body: any, spec: HttpRequestSpec?) -> HttpResponse,
    post: (self: HttpClient, path: string, body: any?, spec: HttpRequestSpec?) -> HttpResponse,
    put: (self: HttpClient, path: string, body: any?, spec: HttpRequestSpec?) -> HttpResponse,
    patch: (self: HttpClient, path: string, body: any?, spec: HttpRequestSpec?) -> HttpResponse,
}

declare function http(base_url: string?, opts: HttpOptions?): HttpClient

-- ── console: logs coloridos e decorados para o terminal ───────────────────
-- Nível: "debug" (0) < "info"/"log" (1) < "warn" (2) < "error" (3) < "silent" (4).
export type ConsoleConfig = {
    level: (string | number)?,
    color: boolean?,
    timestamp: boolean?,
    label: boolean?,
    prefix: string?,
}

declare console: {
    debug: (...any) -> (),
    log: (...any) -> (),
    info: (...any) -> (),
    warn: (...any) -> (),
    error: (...any) -> (),
    -- Lista de tabelas (ou de escalares) → grade ASCII. `columns` escolhe/ordena.
    table: (rows: any, columns: { string }?) -> (),
    -- Atalho de `config({ level = … })`. Devolve o próprio `console`.
    set_level: (level: string | number) -> any,
    config: (opts: ConsoleConfig?) -> any,
}

-- i18n: a locale ativa é a chave `__locale` do contexto (trocá-la reavalia as `{t("…")}` do
-- markup). O catálogo é um JSON por idioma, carregado no daemon (`locales_dir`/`locale_json`)
-- ou aqui com `load`. O que o script monta com `t`/`num`/`date` não se reavalia sozinho.
declare i18n: {
    -- A locale ativa, ou a melhor para o sistema se nenhuma foi escolhida.
    locale: () -> string,
    -- Escolhe a locale (`pt` acha `pt-br`); devolve a que ficou ativa, ou nil se nenhuma serve.
    set_locale: (pedida: string) -> string?,
    available: () -> { string },
    -- Carrega um catálogo (texto JSON); devolve quantas chaves.
    load: (locale: string, json_text: string) -> number,
    -- `args` preenche `{nome}` na mensagem e escolhe o plural (`n` ou `count`).
    t: (chave: string, args: { [string]: any }?) -> string,
    -- Traduz pelo texto de origem (gettext): sem entrada no catálogo devolve o próprio texto.
    tr: (source: string) -> string,
    num: (valor: number, casas: number?) -> string,
    -- `valor`: segundos Unix ou "AAAA-MM-DD" (UTC).
    date: (valor: any) -> string,
}

-- `audio(name = "…")`: controla a fonte pelo nome. Precisa da feature `audio` do glacier-ui.
declare audio: {
    play: (name: string) -> (),
    pause: (name: string) -> (),
    toggle: (name: string) -> (),
    stop: (name: string) -> (),
    -- Segundos.
    seek: (name: string, segundos: number) -> (),
    volume: (name: string, volume: number) -> (),
    position: (name: string) -> number,
    -- nil se o formato não informa a duração.
    duration: (name: string) -> number?,
    playing: (name: string) -> boolean,
    error: (name: string) -> string?,
    -- false se a feature `audio` está desligada neste build.
    available: () -> boolean,
}

declare function write_file(path: string, contents: string): (boolean, string?)
-- Irmão do `write_file`, mas ACRESCENTA em vez de sobrescrever — o que torna um
-- log em Luau viável sem manter o arquivo inteiro em memória. Para LER um
-- arquivo, use `fetch("file://…")`.
declare function append_file(path: string, contents: string): (boolean, string?)
-- Compacta a árvore de `origem` num `.zip` em `destino`. Síncrono (não suspende):
-- é I/O local, não rede nem diálogo.
declare function zip_dir(origem: string, destino: string): (boolean, string?)
-- `.zip` de entradas de TEXTO, sem diretório de staging. Síncronos. `zip_read` entrega
-- todas as entradas (`text` falta em diretório e em conteúdo que não é UTF-8); quem decide
-- quais nomes aceitar é o chamador.
export type ZipEntry = { name: string, dir: boolean?, text: string? }
declare function zip_write(destino: string, entradas: { ZipEntry }): (boolean, string?)
declare function zip_read(origem: string): { ok: boolean, entries: { ZipEntry }?, error: string? }
declare function sse(url: string, opts: StreamOptions?): StreamHandle
declare function websocket(url: string, opts: StreamOptions?): StreamHandle

declare function toast(opts: any): ()
declare function notify(opts: any): ()
declare function copy(text: any): ()
declare function confirm(opts: any): boolean

-- Diálogos suspensivos (Onda 8). `prompt` e `pick_color` param a corrotina e
-- voltam com a resposta; `nil` é a desistência — a mesma convenção do
-- `open_file()` cancelado, e o que separa "não respondeu" de "respondeu vazio".
-- Diálogos de arquivo NATIVOS do SO. Suspendem como o `confirm`, e `nil` é a
-- desistência. `open_files` devolve a lista de caminhos; `default_name` só é
-- lido pelo `save_file`.
export type FileFilter = { name: string, extensions: { string } }

export type FileDialogOptions = {
    title: string?,
    filters: { FileFilter }?,
    starting_dir: string?,
    default_name: string?,
}

declare function open_file(opts: FileDialogOptions?): string?
declare function open_files(opts: FileDialogOptions?): { string }?
declare function save_file(opts: FileDialogOptions?): string?
declare function pick_folder(opts: FileDialogOptions?): string?

declare function prompt(opts: any): string?
declare function pick_color(opts: any): string?
-- O QFontDialog. Devolve a família escolhida, ou `nil` na desistência.
declare function pick_font(opts: any): string?

-- O progresso NÃO suspende: ele acompanha um trabalho que continua rodando.
-- `progress_set(nil)` volta ao indeterminado.
declare function progress(opts: any): ()
declare function progress_set(value: (number | string)?, label: string?): ()
declare function progress_close(): ()

declare function navigate(tela: string): ()
declare function navigate_back(): ()
-- Troca o estilo visual: um `link(rel = style, as = nome)` do app ou um builtin
-- (`fusion`, `fusion-dark`…). Não suspende; no `init`, vale já no 1º quadro.
declare function set_style(nome: string): ()

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
    -- Marca a tabela como OBJETO (mesmo vazia): `ctx.x = json.object({})` grava `{}`;
    -- uma tabela vazia sem marca vai para o `ctx` como `[]`. `tostring(t)` de uma
    -- tabela marcada devolve o JSON dela.
    object: <T>(t: T) -> T,
}

-- Aritmética de data/hora sobre strings ISO (`YYYY-MM-DD`, `HH:MM[:SS]`, ou as
-- duas separadas por espaço) — o formato que os dateedit/timeedit/
-- datetimeedit gravam na chave. Devolve `nil` quando a entrada não é ISO
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

-- ── Testes (`glacier test`): só existem em arquivos *.test.luau ──────────────
-- Um teste é uma função global `test_*`; o `---` acima dela é a documentação.
export type TestComponent = {
    call: (self: TestComponent, action: string, value: string?) -> (),
    get: (self: TestComponent, key: string) -> string?,
    -- Como `ctx.chave = valor` num script: texto, número e booleano viram texto; uma tabela vira
    -- JSON (use `json.array({})` para a lista vazia); `nil` remove a chave.
    set: (self: TestComponent, key: string, value: any) -> (),
    -- Como o SCRIPT enxerga a chave: `boolean`/`number` se foi gravada com tipo (e
    -- ainda vale), senão texto. `get` devolve sempre texto.
    value: (self: TestComponent, key: string) -> any,
    -- O que um widget faz quando é usado: despacha a mesma mensagem no motor real
    -- (o handler de nome `action`, ou, sem ele, a escrita direta no `ctx`). `kind` é o
    -- tipo do widget: "bool" (checkbox, toggle), "number" (slider, rating, dial,
    -- pagination); omitido = texto.
    input: (self: TestComponent, action: string, value: string, kind: ("bool" | "number")?) -> (),
    data: (self: TestComponent) -> { [string]: string },
    toasts: (self: TestComponent) -> { { kind: string, message: string } },
    -- Chaves que o markup LÊ e o ctx não tem (ordenadas). Ver AGENTS.md, "Testes".
    missing: (self: TestComponent) -> { string },
    -- A árvore avaliada (ver AGENTS.md, "A árvore avaliada").
    render: (self: TestComponent) -> RenderNode,
    texts: (self: TestComponent) -> { string },
    buttons: (self: TestComponent) -> { string },
    find: (self: TestComponent, query: RenderQuery) -> RenderNode?,
    find_all: (self: TestComponent, query: RenderQuery) -> { RenderNode },
    set_viewport: (self: TestComponent, width: number, height: number) -> (),
    style: (self: TestComponent, node_or_id: RenderNode | string) -> { [string]: any },
}
export type RenderNode = {
    kind: string,
    text: string?,
    class: string?,
    id: string?,
    hidden: boolean,
    attrs: { [string]: any },
    style: { [string]: any },
    children: { RenderNode },
}
export type RenderQuery = { kind: string?, text: string?, class: string?, id: string? }
declare function assert_eq(got: any, expected: any): ()
declare function assert_ne(got: any, unexpected: any): ()
declare function assert_nil(value: any): ()
declare function assert_not_nil(value: any): ()
declare function assert_true(value: any): ()
declare function assert_false(value: any): ()
declare function assert_has(container: any, item: any): ()
declare function assert_match(text: any, pattern: string): ()
declare function assert_fails(body: () -> (), fragment: string?): ()
declare function assert_renders(path: string): ()
declare function assert_has_text(screen: TestComponent, text: string): ()
declare function assert_no_text(screen: TestComponent, text: string): ()
declare function assert_has_button(screen: TestComponent, label: string): ()
declare function assert_no_button(screen: TestComponent, label: string): ()
declare function assert_complete(screen: TestComponent, expected_missing: { string }?): ()
declare function each(cases: { { any } }, body: (...any) -> ()): ()
declare function fail(message: string?): ()
declare function component(path: string, initial_ctx: { [string]: any }?): TestComponent
