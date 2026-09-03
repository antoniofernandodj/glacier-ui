# Animações de widget no glacier-ui

Como o `<Toggle>` ganhou a bolinha deslizante (`src/animated_toggler.rs`,
0.52.0) — e o padrão a reutilizar para animar qualquer outro widget.

Três widgets seguem este padrão hoje: o `AnimatedToggler` (transição pontual),
o `Spinner` (rotação sem fim, `src/spinner.rs`) e o `Reveal` (o corpo de um
accordion abrindo e fechando, `src/reveal.rs`) — este último acrescenta uma
**quinta peça**, a única que muda o layout em vez do desenho. Ver as duas
seções finais.

## O problema

O iced redesenha **sob demanda**: nada acontece entre um evento e outro, e a
view é reconstruída do zero a cada `update` do app. Isso derruba as duas
abordagens ingênuas de animação:

- guardar o progresso num campo do widget não funciona — o widget morre e
  renasce a cada rebuild da view;
- um `loop`/timer que "vai redesenhando" não existe — ninguém redesenha se
  ninguém pedir.

O `toggler` de fábrica do iced 0.14 convive com isso simplesmente **não
animando**: desenha o knob em uma de duas posições fixas conforme
`is_toggled`. O 0.14 trouxe a API `iced::animation::Animation` (o crate
`lilt` por baixo), mas nenhum widget do próprio iced a usa ainda.

## As quatro peças do padrão

### 1. Estado da animação na árvore de widgets (`tree::State`)

O que precisa sobreviver entre rebuilds da view vai no estado que o iced
mantém **por posição na árvore** — é o mesmo mecanismo que preserva o texto de
um `text_input` enquanto a view é reconstruída em volta dele.

```rust
struct State {
    animation: Animation<bool>,   // progresso 0 ⇄ 1 com easing
    target: bool,                 // o estado que a animação persegue
    now: Instant,                 // relógio do último frame (ver peça 3)
    last_status: Option<toggler::Status>,
}
```

`Animation<T>` funciona por transição: `go_mut(novo_estado, agora)` inicia, e
`interpolate(a, b, instante)` projeta o valor interpolado naquele instante
(com easing e duração configurados na criação — o toggler usa `.quick()` =
200ms + `Easing::EaseOutCubic`). Nasce **assentada** no estado inicial
(`Animation::new(estado)`), então a primeira aparição não anima.

### 2. Transição detectada no `diff()`

O `diff` é o ponto de **reconciliação**: quando a view é reconstruída, o iced
apresenta o widget novo ao estado antigo da árvore. É o único lugar onde "o
valor mudou" fica visível — o widget novo traz o `is_toggled` novo, o estado
carrega o alvo antigo:

```rust
fn diff(&self, tree: &mut Tree) {
    let state = tree.state.downcast_mut::<State>();
    if state.target != self.is_toggled {
        state.target = self.is_toggled;
        state.animation.go_mut(self.is_toggled, Instant::now());
    }
}
```

Sem o campo `target` a detecção seria impossível: `Animation` sabe para onde
vai, mas o `diff` roda a cada rebuild (inclusive os que não mudam nada) e
precisa de um comparando estável.

### 3. Loop de frames auto-sustentado (e auto-desligado)

Ninguém redesenha sozinho — então **cada frame agenda o próximo**, enquanto a
animação corre. O gancho é o evento `RedrawRequested`, que todo widget recebe
no `update()` a cada frame desenhado:

```rust
if let Event::Window(window::Event::RedrawRequested(now)) = event {
    state.now = *now;                          // relógio para o draw (peça 4)
    if state.animation.is_animating(*now) {
        shell.request_redraw();                // agenda o frame seguinte
    }
}
```

Terminada a transição, `is_animating` vira `false`, ninguém pede mais frame e
o widget volta a custar **zero por quadro**. Não há timer, subscription nem
tick global do motor envolvidos — o custo é local ao widget e só durante os
200ms.

### 4. `draw()` interpola pelo progresso

O `draw` não recebe relógio — por isso o `now` guardado na peça 3 (o instante
do `RedrawRequested` que originou este frame). Com o progresso em mãos:

- a **posição** do knob é um lerp entre as duas pontas do trilho;
- as **cores** são o estilo do tema avaliado **nos dois extremos** e misturado
  canal a canal:

