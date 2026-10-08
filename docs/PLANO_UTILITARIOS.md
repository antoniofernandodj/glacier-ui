# Plano: GSS utilitário (um "Tailwind" para o `.gss`)

**Estado:** proposta. Nada aqui existe ainda. Este documento diz o que construir,
em que ordem, como verificar, e quais decisões precisam de resposta antes de começar.

---

## 0. A ideia em cinco linhas

- Hoje, para estilizar um widget você inventa uma classe (`.cartao`), escreve a regra
  no `.gss` (`.cartao { padding: 16; background: #2E3440; }`) e usa `class="cartao"`.
- Com **utilitários**, existe um vocabulário pronto de classes pequenas, uma por
  propriedade: `p-4`, `bg-surface`, `rounded-lg`, `w-fill`, `hover:bg-primary`,
  `md:hidden`. Você as combina no `class` sem escrever regra nenhuma.
- O motor **entende o nome** da classe na hora (não precisa de passo de build), com
  cache, e o que você define no `.gss` continua valendo e vencendo.
- No `.gss`, `@apply` junta utilitários sob um **nome de papel**:
  `.cartao { @apply p-4 bg-surface rounded-lg; }`. É assim que isto convive com a
  convenção do projeto (estilo no `.gss`, classe com nome de papel).
- Tudo o que o `.gss` já faz (pseudo-estados, `@media`, `var()`, escopo) é reaproveitado
  como está: o utilitário só **produz** `StyleRule`s, e quem as aplica é o
  `resolve_classes` de sempre.

---

## 1. O ponto de partida: o que o motor já tem, e o que falta

### 1.1. Já existe (e o plano reaproveita)

| Peça | Onde | Serve para |
|---|---|---|
| `StyleRule` com 21 propriedades | `src/stylesheet.rs` | o alvo de tudo: um utilitário vira uma `StyleRule` |
| Mesclagem esquerda → direita, "a última vence" | `resolve_classes` | `class="p-2 p-4"` dá `p-4`, de forma determinística |
| Especificidade tag < classe < id < inline (compostos logo acima do seu tier, 0.122) | `resolve_classes_in` | utilitário é só mais uma classe |
| Pseudo-estados `:hover`, `:focus`, `:active`, `:disabled`, `:invalid` | `StateStyles`, `resolve_state_classes` | os prefixos `hover:`, `focus:`… |
| `@media (min/max-width/height)` | `MediaQuery` | os prefixos `sm:`, `md:`, `lg:`… |
| `:root { --x }` e `var(--x, fallback)` | `StyleSheet::variables` | os tokens do tema (cores, escalas) |
| Cores `#RRGGBB` e `#RRGGBBAA` | `parse_hex_color` | o modificador de opacidade `bg-primary/50` |
| `class` interpolado a cada avaliação | `eval.rs` | `class="btn {estado}"` continua funcionando |
| Aviso (não erro) para propriedade desconhecida, com sugestão | `closest_property` | o mesmo trato para utilitário desconhecido |

### 1.2. Lacunas do motor (precisam de trabalho **antes** do catálogo)

O Tailwind assume coisas que o motor não tem. Cada lacuna abaixo é uma tarefa da
Fase 1, e algumas viram "não suportado" no catálogo.

