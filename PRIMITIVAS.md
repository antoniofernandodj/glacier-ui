# Primitivas do motor (`src/widget.rs` + `src/parser.rs` + `src/eval.rs`)

Guia prático — e registro de uma armadilha real — para quem for acrescentar
uma **primitiva** nova ao catálogo (ver a tabela de níveis em
[`BUILTINS.md`](BUILTINS.md) e o backlog em [`PLANO_WIDGETS.md`](PLANO_WIDGETS.md)).
Diferente de um builtin (`impl Component` sobre primitivas existentes), uma
primitiva é um nó **nativo** do motor, mapeado 1:1 a um widget do `iced` — e
por isso mexe nos três arquivos centrais do pipeline de avaliação/render.

Motivado pelo `<ProgressBar>` (0.53.0): a primitiva funcionou, mas sumiu da
tela na primeira vez que ganhou uma regra de estilo builtin — não por bug no
widget em si, mas por uma interação sutil com um mecanismo **compartilhado**
de `render_node` que toda primitiva nova herda sem saber. A seção
["A armadilha do `Length::Fill`"](#a-armadilha-do-lengthfill) abaixo é o
porquê; o resto do documento é o passo a passo geral.

## As três paradas de uma primitiva nova

1. **`src/parser.rs`** — um variante em `NodeType` (os campos que o nó
   guarda), um braço no `match` de tags que lê os atributos XML (`Self::get_attr`/
   `get_attr_bool`/`get_attr_num`) e uma entrada em `NodeType::tag_name()`
   (o nome — sempre minúsculo — que um seletor `.gss` de **tag** usa para
   casar o nó, ex.: `ProgressBar {}` == `progressbar {}`).
2. **`src/eval.rs`** — um braço espelhado no `match` gigante de `eval_owned`
   (perto de `NodeType::Select`/`NodeType::Checkbox`): resolve `{template}`
   nos campos string via `process_tpl`, aplica `namespace_action` em ações, e
   — se o widget tiver uma cor/campo próprio (como o `color` do `Button`) —
   cai no fallback `.or_else(|| style.color.clone())` para herdar de uma
   classe `.gss`.
3. **`src/widget.rs`** — um braço no `match` de `render_node` que constrói o
   widget do `iced` de fato. É aqui que mora a armadilha abaixo.

Widgets **bindados a uma variável de contexto** (não um valor literal) seguem
a convenção de `TextInput`/`Checkbox`/`Select`: o atributo guarda o **nome da
chave** (`value="progresso"`), não o valor (`value="{progresso}"` — isso
bindaria contra o valor CRU de `progresso`, uma indireção a mais e quase
sempre um erro de quem está escrevendo o template).

## O mecanismo compartilhado: o wrap de background/borda

No fim de `render_node`, depois do `match` que constrói cada widget, existe um
bloco único que embrulha **qualquer** nó (exceto `Container`) num `container()`
extra, se o nó tiver `background`, `border_radius` ou `border_width > 0`:

```rust
// src/widget.rs, perto do fim de render_node
if node.kind != NodeType::Container {
    let bg_opt = background_for(node);
    let br_opt = node.border_radius;
    let bw_opt = node.border_width.unwrap_or(0.0);
    // ...
    if bg_opt.is_some() || br_opt.is_some() || bw_opt > 0.0 {
        let mut c = container(element);
        c = c.width(parse_length(&node.width)).height(parse_length(&node.height));
        // ... pinta background/borda no `.style()` deste container extra
        element = c.into();
    }
}
```

É o que dá a **qualquer** widget (`Checkbox`, `TextInput`, `Toggle`, …) a
capacidade de ganhar um fundo/borda via `.gss` sem cada `match` arm precisar
implementar isso por conta própria — um atalho genuinamente útil, usado o
tempo todo (é assim que uma classe `.card { background: #222; border-radius: 12; }`
funciona em qualquer coisa).

`node.background`/`node.border_radius`/`node.border_width`/`node.border_color()`
são campos **genéricos** do `UiNode` — resolvidos **uma vez por nó**, iguais
para todo `NodeType` (não são exclusivos de nenhum widget). Uma regra `.gss`
de **tag** (`ProgressBar { background: #ccc; border-radius: 3; }`) escreve
exatamente nesses campos genéricos — não existe um jeito de "escopar" essa
regra só para o `.style()` interno do widget, porque o resolvedor de
`.gss`/`eval.rs` não sabe (nem precisa saber) que tipo de nó está resolvendo.

## Campo direto ou acessor? (0.74)

Nem todo atributo genérico é um campo do `UiNode`. Os quentes — `width`,
`height`, `padding`, `background`, `class`, `id`, os numéricos — continuam
campos; os raros vivem em **grupos** alocados só quando usados
(`Look`, `Interact`, `Cond`, `Drag`, `FormBits`, `Pseudo`) e se leem por
**acessor**:

```rust
node.width.as_deref()      // campo:   width, height, padding, background, class, id, …
node.border_color()        // acessor: align_x, align_y, border_color, font, gradient,
                           //          text_align, text_color, slot_name          (Look)
node.on_press()            // acessor: on_press, on_double_click, cursor, tooltip,
                           //          tooltip_position                       (Interact)
node.set_tooltip(Some(s))  // escrita: sempre `set_<campo>`, aloca o grupo se preciso
```

A regra para escolher, ao acrescentar um atributo genérico novo: **se quase todo
nó de uma tela vai preenchê-lo, é campo; se quase nenhum, é grupo.** O motivo
está no CHANGELOG da 0.74 — cada `Option<String>` no corpo do nó custava 24
bytes em *todos* os nós da árvore, usados ou não.

## O que custa num quadro: caixas pintadas (0.87)

Numa GPU integrada antiga, **pintar** é o item mais caro de uma tela glacier — e
não tem relação com quantos nós ela tem.

Medido numa Intel HD 2500 (Ivy Bridge, 2012), no `examples/componentes_locais`:
**111 nós, 20 caixas pintadas**, janela de 900×720.

| | intervalo por quadro |
|---|---|
| Com pintura | 84–111 ms |
| Sem pintura (`GLACIER_NO_PAINT=1`) | 49–63 ms |
| Render do motor | **0,07 ms** |

As 20 caixas custam **~45 ms por quadro** — metade do total, e seiscentas vezes o
custo do motor inteiro. Uma tela de 300 nós **sem** fundo nenhum roda mais rápido
que uma de 111 nós pintada.

**Por que.** Toda caixa com fundo, borda ou canto arredondado vira um quad que o
`wgpu` sombreia **por pixel** — e canto arredondado é matemática de distância em
cada um deles. O custo é da **área**, não do número: uma caixa que cobre a janela
custa mais que vinte pequenas. E elas se somam por sobreposição: fundo da tela,
mais o do `groupbox`, mais o do `frame` dentro dele são três camadas no mesmo
pixel.

**O que fazer, em ordem de retorno:**

1. **Não pinte a janela inteira.** O tema já pinta o fundo; um
   `<column background="…" width="fill" height="fill">` por cima é uma camada
   redobrada em cada pixel. Prefira `theme` ou `<style>` no `<screen>`.
2. **Menos camadas sobrepostas.** `groupbox` dentro de `frame` dentro de
   container pintado são três passadas na mesma área. Escolha uma.
3. **Canto arredondado só onde se vê.** Numa área grande ele custa o dobro de um
   canto reto e não se nota; guarde-o para as caixas pequenas.
4. **Área importa mais que quantidade.** Vinte crachás pequenos são baratos; um
   painel pintado do tamanho da tela, não.

Isto não é limitação do motor — o mesmo custo aparece em `iced` puro com o mesmo
estilo. Numa GPU moderna, some. Vale medir antes de reescrever estilo: os dois
interruptores abaixo dizem em trinta segundos se é este o seu caso.

## Descobrir onde o quadro está indo (0.78)

Antes de otimizar, meça — e meça **dentro do app**, porque o motor é só uma
parte do quadro. `GLACIER_PERF=1` liga um relatório por segundo no `stderr`:

```sh
GLACIER_PERF=1 ./meu-app
```

```text
[glacier perf] 58 quadros em 1.00s = 58.0 fps  |  render 0.43ms méd, 0.71ms p95
               nós 1682  |  motor 2.5% do quadro  |  fora do motor 16.8ms/quadro
```

O motor cronometra a parte dele (percorrer a árvore e montar os `Element`); o
que vem depois — medir o layout, moldar o texto, desenhar na GPU — acontece
dentro do `iced` e do `wgpu`, e sai **por diferença**, do intervalo entre duas
chamadas de `view`.

A linha que decide é **fora do motor**. Se ela come quase todo o quadro,
otimizar o glacier não vai adiantar: a saída é entregar menos nós ao `iced`
(`virtualize`, logo abaixo), reduzir o tamanho da tela, ou aceitar o hardware.

Duas ressalvas para não ler errado: num app folgado boa parte do "fora do motor"
é espera pelo vsync, não trabalho — compare rolando e parado; e com mais de uma
janela aberta as medidas se somam num relatório só.

Desligada, a instrumentação custa a leitura de um `bool` já resolvido por quadro.

### Dois interruptores para isolar a causa (0.87)

O relatório sozinho diz *quanto*, não *por quê*. Estes dois dizem por quê:

| Variável | O que faz | Para que serve |
|---|---|---|
| `GLACIER_PERF_STRESS=1` | pede um quadro por vsync | mede **capacidade**, não demanda. Sem ela, um app orientado a evento fica ocioso entre eventos e o relatório mede a espera — erro que já se cometeu aqui mais de uma vez |
| `GLACIER_NO_PAINT=1` | pula todo fundo, borda e canto arredondado | separa "lento por nós" de "lento por área pintada". A tela fica feia de propósito |

O procedimento que resolve em dois minutos: rode com `STRESS` ligado, anote o
`intervalo méd`; rode de novo com `NO_PAINT` junto e compare. Se o intervalo cair
bastante, o gargalo é rasterização — e a saída é a seção acima, não otimizar o
motor.

## Virtualizar uma lista longa (0.77)

Um `<scrollable>` entrega ao `iced` **todos** os filhos, e o `iced` mede e
desenha todos — inclusive os que estão fora da tela. Numa lista de 300 cartões
isso é o custo dominante de cada quadro, e nenhuma otimização do motor alcança
(o trabalho é do layout do `iced`, não do render do glacier).

`virtualize` na coluna resolve: o motor monta só os filhos visíveis.

```xml
<scrollable height="fill">
  <column spacing="12" virtualize="300">   <!-- 300 = altura de CADA cartão -->
    <ForEach items="servicos" var="s">
      <container ...>…</container>
    </ForEach>
  </column>
</scrollable>
```

Três coisas para acertar:

1. **A altura é declarada, não medida.** Descobrir a altura real exige o layout,
   que é justamente o trabalho a evitar — a mesma troca do `uniformItemSizes` do
   `QListView`. Todo filho precisa ter a mesma altura, e o valor é a do
   **cartão**; o `spacing` da coluna o motor soma sozinho.
2. **A coluna precisa ser filha direta do `<scrollable>`.** É onde o motor
   procura, e é o que decide se ele pendura o aviso de rolagem.
3. **Errar a altura não quebra**, só desalinha a barra de rolagem: os vãos de
   cima e de baixo são calculados com o valor declarado.

Sem `<scrollable>` acima, com `virtualize="0"` ou com a lista já cabendo inteira
na tela, o nó renderiza normalmente — a virtualização degrada para o
comportamento de sempre, nunca para uma tela vazia.

**Quanto vale** (`tests/perf_arvore.rs`, cartão de ~300px):

| Cartões | Sem | Com |
|---|---|---|
| 40 | 182 µs | 48 µs |
| 80 | 364 µs | 47 µs |
| 300 | 1,81 ms | **47 µs** |

O custo do render vira **constante**: 300 itens custam o mesmo que 10, porque só
10 são montados. E o ganho maior é o que não aparece nessa tabela — o `iced`
deixa de medir e desenhar 290 cartões.

## A armadilha do `Length::Fill`

O `container(element).width(parse_length(&node.width))` do wrap acima usa
`parse_length(&None) == Length::Shrink` quando o nó não tem `width` — o que é
inofensivo para um widget cujo tamanho **natural** já é `Shrink` (um `Button`,
um `Select` sem `width` explícito: embrulhar um `Shrink` num `Shrink` não muda
nada). Mas o `progress_bar` do `iced` é **`Length::Fill` por padrão** — e um
`Container` `Shrink` ao redor de um filho `Fill` não sabe quanto espaço lhe
dar (o `Fill` precisa de um pai com largura determinada para "encher"), e
colapsa o filho a quase-zero.

Foi exatamente isso que aconteceu: os quatro estilos builtin (`src/style.rs`)
declaram `ProgressBar { background: …; border-radius: … }` como regra de tag —
campos genéricos, então o wrap acima entrava em ação sempre que o app não
desse um `width` explícito à barra. O resultado: a barra "sumia" (colapsada a
1-2px), sobrando visível só o que estivesse ao lado dela (no caso, um
`<Spinner>`) — o sintoma reportado foi "isso que foi colocado foi um spinner,
não um progress bar", quando na verdade os dois widgets estavam lá; um só
estava invisível.

**A correção** (`src/widget.rs`): excluir `NodeType::ProgressBar` da condição
do wrap, já que ele pinta o próprio trilho/borda no seu `.style()` (lendo os
MESMOS campos genéricos diretamente) — o wrap seria redundante de qualquer
forma, então tirá-lo do caminho não perde capacidade nenhuma, só evita o
colapso:

```rust
if node.kind != NodeType::Container && !matches!(&node.kind, NodeType::ProgressBar { .. }) {
    // ...
}
```

### A regra geral daqui pra frente

> **Toda primitiva nova cujo tamanho natural no `iced` seja `Length::Fill`
> (não `Shrink`) por padrão precisa ficar de fora do wrap genérico de
> background/borda** — do contrário, a primeira regra `.gss` de tag que
> declarar `background`/`border-radius`/`border-width` para ela (e é comum
> que um estilo builtin faça isso) a colapsa silenciosamente sempre que o
> template não fixar um `width`. Pinte o próprio background/borda dentro do
> `.style()` do widget (lendo `background_for(node)`/`node.border_radius`/
> `node.border_width`/`node.border_color` como o `ProgressBar` já faz) — é
> menos código no fim das contas, e evita a ambiguidade Shrink-ao-redor-de-Fill.
>
> São **três** casos hoje: `ProgressBar`, `Slider` (0.66) e `Reveal` (0.90).
> O terceiro entra por um motivo mais forte que o tamanho natural: a altura
> dele **é** a animação, e um `Container` por fora mediria o filho por conta
> própria, desfazendo o recorte. Fundo e borda de um `<reveal>` vão no filho,
> que é onde precisam ser recortados junto. O `DateTimeEdit` (0.68), o `Calendar` (0.84) e o
> `Pagination`/`Rating` (0.85) não entram na lista por outro motivo: eles não
> são widgets do `iced` embrulhados, são **composições montadas em Rust**
> (`row`/`column`/`button`), então já controlam o próprio tamanho — o wrap
> genérico nunca chega a decidir por eles. O `MaskedInput` (0.85) é a exceção
> dentro da exceção: ele **é** um `text_input` embrulhado, e por isso repete a
> ressalva dele — só chama `.width()` quando o nó declara uma. O `slider`/`vertical_slider` do iced também nasce
> `Length::Fill` no eixo principal, então ele entrou na mesma exclusão do wrap
> e pinta trilho/cursor no próprio `.style()`, lendo `background_for(node)` e
> a `color` — exatamente o que esta seção mandava fazer.

`Button`/`TextInput`/`Select`/`Checkbox`/`Toggle` não precisam dessa exclusão:
o tamanho natural deles é `Shrink` (ou, no caso do `TextInput`, `Fill` mas já
tratado à parte — ver o comentário "iced's own default for `text_input` é
`Length::Fill`" em `render_node`, que só chama `.width(...)` quando
`node.width.is_some()`, nunca deixando o wrap genérico decidir por ele).

## Nem toda primitiva embrulha um widget do `iced`

O passo 3 acima pergunta "qual o tamanho natural deste widget no `iced`?" e
pressupõe que exista um widget do `iced` por trás. Nem sempre existe.

O `DateTimeEdit` (`<dateedit>`/`<timeedit>`/`<datetimeedit>`, 0.68) é uma
primitiva que o `render_node` **compõe**: uma `row` de `button`s (as seções),
`text` (os separadores), uma `column` com as setas, tudo dentro de um
`container` que desenha a borda. Nada disso é um widget novo do `iced`.

O `Calendar` (`<calendar>`/`<monthyearpicker>`/`<daterangepicker>`, 0.84) é o
segundo, e mais extremo: uma `column` de `row`s de `button` — a grade 7×6 — mais
o cabeçalho de navegação, tudo montado num laço. Ele é a demonstração prática do
sinal abaixo: o `PLANO_WIDGETS.md` listou o `Grid` (`QGridLayout`) como
pré-requisito do calendário por três revisões, o que só faria sentido para um
**builtin** (cujo template é markup e precisaria de uma grade declarativa). Em
Rust, grade é um `for`, e o `Grid` deixou de estar no caminho crítico.

Vale saber quando esse caminho é o certo, porque ele parece "coisa de builtin":

- Se dá para compor **em markup**, com props, é **builtin** — é mais barato e o
  app pode sobrescrever.
- Se a composição precisa de dados que o **template não consegue derivar**, é
  primitiva. Foi o caso aqui: para desenhar `13` e `45` em seções separadas, um
  template precisaria ler partes de uma chave cujo *nome* vem de uma prop — a
  indireção `{{value}}` que o interpolador não tem. Em Rust, partir a string é
  uma linha.

O sinal prático: **um builtin que só funcionaria se o interpolador tivesse mais
uma capacidade é, quase sempre, uma primitiva mal classificada.**

### Os três sinais, depois de seis casos (0.85)

O sinal acima é o primeiro de três, e o projeto já usou os três. Vale a lista,
porque ela **prevê** — o `Grid` e o `Flow` da Onda 6 caem no segundo antes de
alguém escrever uma linha deles:

1. **O template precisaria ler uma chave cujo *nome* vem de uma prop** — a
   indireção `{{value}}`. Casos: `<timeedit>` (0.68), `<calendar>` (0.84),
   `<maskedinput>` (0.85).
2. **A repetição é dirigida por um *número*, não por uma coleção.** O `for-each`
   do motor lê uma chave com um array JSON. A janela `4 5 6` de uma paginação e
   as cinco estrelas de uma nota não existem em array nenhum: são derivadas de
   `pagina`/`total` e de `max`. A alternativa de builtin — o app calcular o
   array e passá-lo por `items=` — é exatamente o trabalho que o widget existe
   para poupar. Casos: `<pagination>` e `<rating>` (0.85); previstos:
   `PageIndicator`, `Grid` (`columns="3"`), `Flow`/`Wrap`.
3. **O widget precisa de um evento que o markup não expõe.** Há `on_press`,
   `on_double_click`, `cursor` e `tooltip` em qualquer nó — não há `on_enter`.
   Caso: o hover do `<rating>` (0.85), e o da faixa do `<daterangepicker>`
   (0.84).

O contrário também vale, e a Onda 4 tem o exemplo: `ListView`, `Accordion`,
`ToolBox` e `ButtonBox` são **builtins** porque nenhum dos três sinais aparece
neles — repetem sobre uma coleção de verdade, comparam com `equals`/`contains`
e reagem a clique.

## Uma primitiva que anima o **layout** (0.90)

O `<Reveal>` (`src/reveal.rs` — o corpo de um `<accordion>`/`<toolbox>`
abrindo e fechando) é o primeiro nó cujo **tamanho** muda entre um quadro e
outro sem que a view seja reconstruída. Isso muda duas coisas no passo 3:

1. **O `layout()` lê o estado da animação**, e o `update()` precisa chamar
   `shell.invalidate_layout()` a cada quadro da transição — sem isso o `iced`
   reusa a medida do primeiro quadro e a tela fica parada. O padrão completo
   (as cinco peças) está em [`ANIMACOES.md`](ANIMACOES.md).
2. **O filho transborda**, e tem que ser contido em três frentes: desenho
   (`renderer.with_layer`), ponteiro (cursor mascarado fora da parte visível) e
   overlay (`None` enquanto a transição corre). A tabela em `ANIMACOES.md`
   lista o sintoma de cada uma.

O sinal para reconhecer o caso, quando aparecer o próximo: **se o widget
precisa de um relógio, é primitiva** — não importa o quanto o markup pareça
suficiente. Um builtin não tem onde guardar progresso entre rebuilds da view
nem como pedir o quadro seguinte.

## Checklist para uma primitiva nova

1. `NodeType` em `parser.rs` + braço de parse + `tag_name()`.
2. Braço espelhado em `eval.rs` (`process_tpl`, `namespace_action`, fallback
   pra `style.*` se houver campo de cor/texto próprio).
3. Braço de render em `widget.rs`. Pergunte: **qual o tamanho natural deste
   widget no `iced` (`Length::Fill` ou `Shrink`)?** Se for `Fill`, exclua-o do
   wrap genérico de background/borda (ver acima) e pinte background/borda
   você mesmo no `.style()` do widget.
4. Se o widget aceitar uma cor/valor por classe `.gss`, teste com uma regra
   de **tag** (`MeuWidget { background: … }`) sem `width` no nó — é
   exatamente esse caso que expõe a armadilha acima; um teste só com `class`
   + `width` fixo (como os exemplos costumam escrever) não pega o bug.
5. Exemplo em `examples/` + linha no catálogo do `PLANO_WIDGETS.md` (status
   ✅) + linha na tabela de tags do `README.md`.
