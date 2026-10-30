# Rede do `fetch`: cookies, multipart e cancelamento

Fase 6 do `PLANO_SUBSTITUIBILIDADE.md`. Exemplo completo: `examples/gva/rede_completa` — `fetch`, o
cliente `http()`, cookies, multipart, `abort`, `timeout`, `file://` e `download_file`, em 20
casos na tela (precisa do servidor de teste: `python3 tools/servidor_rede.py`).

O Glacier só traz **dados** de fora, pelo `fetch` da camada Lua. Isto é o que esse `fetch` aprendeu.

## Cookies

Um jar **por host** (com a porta), em memória, ligado por padrão: o `Set-Cookie` de uma resposta
é guardado e o `Cookie` vai sozinho nos pedidos seguintes ao mesmo host.

```lua
fetch("https://api.exemplo/login", { method = "POST", body = creds })   -- guarda a sessão
local res = fetch("https://api.exemplo/perfil")                          -- a sessão vai sozinha
fetch(url, { cookies = false })                                         -- este pedido ignora o jar
clear_cookies()            -- esvazia tudo
clear_cookies("api.exemplo")  -- só deste host
```

- **Só do host que o definiu.** O atributo `Domain` é ignorado: um cookie nunca vai para um
  host que não o escreveu.
- Respeita `Path`, `Secure` (só em `https`) e `Max-Age` (`<= 0` apaga). `Expires` e `SameSite`
  **não** são lidos: um cookie sem `Max-Age` vale até o app fechar.
- Um header `Cookie` escrito por você em `headers` vence o jar.
- O jar **não** vai para o disco: não sobrevive ao app.

## `multipart/form-data`

```lua
local res = fetch(url, {
    method = "POST",
    multipart = {
        fields = { titulo = "olá", versao = 2 },
        files = { { name = "anexo", path = "relatorio.pdf",
                    filename = "r.pdf", content_type = "application/pdf" } },
    },
})
```

Os arquivos são lidos do disco no momento do envio (sem travar a janela) e o `Content-Type`,
com o boundary, é o do motor — vence um que você ponha em `headers`. `filename` padrão é o nome
do arquivo em `path`; `content_type` padrão é `application/octet-stream`. Aspas e quebras de
linha nos nomes são trocadas por `_` (senão quebrariam o cabeçalho).

**Limite:** cada arquivo é lido **inteiro em memória**. Serve para anexos e imagens; não para um
arquivo de gigabytes.

## Cancelamento

```lua
local ctl = abort_controller()
after(5000, function() ctl:abort() end)          -- ou um botão
local res = fetch(url, { signal = ctl })
if res.error == "cancelado" then … end           -- ok = false, status = 0
```

`ctl:abort()` interrompe o pedido onde ele estiver (conexão, envio ou leitura), e `ctl:aborted()`
diz se já foi chamado. Um mesmo controle pode cancelar vários `fetch`.

## No cliente `http()`

O cliente `http(base_url, opts)` repassa tudo ao `fetch`:

```lua
local api = http("https://api.exemplo", { headers = { ["X-Api-Key"] = k }, cookies = true })
api:post("/upload", nil, { multipart = { fields = { t = "x" }, files = {{ name = "f", path = "a.pdf" }} } })
api:get("/lento", { signal = ctl })     -- cancela; e um cancelamento NUNCA é repetido pelos retries
api:set_cookies(false)                  -- este cliente ignora o jar
api:clear_cookies()                     -- esvazia os cookies do host da base_url
```

`cookies` no `http(base, opts)` é o padrão do cliente; um `cookies` na chamada vence.

## Como foi verificado

O `rede_completa` roda contra o servidor de teste e mostra, em cada caso, o que o servidor
**de fato recebeu** (cabeçalhos, corpo, cookies, partes do multipart). Os 20 casos passam
(`docs/REDE.md` não lista os valores: eles estão na tela). Rodado num X virtual (`Xvfb`).

## O que NÃO está feito

- **Progresso** (`on_progress(feito, total)`) de upload e de download. Exige entregar eventos
  repetidos de uma tarefa à corrotina Lua, que hoje só recebe um resultado final. É o item que
  falta para o critério de aceitação do plano ("upload de 50 MB com barra de progresso").
- Streaming do upload e do download: o corpo ainda é montado e lido por inteiro.
