# `.gss` moderno: `opacity`, `transition`, `calc()` e `var()` numérico

Fase 5 do `PLANO_SUBSTITUIBILIDADE.md`. Exemplo: `examples/gva/css_moderno`.

## `var()` em campos numéricos

Antes só os campos de texto aceitavam `var(--x)`; `spacing: var(--gap)` era erro. Agora
todos os numéricos aceitam (`spacing`, `border-radius`, `border-width`, `size`,
`max-width`, `max-height`, `opacity`):

```gss
:root { --gap: 12; --raio: 8; }
.card { spacing: var(--gap); border-radius: var(--raio); }
```

O valor espera as variáveis serem lidas e é resolvido junto com as de texto. Se o resultado
não for um número, o motor avisa no `stderr` e ignora a declaração.

## `calc()`

Aritmética de **números**: `+ - * /`, parênteses, `calc()` aninhado e o sufixo `px`.

```gss
.caixa { width: calc(300 - 2 * 20); border-radius: calc(var(--raio) * 2); }
.tela  { padding: calc(var(--gap) * 2); }
```

Não há `%` do pai nem `em`: o `.gss` não mede o pai, e `calc(100% - 20px)` ficaria sem
significado. Unidade que não seja `px`, divisão por zero ou parêntese aberto deixam o valor
como está, e o erro do número aparece na declaração.

## `opacity`

`opacity: 0.5` (ou `50%`) multiplica o alfa das cores do **próprio** nó: fundo, borda, texto
e a `color` do tipo (fundo de `<button>`, cor de `<text>`). **Os filhos não são afetados** —
o `iced` não compõe um grupo com opacidade —, então num contêiner sem cor própria ela não faz
nada visível. Um gradiente não é alterado.

## `transition`

```gss
.botao        { color: #5865F2; }
.botao:hover  { background: #E07A5F; }
.botao        { transition: background 400ms ease-out; }
```

Só em `<button>`, e só com `color` (ou `gradient`) definido — no botão, o fundo de repouso é
`color` e o dos estados é `background`. Anima a troca entre repouso, hover e pressionado.

- **Propriedades:** `background`, `color`/`text-color` (o rótulo), `border-color`, `opacity` e
  `all`.
- **Duração:** `150ms` ou `0.2s`. **Uma duração só** vale para todas as propriedades (a
  primeira que aparecer).
- **Curva:** `linear`, `ease` (padrão), `ease-in`, `ease-out`, `ease-in-out`.
- Espessura e raio da borda trocam de uma vez. Um fundo em gradiente de verdade (cores
  diferentes) também troca de uma vez; um gradiente de paradas iguais — como o botão monta o
  fundo de um estado — anima como cor sólida.

Como funciona: o estado da animação mora na árvore de widgets (um embrulho, `Transicionado`),
e o embrulho pede um quadro enquanto ela anda. Ver `src/transition.rs` e `docs/ANIMACOES.md`.

### Como foi verificado — e o que não foi

`tools/transition_check` (`cd tools/transition_check && cargo run`) roda a lógica sem janela,
com o `Button` do iced e um renderizador nulo: a cor sobe em rampa, o embrulho pede quadros
enquanto anima e para depois, e na saída volta animando. **Não foi medida numa janela real**:
a medição por captura de tela precisa do ponteiro, e nesta sessão o ponteiro injetado deixou
de se mover. Quem rodar `css_moderno` deve ver o botão "Com transition" clarear/escurecer
em ~400 ms, e o "Sem transition" saltar.

## Fora desta fase

`:first-child`, `:last-child`, `:nth-child` e `@keyframes` não foram feitos. `@font-face` já é
coberto pelo `<font>` no markup do `<app>`.
