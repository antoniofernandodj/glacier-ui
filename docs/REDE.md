# Rede: `fetch`, `http()`, cookies, multipart, cancelamento e progresso

Fase 6 do `PLANO_SUBSTITUIBILIDADE.md`. Exemplo completo, em quatro formas — `fetch`, o cliente `http()`, cookies, multipart, `abort`,
`timeout`, `file://`, `download_file` e o método `QUERY`, em 21 casos na tela. Os exemplos **ligam e
desligam sozinhos** o servidor de teste (`tools/servidor_rede.py`, porta 8096; se já houver um na
porta, usam o existente). Ao rodar o script à mão: `python3 tools/servidor_rede.py`.

| | estado em Rust | estado em Luau |
|---|---|---|
| `.gva` | `rede_completa` | `rede_completa_luau` |
| `.gvb` | `gvb_rede_completa` | `gvb_rede_completa_luau` |

Os de Rust usam a API pública `glacier_ui::net` (abaixo: `Request`, `Client`…) por
`examples/comum/rede_rust.rs`; os de Luau, o `fetch` e o `http()` do prelúdio. Para o progresso
há `rede_progresso` (Luau) e `rede_progresso_rust` (Rust, com `Request::on_progress` e uma
`subscription` que leva o número à UI).

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

## Em Rust: `glacier_ui::net`

O mesmo que o `fetch` e o `http()` fazem está disponível a quem escreve um `Component` em Rust,
dentro de um `ctx.perform(async { … })`, pela mesma camada (o mesmo jar, o mesmo multipart, o
mesmo cancelamento). Dois níveis:

```rust
use glacier_ui::net::{AbortHandle, Client, Multipart, MultipartFile, Request, clear_cookies};

// Request: um pedido (o `fetch` do Luau)
let res = Request::post("https://api.example.com/upload")
    .header("Authorization", "Bearer …")
    .timeout_ms(5000)
    .multipart(Multipart { fields: vec![("title".into(), "hi".into())],
                           files: vec![MultipartFile { name: "file".into(), path: "a.pdf".into(),
                                                       filename: None, content_type: None }] })
    .send().await;                          // FetchResult { ok, status, body, error }

let ctl = AbortHandle::new();
Request::get(url).abortable(&ctl).send().await;       // ctl.abort() de outra thread
Request::get(url).no_cookies().download_to("/tmp/x.bin").send().await;
clear_cookies(None);

// Client: o `http()` do Luau, com um atalho por método
let api = Client::new("https://api.example.com")
    .default_header("Authorization", "Bearer …")
    .default_param("version", "1")
    .retries(2)
    .on_error(|e| eprintln!("{} {} → {}", e.method, e.url, e.status));

let r = api.get("/items").param("page", "2").send().await;     // r.ok(), r.status(), r.json()
api.post("/items").json(&serde_json::json!({ "name": "x" })).send().await;
api.put("/items/1").json(&body).send().await;
api.patch("/items/1").json(&patch).send().await;
api.delete("/items/1").send().await;
api.head("/items").send().await;
api.options("/items").send().await;
api.query("/search").json(&serde_json::json!({ "term": "glacier" })).send().await;
```

### Progresso (`on_progress`)

`fetch(url, { on_progress = function(p) … end })` (e a mesma opção numa chamada do `http()`)
recebe `{ phase = "upload" | "download", done, total?, fraction? }` enquanto o corpo do pedido
sobe e o da resposta desce — no máximo ~20 vezes por segundo, mais uma ao fim de cada fase.
`total` é o tamanho do corpo enviado, ou o `Content-Length` da resposta (ausente se não vier).
O handler roda como uma ação qualquer: escreve em `ctx` e a barra anda.

```lua
local r = fetch(url, { method = "POST", body = grande, on_progress = function(p)
    ctx.pct = (p.fraction or 0) * 100
end })
```

Em Rust, `Request::on_progress(|p| …)` / `RequestBuilder::on_progress` chamam o closure **na
tarefa de rede**; entregue o número à UI por um canal ou um `Arc<Mutex<_>>`.
Os exemplos `rede_progresso` e `rede_progresso_rust` sobem e baixam 50 MB com um
`<progressbar>`. Não há *streaming* do corpo da resposta (ele ainda chega inteiro no `body`).

### O método `QUERY` (RFC 10008)

`QUERY` (RFC 10008, 2026) é um método **seguro e idempotente com corpo**: como o `GET`, não muda
nada no servidor; como o `POST`, leva a consulta no corpo — para consultas grandes ou
estruturadas demais para uma URL. O servidor deve rejeitar um `QUERY` sem conteúdo ou com
`Content-Type` ausente/incoerente (§2), então **o cliente nem envia** um `QUERY` sem corpo ou sem
`Content-Type` (`.json(…)` põe os dois): volta `ok = false`, `status = 0`, sem tocar na rede. Em
Luau: `api:query("/search", { termo = "glacier" })`.

### Retries

`retries(n)` refaz a chamada em falha de rede, `429` e `5xx` — **só para métodos idempotentes**
(`GET`, `HEAD`, `OPTIONS`, `PUT`, `DELETE`, `QUERY`): o primeiro `POST`/`PATCH` pode ter chegado ao
servidor. `retry_on` troca a regra; uma chamada cancelada nunca se repete. Vale igual para o
`http()` do Luau.

## Como foi verificado

O `rede_completa` roda contra o servidor de teste e mostra, em cada caso, o que o servidor
**de fato recebeu** (cabeçalhos, corpo, cookies, partes do multipart). Os 20 casos passam nas
quatro variantes, com os mesmos valores (a única diferença esperada é o caso 5, em que o Rust
confere o tamanho do arquivo gravado e o Luau o relê em base64). Rodado num X virtual (`Xvfb`).

## O que NÃO está feito

- **Progresso** (`on_progress(feito, total)`) de upload e de download. Exige entregar eventos
  repetidos de uma tarefa à corrotina Lua, que hoje só recebe um resultado final. É o item que
  falta para o critério de aceitação do plano ("upload de 50 MB com barra de progresso").
- Streaming do upload e do download: o corpo ainda é montado e lido por inteiro.
