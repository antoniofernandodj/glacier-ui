# Plano: `@msg.ola` — ler um campo de uma tabela do `ctx` no markup

**Estado:** **implementado na glacier-ui 0.138.0** (todas as fases; as decisões da seção 4 valeram). Nasce de um app de uma tela
(`meu-app`) que faz `ctx.msg = { ola = "mundo" }` no `init` e escreve
`text(content = "@msg.ola")` no markup: o texto não aparece, sem erro e sem aviso.

> **Resultado medido** (build de depuração; cada medição recarrega a tela): 2000 linhas × 3
> leituras — plana `{i.nome}` 408 ms; aninhada `{i.endereco.cidade}` 439 ms (+8%, desce direto
> no JSON do item); global `{cfg.a.b.c}` 471 ms (+16%, memo do JSON + uma `String`). A leitura
> plana não muda: a chave exata é tentada primeiro.

## 1. Diagnóstico

O markup não consegue ler `msg.ola` porque **o contexto é um mapa plano e o motor só
resolve `var.campo` para a variável de um `for-each`**.

- `@msg.ola` vira o placeholder `{msg.ola}`; `process_tpl` (`src/eval.rs:1105`) faz
  `context.get("msg.ola")`.
- `EvalCtx::lookup` (`src/eval.rs:701`) — o **único** caminho de leitura do avaliador —
  procura a chave exata, primeiro nas camadas locais e depois em `self.base.get(key)`,
  que é o `ContextMap` plano.
- A única coisa que entende `var.campo` é `ItemVars::get` (`src/eval.rs:540`), e só para a
  variável de um `for-each` (`var = t` → `@t.tag`). Nem aí há caminho aninhado:
  `@t.endereco.cidade` procura um campo literalmente chamado `endereco.cidade`.
- `ctx.msg = { ola = "mundo" }` grava **uma** chave, `msg`, com o texto
  `{"ola":"mundo"}`. A chave `msg.ola` nunca existe, e uma chave ausente interpola como
  texto vazio — daí a tela em branco.

**Não é uma regressão do `ctx` tipado.** Na glacier-ui 0.133 a mesma tabela também virava a
string `{"ola":"mundo"}` na chave `msg` e `msg.ola` também não existia. O que mudou é a
expectativa: desde a 0.136 o script lê `ctx.msg.ola` como tabela (o `console.info(ctx.msg.ola)`
do `logar` funciona), e é natural esperar o mesmo caminho no markup. É uma **lacuna de
coerência** entre os dois lados.

O que já acusa o problema, sem rodar a tela: `tela:missing()` lista `msg.ola` (uma chave que
o markup lê e ninguém publicou), então o `assert_complete` do teste do app falha.

## 2. Contorno imediato (sem mudar o motor)

Publicar a chave plana que o markup lê: `ctx.msg_ola = msg.ola` (ou `ctx["msg.ola"] = …`) e
`@msg_ola`. Serve para um campo; não serve para listas de objetos aninhados.

## 3. A correção

### 3.1 Desenho

Resolver o caminho **dentro do `EvalCtx::lookup`**, como fallback — não antes:

1. Chave exata (camadas, depois base): comportamento de hoje, sem custo novo. Uma chave
   que contém ponto literalmente (`ctx["a.b"]`) continua ganhando.
2. Só se **não achou** e a chave tem `.`: procurar o **maior prefixo** que existe
   (`msg`), interpretar o valor dele como JSON e descer pelos segmentos restantes.
   - objeto: campo pelo nome; array: segmento numérico é o índice (`@lista.0.nome`);
   - folha string sai crua; número/booleano pelo texto canônico (`3`, `true`); objeto ou
     array sai como JSON (é o que `spread=` e `for-each` esperam); `null` ou caminho
     inexistente = ausente (e então vale o `{msg.x|padrão}`);
   - o mesmo para uma variável de `for-each` (`@t.endereco.cidade`).

Por que no `lookup`: ele é o único ponto de leitura, então `{…}`, `{k|padrão}`, `if`/`equals`,
atributos interpolados e o rastreamento de dependências ganham o caminho de uma vez. O cache
de avaliação confere dependências relendo pelo mesmo `lookup` (`deps_hold`, `src/eval.rs:758`),
então uma dependência `msg.ola` invalida quando o **valor** dela muda, e não quando outro campo
de `msg` muda.