| # | Lacuna | Por que atrapalha | Proposta |
|---|---|---|---|
| L1 | `padding` é **uma** string (`"10 20"`) | `px-4 py-2` precisaria combinar dois campos; hoje o segundo apagaria o primeiro | quatro campos por lado (`padding_top/right/bottom/left`), mesclados lado a lado; o `padding: "10 20"` do `.gss` continua valendo e preenche os quatro |
| L2 | `color` quer dizer coisas diferentes: cor do **texto** num `Text`, **fundo** num `Button`; o rótulo do botão é `text_color` | `text-primary` e `bg-primary` não podem depender do widget | `text-*` grava `text_color`; `bg-*` grava `background`; o `Text` passa a ler `text_color` (com `color` de reserva) e o `Button` a ler `background` (com `color` de reserva). O `color` antigo segue valendo |
| L3 | O eixo cruzado muda com o widget: numa `column` os filhos se alinham por `align_x`, numa `row` por `align_y`, num `container` o conteúdo usa os dois | `items-center` precisa de uma forma só | campo virtual `items`, resolvido no `eval` pelo tipo do nó (column → `align_x`, row → `align_y`, container → ambos) |
| L4 | Não há **sombra** nem **opacidade** | `shadow-*` e `opacity-*` são os utilitários mais pedidos depois de cor e espaço | campos `shadow` e `opacity` na `StyleRule`; o iced já tem `Shadow` em container e botão. Fase própria, depois do núcleo |
| L5 | Borda: largura, cor e raio são **únicos** (os quatro lados) | `border-b`, `border-t` não existem | **não suportar** `border-t/b/l/r`; só `border`, `border-N`, `border-<cor>`, `rounded-*`. Aviso claro, como o do `border-bottom:` que já existe |
| L6 | Não há `min-width`/`min-height` | `min-w-*` | adicionar `min_width`/`min_height` (o `max_*` já existe e embrulha num `container`); Fase 3 |
| L7 | Sem eixo principal (`justify-*`) | `justify-between` não tem equivalente no iced | **não suportar**. O idioma do motor é o `<space />` flexível; a documentação aponta para ele |
| L8 | Não há estado de **pai** (`group-hover:`) | o efeito depende do pai estar com o mouse em cima | **fora de escopo** da v1 |
| L9 | `spacing` é único (`gap-x`/`gap-y` não existem) | só `gap-N` | suportar `gap-N` e `space-x/y-N` como o mesmo campo; avisar quando o eixo não casa |

---

## 2. Decisões a tomar (com recomendação)

| # | Pergunta | Opções | Recomendação |
|---|---|---|---|
| D1 | **Convive com a convenção do projeto?** O `CLAUDE.md` diz que estilo mora no `.gss` e que a classe tem nome de papel; utilitário no markup é o oposto | (a) proibir no markup, só `@apply`; (b) permitir no markup sem restrição; (c) permitir, mas a convenção continua dizendo "papel no markup, utilitário no `.gss`" | **(c).** O vocabulário nasce igual para os dois lados; `@apply` é o caminho recomendado, e o utilitário solto no markup fica para protótipo e para cola de layout (`w-fill`, `gap-2`). O `CLAUDE.md` ganha uma regra explícita |
| D2 | **Nomes em inglês (Tailwind) ou em português?** | `p-4`/`bg-*`/`rounded` vs `pad-4`/`fundo-*`/`arredondado` | **Inglês do Tailwind.** Quem conhece, conhece; a documentação e os modelos de linguagem já falam esse vocabulário. O resto do markup (`on_click`, `padding`) já é inglês. Apelidos em português podem entrar depois, sem mudar a arquitetura |
| D3 | **Resolver em tempo de execução ou gerar um `.gss`?** | (A) o motor parseia o nome da classe quando não acha regra; (B) uma ferramenta varre o markup e gera um `.gss` com as regras usadas (o JIT do Tailwind) | **(A) como caminho principal**, porque o motor tem hot-reload, `class` interpolado e não tem passo de build. **(B) como ferramenta de apoio** (`glacier utilities --emit`), útil para inspecionar e para o build web |
| D4 | **Escala de espaço** | `p-4` = 16 px (como o Tailwind) ou outra base | **Base 4, configurável** por `--space: 4` em `:root`. Os valores do Tailwind (`1`, `2`, `3`…`96`) com o mesmo significado |
| D5 | **Conflito de classes** (`p-2 p-4`) | ordem do CSS (Tailwind) ou ordem do `class` | **Ordem do `class`, última vence.** É a regra que o motor já tem, e é mais previsível que a do CSS |
| D6 | **`dark:`** | implementar já, adiar, ou não ter | **Adiar.** O motor troca de estilo por ação (`style:fusion-dark`), e não há `prefers-color-scheme`. Reservar o prefixo e decidir na Fase 3 (ligar à chave `glacier_style`) |
| D7 | **Utilitário e regra de mesmo nome** | quem vence | **A regra do `.gss` vence**: o utilitário só é consultado quando nenhuma folha define a classe. Assim `.p-4 { padding: 99; }` no app é uma escolha deliberada |

---

## 3. A gramática

Uma classe utilitária é:

```
[variante:]* [!]utilitário[-valor][/alfa]
```

