# Plano: uma sintaxe própria para os testes Luau

**Estado:** **implementado** no glacier-ui 0.128 (runner) e no glacier-cli 0.9 (flags e presets); a sintaxe
antiga (`describe`/`expect`) foi **removida** no glacier-ui 0.129 e no glacier-cli 0.10.
Escrito em 2026-10-21, depois de 340 testes escritos no `rustploy-gui` com a sintaxe antiga. As
decisões da última seção foram tomadas; a seção 2.7 (verbosidade) entrou junto com a
implementação.

## 1. O problema

O runner do `glacier test` (0.124) fala Jest:

```lua
describe("contador", function()
    before_each(function() ... end)

    test("soma e subtrai", function()
        local tela = component("views/app.gvb")
        tela:call("somar")
        expect(tela:get("contador")):toBe("1")
        expect(tela:toasts()).never:toEqual({})
    end)
end)
```

Três coisas incomodam:

- **Parece coisa de Node.** `describe`/`it`/`expect` vêm do Jest, e o resto do motor não fala
  assim. Os globais do Luau do motor são `toast`, `open_window`, `append_textarea`: verbos curtos
  em snake_case.
- **camelCase.** `toBe`, `toEqual`, `toBeNil`, `toBeTruthy`, `toContain`, `toThrow`, `.never`.
  A convenção do repositório é `snake_case` (ver o `CLAUDE.md`), e esta é a única API do motor
  que a quebra.
- **Encadeamento.** `expect(a):toBe(b)` é uma frase com sujeito, verbo e advérbio para dizer
  `a == b`. Negar exige outra peça (`.never`), e a falha não diz em que linha do teste ela está
  (hoje sai `[string "<script:…>"]:47`).

Há também um custo prático, visto ao escrever 328 testes: metade deles são laços
`for _, caso in {...}`, e a conta "essas chaves, nem mais nem menos" do `tela:missing()` foi
reescrita à mão num teste inteiro.

## 2. A proposta

### 2.1 O teste é uma função `test_*`

Sem `describe`, sem registro, sem `it`. O motor já descobre `init` e os handlers **pelo nome**, e
descobrir teste pelo nome segue a mesma regra (Rust, Go e Python fazem igual):

```lua
function test_filtrar_deixa_so_o_trecho()
    local tela = nova()
    tela:call("filtrar", "API")
    assert_eq(nomes(tela), { "api-gateway" })
    assert_eq(tela:get("total"), "3")
end
```

Descoberta: depois de carregar o arquivo, o runner percorre `_G`, pega as funções cujo nome
começa com `test_` e as ordena pela **linha de definição** (`debug.info(fn, "l")`), para a saída
seguir a ordem do arquivo (`_G` não tem ordem).

### 2.2 Asserções são funções `assert_*`, `(obtido, esperado)`

No estilo dos `assert_eq!`/`assert_ne!` do Rust. A ordem é sempre `(obtido, esperado)`. O negado
é outra função, não um `.never`.

| hoje | proposta |
|---|---|
| `expect(a):toBe(b)` / `:toEqual(b)` | `assert_eq(a, b)` |
| `expect(a).never:toBe(b)` | `assert_ne(a, b)` |
| `expect(t):toContain(x)` | `assert_has(t, x)` |
| `expect(t):toMatch(p)` | `assert_match(t, p)` |
| `expect(v):toBeNil()` / `.never:toBeNil()` | `assert_nil(v)` / `assert_not_nil(v)` |
| `expect(v):toBeTruthy()` / `:toBeFalsy()` | `assert_true(v)` / `assert_false(v)` |
| `expect(fn):toThrow("x")` | `assert_fails(fn, "x")` |
| `fail("msg")` | `fail("msg")` (fica) |
| `expect(tela:missing()):toEqual({})` | `assert_complete(tela, { "ausente_esperada" })` |

`assert_complete(tela, esperadas)` esconde a conta de conjuntos do `tela:missing()`: passa se as
chaves ausentes forem **exatamente** `esperadas`, e a falha nomeia as que sobraram e as que
faltaram (hoje cada projeto reescreve essa comparação).

