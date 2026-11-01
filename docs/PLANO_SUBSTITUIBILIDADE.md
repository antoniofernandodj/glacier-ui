# Plano: Glacier como "micro navegador" (substituibilidade da web)

**Estado:** em andamento (2026-10-31), glacier-ui 0.146.1. **Fases 0, 1, 5 e 7 feitas**, com as
ressalvas de cada uma (`docs/MEDICOES.md`, `docs/TEXTO.md`, `docs/REDE.md`, `docs/I18N.md`);
**Fase 4 feita, menos `@keyframes` e `@font-face`** (ver `docs/GSS_MODERNO.md`);
**Fase 2 parcial** (árvore no AT-SPI, teclado nos botões e aviso de contraste; **falta** o teste
com o Orca, o foco no leitor e o teclado nos demais widgets — `docs/ACESSIBILIDADE.md`);
**Fase 6 parcial: só o `audio`** (`docs/AUDIO.md`), **o `video` está INCOMPLETO** (não foi feito);
**Fase 3 adiada** por decisão (WASM). Carga remota de app, tela ou asset foi **descartada**
(nada remoto além de dados; ver "Fora do escopo"). Proposta de 2026-10-29, glacier-ui 0.139.0. Nasce de uma auditoria do que o
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
| HTML | `.gv`/`.gv`, 60+ tags (`grid`, `flow`, `tableview`, `treeview`, `form`, `select`, `calendar`, gráficos…) | forte |
| CSS | `.gss`: classes, `#id`, tag, combinadores, pseudo-estados, `@media`, `var(--x)`, `gradient`, escopo por componente | bom |
| JS | Luau em `script`: corrotinas, `require`, timers, `storage` | bom |
| `fetch` | HTTP/HTTPS (hyper + rustls), gzip, timeout, headers | cobre API JSON |
| SSE / WebSocket | `sse()` / `websocket()` | presente |
| localStorage | `storage` | presente |
| Roteamento | `screen`, `navigate`, multi-janela | presente |
| Formulários | Reactive Forms | melhor que o nativo |
| Dev loop | hot-reload de `.gv`/`.gss`/JSON/Luau | presente |
| Empacotamento | `embed_assets!`, binário único | melhor que a web |

### 1.2 O que falta

**Bloqueadores** (decidem se dá para chamar de substituto):

1. **Texto.** Sem `a`, sem texto inline misto (negrito no meio da frase), sem
   Markdown; seleção e cópia de texto estático não apareceram.
2. **Acessibilidade.** Nenhuma menção a AccessKit, ARIA ou leitor de tela.
3. **Mídia.** Sem `audio` nem `video`. Antes havia a `webview` nativa como saída; foi removida (nada remoto além de dados).
4. **Alvo WASM incompleto.** Sem Luau e sem `fetch` no navegador (`WEB.md`); negrito e
   itálico somem sem fonte própria; GPU desligada por padrão; WebGPU/WebGL nunca
   vistos funcionando.

**Lacunas de plataforma:**

- Rede: sem cookies/sessão, multipart, download com progresso, cache, offline.
- CSS: sem `transition`, `@keyframes`, `calc()`, `@font-face`, `opacity`, `filter`,
  `position`/`z-index` genéricos, `::before`, `:nth-*`.
- i18n: sem tradução, plural, formato por locale, RTL/bidi.
- Descoberta: sem URL por tela, sem conteúdo indexável (na web o app é um `canvas`).

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

### Fase 1 — Texto rico, links e seleção

- `a(href = "…", on_click = "…")` com estados de foco/hover, abrindo URL externa ou
  navegando entre telas.
- Texto inline misto: `text` com filhos `b`, `i`, `code`, `a`. O `iced` tem
  `rich_text`/`span`; o trabalho é o parser e o `{interpolação}` dentro dos spans.
- `markdown` (via `iced::widget::markdown` ou `pulldown-cmark`).
- Seleção e cópia de texto estático (`selectable="true"`); avaliar o custo no upstream.

**Aceitação:** uma tela de "Sobre" com parágrafo, link, código inline e lista, tudo
selecionável; verificar a árvore avaliada dos spans (não só que renderiza).

**O que saiu, além do pedido:** o texto de leitura virou uma lista de **blocos** no
`prose` (parágrafo, `h1`–`h6`, citação com autoria, listas, **figura com legenda**,
código, régua), e o `prose(syntax, content)` lê o texto de uma chave do contexto — arquivo,
`fetch` ou banco — em `gv`, `html` ou `markdown`, tratando-o como **não-confiável**
(só links `http`/`https`/`mailto`, sem `on_click`/`navigate_to`, imagem só local). O foco de
teclado nos links e a seleção de texto estão feitos. Ver `docs/TEXTO.md`.

### Fase 2 — Acessibilidade

Decide o uso público e, em alguns mercados, a conformidade legal.

- Levantar o estado do AccessKit no `iced`/`winit` na versão usada; **a viabilidade
  depende do upstream**. Se não houver caminho, documentar e considerar contribuir.