### 3.2 Pontos delicados

- **Dono do `&str` devolvido.** Hoje `lookup` devolve `Option<&str>` emprestado do mapa. Um
  campo resolvido por caminho é calculado. Opções: devolver `Cow<str>` (só o caminho novo
  aloca; o exato continua emprestado) ou guardar o JSON já interpretado num memo que o
  `EvalCtx` empresta. A primeira é mais simples; medir.
- **Custo de interpretar o JSON.** Não interpretar `msg` a cada leitura: um memo
  `chave → (texto, Arc<Value>)`, comparando o texto (como o `EvalCache::array` de
  `src/eval.rs:491` já faz), vive no `EvalCache` e é emprestado ao `EvalCtx` na avaliação.
- **Leitura somente.** Os widgets de duas vias (`:value = msg.nome`) escrevem na chave; não
  existe "escrever num caminho". Documentar que a chave ligada a um campo é plana.
- **Coleções.** `for-each = msg.itens` e os `:items`/`:options` dos widgets leem a chave direto
  (`context.get`, `src/widget.rs`). Passar por um mesmo auxiliar `resolve(context, chave)`.
- **`tela:missing()` / `binding_keys`.** Uma chave com ponto que o caminho resolve deixa de
  ser "ausente". Se o prefixo nem existe, acusar o prefixo (`msg`), que é a causa.
- **Aviso, mesmo antes da correção.** Quando uma chave com ponto não existe mas o prefixo
  existe, avisar uma vez em `stderr` ("`@msg.ola` lê a chave 'msg.ola'; 'msg' existe mas…").
  Vale como rede de segurança permanente para um caminho que não resolve.

### 3.3 Fases

1. **Teste que reproduz.** No harness (`glacier test`): `ctx.msg = { ola = "mundo" }` +
   `assert_has_text(tela, "mundo")` (falha hoje) e `assert_complete` listando `msg.ola`.
   Casos: caminho aninhado; índice de array; folha número/booleano; objeto/array como folha;
   `null` e inexistente com `{msg.x|padrão}`; chave com ponto literal vence; `for-each` com
   `@t.endereco.cidade`; o cache invalida quando só o campo lido muda, e **não** quando muda
   outro campo.
2. **Motor.** O fallback no `EvalCtx::lookup` + o memo do JSON; manter o caminho exato intacto.
3. **Coleções e widgets.** O auxiliar compartilhado em `for-each`, `:items`, `:options`,
   `:columns` etc.
4. **Diagnóstico.** O ajuste do `missing()` e o aviso em `stderr`.
5. **Medição** (o mesmo harness da Fase 2 do `PLANO_CTX_TIPADO.md`): leitura **plana** sem
   regressão (o caminho exato não muda); lista de 2000 linhas lendo `@t.a.b` por célula; 10
   leituras do mesmo `@msg.x` na mesma avaliação (o memo tem de evitar 10 interpretações).
6. **Docs e release.** `AGENTS.md` do template ("O contexto é um mapa plano…", que hoje diz
   que o markup lê texto e não menciona caminho), CHANGELOG, e a glacier-ui `0.138.0` +
   glacier-cli. Depois subir o `meu-app` para a versão nova.

## 4. Decisões (tomadas)

1. **Índice de array no caminho** (`@lista.0.nome`): **entra agora.** Custa uma linha e fecha
   o caso `@itens.0`. O segmento numérico indexa um array (base 0); num objeto, é o nome do
   campo.
2. **Folha objeto/array:** **sai como o JSON em texto** (casa com `spread=` e com `for-each`).
3. **Escrever em caminho** (`ctx["msg.ola"] = x` atualizando `msg.ola` dentro da tabela): **fora
   do escopo.** Hoje isso grava uma chave plana de nome `msg.ola` — e, com a leitura por
   caminho, ela passa a **sombrear** o campo da tabela (a chave exata vence). Aceitável e
   previsível; fica escrito no `AGENTS.md`.

## 5. O que esta correção não é

Não é um `ctx` aninhado de verdade (estado reativo por campo). A tabela continua sendo uma
chave de contexto com o JSON dela; o caminho é só uma forma de **ler** um pedaço dela no
markup, sem o script precisar achatar à mão.
