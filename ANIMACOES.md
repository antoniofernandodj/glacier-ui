# Animações de widget no glacier-ui

Como o `<Toggle>` ganhou a bolinha deslizante (`src/animated_toggler.rs`,
0.52.0) — e o padrão a reutilizar para animar qualquer outro widget.

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