| parte | exemplos | significado |
|---|---|---|
| variante | `hover:`, `focus:`, `active:`, `disabled:`, `invalid:`, `sm:`, `md:`, `lg:`, `xl:`, `max-md:` | quando vale. Várias se empilham: `md:hover:bg-primary` |
| utilitário | `p`, `px`, `bg`, `text`, `w`, `rounded`, `gap`, `items` | a propriedade |
| valor | `4`, `fill`, `primary`, `lg`, `[240]`, `[#112233]` | da escala, do tema, ou **arbitrário** entre colchetes |
| alfa | `/50` | opacidade da cor (`bg-primary/50` → `#RRGGBB80`) |
| `!` | `!w-fill` | reservado; **não** é necessário (o inline já vence), não implementar na v1 |

Regras de leitura:

- Valor arbitrário usa `_` no lugar de espaço: `p-[10_20]` → `padding: 10 20`.
- Valor da escala vira número; valor do tema vira `var(--…)`, resolvido pelo caminho que o `.gss` já tem.
- `text-<x>` é ambíguo (tamanho ou cor), e o desempate é **o namespace do token**: se `<x>`
  está em `--text-*` é tamanho; se está em `--color-*` é cor; senão, é erro com sugestão.
- Token que o parser não entende → **aviso** (uma vez por token) com "você quis dizer…?",
  e a classe é ignorada. Nunca erro fatal, como a propriedade GSS desconhecida.

---

## 4. O catálogo da v1

Mapeamento de cada família para as propriedades que o `.gss` **já** tem (ou que a
Fase 1 cria).

### 4.1. Espaço e tamanho

| Utilitário | Propriedade | Observação |
|---|---|---|
| `p-4`, `px-4`, `py-2`, `pt-1`, `pr-1`, `pb-1`, `pl-1` | `padding_*` | depende da L1 |
| `gap-4`, `space-x-4`, `space-y-4` | `spacing` | os dois eixos são o mesmo campo (L9) |
| `w-fill`, `w-shrink`, `w-fill-2`, `w-64`, `w-[240]` | `width` | `w-1/2` **não**: sem pai conhecido não há fração; use `w-fill-1` + `w-fill-1` |
| `h-fill`, `h-64`, … | `height` | idem |
| `size-8` | `width` + `height` | |
| `max-w-96`, `max-h-96` | `max_width`, `max_height` | |
| `min-w-*`, `min-h-*` | `min_width`, `min_height` | Fase 3 (L6) |

### 4.2. Alinhamento

| Utilitário | Propriedade | Observação |
|---|---|---|
| `items-start`, `items-center`, `items-end` | `items` (virtual) | resolvido pelo tipo do nó (L3) |
| `place-x-center`, `place-y-center` | `align_x`, `align_y` | o conteúdo de um `container` |
| `text-left`, `text-center`, `text-right` | `text_align` | |

### 4.3. Cor

| Utilitário | Propriedade |
|---|---|
| `bg-primary`, `bg-[#112233]`, `bg-primary/50` | `background` |
| `text-primary`, `text-[#fff]` | `text_color` (L2) |
| `border-primary` | `border_color` |
| `bg-gradient-*` | **fora da v1** (`gradient` aceita só o que o motor aceita hoje; revisar na Fase 3) |

### 4.4. Tipografia

| Utilitário | Propriedade |
|---|---|
| `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`… `text-5xl`, `text-[13]` | `size` |
| `font-bold`, `font-normal` | `bold` |
| `font-sans`, `font-mono`, `font-[Inter]` | `font` (família registrada com `.font_named`) |

### 4.5. Borda e forma

| Utilitário | Propriedade |
|---|---|
| `border`, `border-2` | `border_width` |
| `rounded`, `rounded-md`, `rounded-lg`, `rounded-full`, `rounded-[12]` | `border_radius` (`full` = raio grande, limitado ao metade do menor lado) |

### 4.6. Efeitos e interação

| Utilitário | Propriedade | Fase |
|---|---|---|
| `cursor-pointer`, `cursor-text`, `cursor-not-allowed`… | `cursor` | 2 |
| `hidden` | `hidden` | 2 |
| `shadow`, `shadow-md`, `shadow-lg`, `shadow-none` | `shadow` (L4) | 4 |
| `opacity-50` | `opacity` (L4) | 4 |

### 4.7. O que **não** haverá (e a mensagem que o aviso dá)

`justify-*` (use `<space />`), `border-t/b/l/r`, `flex-*`, `grid-cols-*` (use o widget
`<grid>`), `absolute/relative/z-*` (use `<stack>` com `x`/`y`/`anchor`), `group-*`/`peer-*`,
`transition-*`/`animate-*` (as animações têm o sistema próprio, ver `ANIMACOES.md`),
`ring-*`, `blur-*`, `italic`, `underline`, `leading-*`, `tracking-*`.

