# Plano: expressividade e coerência do `.gv` e do `.gss`

**Estado:** **implementado, não publicado** (2026-11-25; seria a 0.169.0). Base: glacier-ui 0.168.3. Ver "Estado da execução" abaixo.

## Decisões do dono (2026-11-25)

- **Duas formas por conceito, não uma.** A forma curta (um nó: `button(…, if = expr)`,
  `Componente(each = @lista as v, …)`) e a de bloco (`if expr { … } else { … }`,
  `each @lista as v { … }`) convivem, cada uma com seu papel, e as duas usam **a mesma
  gramática de expressão**. O que sai são as formas que só repetem outra com regras
  próprias (`equals`/`not_equals`/`one_of`/… como atributo, `template(if = …)`,
  `template(:for_each = …)`).
- **O `else` como atributo fica, com erro de posição:** o parser exige que o irmão
  imediatamente anterior tenha `if =` (ou seja um `else-if`), e senão a tela não
  carrega, com `arquivo:linha` e a dica. A forma curta continua; a falha calada acaba.
- Todas as demais recomendações deste plano foram aceitas.

## Estado da execução (2026-11-25) — implementado, não publicado (0.169.0)

Todas as fases foram implementadas no glacier-ui; testes em `tests/linguagem_*.rs`,
`tests/corpus_gv.rs` e nos módulos (`expr`, `ferramentas`, `stylesheet`, `widget`,
`luau`). A migração foi provada numa **cópia** das 76 telas do rustploy-gui: 1725 trocas
em 73 arquivos, zero formas depreciadas restantes, todos os arquivos parseando, e uma
segunda passada não muda nada (idempotente). O rustploy em si **não** foi migrado (é a
seção 14, depois da publicação).

**Desvios do plano, cada um com o motivo:**

1. **Bareword não foi restringido a `[A-Za-z0-9_.-]`** (6.1). O corpus usa de propósito
   `text = +`, cores e `open_project:@id` sem aspas; restringir quebraria formas
   legítimas e legíveis. A regra dos sigilos ficou documentada (`LINGUAGEM.md`), e a
   ambiguidade real (expressão × texto) é resolvida pela detecção de expressão.
2. **`:attr = @x` não virou erro** (6.1). É a ligação **dinâmica** (o nome da chave vem
   de outra chave), usada pelos builtins `wizard`, `tabs` e `command`
   (`:value = "{value}"`). Virar erro quebraria um recurso.
3. **A semântica de cor CSS é opt-in (`app(gss_color = css)`), não o padrão** (8.1).
   Trocar o padrão repintaria em silêncio todo botão existente — os estilos embutidos,
   82 `.gss` de exemplo e o rustploy. O padrão vira `css` numa versão `feat!` depois do
   rustploy migrado com `glacier fmt --migrate-color`. Os estilos embutidos já são
   convertidos sozinhos no modo `css`.
4. **`:estado` num ancestral e `+`/`~` não entraram** (8.2, passos 3–4). O estado vem do
   `status` do widget do iced no desenho, só do próprio widget; um contêiner não tem
   hover, e dar isso a ele exige um embrulho de rastreio por ancestral em toda lista. Os
   erros de parse agora dizem a alternativa (fazer do cartão um `button`; classe
   condicional para o vizinho). `[attr]` ficou fora, como previsto.
5. **`key =` no `each` é apelido do `reorder_key`** (5.1b): é onde o motor já usa
   identidade por item. Preservar estado por item ao inserir (fora do arrasto) exige
   trocar a coluna por uma `keyed_column` do iced — fica como próximo passo.
6. **Pintura (8.5)**: o fundo opaco igual ao do ancestral (sem borda, raio nem sombra) não
   é pintado — idêntico na tela, e restrito a `column`/`row`/`container`/`scrollable`
   (camadas de `stack`, `popover` & cia. não herdam). O "raio que não aparece numa caixa
   de tela inteira" **não** foi feito: na montagem o motor não sabe o tamanho da caixa.
   Nenhuma das duas foi medida numa GPU (sessão sem janela); o `check` avisa da regra
   que pinta a cor do tema.
7. **Argumentos de ação vindos de uma chave chegam como texto** (6.2); só os literais
   chegam tipados. O tipo da chave mora no `ctx` tipado do motor, que a avaliação do
   template não enxerga.
8. **`glacier fmt` não reindenta** (9.2): o parser descarta comentários, e reimprimir da
   árvore os perderia. O `fmt` faz as trocas pontuais (migração) e normaliza o espaço no
   fim das linhas e a quebra final.
9. **G4 estava em parte errado**: prop sem `default` já era obrigatória e prop não
   declarada já era erro. O que faltava — o tipo — entrou.
10. **`@x == "true"` → `@x`** não é migrado automaticamente: depende do tipo da prop, que
    o `fmt` não infere.
11. **Botão ocupado**: o motor faz a parte dele; apagar `busy.luau`, `busy_actions.luau`,
    o script Python e os `disabled = @busy_*` é trabalho do rustploy (seção 14).
12. **G13 (tag em comentário) e S4 (vírgula) já estavam resolvidos**; ficaram travados por
    teste. A análise original também errou ao dizer que não havia `opacity`.
13. **`matches =` não foi depreciado**: é a busca de texto por palavras (filtrar uma lista
    enquanto se digita), e a linguagem de expressão não tem equivalente. Continua sendo a
    forma, sem aviso.

Nasce de uma análise das duas linguagens feita sobre o maior app real do motor, o
`rustploy-gui` (~13 mil linhas de `.gv`, ~820 de `app.gss` mais os `style """…"""` dos
componentes). A conclusão foi: **a estrutura é muito legível, a expressão é fraca**. O que
não cabe na linguagem vira duplicação: nós repetidos no `.gv`, regras repetidas no `.gss` e
flags pré-calculadas no Luau só para o template poder compará-las por igualdade.

