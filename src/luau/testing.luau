-- Ambiente de testes Luau (`glacier test`). Carregado na VM de cada arquivo
-- `*.test.luau` antes do corpo dele: registra `describe`/`test`/`expect` e
-- `__glacier_run`, que o runner em Rust chama depois que o arquivo carregou.

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

function fail(message: string?)
    error(message or "falhou", 2)
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

-- Chamado pelo runner: roda cada teste isolado e devolve { {name, ok, err} }.
function __glacier_run(filter: string?)
    local results = {}
    for _, t in ipairs(tests) do
        if filter == nil or string.find(t.name, filter, 1, true) then
            local ok, err = pcall(function()
                for _, fn in ipairs(t.before) do
                    fn()
                end
                t.body()
            end)
            table.insert(results, { name = t.name, ok = ok, err = (not ok) and tostring(err) or nil })
        end
    end
    return results
end