---

## 5. Tema e tokens

Os tokens são variáveis `:root` do `.gss`, com um **namespace** por tipo. Assim o que
já existe (`:root { --fraco: … }`) continua valendo e o utilitário só lê:

```gss
:root {
  --space: 4;                       /* a base da escala: p-4 = 16 */
  --color-primary: #89B4FA;         /* bg-primary, text-primary, border-primary */
  --color-surface: #313244;
  --text-sm: 12;                    /* text-sm */
  --text-lg: 18;
  --radius-lg: 12;                  /* rounded-lg */
  --screen-md: 720;                 /* md: → @media (min-width: 720) */
}
```

Ordem de resolução de um token, do mais fraco ao mais forte:

1. **padrões do motor** (a escala do Tailwind, os breakpoints `640/768/1024/1280`);
2. **paleta do tema** (`theme.json`: `primary`, `success`, `warning`, `danger`, `text`,
   `background` viram `--color-*` automaticamente), e a do estilo embutido ativo
   (`style:fusion-dark`);
3. **`:root` do app**, que vence tudo.

Efeito prático: um app sem `:root` nenhum já tem `bg-primary` funcionando com a cor do
tema, e trocar de tema (`style:…`) troca os utilitários sozinho.

---

## 6. Arquitetura

### 6.1. Peças

```
src/utilities/
  mod.rs        a API pública: `resolve_utility(token, &Theme, viewport) -> Option<UtilityRule>`
  grammar.rs    tokenização: variantes, utilitário, valor, alfa, arbitrário
  catalog.rs    a TABELA (utilitário → função que escreve na StyleRule); é a fonte única
  theme.rs      a leitura dos tokens (padrões, tema, :root)
  color.rs      `#RRGGBB` + alfa, mistura
  explain.rs    texto legível de "o que este token faz" (CLI e editor)
```

A tabela do `catalog.rs` é **dados**: uma lista de `(nome, tipo de valor, escreve em)`.
O mesmo dado alimenta o parser, a documentação gerada, o `--explain` da CLI e o
autocompletar do editor, para os quatro nunca divergirem.

### 6.2. Onde entra no motor

Um único ponto: dentro de `resolve_classes` e `resolve_state_classes`, no laço
`for name in classes.split_whitespace()`:

```
para cada token:
    se alguma folha define a classe  → usa a regra da folha        (D7)
    senão se parece utilitário       → resolve_utility(token)      (novo)
    senão                            → ignora, como hoje