Este plano cobre **todos** os pontos levantados na análise, inclusive os que, ao abrir o
código, se mostraram já resolvidos (ficam como verificação e correção de documentação, ver a
seção 1). Cada ponto tem um número (`G1`…, `S1`…) e a seção 9 confere que nenhum ficou sem
fase.

---

## 0. Os pontos, com a evidência

### `.gv`

| # | Ponto | Evidência (rustploy-gui / glacier) |
|---|---|---|
| G1 | Condição só compara igualdade: `==`, `!=`, `in`, `is empty`, truthy. Não há `&&`, `||`, `!`, `<`, `>` | `src/gv.rs::Parser::condition`; `src/eval.rs::eval_condition`. No app: 199 `equals`, 131 `not_equals`, 1 `one_of` |
| G2 | Flags criadas no Luau só porque o template não sabe combinar condições | `has_warm`, `has_pending`, `filler`, `status_color`, `mark` em `service_card` / `service_export_window` |
| G3 | Tudo é string no template: `@badge == "true"`, `@count != "0"`, `default = "false"` | `nav_item.component.gv`, `gitsync.component.gv`, `service_card.component.gv`. O `ctx` já é tipado desde a 0.138 (`PLANO_CTX_TIPADO`), mas o template compara texto |
| G4 | Props sem tipo; errar `"true"` × `true` passa calado; prop obrigatória ausente vira vazio | `prop(name = x, default = "false")` |
| G5 | Não há classe nem atributo condicional, então o nó é duplicado para trocar uma classe ou um ícone | `nav_item` repete o `button` inteiro (com filhos) por `nav-item--active`; `app.gv` repete o botão de maximizar para trocar o SVG |
| G6 | Três formas de condicionar: bloco `if … { }`, atributo `if = @x, equals = …` e `else` como atributo do irmão | `service_card`: `column(if = @filler, equals = 1)` + `column(else, …)` |
| G7 | Três formas de iterar: `each @x as v (fallback = …)`, `template(:for_each = x, var = v)` e `:for_each` direto no componente | `advanced.component.gv`, `service_export_window`, `new_job_window`, `services_reorder_window` |
| G8 | Sigilos demais: `@x`, `"@x"`, `{expr}`, `:attr = chave`, e bareword que também é string | `class = u-muted`, `text("@name")`, `"{t('…')}"`, `:value = import_name` |
| G9 | A ação é uma string `handler:arg` com um argumento só, sem tipo | `on_click = copy_container_name:@container_name_full` |
| G10 | i18n verboso: `"{t('common.select')}"` em todo rótulo | ~todas as telas |
| G11 | `disabled = @busy_<handler>` repetido à mão ao lado de cada `on_click` | 341 `on_click`; `rustploy-gui/views/scripts/busy.luau` + `busy_actions.luau` + `tools/busy_actions.py --check` só para isso |
| G12 | Armadilhas de condição que falham calado: `empty` / `not_empty` só entendem array JSON (string conta como vazia); `one_of` separa por espaço, `contains` por vírgula **e** espaço | `docs/PRIMITIVAS.md`, "Quatro armadilhas que falham EM SILÊNCIO" |
| G13 | Tag literal dentro de comentário quebra o parser e o erro aponta a linha errada | **Já resolvido** no `.gv` (era do XML). Verificado em 2026-11-25 por `tests/linguagem_fase0.rs` |
| G14 | `if =` na **raiz** de um componente extraído é ignorado em silêncio: o componente aparece sempre | `AGENTS.md` do rustploy, "A condição vai na CHAMADA, nunca na raiz do arquivo extraído" |

### `.gss`

| # | Ponto | Evidência |
|---|---|---|
| S1 | `color` muda de sentido por widget: é o texto num `text` e o **fundo** num `button`, cujo texto é `text_color`; ao lado disso existe `background`. São três propriedades para dois conceitos | `src/stylesheet.rs::parse_rule_body`; `.btn { color: var(--border) }` |
| S2 | Estado × modificador multiplica regras: `.btn--primary:disabled`, `.btn--danger:disabled`, `.btn--blue:disabled` com o mesmo valor; `:hover` e `:focus` iguais; `transition` repetido em cada bloco | `rustploy-gui/views/styles/app.gss` (l. 198–218, 271–282) |
| S3 | Faltam aninhamento, `:not()`, `:is()` e `:estado` num ancestral; `+`, `~` e `[attr]` são erro de parse | `src/stylesheet.rs::parse_selector` |
| S4 | Vírgula "falha calada" | **Já resolvido** desde a 0.38 (`parse_gss_in`, l. ~1427), inclusive dentro de `@media` (o interior é reparseado). Falta o teste do caso `@media`. A documentação do rustploy está desatualizada, ver 1.1 |
| S5 | Tipografia mínima: `size`, `bold` e `font`; sem peso numérico, `line_height`, `letter_spacing`, itálico nem sombra | `parse_rule_body`. `opacity` **já existe** (`GSS_MODERNO.md`), a análise errou nisso |
| S6 | `bold: true` ignora a família da regra e cai na sans do sistema | **Bug**: `widget.rs::font_for` e `texto_de_faixa` montam `Font { weight: Bold, ..Default::default() }`, que zera a `family` |
| S7 | `font: mono` é a mono **do sistema** (`Font::MONOSPACE`), não a mono que o app embutiu | `widget.rs::font_for` |
| S8 | `column` / `row` sem `width` encolhe e zera o filho `fill`. O bloco some sem erro e mantém a altura reservada | `PRIMITIVAS.md`, "A armadilha do `Length::Fill`" e as quatro variações; caso real no rustploy em 2026-11-19 |
| S9 | O `glacier test` avalia a árvore mas não faz layout, então S8 passa no teste | `crates/glacier-cli/src/serve.rs::testar` |
| S10 | Custo de pintura vaza para o autor: fundo redundante igual ao do tema e raio em caixa de tela inteira | Comentários em `app.gss` (`.screen`, `.content`, `.panel`); `PRIMITIVAS.md`, "O que custa num quadro" |
| S11 | Nomes incoerentes: propriedade em `snake_case`, variável e `@media` com hífen. **Bug** junto: a lista de sugestão (`KNOWN_PROPERTIES`) está em hífen (`align-x`, `border-radius`…), então o "você quis dizer" sugere uma grafia que o parser **rejeita** | `src/stylesheet.rs::KNOWN_PROPERTIES` × o `match key` logo acima |
| S12 | A paleta existe duas vezes: `theme.json` e `:root` | `rustploy-gui/views/styles/theme.json` × `app.gss :root` (`#0A0E14` nos dois) |
| S13 | `hidden: true` e `display: none` são a mesma coisa com dois nomes, e misturam visibilidade com estilo | `parse_rule_body` |
| S14 | Propriedade desconhecida é aviso no `stderr`, não erro; classe usada no markup e nunca definida (ou o contrário) não avisa nada | `parse_rule_body` (ramo `other`); não existe checagem de classe |