```rust
let progress = state.animation.interpolate(0.0_f32, 1.0, state.now);
let off = toggler::default(theme, /* status com is_toggled = false */);
let on  = toggler::default(theme, /* status com is_toggled = true  */);
// fundo do trilho: mistura off.background → on.background por `progress`
// knob: x = off_x + (on_x - off_x) * progress
```

Avaliar o catálogo nos dois extremos (em vez de guardar cores no widget) é o
que faz o trilho escorregar de cinza para o `primary` **de qualquer paleta**
— temas custom e os estilos builtin de `crate::style` funcionam sem uma linha
de código extra.

## O fluxo completo de um clique

```
clique → on_toggle publica a mensagem
       → componente troca a variável de contexto
       → view reconstruída com is_toggled novo
       → diff() percebe alvo ≠ novo e dispara go_mut
       → cada RedrawRequested desenha um passo e agenda o próximo
       → 200ms depois: is_animating = false, silêncio
```

Repare que o widget **não** anima "ao ser clicado" — anima ao **receber um
`is_toggled` diferente**. Consequência boa: mudar a variável por qualquer
outro caminho (script Luau, `ctx.set` num update, broadcast) anima igual.

## Detalhes práticos

- Exige a feature **`advanced`** do iced (expõe `Widget`, `Shell`, `Tree`,
  `layout`, `renderer` — o `Cargo.toml` da lib já a liga).
- O `AnimatedToggler` **não desenha rótulo**: `widget.rs` compõe
  `row![toggler, text(label)]`. Isso poupou replicar a máquina de
  layout/draw de parágrafo do iced (a maior parte do fonte do toggler
  original é isso).
- `mix_background` só interpola `Background::Color` (é tudo que o catálogo do
  toggler produz); um gradiente cairia no extremo mais próximo.
- O `last_status` replica o contrato do widget original: `update` registra o
  status em cada `RedrawRequested` e pede redraw quando ele muda entre frames
  (é o que dá o feedback de hover sem animação de estado).

## Checklist para animar outro widget

1. Copie o fonte do widget do iced (MIT) para um módulo novo em `src/`;
   especialize `Theme`/`Renderer` para os concretos do iced (menos genéricos
   para arrastar).
2. Defina o `State` com `Animation<T>` + o campo-alvo + `now`, e devolva-o em
   `tag()`/`state()`.
3. Detecte a mudança no `diff()` e chame `go_mut`.
4. No `update()`, trate `RedrawRequested`: guarde `now`, e `request_redraw()`
   enquanto `is_animating`.
5. No `draw()`, projete com `interpolate(...)` usando `state.now`; para cores,
   avalie o catálogo de estilo nos dois extremos e misture pelo progresso.
6. Componha texto/rótulos fora do widget se puder — some layout de parágrafo
   só se for inevitável.

## Uma segunda variante: rotação sem fim (`repeat_forever`)

O `<Spinner>` (`src/spinner.rs`, 0.53.0 — o indicador "busy"/indeterminado)
usa o mesmo esqueleto, mas troca a peça 2 (`diff()` detecta uma transição
pontual) por algo mais simples: **não há transição nenhuma para detectar**,
só uma fase que avança pra sempre.

- `Animation::new(0.0).duration(REVOLUTION).easing(Easing::Linear).repeat_forever()`,
  com `go_mut(1.0, agora)` chamado **uma única vez**, em `state()` (na
  criação) — nunca de novo no `diff()`. Sem `diff()` algum: o spinner não tem
  "estado alvo" para comparar entre um rebuild e outro.
- `Easing::Linear` importa aqui de um jeito que não importa no toggler: um
  spinner tem que girar em velocidade **constante** — qualquer easing
  ease-in/ease-out faria o rastro acelerar e desacelerar a cada volta, o que
  lê como "engasgo", não como rotação.
- `is_animating(agora)` de uma `Animation` com `repeat_forever()` nunca vira
  `false` — é exatamente o sinal de "continue pedindo quadro" que a peça 3
  (`update()` + `request_redraw()`) já sabia usar, sem mudar nada nela.
- `Animation<f32>` (ao contrário de `Animation<bool>`, o caso do toggler) não
  tem o açúcar `interpolate(start, end, at)` — projete o progresso cru com
  `interpolate_with(|t| t, at)` e escale você mesmo (aqui, `progresso * TAU`
  vira o ângulo).