```

- **Variante de estado** (`hover:bg-primary`) escreve em `StateStyles.hover`, pelo
  mesmo caminho do `.classe:hover { }`.
- **Variante responsiva** (`md:p-4`) consulta o `viewport` que o `resolve_classes` já recebe.
- **Cache:** um mapa `token → UtilityRule` (por processo, `thread_local`), preenchido na
  primeira vez. O custo de um token conhecido é uma consulta de hash, igual ao de hoje
  para uma classe de folha. O *fast path* dos nós sem `class` não muda.
- **Hot-reload:** o cache guarda só o que é função **pura** do token e do tema. Mudou o
  tema (`:root`, `theme.json`, `style:`), o cache de tokens que dependem dele é limpo.

### 6.3. `@apply`

No `.gss`, uma declaração `@apply a b c;` dentro de uma regra expande os utilitários **na
hora de ler a folha**, para a mesma `StyleRule` (e estados, e `@media`):

```gss
.cartao       { @apply p-4 bg-surface rounded-lg gap-2; }
.cartao:hover { @apply bg-primary/20; }          /* já é `hover:`, escrito como pseudo-estado */
.botao        { @apply px-4 py-2 rounded-md bg-primary text-base font-bold; }
```

Duas consequências boas: o markup mantém `class="cartao"` (a convenção do projeto) e a
**validação acontece na leitura da folha**, com `arquivo:linha`, sem esperar a tela
renderizar.

### 6.4. Erros e avisos

- Utilitário desconhecido → aviso único por token: `utilitário desconhecido 'bg-prymary'
  (ignorado) — você quis dizer 'bg-primary'?`, com a lista curta de candidatos.
- Valor fora da escala (`p-37` sem `p-[37]`) → aviso com a escala disponível.
- Utilitário **não suportado** (`justify-between`, `border-b`) → aviso com a alternativa do
  §4.7.
- Em `class` **interpolado** (`class="bg-{cor}"`) o token só é conhecido na avaliação; o
  aviso sai nessa hora, uma vez, e o catálogo estático (CLI) não o enxerga. Isso é
  esperado e documentado.

---

## 7. As fases

Cada fase termina com algo que roda, e o critério de aceite diz como saber.

### Fase 0. Especificação (sem código)

- Decidir D1–D7 (§2) com você.
- Congelar a gramática (§3) e o catálogo da v1 (§4) num `docs/UTILITARIOS.md` de
  referência, como um dado (a tabela), não como prosa.
- Auditar a lista de lacunas (§1.2) contra o `widget.rs`: confirmar, widget por widget,
  quais leem `color`, `text_color`, `background`.

**Aceite:** documento revisado; cada linha do catálogo aponta para uma propriedade que
existe ou para uma lacuna numerada.

### Fase 1. Pré-requisitos do motor (as lacunas)

- **L1** padding por lado; **L2** `text_color` no `Text` e `background` no `Button`;
  **L3** campo virtual `items` resolvido por tipo de nó. L5, L7 e L8 viram só texto de
  aviso.
- Cada mudança **mantém o comportamento antigo**: todo `.gss` e todo markup existente
  produz a mesma árvore.

**Aceite:** os exemplos `.gva` e `.gvb` carregam sem diferença de aviso; a árvore avaliada
de uma amostra de telas é idêntica à de antes (comparação por script, como se fez na
migração do `.gvb`).

### Fase 2. O núcleo

- `src/utilities/` com `grammar`, `catalog` (espaço, tamanho, alinhamento, cor,
  tipografia, borda, `cursor`, `hidden`), `theme`, `color` e o cache.
- O gancho em `resolve_classes` (§6.2), valores arbitrários e `/alfa`.
- `glacier utilities --explain "<token>"` e `--list`: mostram a `StyleRule` resultante.

**Aceite:** uma tabela de testes `token → StyleRule` cobre **cada linha** do catálogo
(uma entrada por utilitário, mais os casos de erro); `--explain "hover:bg-primary/50"`
imprime o estado, o campo e a cor `#…80` esperados; um exemplo novo escrito só com
utilitários renderiza.

### Fase 3. Variantes e o resto do catálogo

- Estado: `hover:`, `focus:`, `active:`, `disabled:`, `invalid:`.
- Responsivo: `sm: md: lg: xl:` e `max-*:`, ligados a `--screen-*`.
- `min-w/min-h` (L6), `size-*`, e a decisão final do `dark:` (D6).

**Aceite:** variantes empilhadas (`md:hover:bg-primary`) resolvem na ordem certa; um teste
muda o viewport e o resultado muda; o `StateStyles` é idêntico ao de
`.classe:hover { }` escrito à mão.

### Fase 4. `@apply`, `@theme` e os efeitos

- `@apply` no `.gss` (§6.3), com erro `arquivo:linha` para utilitário inválido.
- Leitura dos tokens por namespace (`--color-*`, `--text-*`, `--radius-*`, `--screen-*`).
- `shadow`/`opacity` (L4), com o suporte no `widget.rs`.

**Aceite:** um `.gss` só de `@apply` reproduz uma tela existente; trocar `style:fusion-dark`
em tempo de execução troca as cores dos utilitários; a sombra aparece nos widgets que
têm `container`.

### Fase 5. Ferramentas

- **CLI:** `glacier utilities --list|--explain|--emit|--check`. O `--check` varre os
  `.gva`/`.gvb`/`.gss` e lista utilitários desconhecidos, sem abrir janela (serve de
  passo de CI).
- **VS Code:** autocompletar de utilitários dentro de `class="…"`/`class = …`, *hover*
  mostrando a `StyleRule`, amostra de cor, e diagnóstico de token desconhecido. Tudo lido
  do **mesmo catálogo** (exportado como JSON pela CLI), sem uma segunda lista para manter.
- **`gvb_check`:** a opção `--classes`, com o mesmo relatório.

**Aceite:** o autocompletar sugere `bg-primary` ao digitar `bg-pr`; o `--check` falha com
código de saída diferente de zero num arquivo com `bg-prymary`.

### Fase 6. Documentação, convenção e exemplos

