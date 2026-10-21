-- Ambiente de testes Luau (`glacier test`). Carregado na VM de cada arquivo
-- `*.test.luau` antes do corpo dele: registra `assert_*`/`each` e `__glacier_run`,
-- que o runner em Rust chama depois que o arquivo carregou. Um teste é uma função
-- global `test_*` (descoberta pelo nome, ordenada pela linha); o formato antigo
-- (`describe`/`test`/`expect`) segue valendo, obsoleto.

local tests = {}
local names: { string } = {}
local befores: { { () -> () } } = { {} }

function describe(name: string, body: () -> ())
    table.insert(names, name)
    -- cada describe herda os before_each do pai, sem vazar para os irmãos
    local inherited = {}
    for _, fn in ipairs(befores[#befores]) do
        table.insert(inherited, fn)
    end
    table.insert(befores, inherited)
    body()
    table.remove(befores)
    table.remove(names)
end

function before_each(fn: () -> ())
    table.insert(befores[#befores], fn)
end

function test(name: string, body: () -> ())
    local full = table.concat(names, " › ")
    if full ~= "" then
        full = full .. " › "
    end
    table.insert(tests, { name = full .. name, body = body, before = table.clone(befores[#befores]) })
end
it = test

local function show(v: any): string
    if type(v) == "string" then
        return string.format("%q", v)
    elseif type(v) == "table" then
        local n = #v
        local count = 0
        for _ in pairs(v) do
            count = count + 1
        end
        if count == n then
            -- sequência: `{ "a", "b" }`, como se escreve
            local items = {}
            for i = 1, n do
                table.insert(items, show(v[i]))
            end
            return n == 0 and "{}" or ("{ " .. table.concat(items, ", ") .. " }")
        end
        local parts = {}
        local keys = {}
        for k in pairs(v) do
            table.insert(keys, k)
        end
        table.sort(keys, function(a, b)
            return tostring(a) < tostring(b)
        end)
        for _, k in ipairs(keys) do
            table.insert(parts, tostring(k) .. " = " .. show(v[k]))
        end
        return "{ " .. table.concat(parts, ", ") .. " }"
    end
    return tostring(v)
end

local function deep_equal(a: any, b: any): boolean
    if a == b then
        return true
    end
    if type(a) ~= "table" or type(b) ~= "table" then
        return false
    end
    for k, v in pairs(a) do
        if not deep_equal(v, b[k]) then
            return false
        end
    end
    for k in pairs(b) do
        if a[k] == nil then
            return false
        end
    end
    return true
end

-- Uma falha de asserção é uma tabela, não uma string: o runner mostra o `tipo`,
-- o `detalhe` (obtido/esperado) e a LINHA do `assert_*` que falhou.
local function raise(kind: string, detail: string)
    error({ __assert = true, kind = kind, detail = detail, line = debug.info(3, "l") }, 0)
end

function fail(message: string?)
    error({ __assert = true, kind = "fail", detail = message or "falhou", line = debug.info(2, "l") }, 0)
end

local Matchers = {}
Matchers.__index = Matchers

local function check(self: any, passed: boolean, expectation: string, detail: string)
    if self.negated then
        passed = not passed
        expectation = "não " .. expectation
    end
    if not passed then
        error("esperava " .. expectation .. "\n    " .. detail, 3)
    end
end

function Matchers.toBe(self, expected)
    check(self, self.actual == expected, "ser " .. show(expected), "recebeu " .. show(self.actual))
end

function Matchers.toEqual(self, expected)
    check(self, deep_equal(self.actual, expected), "ser igual a " .. show(expected), "recebeu " .. show(self.actual))
end

function Matchers.toBeNil(self)
    check(self, self.actual == nil, "ser nil", "recebeu " .. show(self.actual))
end

function Matchers.toBeTruthy(self)
    check(self, self.actual ~= nil and self.actual ~= false, "ser verdadeiro", "recebeu " .. show(self.actual))
end

function Matchers.toBeFalsy(self)
    check(self, self.actual == nil or self.actual == false, "ser falso", "recebeu " .. show(self.actual))
end

function Matchers.toContain(self, item)
    local found = false
    if type(self.actual) == "string" then
        found = string.find(self.actual, item, 1, true) ~= nil
    elseif type(self.actual) == "table" then
        for _, v in pairs(self.actual) do
            if deep_equal(v, item) then
                found = true
                break
            end
        end
    end
    check(self, found, "conter " .. show(item), "em " .. show(self.actual))
end

function Matchers.toMatch(self, pattern)
    local ok = type(self.actual) == "string" and string.find(self.actual, pattern) ~= nil
    check(self, ok, "casar com o padrão " .. show(pattern), "recebeu " .. show(self.actual))
end

-- `expect(function() … end):toThrow("trecho da mensagem")`
function Matchers.toThrow(self, fragment)
    local ok, err = pcall(self.actual)
    local threw = not ok
    if threw and fragment ~= nil then
        threw = string.find(tostring(err), fragment, 1, true) ~= nil
    end
    check(self, threw, "lançar erro" .. (fragment and (" contendo " .. show(fragment)) or ""),
        ok and "a função não lançou" or ("lançou " .. show(tostring(err))))
end

function expect(actual: any)
    local self = { actual = actual, negated = false }
    self.never = setmetatable({ actual = actual, negated = true }, Matchers)
    return setmetatable(self, Matchers)
end


-- ── asserções: (obtido, esperado) ────────────────────────────────────────────

local function got_expected(got: any, expected: any): string
    return "  obtido:   " .. show(got) .. "\n  esperado: " .. show(expected)
end

function assert_eq(got: any, expected: any)
    if not deep_equal(got, expected) then
        raise("assert_eq", got_expected(got, expected))
    end
end

function assert_ne(got: any, unexpected: any)
    if deep_equal(got, unexpected) then
        raise("assert_ne", "  obtido: " .. show(got) .. "\n  (devia ser diferente)")
    end
end

function assert_nil(v: any)
    if v ~= nil then
        raise("assert_nil", "  obtido: " .. show(v))
    end
end

function assert_not_nil(v: any)
    if v == nil then
        raise("assert_not_nil", "  obtido: nil")
    end
end

function assert_true(v: any)
    if v == nil or v == false then
        raise("assert_true", "  obtido: " .. show(v))
    end
end

function assert_false(v: any)
    if v ~= nil and v ~= false then
        raise("assert_false", "  obtido: " .. show(v))
    end
end

local function contains(container: any, item: any): boolean
    if type(container) == "string" then
        return type(item) == "string" and string.find(container, item, 1, true) ~= nil
    elseif type(container) == "table" then
        for _, v in pairs(container) do
            if deep_equal(v, item) then
                return true
            end
        end
    end
    return false
end

function assert_has(container: any, item: any)
    if not contains(container, item) then
        raise("assert_has", "  em:       " .. show(container) .. "\n  faltou:   " .. show(item))
    end
end

function assert_match(text: any, pattern: string)
    if type(text) ~= "string" or string.find(text, pattern) == nil then
        raise("assert_match", "  obtido:  " .. show(text) .. "\n  padrão:  " .. show(pattern))
    end
end

-- `assert_fails(function() … end, "trecho da mensagem")`
function assert_fails(fn: () -> (), fragment: string?)
    local ok, err = pcall(fn)
    if ok then
        raise("assert_fails", "  a função não lançou" .. (fragment and (" (esperava " .. show(fragment) .. ")") or ""))
    end
    if type(err) == "table" and err.__assert then
        err = err.detail
    end
    if fragment ~= nil and string.find(tostring(err), fragment, 1, true) == nil then
        raise("assert_fails", "  lançou:   " .. show(tostring(err)) .. "\n  esperado: " .. show(fragment))
    end
end

-- `assert_complete(tela, { "ausente_esperada" })`: as chaves que o markup lê e o
-- ctx não tem (`tela:missing()`) devem ser EXATAMENTE `expected`.
function assert_complete(screen: any, expected: { string }?)
    local want = {}
    for _, k in ipairs(expected or {}) do
        want[k] = true
    end
    local have = {}
    local extra = {}
    for _, k in ipairs(screen:missing()) do
        have[k] = true
        if not want[k] then
            table.insert(extra, k)
        end
    end
    local lacking = {}
    for _, k in ipairs(expected or {}) do
        if not have[k] then
            table.insert(lacking, k)
        end
    end
    if #extra > 0 or #lacking > 0 then
        local lines = {}
        if #extra > 0 then
            table.insert(lines, "  o markup lê e ninguém publicou: " .. table.concat(extra, ", "))
        end
        if #lacking > 0 then
            table.insert(lines, "  esperadas como ausentes, mas existem: " .. table.concat(lacking, ", "))
        end
        raise("assert_complete", table.concat(lines, "\n"))
    end
end

-- `each({ { entrada, esperado }, … }, function(entrada, esperado) … end)`: roda a
-- função em cada linha; a falha diz QUAL linha foi.
function each(cases: { { any } }, fn: (...any) -> ())
    for i, row in ipairs(cases) do
        local n = 0
        for k in pairs(row) do
            if type(k) == "number" and k > n then
                n = k
            end
        end
        local ok, err = pcall(fn, table.unpack(row, 1, n))
        if not ok then
            local label = "caso #" .. i .. " " .. show(row)
            if type(err) == "table" and err.__assert then
                err.detail = "  " .. label .. "\n" .. err.detail
                error(err, 0)
            end
            error(label .. ": " .. tostring(err), 0)
        end
    end
end

-- ── o runner ────────────────────────────────────────────────────────────────

-- "grupo__caso_x" → "grupo › caso x"; sem `__`, só troca `_` por espaço.
local function display_name(raw: string): string
    local group, rest = string.match(raw, "^(.-)__(.+)$")
    if group ~= nil and group ~= "" then
        return (string.gsub(group, "_", " ")) .. " › " .. (string.gsub(rest, "_", " "))
    end
    return (string.gsub(raw, "_", " "))
end

-- Chamado pelo runner: roda cada teste isolado e devolve uma lista de
-- { name, ok, line, kind, detail, assert_line, trace, ms, logs }.
function __glacier_run(filter: string?)
    local queue = {}
    for _, t in ipairs(tests) do
        table.insert(queue, { name = t.name, raw = t.name, body = t.body, before = t.before })
    end
    local found = {}
    for key, fn in pairs(_G) do
        if type(key) == "string" and type(fn) == "function" and string.sub(key, 1, 5) == "test_" then
            local raw = string.sub(key, 6)
            table.insert(found, { name = display_name(raw), raw = raw, body = fn, line = debug.info(fn, "l") })
        end
    end
    table.sort(found, function(a, b)
        return (a.line or 0) < (b.line or 0)
    end)
    for _, t in ipairs(found) do
        table.insert(queue, t)
    end

    local setup_fn = rawget(_G, "setup")
    local real_print = print
    local logs: { string } = {}
    print = function(...)
        local parts = {}
        for i = 1, select("#", ...) do
            parts[i] = tostring((select(i, ...)))
        end
        table.insert(logs, table.concat(parts, "\t"))
    end

    local results = {}
    for _, t in ipairs(queue) do
        if filter == nil or string.find(t.name, filter, 1, true) or string.find(t.raw, filter, 1, true) then
            logs = {}
            local started = os.clock()
            local ok, err = xpcall(function()
                for _, fn in ipairs(t.before or {}) do
                    fn()
                end
                if t.line ~= nil and type(setup_fn) == "function" then
                    setup_fn()
                end
                t.body()
            end, function(e)
                return { e = e, trace = debug.traceback("", 2) }
            end)
            local r = { name = t.name, ok = ok, line = t.line, ms = (os.clock() - started) * 1000, logs = logs }
            if not ok then
                local e = err.e
                r.trace = err.trace
                if type(e) == "table" and e.__assert then
                    r.kind = e.kind
                    r.detail = e.detail
                    r.assert_line = e.line
                else
                    local msg = tostring(e)
                    r.detail = msg
                    r.assert_line = tonumber(string.match(msg, '"%]:(%d+):'))
                end
            end
            table.insert(results, r)
        end
    end
    print = real_print
    return results
end