- O anel em si é desenhado só com `fill_quad` (N pontos circulares, cada um
  com opacidade decaindo pela distância angular até a "cabeça" do rastro) —
  a mesma primitiva de baixo nível do knob do toggler, evitando puxar o
  trait `canvas`/`geometry::Renderer` só para um indicador.

Efeito prático: dois `<Spinner>` na mesma tela giram cada um com seu próprio
relógio (cada um tem seu `tree::State`), sem escrever nada no contexto do
app — o que também é a razão de o `PLANO_WIDGETS.md` ter reclassificado esse
widget: ele não precisa do desbloqueio de "estado por instância" que trava
boa parte do catálogo Qt, porque não guarda valor nenhum, só fase.

## Uma terceira variante: animar o **layout** (`<Reveal>`, 0.90)

O toggler e o spinner animam o que **desenham**. O `<Reveal>`
(`src/reveal.rs` — o corpo de um `<accordion>`/`<toolbox>` abrindo e fechando)
anima o que **mede**: a altura do nó, e portanto o layout de tudo que está
abaixo dele na tela.

As quatro peças continuam valendo palavra por palavra. O que se acrescenta é
uma quinta, e ela é obrigatória:

### 5. `shell.invalidate_layout()` a cada quadro da transição

O `iced` mede a árvore **uma vez**, quando a view é reconstruída, e reusa essa
medida em todos os quadros seguintes. Um widget que só muda o `draw` não se
importa; um que muda de tamanho, sim — sem invalidar o layout, a animação corre
no relógio interno e a tela fica parada, com a altura do primeiro quadro.

```rust
if let Event::Window(window::Event::RedrawRequested(now)) = event {
    state.now = *now;
    if state.animation.is_animating(*now) {
        shell.invalidate_layout();   // ← a quinta peça
        shell.request_redraw();
    }
}
```

`invalidate_layout` faz o `UserInterface::update` remedir a árvore inteira
**antes de desenhar, no mesmo quadro** (é o mesmo gancho que um `text_input`
usa quando o texto digitado muda a largura dele). Terminada a transição,
`is_animating` vira `false` e as duas chamadas param juntas.

O `layout()` então projeta a altura pelo progresso, e o filho continua medido
inteiro:

```rust
let child = self.content.as_widget_mut().layout(&mut tree.children[0], renderer, &limits.loose());
let natural = child.size();
let progress = tree.state.downcast_ref::<State>().progress();
layout::Node::with_children(
    Size::new(natural.width, natural.height * progress),
    vec![child],          // ancorado no topo: o corpo é descoberto, não empurrado
)
```

### O que vem de brinde com isso, e precisa ser tratado

Um filho maior que o pai transborda — e transborda de **três** jeitos:

| Transbordo | Sintoma se ignorado | Tratamento |
|---|---|---|
| Desenho | o corpo aparece por cima do que vem depois | `renderer.with_layer(bounds, …)` no `draw`, como o `scrollable` faz |
| Ponteiro | um clique abaixo da dobra cai no filho invisível, não no widget que se vê ali | entregar `mouse::Cursor::Unavailable` ao filho fora da parte visível |
| Overlay | o menu de um `<select>` escapa da dobra (overlay não é recortado por camada) | devolver `None` em `overlay()` enquanto `progress < 1.0` |

E uma consequência de projeto que não tem tratamento, só escolha: **o filho
existe mesmo fechado**. É de onde a altura encolhe ao fechar — um nó que não
está na árvore não tem de onde animar. O preço é montá-lo e medi-lo a cada
quadro; o que ele não faz é desenhar nem receber clique.

Para um builtin, isso vira uma regra dura no template: os dois braços do
`if`/`else` precisam ter a **mesma forma** de árvore (o `<accordionitem>` tem
cabeçalho + `<Reveal>` nos dois), porque o `iced` casa o estado do widget por
posição. Um braço com um filho a mais e o `<Reveal>` do outro braço nasce
zerado em vez de continuar de onde o antigo parou — a animação simplesmente
some. `tests/engine_tests.rs::accordion_tem_a_mesma_forma_aberto_e_fechado`
existe para pegar isso.