### 2.3 Tabela de casos: `each`

```lua
function test_rotulo_de_recorrencia()
    each({
        { nil,                                  "manual" },
        { { IntervalHours = 6 },                "a cada 6h" },
        { { Daily = { hour = 3, minute = 5 } }, "diariamente às 03:05" },
    }, function(entrada, esperado)
        assert_eq(Jobs.recurrence_label(entrada), esperado)
    end)
end
```

Cada linha da tabela é um caso. Quando um falha, a mensagem diz **qual** (o índice e os valores
da linha), em vez de apontar o `assert_eq` de dentro do laço.

### 2.4 `setup()` e grupos

- `setup()`, se existir, roda antes de cada teste do arquivo (no lugar de `before_each`).
- O **grupo é o arquivo**. Se um arquivo precisar de subgrupos, o sufixo `__` separa:
  `test_nav__abas_gravam_a_chave`; o runner imprime o que vem antes de `__` como cabeçalho.

### 2.5 Documentação do teste: comentário `---`

O Luau não tem docstring (uma string solta no começo da função não é instrução válida). A
documentação vai num bloco de `---` logo acima da função:

```lua
--- Filtrar por "API" deixa só o serviço que contém o trecho.
--- O total continua 3: o filtro esconde linhas, não apaga serviços.
function test_filtrar_deixa_so_o_trecho()
```

Como funciona:

- O runner é Rust e já lê o arquivo (`read_to_string`). Para cada `test_*` ele pega a linha de
  definição e junta os `---` consecutivos imediatamente acima. Os números de linha batem com o
  arquivo (o chunk é carregado com o nome do arquivo, e hoje o erro já aponta a linha real).
- Custo zero em execução, nenhuma sintaxe nova: o Luau vê comentário.
- O `luau-lsp` já trata `---` como documentação: o texto aparece no hover do editor.

Onde aparece:

- **Sem `-v`:** nada, a saída continua enxuta.
- **Com `-v`:** o **primeiro parágrafo** (até a linha em branco) sob o nome do teste.
- **Em falha, sempre:** o bloco inteiro, antes do `obtido`/`esperado`. É a *intenção* do teste, que
  é o que falta quando só se vê dois valores diferentes.

### 2.7 Verbosidade e logs

| flag | mostra |
|---|---|
| (nenhuma) | uma linha por arquivo (`N ✓  M ✗`); cada falha por inteiro |
| `-v` | o nome de cada teste, `✓`/`✗` |
| `-vv` | + a documentação (primeiro parágrafo do `---`) |
| `-vvv` | + o tempo de cada teste e o stacktrace das falhas |
| `-s`, `--silent` | esconde os logs |

As letras se combinam: `glacier test -svvv`. Os **logs** (`print`, `console.*` de dentro de um
teste) aparecem por padrão, recuados sob o nome do teste que os escreveu; o runner substitui o
`print` durante cada teste e devolve o que foi escrito. Uma falha mostra sempre, qualquer que seja o
nível: a linha do teste, a documentação inteira e o obtido/esperado. A flag chega ao app por
`GLACIER_TEST_VERBOSE` (0–3) e `GLACIER_TEST_SILENT` (`1`), como `GLACIER_TEST_FILTER`.

### 2.6 Como fica a saída (com `-v`, `-vv` e numa falha)

Sem `-v`:

```
tests/itens.test.luau
  ✓ filtrar deixa so o trecho
  ✓ editar e excluir sem selecao so avisam

2 passaram, 0 falharam
```

Com `-v`:

```
tests/itens.test.luau
  ✓ filtrar deixa so o trecho
      Filtrar por "API" deixa só o serviço que contém o trecho.
  ✓ editar e excluir sem selecao so avisam
      Sem linha selecionada, os dois só mostram um toast de aviso.
```

Numa falha (note a linha do teste, que hoje não existe):