- `docs/UTILITARIOS.md` (gerado do catálogo) e uma seção no `AGENTS.md` dos projetos.
- O `CLAUDE.md`: regra explícita sobre quando usar utilitário no markup e quando `@apply`
  (D1).
- Exemplos: `examples/gvb/utilitarios` (só utilitários) e `examples/gvb/utilitarios_apply`
  (`@apply`), mais um preset opcional no `glacier new`.
- Migrar **um** dos exemplos existentes para `@apply`, como prova de que o vocabulário
  cobre o que as telas reais precisam.

**Aceite:** um projeto novo gerado pela CLI usa um utilitário sem leitura de documentação
extra; os exemplos carregam sem aviso.

### Fase 7. Desempenho e robustez

- Medir (com o `perf.rs` e o método de `AGENTS.md`, "Como medir, em vez de adivinhar"):
  tempo de avaliação de uma tela grande com e sem utilitários.
- Fuzz do parser de token: nenhuma entrada pode dar *panic*, nem laço infinito, nem alocar
  sem limite (comprimento máximo do token, profundidade máxima de variantes).
- Paridade na versão web (WASM) e com o `AssetSource` embutido.

**Aceite:** a avaliação com utilitários fica dentro de ±5% da mesma tela com classes de
folha; o fuzz roda um número fixo de iterações sem falha.

---

## 8. Como verificar sem `cargo test` local

Neste ambiente o `cargo test` não é uma opção (cada exemplo/teste linka o motor inteiro).
A verificação se apoia em três coisas:

1. **A tabela de testes** é escrita em `tests/utilities.rs` e roda em CI/outra máquina; aqui
   se garante que ela **compila** (`cargo build --tests`).
2. **`glacier utilities --explain`** é o teste manual: imprime a regra que um token produz,
   então qualquer pessoa confere um utilitário em um comando.
3. **`gvb_check --classes` e `--load`** sobre os exemplos: carregam, validam, e acusam
   token desconhecido.

---

## 9. Riscos

| Risco | Gravidade | Mitigação |
|---|---|---|
| Duas formas de fazer a mesma coisa (`size="13"` inline, `.nota {}` no `.gss`, `text-sm`) criam estilo inconsistente | alta | D1(c): a convenção diz qual usar quando; o `--check` pode ter uma regra opcional `--no-inline-utilities` |
| L2 muda o que `color` quer dizer e quebra telas existentes | alta | o campo antigo continua valendo; a Fase 1 só **adiciona** leitura; aceite por comparação de árvore |
| O cache de tokens fica velho quando o tema muda | média | só entram no cache funções puras; a troca de tema limpa os tokens dependentes; teste de hot-reload |
| Utilitário que o motor não consegue honrar em todos os widgets (`rounded-lg` num `Text`) falha em silêncio | média | cada linha do catálogo declara os widgets que a honram, e o `--explain` avisa quando o nó não a suporta |
| Catálogo cresce sem controle ("só mais um utilitário") | média | o catálogo é uma tabela com dono; todo item novo exige a propriedade correspondente no `.gss` |
| Diferença de ordem em relação ao Tailwind (D5) surpreende quem vem de lá | baixa | está documentada em uma frase, no topo do `UTILITARIOS.md` |

---

## 10. Fora de escopo (e por quê)

- **Ser um clone do Tailwind.** O alvo é o *modelo mental* (utilitário + variante + tema +
  `@apply`), limitado ao que o motor desenha. Onde o iced não tem o conceito, o plano diz
  "não" em vez de simular.
- **Passo de build obrigatório.** Nenhuma ferramenta precisa rodar antes do app.
- **`group-*`/`peer-*`**, animações, `ring`, `blur`: precisam de capacidades do motor que
  não existem e têm retorno menor.
- **Um arquivo `tailwind.config`.** O tema é o `:root` do `.gss` e o `theme.json`, que já
  existem.

---

## 11. O que preciso de você antes de começar

1. **D1:** utilitário solto no markup é permitido (recomendação c) ou só `@apply`?
2. **D2:** nomes em inglês do Tailwind (recomendação), ou em português desde já?
3. **D3:** resolver em tempo de execução, com a geração de `.gss` só como ferramenta
   (recomendação)?
4. **Escopo da v1:** parar na Fase 4 (núcleo, variantes, `@apply`, tema), ou incluir as
   ferramentas (Fase 5) já na primeira entrega?
5. **L2:** posso mexer em como `Text` e `Button` leem cor (Fase 1), mantendo o
   comportamento antigo?
