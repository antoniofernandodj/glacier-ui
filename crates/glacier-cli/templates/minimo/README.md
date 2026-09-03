# {{titulo}}

Interface declarativa com [glacier-ui](https://crates.io/crates/glacier-ui): o
layout mora em XML (`.gv`), o estilo num `.gss` CSS-like e o comportamento num
`<script>` Luau interpretado em runtime.

```
cargo run
```

## O mapa

| Arquivo | O que é |
|---|---|
| `src/main.rs` | a casca: sobe o runner e registra a tela |
| `views/contador.gv` | a tela: `<screen>` + `<resources>` + layout + `<script>` |
| `views/styles/app.gss` | a paleta (`:root`) e as classes |
| `views/scripts/glacier.d.luau` | tipos dos globais do motor, só para o luau-lsp |
| `.luaurc` | declara esses globais para o type-checker |

## Como as peças se ligam

**`src/main.rs` é só a casca.** Ele sobe o runner, registra a tela e diz qual
abre primeiro. Tudo o que a tela *é* mora no `.gv` — e muda sem recompilar.

O caminho `views/contador.gv` é relativo ao **diretório de onde o app roda** (a
raiz do projeto, num `cargo run`), não ao `src/`. O mesmo vale para o `href` de
uma stylesheet.

Título e tamanho da janela **não** estão no Rust: quem os declara é o
`<screen>` do próprio `.gv`, junto da tela que eles descrevem — e assim o
título recarrega a quente. O erro de `register_component` já traz
`arquivo:linha:coluna`, o trecho e uma dica, então basta imprimi-lo.

**O comportamento vive no `<script>`.** Cada função global do bloco é um destino
possível para um `on_click`. `init()` é chamada pelo motor quando a tela entra.

O contexto (`ctx`) guarda **strings** — é o que os `{marcadores}` do markup
leem. Daí o `tostring`/`tonumber` no contador.

O corpo do `<script>` é recortado por texto **antes** do parse de XML, então
`<`, `>` e `&` podem aparecer no Luau à vontade. Quando ele crescer, mova-o para
um arquivo e use `<script src="scripts/x.luau">` — o `src` resolve relativo ao
`.gv`, e não ao diretório de onde o app roda.

## Estilo

No `.gss`, precedência do mais fraco ao mais forte: **tag < classe < id <
atributo inline** no nó; num `class="a b"`, `b` sobrepõe `a`.

Os tokens em `:root` são a fonte única da paleta, e `var()` atravessa
stylesheets — os mesmos nomes valem dentro de um `<style>` no template.

Num `Button`, `color` é o **fundo** e `text-color` é o texto. Cada bloco de
pseudo-estado (`:hover`, `:active`, `:focus`, `:disabled`) sobrescreve só o
campo que declara, por cima da regra base.

## Hot-reload

Com o app aberto, salve o `.gv` ou o `.gss`: o motor relê o arquivo e redesenha.
Só o `src/main.rs` exige recompilar — e ele quase não muda.

## Próximo passo

`glacier presets` lista presets maiores (navegação entre telas, componentes com
props, multi-janela, bandeja). A referência completa da linguagem está no
[README do glacier-ui](https://github.com/antoniofernandodj/glacier-ui).
