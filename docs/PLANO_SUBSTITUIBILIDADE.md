# Plano: Glacier como "micro navegador" (substituibilidade da web)

**Estado:** em andamento (2026-10-30). Fases 0 e 2 feitas, com as ressalvas listadas em
cada uma (`docs/MEDICOES.md`, `docs/TEXTO.md`); **Fase 1 descartada** (nada remoto
além de dados); 3 a 8 não começaram. Proposta de 2026-10-29, glacier-ui 0.139.0. Nasce de uma auditoria do que o
motor cobre da plataforma web. A premissa: alguém que sobe **hoje** um serviço novo e
quer entregá-lo como markup + stylesheet + script + camada de rede, embutidos num
binário pequeno, em vez de uma stack web.

> A auditoria foi feita **lendo código e docs**. Nada foi medido: consumo de memória,
> tamanho de binário e tempo de início ficam como a Fase 0 deste plano. Sem número, a
> afirmação "consome menos que a web" é hipótese, não resultado.

---

## 1. Diagnóstico

### 1.1 O que já substitui

| Peça da web | No Glacier | Estado |
|---|---|---|
| HTML | `.gv`/`.gvb`, 60+ tags (`grid`, `flow`, `tableview`, `treeview`, `form`, `select`, `calendar`, gráficos…) | forte |
| CSS | `.gss`: classes, `#id`, tag, combinadores, pseudo-estados, `@media`, `var(--x)`, `gradient`, escopo por componente | bom |
| JS | Luau em `<script>`: corrotinas, `require`, timers, `storage` | bom |
| `fetch` | HTTP/HTTPS (hyper + rustls), gzip, timeout, headers | cobre API JSON |
| SSE / WebSocket | `sse()` / `websocket()` | presente |
| localStorage | `storage` | presente |
| Roteamento | `screen`, `navigate`, multi-janela | presente |
| Formulários | Reactive Forms | melhor que o nativo |
| Dev loop | hot-reload de `.gv`/`.gss`/JSON/Luau | presente |
| Empacotamento | `embed_assets!`, binário único | melhor que a web |

### 1.2 O que falta

**Bloqueadores** (decidem se dá para chamar de substituto):

1. **Texto.** Sem `<a>`, sem texto inline misto (negrito no meio da frase), sem
   Markdown; seleção e cópia de texto estático não apareceram.
2. **Acessibilidade.** Nenhuma menção a AccessKit, ARIA ou leitor de tela.
3. **Carga remota** (descartada, ver Fase 1). O `AssetSource` só tem disco e embutido. Não existe "abrir uma
   URL e carregar o app"; cada atualização é um binário novo.
4. **Mídia.** Sem `<audio>` nem `<video>`. Hoje só a `webview` nativa quebra o galho.
5. **Alvo WASM incompleto.** Sem Luau e sem `fetch` no navegador (`WEB.md`); negrito e
   itálico somem sem fonte própria; GPU desligada por padrão; WebGPU/WebGL nunca
   vistos funcionando.

**Lacunas de plataforma:**

- Rede: sem cookies/sessão, multipart, download com progresso, cache, offline.
- CSS: sem `transition`, `@keyframes`, `calc()`, `@font-face`, `opacity`, `filter`,
  `position`/`z-index` genéricos, `::before`, `:nth-*`.
- i18n: sem tradução, plural, formato por locale, RTL/bidi.
- Descoberta: sem URL por tela, sem conteúdo indexável (na web o app é um `<canvas>`).

### 1.3 Nota de partida

| Cenário | Nota |
|---|---|
| App interno / dashboard (CRUD, API JSON, tempo real) | 0,6 |
| SaaS público voltado ao cliente final | 0,25 |
| Web inteira, ponderada por uso | 0,10 a 0,15 |
| **Geral para o caso descrito (serviço novo, simples)** | **0,3** |

**Meta do plano:** levar o geral a **~0,5** e o cenário de app interno a **~0,8**. Não
é meta chegar perto da web de conteúdo (SEO, mídia aberta): isso fica fora do escopo
(seção 5).

---

## 2. Fases

Ordem por **retorno sobre custo**, não por tamanho. Cada fase tem um critério de
aceitação verificável (convenção do `CLAUDE.md`: olhar a árvore avaliada e rodar o
exemplo, não só o teste).