---

## 1. Fase 0: verificar o que já existe e corrigir bugs pequenos

Não muda a linguagem. Corrige o que é bug e trava com teste o que já funciona.

### 1.1 Vírgula (S4)

- Teste novo em `src/stylesheet.rs`: `.a, .b { … }` **dentro** de `@media` aplica às duas
  partes (o caminho existe, porque o interior passa por `parse_gss_in`, mas não tem teste).
- Teste de vírgula com estado e combinador ao mesmo tempo (`.a:hover, .b > .c { }`).
- `CLAUDE.md` deste repo: dizer explicitamente que a vírgula é suportada.
- **Fora deste repo:** o `AGENTS.md` do rustploy diz que a vírgula "falha em silêncio" e
  ainda fala em `.gvb` e `<template if=>`. Corrigir lá (seção 8).

### 1.2 Sugestão de propriedade na grafia certa (S11, parte do bug)

- `KNOWN_PROPERTIES` passa a `snake_case` (`align_x`, `border_radius`, `text_color`,
  `max_width`, `font_family`, `stroke_width`…), gerado da **mesma** fonte que o `match` do
  `parse_rule_body`. Assim as duas listas não podem divergir de novo: uma tabela
  `const PROPRIEDADES: &[(&str, Setter)]`, ou ao menos um teste que percorre a lista e confirma
  que cada nome é aceito pelo parser.
- Propriedade escrita com hífen (`border-radius`) vira aviso específico: "o GSS usa
  `border_radius`" (hoje é o aviso genérico, com a sugestão errada).

### 1.3 Tag dentro de comentário (G13)

- Teste em `src/gv.rs`: `// <column>` numa linha, `/* <text> */` dentro de `style """…"""` e
  `<b>` num comentário entre atributos. Nos três casos a tela tem que parsear e um erro
  posterior tem que apontar a linha certa.
- Se algum caso quebrar, a correção é no emissor (`gv.rs::escape` / `Out::node`): comentário
  não é emitido no XML, ou é emitido escapado.
- Se nenhum quebrar, o ponto é só documentação (remover a armadilha do `AGENTS.md` do rustploy).

### 1.4 Corpus de medição

- `examples/gv_corpus_check` (ou um teste ignorado por padrão): roda o parser sobre um
  diretório de `.gv` e conta cada forma (`equals`, `else` como atributo, `:for_each`, ação
  `h:arg`, `"{t('…')}"`, `@x == "true"`…).
- Rodado sobre `rustploy-gui/views` antes e depois de cada fase, é a régua de "a migração
  terminou" e de "a forma velha pode sair".

**Verificação:** `cargo test` + os testes novos.

---

## 2. Fase 1: uma linguagem de expressão (G1, G2, G3, G12)

O núcleo do plano. Todas as fases do `.gv` dependem dela.

### 2.1 Gramática

```
expr    := ternary
ternary := or ( '?' expr ( ':' expr )? )?        // sem ':' → "" / falso
or      := and ( '||' and )*
and     := not ( '&&' not )*
not     := '!' not | cmp
cmp     := primary ( op primary )?
         | primary 'in' primary                   // item in lista
         | primary 'is' 'empty' | primary 'is' 'not' 'empty'
op      := '==' | '!=' | '<' | '<=' | '>' | '>='
primary := '@' caminho | string | número | 'true' | 'false' | 'null'
         | '[' expr ( ',' expr )* ']'             // lista literal
         | chamada | '(' expr ')'
chamada := ident '(' args ')'                     // t, num, date, len, lower, upper, trim
```

### 2.2 Semântica (com o `ctx` tipado)

- `@x` lê o valor **tipado** do `ctx`, não a string. `@badge` sozinho é truthy pelo tipo
  (bool, número ≠ 0, string não vazia, lista não vazia).
- `==` / `!=`: se os dois lados são números (ou strings que são número), compara número. Se um
  lado é bool, compara bool, e `"true"` / `"false"` vindos de string contam como bool. Isso
  mantém o markup antigo (`@badge == "true"`) funcionando enquanto o Luau passa a gravar bool.
- `<` `>` `<=` `>=`: só número. Se um lado não for número, o resultado é falso, com aviso único
  por nó (`arquivo:linha`).
- `is empty` vale para **string vazia, lista vazia, objeto vazio, null e chave ausente**
  (resolve G12a). `is not empty` é o contrário.
- `in`: a lista pode ser literal (`["int", "double"]`), um array do `ctx` ou, para manter o
  que existe, uma string separada por vírgula ou espaço (resolve G12b: um só separador
  aceito, os dois).

### 2.3 Onde a expressão vale

