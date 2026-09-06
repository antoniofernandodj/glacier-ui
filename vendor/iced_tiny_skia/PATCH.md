# `iced_tiny_skia` 0.14.0 com uma linha corrigida

Cópia do crate publicado, com **uma** alteração. Ela some quando sair a 0.14.1:
a correção já está no `master` do `iced`, só não entrou na versão publicada.

## O que muda

`src/lib.rs`, no laço que desenha as primitivas de `canvas`:

```diff
- let Some(group_bounds) = (group.clip_bounds()
-     * group.transformation()
-     * scale_factor)
-     .intersection(&layer_bounds)
+ let Some(group_bounds) = (group.clip_bounds()
+     * scale_factor)
+     .intersection(&layer_bounds)
```

`Layer::draw_primitive_group` já guarda o recorte **transformado**
(`Item::Group(primitives, clip_bounds * transformation, transformation)`).
Multiplicá-lo por `group.transformation()` outra vez aplica a translação **duas
vezes**.

## O que isso quebrava

Um canvas em (20, 50) tinha o recorte calculado em (40, 100): o desenho saía
cortado em cima e à esquerda. E como o erro cresce com a posição, **todo canvas
mais abaixo na tela ficava com o recorte fora de si mesmo e não desenhava nada**
— só o texto, que segue outro caminho.

Na prática: uma tela com mais de um `<gauge>`, `<dial>`, `<linechart>` ou
`<piechart>` mostrava o primeiro cortado e nenhum dos outros. Foi o que segurou
a Onda 7 (0.93.0).

## Por que isto está aqui, e não numa nota de "problema conhecido"

O `iced_tiny_skia` é o renderizador de **software** — o fallback que o `iced`
usa quando o `wgpu` não sobe. Numa GPU sã ninguém passa por ele; numa máquina
cujo driver Vulkan está incompleto (ver `TROUBLESHOOTING.md`), ele é o
renderizador **normal**, e a onda inteira ficava invisível.

O `[patch.crates-io]` no `Cargo.toml` da raiz vale só para quem compila **este**
repositório: um `[patch]` não atravessa para quem consome o `glacier-ui`
publicado no crates.io.

## Como remover

Quando a 0.14.1 (ou posterior) sair:

1. apague `vendor/iced_tiny_skia/`;
2. apague a seção `[patch.crates-io]` do `Cargo.toml` da raiz;
3. `cargo update -p iced_tiny_skia`.

O resto do diretório é o crate publicado, sem alteração, sob a licença MIT do
`iced` (ver `LICENSE` ao lado, se presente, ou <https://github.com/iced-rs/iced>).