### Fase 0 — Medir (pré-requisito)

Sem isso não há como dizer "menos recursos que o navegador".

- Exemplo de referência: um painel com lista, formulário, `fetch` e um gráfico.
- Medir: memória residente (Resident Set Size, `VmRSS` do kernel) em repouso e em uso, tamanho do binário (release, stripped), tempo até a
  primeira tela. Comparar com o mesmo painel numa aba do Chrome (memória residente da aba + processo
  GPU) e com um equivalente Tauri.
- Resultado em `docs/MEDICOES.md`, com a máquina e o backend (`WGPU_BACKEND=gl` aqui).

**Aceitação:** três números por alvo, reproduzíveis por um `make` documentado.

### Fase 1 — Carga remota (DESCARTADA)

**Decidido em 2026-10-30: o Glacier não carrega app, tela nem asset de fora.** Tudo
é local — `.gv`/`.gva`/`.gvb`, `.gss`, scripts, imagens, telas. A única coisa que vem
de fora é dado, pelo `fetch` da camada Lua.

A Fase 1 foi implementada (`HttpAssets` com manifesto ed25519, cache e modo offline;
`RoteadorAssets` para telas remotas) e depois **removida por completo**: carregar
markup e script de uma URL é execução remota de código, a superfície de ataque que o
projeto decidiu não assumir (assinatura, rollback de manifesto, política de confiança
por origem, confiança em chave…). O histórico git guarda a implementação
(commit `5fb9487`) para quem precisar consultar; não há plano de retomá-la.

A idéia de "micro navegador" fica, portanto, restrita a: motor pequeno + app local +
dados remotos. Ver "Fora do escopo".

### Fase 2 — Texto rico, links e seleção

- `<a href="…" on_click="…">` com estados de foco/hover, abrindo URL externa ou
  navegando entre telas.
- Texto inline misto: `<text>` com filhos `<b>`, `<i>`, `<code>`, `<a>`. O `iced` tem
  `rich_text`/`span`; o trabalho é o parser e o `{interpolação}` dentro dos spans.
- `<markdown>` (via `iced::widget::markdown` ou `pulldown-cmark`).
- Seleção e cópia de texto estático (`selectable="true"`); avaliar o custo no upstream.

**Aceitação:** uma tela de "Sobre" com parágrafo, link, código inline e lista, tudo
selecionável; verificar a árvore avaliada dos spans (não só que renderiza).

### Fase 3 — Acessibilidade

Decide o uso público e, em alguns mercados, a conformidade legal.

- Levantar o estado do AccessKit no `iced`/`winit` na versão usada; **a viabilidade
  depende do upstream**. Se não houver caminho, documentar e considerar contribuir.
- Papéis e rótulos no markup: `role`, `label`/`aria_label`, ordem de foco.
- Navegação por teclado completa (Tab, setas, Enter/Espaço) nos builtins.
- Contraste: aviso do `.gss` quando o par `color`/`background` falha no WCAG AA.

**Aceitação:** o exemplo da Fase 0 navegável só com teclado e lido por Orca (Linux).

### Fase 4 — Alvo WASM de verdade

- `fetch`/SSE/WebSocket no WASM via `web-sys` (hoje `net_web.rs` só devolve erro).
- Lógica no navegador sem Luau: avaliar o MicroPython (já no repo) ou um interpretador
  compilável para `wasm32-unknown-unknown`. **Decidir uma, não as duas.**
- Fonte padrão com Bold/Italic embutidos, para o texto não sumir em silêncio.
- Validar WebGL/WebGPU numa máquina com GPU funcional e só então tornar `web-gpu` o
  padrão.

**Aceitação:** o exemplo da Fase 0 rodando no Chrome com `fetch` real.

### Fase 5 — CSS que falta

Ordem pelo que mais aparece em tela real:

1. `opacity`
2. `transition` (propriedades já animáveis: cor, tamanho, `opacity`)
3. `calc()`
4. `@font-face` / fonte por arquivo no `.gss`
5. `:nth-child`, `:first/last-child`
6. `@keyframes` simples

Regra do projeto: propriedade desconhecida é **ignorada com aviso** — manter. Cada
propriedade nova entra com o aviso de valor inválido e com teste na árvore avaliada.