1. `if <expr> { }` / `else if <expr> { }`: a condição de bloco.
2. `{ … }` dentro de qualquer string: `text("{@cpu > 80 ? '⚠ ' : ''}@cpu")`.
3. Atributo cujo valor **começa** com `@`, `!`, `(` ou um nome de função conhecido, sem
   aspas: `disabled = @busy || !@valido`. Ver a Fase 5 para a regra exata.

### 2.4 Implementação

- Módulo novo `src/expr.rs`: lexer, parser recursivo e `Expr` (AST), mais `eval(&Expr,
  &EvalCtx) -> Valor`. Cada leitura passa por `context.get`, para a **dependência continuar
  registrada** e a reavaliação escopada continuar funcionando.
- Parse **uma vez** (no `gv.rs`, ou ao montar o `UiNode`), nunca por quadro. O `UiNode`
  guarda a `Expr` pronta.
- `NodeType::If` / `ElseIf` ganham `expr: Option<Expr>`. Os campos atuais (`equals`,
  `not_equals`, `one_of`, `contains`, `matches`, `empty`, `not_empty`) passam a ser
  **compilados** para uma `Expr` no parse, e o `eval_condition` vira só `expr::eval`. Uma
  semântica, um caminho.
- `process_tpl` passa a reconhecer `{expr}`. O que hoje é `{chave}`, `{chave|padrão}` e
  `{t(…)}` continua igual (são casos particulares da gramática; `|padrão` vira o operador de
  "valor ausente").
- Erro de sintaxe na expressão é **erro de parse** com linha e coluna (`Diagnostic`), não
  aviso.

### 2.5 Verificação

- Testes unitários de `expr.rs`: precedência, tipos, `is empty` em cada tipo, `in` com os três
  formatos de lista, curto-circuito.
- Teste de reavaliação: uma condição `@a && @b` reavalia quando muda `b`.
- Medição com `GLACIER_PERF` numa tela grande do rustploy, antes e depois. O alvo é custo de
  avaliação igual ou menor (a AST evita reinterpolar strings).
- No rustploy, as flags de G2 somem (`has_warm`, `has_pending`, `filler`, `mark`) e o
  `glacier test -vvv` continua verde.

---

## 3. Fase 2: atributo e classe condicionais (G5)

### 3.1 Classe condicional

```gv
button(
  class = nav-item,
  class:nav-item--active = @view in @target,
  on_click = @action,
) { … }
```

- Diretiva `class:<nome> = <expr>`: acrescenta `<nome>` à lista de classes quando a
  expressão é verdadeira. Pode repetir no mesmo nó.
- Também funciona por interpolação (`class = "nav-item {@ativo ? 'nav-item--active'}"`). A
  diretiva é a forma recomendada, porque o nome da classe fica legível para busca e para o
  `glacier check`.
- Na resolução de estilo (`eval.rs`, onde o `class` já é interpolado) a lista final é montada
  antes do `resolve_classes`, então o cache por string de classe continua valendo.

### 3.2 Qualquer atributo aceita expressão

```gv
svg(src = @glacier_window_maximized ? "assets/icons/win_restore.svg"
                                    : "assets/icons/win_maximize.svg")
```

- Isso acaba com a duplicação de nós da barra de título e do `nav_item` (os dois são casos de
  aceitação desta fase).

### 3.3 Verificação

- `nav_item.component.gv` e a barra de título de `app.gv` reescritos com um nó só. O teste
  existente `nav_item_aceso_recolore_os_filhos_por_combinador` continua passando.
- Teste novo: trocar `@view` troca a classe sem recriar o nó (o estado de hover e de foco se
  mantém).

---

## 4. Fase 3: props tipados (G4)

```gv
props {
  prop(name = id,          type = string, required = true)
  prop(name = has_pending, type = bool,   default = false)
  prop(name = cpu,         type = number)
  prop(name = rows,        type = list,   default = [])
  prop(name = on_open,     type = action)
  prop(name = empty_view,  type = component)
}
```

- Tipos: `string`, `number`, `bool`, `list`, `object`, `action`, `component`, `any` (o padrão,
  para não quebrar o que existe).
- **No parse:** um valor literal incompatível no uso do componente (`has_pending = "talvez"`
  num `bool`) é erro com posição. Um `required` ausente é erro.
- **Em tempo de execução:** um valor vindo do `ctx` com tipo errado gera um aviso único por
  (componente, prop) e é convertido pela regra da 2.2.
- `default` aceita literal tipado (`false`, `0`, `[]`), não só string.
- O valor do prop entra no escopo do componente **com o tipo**, e a expressão `@has_pending`
  sozinha funciona sem `== "true"`.

**Verificação:** testes de parse (literal errado, required ausente) e de runtime (aviso único).
No rustploy, `service_card` e `nav_item` passam a declarar tipos.

---

## 5. Fase 4: duas formas com papéis distintos, uma semântica (G6, G7, G14)

### 5.1 As formas que ficam

| Conceito | Curta (um nó) | Bloco (vários nós, ou com `else`) |
|---|---|---|
| Condição | `button("Salvar", if = @dirty && !@busy)` | `if expr { … } else if expr { … } else { … }` |
| Senão de um nó | `column(else, …)` / `column(else_if = expr, …)` logo depois do irmão com `if =` | o `else` do bloco |
| Laço | `PickerRow(each = @services as s, title = @s.name)` | `each @services as s (key = id, fallback = Vazio) { … }` |
| Reordenar | — | `each @lista as v (key = id, on_reorder = h) { … }` |

- **Mesma gramática nos dois lados:** o `if =` do nó e o `if` do bloco recebem a mesma
  `expr` da Fase 1; o `each =` do nó e o `each` do bloco, a mesma forma `@lista as v`.
- **`else` como atributo com erro de posição:** o irmão imediatamente anterior (ignorando
  comentários) precisa ter `if =` ou `else_if =`; senão é erro de parse com `arquivo:linha`,
  a linha do irmão e a dica ("mova o `else` para logo depois do `if`, ou use um bloco").
