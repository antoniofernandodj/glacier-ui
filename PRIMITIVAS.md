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

`node.background`/`node.border_radius`/`node.border_width`/`node.border_color`
são campos **genéricos** do `UiNode` — resolvidos **uma vez por nó**, iguais
para todo `NodeType` (não são exclusivos de nenhum widget). Uma regra `.gss`
de **tag** (`ProgressBar { background: #ccc; border-radius: 3; }`) escreve
exatamente nesses campos genéricos — não existe um jeito de "escopar" essa
regra só para o `.style()` interno do widget, porque o resolvedor de
`.gss`/`eval.rs` não sabe (nem precisa saber) que tipo de nó está resolvendo.

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
> Hoje, `ProgressBar` é o único caso (`progress_bar` do iced é o único
> primitivo builtin com esse default). Um `Slider`/`vertical_slider` (P1 no
> `PLANO_WIDGETS.md`) também nasce `Length::Fill` no eixo principal — aplique
> a mesma exclusão quando ele for implementado.

`Button`/`TextInput`/`Select`/`Checkbox`/`Toggle` não precisam dessa exclusão:
o tamanho natural deles é `Shrink` (ou, no caso do `TextInput`, `Fill` mas já
tratado à parte — ver o comentário "iced's own default for `text_input` é
`Length::Fill`" em `render_node`, que só chama `.width(...)` quando
`node.width.is_some()`, nunca deixando o wrap genérico decidir por ele).

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