```
✗ filtrar deixa so o trecho              tests/itens.test.luau:12
    Filtrar por "API" deixa só o serviço que contém o trecho.
    O total continua 3: o filtro esconde linhas, não apaga serviços.

    assert_eq
      obtido:   { "api-gateway", "fila-jobs" }
      esperado: { "api-gateway" }
```

O nome impresso é o da função sem o `test_`, com `_` trocado por espaço. A linha do `assert_*` que
falhou vem de `debug.info(2, "l")` no momento da chamada.

## 3. Alternativa mais leve

Manter a chamada de registro, só sem o jeito Node:

```lua
--- Filtrar por "API" deixa só o serviço que contém o trecho.
test("filtrar deixa só o trecho", function()
    assert_eq(...)
end)
```

com as mesmas `assert_*` e `each`. O `---` continua funcionando: o runner procura os comentários
acima da **linha da chamada** `test(...)`, em vez da linha da função. Conserva os nomes com acento
e espaço e a ordem natural do arquivo; perde só a descoberta por nome (e, com ela, o aviso de
nome duplicado, já que o nome passa a ser um texto livre).

## 4. O que isso custa

- **O nome perde acento e espaço**: "ida e volta preserva yaml e toml" vira
  `test_ida_e_volta_preserva_yaml_e_toml`. Lê bem, mas é uma troca.
- **Colisão de nomes**: duas funções com o mesmo nome no mesmo arquivo — a segunda sobrescreve a
  primeira, em silêncio. O runner pode avisar quando o arquivo *define* `test_x` duas vezes (ele
  lê o texto, então consegue contar).
- **Função `test_*` que não é teste**: um helper chamado `test_algo` seria executado. É a mesma
  armadilha de qualquer descoberta por prefixo; a regra "helper não começa com `test_`" cabe na
  documentação.
- **Migração**: 10 presets do `glacier new`, a seção "Testes" do `AGENTS.md` gerado, o
  `glacier.d.luau` e os testes dos projetos que já usam o formato atual (o `rustploy-gui` tem 340).

## 5. Migração

Houve uma versão de transição (glacier-ui 0.128), em que o runner rodava os dois formatos. Na
0.129 a sintaxe antiga saiu: `describe`, `test`, `it`, `before_each` e `expect` não existem mais, e
um arquivo que ainda os usa falha ao carregar com `attempt to call a nil value`. Cada
`expect(a):toBe(b)` vira `assert_eq(a, b)` (tabela da seção 2.2) e cada `test("nome", …)` vira
`--- nome` + `function test_nome()`.

## 6. Passos

1. `src/luau/testing.luau`: `assert_*`, `each`, `setup`, o contador de casos e o formato das
   mensagens com `obtido`/`esperado` e a linha (`debug.info`).
2. `src/luau/testing.rs`: descoberta de `test_*` por prefixo e ordem por linha; extração dos `---`
   acima de cada função; aviso de nome duplicado; formatação da saída (`-v`, doc em falha).
3. `crates/glacier-cli`: a flag `-v`/`--verbose` em `glacier test` (repassada ao app por uma
   variável de ambiente, como `GLACIER_TEST_FILTER`); a ajuda; `templates/_comum/views/scripts/glacier.d.luau`
   com as declarações novas.
4. Os testes dos 10 presets e a seção "Testes" do `AGENTS.md` passam para a sintaxe nova.
5. `CHANGELOG`: motor (runner) e CLI (`-v`, templates).
6. Os testes do `rustploy-gui` migram num projeto à parte, depois.

## 7. Decisões

1. **Descoberta por nome** (`test_*`), não registro.
2. **Asserções em inglês** (`assert_eq`), como os globais do motor.
3. **Globais**, sem `require`.
4. **`-vv` mostra o primeiro parágrafo** do doc; a falha mostra o doc inteiro.
5. **`---` em helper e em `setup()`** não tem efeito no runner (só os `test_*` são lidos).
6. **Sem `teardown()`**: só `setup()`, como era o `before_each`.
7. **Sintaxe antiga**: aceita só na 0.128; removida na 0.129.
8. **Grupo**: `grupo__caso` aparece como `grupo › caso`.