**Aceitação:** um botão com `transition: background 150ms` visível e medido (frames).

### Fase 6 — Rede completa

- Cookies e jar por host, só para o `fetch` (a Fase 1 foi descartada).
- `multipart/form-data` e upload de arquivo.
- Download em streaming com progresso (`on_progress`).
- Cancelamento (`abort`) de `fetch`.

**Aceitação:** upload de um arquivo de 50 MB com barra de progresso, sem travar a UI.

### Fase 7 — Mídia

- `<audio>` primeiro (barato: `rodio`/`symphonia`), com `play/pause/seek` e eventos.
- `<video>`: decidir entre decodificação própria (`ffmpeg`, pesado e com licença) e
  manter a `webview` como saída oficial. **Recomendação:** documentar a `webview`
  como a resposta suportada e só investir em `<video>` próprio se a Fase 0 mostrar que
  vale a pena.

**Aceitação:** `<audio>` tocando um MP3 remoto com barra de posição.

### Fase 8 — i18n

- `{t("chave")}` com catálogo JSON por locale e troca ao vivo.
- Plural e formato de número/data por locale.
- RTL/bidi: depende do `cosmic-text`; avaliar e documentar o limite.

**Aceitação:** o exemplo da Fase 0 trocando pt-BR ↔ en sem reiniciar.

---

## 3. Dependências entre fases

```
Fase 0 (medir)
   ├── Fase 6 (rede do fetch: cookies, upload)
   ├── Fase 2 (texto) ── Fase 3 (acessibilidade)
   ├── Fase 4 (WASM)
   ├── Fase 5 (CSS)
   ├── Fase 7 (mídia)
   └── Fase 8 (i18n)
```

A Fase 0 vem primeiro. Depois, 1 e 2 são as de maior retorno e podem andar em
paralelo; a 3 espera o desenho do texto da 2, porque os papéis moram nos mesmos nós.

## 4. Quanto cada fase move a nota

Estimativa, não medição. Vale revisar depois da Fase 0.

| Fase | Geral | App interno |
|---|---|---|
| 1 carga remota | descartada | descartada |
| 2 texto rico | +0,05 | +0,05 |
| 3 acessibilidade | +0,05 | +0,03 |
| 4 WASM | +0,04 | +0,02 |
| 5 CSS | +0,03 | +0,03 |
| 6 rede | +0,02 | +0,03 |
| 7 mídia | +0,02 | +0,01 |
| 8 i18n | +0,01 | +0,02 |

## 5. Fora do escopo (de propósito)

- **SEO e indexação.** Um app em `<canvas>` não tem DOM. Quem precisa disso usa web.
- **Carregar app, tela ou asset de uma URL** (Fase 1, descartada): só dados remotos.
- **Compatibilidade com HTML/CSS/JS existentes.** O Glacier não é um motor de
  renderização da web; o markup é o dele.
- **Extensões de navegador, DevTools completo, service workers.**
- **Ecossistema npm.** A resposta é `LuaExtension` e componentes em Rust.

## 6. Riscos

- **Acessibilidade depende do upstream.** Se o `iced` não expuser AccessKit, a Fase 3
  vira "contribuir para o iced", com prazo fora do controle do projeto.
- **Luau no WASM pode não ter caminho** (o `luau0-src` só gera wasm para emscripten).
  A Fase 4 pode terminar em "outra linguagem para o navegador", o que parte a
  promessa de um script só.
- **Superfície crescendo sem teste de ponta a ponta.** O histórico do repo (widget que
  passa nos testes e aparece vazio) vale para toda fase: o critério de aceitação é
  rodar o exemplo, não só passar o teste.

## 7. O que ficou decidido

Fase 1 (2026-10-30): **descartada.** Sem app, tela ou asset remoto; só dados via
`fetch`. A implementação foi removida (ver a seção da Fase 1).

Fase 2 (2026-10-30): estilo inline só dentro de `<prose>` (com `<p>`); `<a>` com hover;
`selectable` em `<text>`/`<prose>` (via `<textarea readonly>`, perdendo estilo e link);
`<markdown>` com tabelas e imagens **locais**. Falta o foco de teclado nos links
(ver `docs/TEXTO.md`).