- **G14:** `if =` na raiz de um `component` passa a valer (é aplicado a cada instância,
  combinado com o `if =` da chamada por "e"). Se a implementação mostrar que isso não cabe
  sem custo, vira erro de parse com a dica de pôr a condição na chamada; calado, não fica.

### 5.1b As formas que saem (com aviso e migração)

- `equals`, `not_equals`/`ne`, `eq`, `one_of`/`equals_any`, `contains`/`has`, `matches`,
  `empty`, `not_empty` como atributos ao lado de `if =` → a expressão
  (`if = @kind in ["int", "double"]`).
- `template(if = …)` / `template(else)` → o bloco `if`.
- `template(:for_each = x, var = v)` → o bloco `each @x as v`.
- `:for_each = x, var = v` num nó → `each = @x as v`.
- `each` ganha `key =`, a identidade estável do item (hoje só o `reorder_key`), para o motor
  preservar estado por item ao reordenar ou inserir.

### 5.2 Mecanismo de depreciação

1. **Aviso** com `arquivo:linha` e a forma nova já escrita ("`column(else, …)` → use `if
   … { } else { }`; `glacier fmt --migrate` faz isso"). Um aviso por forma por arquivo, para
   não inundar o terminal.
2. **`glacier fmt --migrate`** (Fase 9) reescreve tudo automaticamente.
3. **Remoção** só quando o corpus (1.4) do rustploy-gui contar zero usos, numa versão marcada
   como `feat!` no CHANGELOG.

**Verificação:** testes de que cada forma velha produz exatamente a mesma árvore que a nova
(equivalência por `UiNode`), mais os testes de migração do `fmt`.

---

## 6. Fase 5: sigilos, strings e ações (G8, G9, G10)

### 6.1 Uma regra por sigilo, escrita no `CLAUDE.md`

| Escrita | Significa | Onde |
|---|---|---|
| `@x` | lê a chave `x` (tipada) | expressão; dentro de string, atalho de `{@x}` |
| `{expr}` | expressão dentro de string | qualquer string |
| `:attr = x` | **liga** o widget à chave `x`, nos dois sentidos | só widget de valor (`value`, `checked`, `options`…) |
| `nome` sem aspas | identificador: classe, nome de handler, valor de enum (`center`, `fill`) | atributos de nome |
| `"texto"` | texto literal | qualquer lugar |

- Bareword passa a aceitar só `[A-Za-z0-9_.-]`. Qualquer outra coisa sem aspas é erro, com a
  dica "ponha entre aspas". Hoje `size = "1280 820"` precisa de aspas e `class = u-muted` não,
  e a regra fica explícita em vez de implícita.
- `:attr = @x` (sigilo junto do bind) vira erro com dica, porque a ligação recebe o **nome** da
  chave, não o valor. É a "armadilha 3" do `PRIMITIVAS.md` virando erro.

### 6.2 Ação com argumentos

```gv
on_click = copy_container_name(@container_name_full)
on_click = export_toggle(@v.key, true)
on_click = window.resize("nw")
```

- Forma de chamada: nome do handler e argumentos que são **expressões** (Fase 1). O handler
  Luau recebe os argumentos **tipados e separados** (`function export_toggle(key, on)`).
- A forma `handler:arg` continua aceita e vira `handler("arg")` (um argumento string, como
  hoje). Aviso de depreciação com migração automática.
- Os handlers embutidos com prefixo (`window:resize:nw`, `field:nome`, `tray:open`) ganham a
  forma `window.resize("nw")` / `field.set("nome")`. Os prefixos atuais ficam como apelido.
- Prop `type = action` (Fase 3) aceita a forma de chamada.

### 6.3 i18n curto

```gv
text(t("common.select"))
text(t("service_card.name", container_name = @container_name))
button(t("common.open"), …)
```

- Com a Fase 1, `t(…)` é uma chamada de expressão e pode ser o valor do atributo **sem aspas
  nem chaves**. O `"{t('…')}"` continua valendo dentro de texto composto.
- O argumento nomeado passa a aceitar expressão (`container_name = @x`) em vez de só nome de
  chave.

**Verificação:** testes de parse para cada linha da tabela 6.1; teste de ação com dois
argumentos tipados chegando ao Luau (`src/luau/testing.rs`); o `fmt --migrate` converte
`h:arg` e `"{t('k')}"`.

---

## 7. Fase 6: botão ocupado automático (G11)

Hoje o rustploy embrulha as funções globais do Luau (`busy.luau`), mantém a lista à mão
(`busy_actions.luau`), confere a lista com um script Python e escreve `disabled =
@busy_<handler>` em 341 botões. O motor sabe quando um handler está rodando, porque o handler
que espera (`http`, `confirm`) é uma corrotina.

- O runtime grava `__busy.<handler>` (bool) no `ctx` enquanto a corrotina daquele handler
  estiver viva, inclusive quando a ação é a forma de chamada da Fase 5 (a chave é o nome do
  handler, sem os argumentos, como hoje).
- `app(busy_buttons = true)`: todo widget com `on_click` / `on_submit` fica desabilitado
  enquanto o próprio handler roda. O atributo `busy = false` num nó desliga para aquele nó, e
  `busy = @outra_chave` usa outra condição.
- **A verificar no começo da fase:** se todo handler que faz I/O passa de fato por corrotina
  (`src/luau/mod.rs`, prelúdio de `coroutine.yield`). Se algum caminho for síncrono, ele fica
  de fora (não há tempo "ocupado" visível) e a fase registra isso.

**Verificação:** teste em `luau/testing.rs` com um handler que espera `http` (servidor falso
de `tools/servidor_rede.py`): o botão fica desabilitado durante a espera e volta depois. No
rustploy, apagar `busy.luau`, `busy_actions.luau`, `tools/busy_actions.py` e os 341 `disabled
= @busy_*`.

---

## 8. Fases do `.gss`

### 8.1 Fase 7: semântica de cor (S1)

**Regra nova, igual ao CSS:** `background` é o fundo, em todo widget. `color` é a cor do
**conteúdo** (texto, ícone, traço do spinner, preenchimento do progressbar…), em todo widget.
`text_color` vira apelido de `color`.

- **Auditoria primeiro:** cada `get_attr(&node, &["color"])` em `src/parser.rs` (cerca de 15
  sítios: `button`, `progressbar`, `slider`, `badge`, `canvas`, `colorwheel`…) e cada leitura
  de `rule.color` em `src/widget.rs`. Tabela no próprio plano de execução: widget, o que
  `color` faz hoje, o que fará. Onde `color` é **valor** (o `color` de um seletor de cor) não é
  estilo e não muda.
- **Compatibilidade:** `app(gss_color = legacy)` mantém o comportamento atual (o `color` do
  botão como fundo) durante a transição. Sem a flag, vale a regra nova. Em modo `legacy`, uma
  regra com `color` que se aplica a um `button` sem `background` gera um aviso único por
  seletor, com a reescrita sugerida.
- `transition` e a checagem de contraste (`a11y::aviso_de_contraste`) simplificam: o par passa
  a ser sempre `color` × `background`, sem o caso especial do botão.
- `fmt --migrate` reescreve regras de botão (`color` → `background`, `text_color` → `color`).
  Como não dá para saber, só pelo `.gss`, se uma classe é aplicada a um botão, o `fmt` cruza
  com os `.gv` do mesmo projeto, e o que ficar ambíguo vira um comentário `/* TODO
  glacier-migrate */` na regra.

### 8.2 Fase 8: seletores que não obrigam a repetir (S2, S3)

1. **Aninhamento com `&`**, incluindo concatenação no estilo BEM:

   ```gss
   .btn {
     padding: 8 15;
     transition: all 150ms ease-out;
     &:disabled   { color: var(--faint); }
     &--primary   { background: var(--green); &:hover { background: var(--green-a); } }
     &--danger    { background: var(--danger); }
   }
   ```

   O `&--x` não é CSS padrão (é SCSS). Entra porque o BEM é a convenção deste motor
   (`CLAUDE.md`). A expansão acontece no parse, então é só açúcar: o resto do motor vê as
   regras planas de hoje.
2. **`:is(…)` e `:not(…)`** no último composto (`.btn:is(:hover, :focus)`,
   `.row:not(.row--header)`).
3. **Estado num ancestral** (`.card:hover > .card__title`). Exige saber o estado de hover e de
   foco de cada ancestral na resolução. Hoje o estado só existe no nó (`StateStyles`). Este
   passo tem medição própria (`GLACIER_PERF`, lista longa com hover) e só entra se o custo por
   quadro ficar dentro do ruído.
4. **`+` e `~` (irmão)**. A posição entre irmãos já existe para `:nth-child`
   (`Element::com_posicao`). Falta guardar o elemento anterior. Entra depois do passo 3, com o
   mesmo critério de custo.
5. **`[attr]`**. O modelo de estilo não enxerga atributos do nó. Fica **fora**, com motivo: a
   classe condicional da Fase 2 cobre o caso de uso (`class:x--on = @attr`) sem expor
   atributos ao seletor. O erro de parse passa a dizer exatamente isso.

### 8.3 Fase 9 (GSS): tipografia (S5, S6, S7)

- **S6, bug:** `bold` passa a ser "peso 700 **na família corrente**". `font_for` e todos os
  `Font { weight: Bold, ..Default::default() }` (`widget.rs` l. ~423, ~1123, ~3635, ~4401)
  passam a partir da `Font` já resolvida para o nó (família da regra, ou a padrão do `app`),
  mudando só o `weight`. Se a família não tiver a face em negrito carregada, o motor avisa uma
  vez (hoje troca de família calado).
- **S7:** `font: mono` passa a significar "a mono do app": `app(mono_font = "JetBrains Mono")`
  ou `font(src = …, role = mono)` no `resources`. Sem isso declarado, continua sendo a do
  sistema.
- **Propriedades novas:**
  - `font_weight: 100…900 | normal | bold` (`bold: true` vira apelido de `font_weight: 700`);
  - `font_style: italic | normal`;
  - `line_height: 1.4` (múltiplo) ou `line_height: 20` (px), via `LineHeight` do iced;
  - `box_shadow: <x> <y> <blur> <cor>`, via `iced::Shadow` no wrap de fundo e borda (o
    `container::Style` do iced 0.14 já tem o campo);
  - `letter_spacing`: o iced 0.14 **não** tem espaçamento entre letras no `text`. Fica
    registrado como não suportado: erro claro ("o iced 0.14 não suporta `letter_spacing`") em
    vez de propriedade desconhecida. Reabrir quando o iced suportar.
- A `opacity` já existe (`GSS_MODERNO.md`). A análise que deu origem a este plano errou nesse
  ponto, e não há trabalho aqui.

### 8.4 Fase 10: a armadilha do `Fill` (S8, S9)

1. **Corrigir na origem.** O `iced` resolve sozinho um `column` `Shrink` com filho `Fill`
   (`Column::push` faz `width.enclose(...)`). O colapso vem do **wrap de fundo e borda do
   glacier**, que faz `container(element).width(parse_length(&node.width))` com
   `parse_length(&None) == Shrink` (`PRIMITIVAS.md`, "A armadilha do `Length::Fill`").
   Correção proposta: sem `width` declarado, o wrap herda o `size_hint()` do elemento
   embrulhado (`Length::enclose`) em vez de forçar `Shrink`. O mesmo para a `height`.
   - Começar por um exemplo mínimo que reproduz o caso do rustploy (o formulário de backup de
     2026-11-19: `column` com só `field` dentro), em `examples/fill_shrink_check`.
   - Conferir as quatro variações documentadas no `PRIMITIVAS.md` (scrollable, prop de
     builtin, tamanho do irmão, abaixo da dobra) depois da correção. A que continuar existindo
     ganha o detector abaixo.
2. **Detectar o que sobrar.** `glacier test --layout` (e `glacier check`, Fase 11) roda o
   layout do iced sem janela, com o renderizador nulo que o `tools/transition_check` já usa,
   num tamanho de janela configurável (`--size 1280x820`; por padrão o `min_size` e o `size`
   do `app`). Todo nó com conteúdo e largura ou altura **zero** vira aviso com
   `arquivo:linha`, e falha com `--strict`.
3. A regra "container que agrupa controles leva `width = fill`" sai do `AGENTS.md` do rustploy
   quando o passo 1 estiver no ar.

### 8.5 Fase 11: custo de pintura no motor, não no autor (S10)

- **Fundo redundante:** na montagem, um nó cujo `background` resolvido é opaco e igual ao do
  ancestral pintado mais próximo (ou ao do tema, na raiz) não pinta. É o que o `app.gss` do
  rustploy faz à mão em `.screen` e `.content`.
- **Raio que não aparece:** o raio de uma caixa que ocupa a janela inteira não é visível
  (decorations do `app` desligadas ou não) e pode ser descartado.
- As duas otimizações ficam sob a medição de `MEDICOES.md` / `GLACIER_PERF`, antes e depois,
  na GPU integrada de referência. Se uma delas não medir ganho, sai do plano.
- O `glacier check` avisa quando uma regra pinta a mesma cor do tema numa caixa de tela
  inteira, para o autor saber sem ler o `PRIMITIVAS.md`.

### 8.6 Fase 12: nomes, tema e visibilidade (S11, S12, S13, S14)

- **S11, nomes.** Fica decidido e escrito: **propriedade em `snake_case`** (igual aos
  atributos do `.gv`, que é o motivo da regra), **variável com `--hífen`** (sintaxe de custom
  property do CSS) e **feature de `@media` com hífen** (sintaxe do CSS). A incoerência vira
  regra com motivo, e a 1.2 garante que o erro sempre sugere a grafia certa.
- **S12, uma paleta só.** Bloco `@theme { background: var(--bg); text: var(--text); primary:
  var(--accent); success: var(--ok); danger: var(--error); }` no `.gss` global, resolvido
  contra o `:root`. O `link(rel = theme, href = …json)` continua funcionando. Com os dois
  declarados, o `@theme` vence e o motor avisa.
- **S13, visibilidade.** `display: none` é a forma canônica (CSS). `hidden: true` vira apelido
  com aviso de depreciação e migração pelo `fmt`.
- **S14, nada calado.**
  - `app(gss_strict = true)` (e `glacier check --strict`): propriedade desconhecida vira
    **erro**. Fora do modo estrito continua aviso, porque um typo não deve derrubar a tela
    inteira em produção.
  - O `glacier check` cruza classes: usada em `.gv` e nunca definida em nenhum `.gss` ou
    `style`, e definida e nunca usada. As classes montadas por expressão (`class:x = …`) entram
    pelo nome literal. As montadas por interpolação livre (`"x--{@estado}"`) entram como
    padrão (`x--*`) e não geram falso positivo.

---

## 9. Ferramental (transversal)

### 9.1 `glacier check` (subcomando novo em `crates/glacier-cli`)

Lint de projeto inteiro, sem abrir janela: parse de todo `.gv` e `.gss`; formas depreciadas
(Fases 4 e 5); tipos de prop (Fase 3); classes (8.6); propriedade desconhecida (8.6); fundo
redundante (8.5); layout zero (8.4, com `--layout`). Saída `arquivo:linha: mensagem`, código de
saída diferente de zero com `--strict`. Entra no `make check` deste repo e no CI do rustploy.

### 9.2 `glacier fmt` e `glacier fmt --migrate`

- `fmt`: formatação canônica (dois espaços, uma declaração por linha no `.gss`, vírgula final
  nos atributos multilinha), a convenção do `CLAUDE.md` aplicada por ferramenta.
- `--migrate`: as reescritas de todas as fases (`else` como atributo, `equals` e similares,
  `:for_each`, `h:arg`, `"{t('k')}"`, `@x == "true"` → `@x` quando o prop é `bool`, `color`
  de botão, `hidden`). Cada reescrita tem teste de ida (forma velha → nova) e de equivalência
  (a árvore `UiNode` é a mesma).

### 9.3 Editor

- `editors/vscode-gv`: gramática TextMate para expressões, `class:x =`, ação com argumentos,
  `t(…)` sem aspas, aninhamento `&` no `.gss`, `@theme`, propriedades novas.
- Diagnósticos do `glacier check` na extensão, se ela já roda o CLI. Se não roda, fica
  registrado como próximo passo.

---

## 10. Documentação

- `CLAUDE.md`: a tabela de sigilos (6.1), a forma canônica (5.1), a regra de nomes (8.6) e a
  semântica de cor (8.1).
- `docs/PRIMITIVAS.md`: as "Quatro armadilhas" e as variações do `Fill` marcadas como
  resolvidas na versão em que cada uma for, com o que restou.
- `docs/GSS_MODERNO.md`: aninhamento, `:is` / `:not`, estado no ancestral, tipografia,
  `box_shadow`, `@theme`.
- Referência nova `docs/LINGUAGEM.md`: gramática do `.gv` (com a de expressão) e do `.gss`
  num lugar só. Hoje ela está espalhada entre `CLAUDE.md`, `PRIMITIVAS.md`, `BUILTINS.md` e
  comentários do `gv.rs`.
- `CHANGELOG.md` por fase. As remoções de forma velha entram como `feat!`.
- Este plano vai para `docs/arquivados/` quando a última fase fechar.

---

## 11. Ordem, versões e dependências

| Ordem | Fase | Depende de | Quebra compatibilidade? |
|---|---|---|---|
| 1 | 0: verificar e corrigir (1.1–1.4) | — | Não |
| 2 | 1: expressões | 0 | Não (as formas velhas compilam para `Expr`) |
| 3 | 2: classe e atributo condicionais | 1 | Não |
| 4 | 9 (GSS): tipografia, bug do `bold` | — | Pode mudar a fonte de texto em negrito que hoje caía na sans; aviso no CHANGELOG |
| 5 | 10: `Fill` | — | Pode **aparecer** conteúdo que hoje some; conferir telas |
| 6 | 3: props tipados | 1 | Não (`type = any` por padrão) |
| 7 | 5: sigilos, ações, i18n | 1 | Não (formas velhas como apelido) |
| 8 | 6: botão ocupado | 5 | Não (opt-in por `app(busy_buttons)`) |
| 9 | 8 (GSS): aninhamento e seletores | — | Não |
| 10 | 7 (GSS): semântica de cor | 9.2 | **Sim**, atrás de `gss_color = legacy` |
| 11 | 11 (GSS): pintura | — | Não |
| 12 | 12 (GSS): nomes, tema, visibilidade, modo estrito | 9.1 | Não (modo estrito é opt-in) |
| 13 | 4: duas formas, uma semântica (avisos, `else` com erro) | 1, 2, 5, 9.2 | Não (só aviso) |
| 14 | Remoção das formas velhas e da `legacy` | rustploy migrado (corpus = 0) | **Sim** (`feat!`) |

- Uma versão minor por fase, publicada no crates.io (o rustploy-gui nunca usa `path` nem
  `[patch]`).
- As ferramentas da seção 9 nascem junto com a primeira fase que precisa delas: o `check` na
  Fase 0 (só parse), crescendo a cada fase, e o `fmt --migrate` na primeira fase com forma
  depreciada.

## 12. Riscos

- **Custo de avaliação por quadro:** a expressão tem que ser parseada uma vez e avaliada
  lendo o `ctx` sem cópia (`PLANO_CTX_TIPADO`, 0.168.1). A medição está prevista nas Fases 1 e
  8.
- **Rastreamento de dependência:** uma expressão que lê `@a` dentro de um ramo não avaliado
  (curto-circuito) não registra `a`. Isso é correto (quando o ramo mudar, a outra leitura
  dispara a reavaliação), mas precisa de teste.
- **Semântica de cor:** é a mudança mais invasiva. Por isso fica atrás de flag, tem auditoria
  de widgets e é migrada por ferramenta.
- **Duas sintaxes convivendo** durante a transição: o aviso e o `fmt --migrate` existem
  justamente para encurtar esse período. O corpus (1.4) diz quando acabou.
- **Estado em ancestral e `+` / `~`** podem não caber no orçamento de quadro. Os dois têm
  critério de saída explícito (8.2).

## 13. Verificação geral (antes de dizer que funciona)

Segue a regra do `CLAUDE.md`: um teste que escreve uma chave e lê a mesma chave de volta não
prova que a tela desenha.

- `cargo test` e os exemplos de cada fase rodando **numa janela**, com captura de tela quando
  a fase mexe em pintura ou layout (Fases 2, 7, 9, 10, 11).
- `glacier test -vvv` no `rustploy-gui` a cada versão publicada, mais o `glacier check
  --strict` depois da Fase 12.
- Ao fim de cada fase, o corpus do rustploy-gui mostra a contagem das formas velhas caindo.

## 14. Fora deste repositório (rustploy)

Entram no rustploy, na ordem em que as versões do glacier forem publicadas:

- Corrigir o `AGENTS.md`: a vírgula funciona (S4), os arquivos são `.gv` e não `.gvb`, a
  condição não é mais `<template if=>`, e a armadilha da tag em comentário sai ou fica
  conforme a 1.3.
- Migrar o `rustploy-gui` fase a fase (`glacier fmt --migrate`), apagar as flags de G2 e o
  mecanismo de `busy` de G11, unificar a paleta (S12) e ligar `gss_strict` e `busy_buttons`.
- É o teste de aceitação do plano inteiro: o corpus chegar a zero formas velhas.

---

## Apêndice: conferência ponto a ponto

| Ponto | Fase |
|---|---|
| G1 só igualdade | 1 |
| G2 flags no Luau | 1, 2 (aceitação no rustploy) |
| G3 tudo é string | 1 (tipos na comparação), 3 |
| G4 props sem tipo | 3 |
| G5 sem classe ou atributo condicional | 2 |
| G6 três formas de `if` | 4 (duas formas, uma semântica; `else` com erro de posição) |
| G7 três formas de laço | 4 |
| G14 `if` na raiz do componente ignorado | 4 |
| G8 sigilos e bareword | 5 (6.1) |
| G9 ação com um argumento | 5 (6.2) |
| G10 i18n verboso | 5 (6.3) |
| G11 `busy` manual | 6 |
| G12 `empty` e `one_of` calados | 1 (2.2) |
| G13 tag em comentário | 0 (1.3): já resolvido, travado por teste |
| S1 `color` ambíguo | 7 (8.1) |
| S2 regras repetidas | 8 (8.2) |
| S3 seletores que faltam | 8 (8.2) |
| S4 vírgula | 0 (1.1): já resolvido, falta teste e doc |
| S5 tipografia mínima | 9 (8.3); `opacity` já existe |
| S6 `bold` perde a família | 9 (8.3) |
| S7 `font: mono` do sistema | 9 (8.3) |
| S8 `Fill` some | 10 (8.4) |
| S9 teste não faz layout | 10 (8.4), 9.1 |
| S10 custo de pintura | 11 (8.5) |
| S11 snake × hífen, sugestão errada | 0 (1.2), 12 (8.6) |
| S12 paleta duplicada | 12 (8.6) |
| S13 `hidden` × `display` | 12 (8.6) |
| S14 desconhecido não é erro, classe órfã | 12 (8.6), 9.1 |