- Papéis e rótulos no markup: `role`, `label`/`aria_label`, ordem de foco.
- Navegação por teclado completa (Tab, setas, Enter/Espaço) nos builtins.
- Contraste: aviso do `.gss` quando o par `color`/`background` falha no WCAG AA.

**Aceitação:** o exemplo da Fase 0 navegável só com teclado e lido por Orca (Linux).

**Estado (2026-10-31): parcial.** Feito: `role`/`aria_label`/`aria_hidden` no markup; a
árvore de acessibilidade da tela avaliada publicada no AT-SPI pelo `accesskit_unix` (feature
`a11y`, só Linux — o `iced`/`winit` não expõem o AccessKit, então a ponte não passa pelo
`winit`); `button` com Tab, anel de foco e Enter/Espaço; aviso de contraste WCAG AA no
`.gss` (primeiro achado real: `.env__del` no rustploy, 3,30:1). **Medido:** a árvore lida
pelo barramento AT-SPI (papéis e nomes certos) e `DoAction` num botão fechando o ciclo; o
teclado numa janela real. **Em aberto:** a aceitação ("lido por Orca") **não foi
verificada** — o Orca não foi usado; o foco do `iced` não chega ao leitor e os nós não têm
geometria; só o botão ganhou teclado (checkbox, radio, slider, select, abas, tabela…
não auditados); só Linux. Ver `docs/ACESSIBILIDADE.md`.

### Fase 3 — Alvo WASM de verdade

- `fetch`/SSE/WebSocket no WASM via `web-sys` (hoje `net_web.rs` só devolve erro).
- Lógica no navegador sem Luau: avaliar o MicroPython (já no repo) ou um interpretador
  compilável para `wasm32-unknown-unknown`. **Decidir uma, não as duas.**
- Fonte padrão com Bold/Italic embutidos, para o texto não sumir em silêncio.
- Validar WebGL/WebGPU numa máquina com GPU funcional e só então tornar `web-gpu` o
  padrão.

**Aceitação:** o exemplo da Fase 0 rodando no Chrome com `fetch` real.

### Fase 4 — CSS que falta

Ordem pelo que mais aparece em tela real:

1. `opacity`
2. `transition` (propriedades já animáveis: cor, tamanho, `opacity`)
3. `calc()`
4. `@font-face` / fonte por arquivo no `.gss`
5. `:nth-child`, `:first/last-child`
6. `@keyframes` simples

Regra do projeto: propriedade desconhecida é **ignorada com aviso** — manter. Cada
propriedade nova entra com o aviso de valor inválido e com teste na árvore avaliada.

**Aceitação:** um botão com `transition: background 150ms` visível e medido (frames). **Medida** numa janela real (Xvfb, ponteiro por XTEST): sem `transition` a cor salta de uma vez; com `transition: background 400ms ease-out` ela sobe de (88,101,242) a (224,122,95) em curva suave.

### Fase 5 — Rede completa

- Cookies e jar por host, só para o `fetch`.
- `multipart/form-data` e upload de arquivo.
- Progresso de envio e recebimento (`on_progress`) — feito; o *streaming* do corpo (entregar pedaços antes do fim) não.
- Cancelamento (`abort`) de `fetch`.

**Aceitação:** upload de um arquivo de 50 MB com barra de progresso, sem travar a UI. **Medida** com o exemplo `rede_progresso` sob Xvfb: a barra vai de 0 a 100% em ~3,4 s no envio e ~5 s no download, linear.

### Fase 6 — Mídia

- `audio` primeiro (barato: `rodio`/`symphonia`), com `play/pause/seek` e eventos.
- `video`: a `webview` nativa foi removida, então não há saída pronta. Decodificação
  própria (`ffmpeg`, pesado e com licença) só se a Fase 0 mostrar que vale a pena;
  senão, fora do escopo.

**Aceitação:** `audio` tocando um MP3 **local** com barra de posição.

**Estado (2026-10-31): parcial — só o `audio`.** Feito: `audio` (MP3, WAV, OGG/Vorbis,
FLAC; feature `audio`, `rodio`) com player (botão, barra de posição com *seek*, tempo),
`on_ended`/`on_error`, ações `audio:play|pause|toggle|stop|seek:NOME` e o global Luau `audio`.
**Medido** numa janela real com o dispositivo de áudio da máquina: a barra anda (0:03 / 0:08
após 3 s), o clique na barra faz seek, o fim volta o ícone a ▶ e dispara `on_ended`, tocar de
novo recomeça. **`video` INCOMPLETO: não foi feito** (pedido para ficar de fora desta
rodada; continua sem saída pronta, ver acima). Também em aberto: ler o `src` do `AssetSource`
(`embed_assets!`) em vez do disco; macOS/Windows não rodados. Ver `docs/AUDIO.md`.

### Fase 7 — i18n

- `{t("chave")}` com catálogo JSON por locale e troca ao vivo.
- Plural e formato de número/data por locale.
- RTL/bidi: depende do `cosmic-text`; avaliar e documentar o limite.

**Aceitação:** o exemplo da Fase 0 trocando pt-BR ↔ en sem reiniciar.

**Estado (2026-10-31): feita, com ressalvas.** `{t("chave", n=…)}`, `{num(…)}` e `{date(…)}` no
markup (`.gv`), catálogo JSON por locale com plural (CLDR para pt/en/ru/ar/fr/pl…),
locale ativa na chave `__locale` (a do sistema por padrão), `locales_dir`/`locale_json` no
daemon e o global Luau `i18n`. **Medida** com o exemplo `i18n` (Rust) e `i18n_luau`
em janela real: en → pt-BR → ru trocando ao vivo, com plural, número e data. **Ressalvas:**
conferida também no **painel da Fase 0** (`examples/painel`, com botões Português/English;
o `web/index.html` ganhou os mesmos; as colunas do Glacier e do Chrome de `docs/MEDICOES.md` foram **refeitas** com ele — sem diferença mensurável); **RTL**: o `cosmic-text` faz o
bidi *dentro* da linha (conferido em árabe), mas o **layout não espelha**; sem dígitos/
calendários locais, hora, moeda, gênero/ICU `select`, hot-reload do catálogo. Ver
`docs/I18N.md`.

---

## 3. Dependências entre fases

```
Fase 0 (medir)
   ├── Fase 5 (rede do fetch: cookies, upload)
   ├── Fase 1 (texto) ── Fase 2 (acessibilidade)
   ├── Fase 3 (WASM)
   ├── Fase 4 (CSS)
   ├── Fase 6 (mídia)
   └── Fase 7 (i18n)
```

A Fase 0 vem primeiro. Depois, a 1 (texto) é a de maior retorno e as 4 e 5 podem andar em
paralelo; a 2 espera o desenho do texto da 1, porque os papéis moram nos mesmos nós.

## 4. Quanto cada fase move a nota

Estimativa, não medição. Vale revisar depois da Fase 0.

| Fase | Geral | App interno |
|---|---|---|
| 1 texto rico | +0,05 | +0,05 |
| 2 acessibilidade | +0,05 | +0,03 |
| 3 WASM | +0,04 | +0,02 |
| 4 CSS | +0,03 | +0,03 |
| 5 rede | +0,02 | +0,03 |
| 6 mídia | +0,02 | +0,01 |
| 7 i18n | +0,01 | +0,02 |

## 5. Fora do escopo (de propósito)

- **SEO e indexação.** Um app em `canvas` não tem DOM. Quem precisa disso usa web.
- **Carregar app, tela ou asset de uma URL.** Foi implementado (`HttpAssets` com manifesto
  ed25519, cache e modo offline; commit `5fb9487`) e **removido por completo em 2026-10-30**:
  é execução remota de código, a superfície de ataque que o projeto decidiu não assumir. Só
  dados remotos, pelo `fetch`.
- **Compatibilidade com HTML/CSS/JS existentes.** O Glacier não é um motor de
  renderização da web; o markup é o dele.
- **Extensões de navegador, DevTools completo, service workers.**
- **Ecossistema npm.** A resposta é `LuaExtension` e componentes em Rust.

## 6. Riscos

- **Acessibilidade depende do upstream.** Se o `iced` não expuser AccessKit, a Fase 2
  vira "contribuir para o iced", com prazo fora do controle do projeto.
- **Luau no WASM pode não ter caminho** (o `luau0-src` só gera wasm para emscripten).
  A Fase 3 pode terminar em "outra linguagem para o navegador", o que parte a
  promessa de um script só.
- **Superfície crescendo sem teste de ponta a ponta.** O histórico do repo (widget que
  passa nos testes e aparece vazio) vale para toda fase: o critério de aceitação é
  rodar o exemplo, não só passar o teste.

## 7. O que ficou decidido

Fase 0 (2026-10-30): a primeira medição era de um painel que não carregava os dados nem a
lista (defeito do motor no `init` + erro do painel); refeita. Ver `docs/MEDICOES.md`.

Fase 1 (2026-10-30): estilo inline só dentro de `prose`, em blocos; `a` com hover e foco
de teclado (Tab/Enter/Espaço); `selectable` em `text`/`prose` (via `textarea(readonly)`,
perdendo estilo e link); `markdown` com tabelas, imagens **locais** e legenda;
`prose(syntax, content)` em `gv`/`html`/`markdown`, com o conteúdo dinâmico tratado
como não-confiável. Fica fora: lista aninhada e tabela dentro do `prose`.

Remoção do `webview_url` e da feature `webview` (2026-10-30): era uma tela remota por outro
caminho, e a Fase 6 já não conta com ela para `video`.

Fases 2 e 3 (2026-10-30): adiadas. A Fase 4 (CSS) e a 5 (rede do `fetch`) vêm antes.
Depois (2026-10-31): a Fase 2 foi retomada, parcialmente; a Fase 6 foi feita só no `audio`, com o
`video` **deixado de fora e marcado como incompleto**; a Fase 7 foi feita. A Fase 3 segue adiada.
