# A linguagem do `.gv` e do `.gss` — referência

Um lugar só para a gramática das duas linguagens. A convenção de **escrita** (caixa,
indentação, o que vai no `.gss`) está no `CLAUDE.md`; o porquê de cada decisão, no
`docs/PLANO_LINGUAGEM_GV_GSS.md`.

## `.gv`

### Estrutura

```gv
tag(atributo = valor, outro = valor) { filhos }
```

Parênteses e chaves são opcionais; a vírgula final é opcional; a quebra de linha não
significa nada. `CamelCase` é componente do app, minúscula é widget do motor.

### Valores e sigilos

| Escrita | Significa | Onde |
|---|---|---|
| `"texto"` | texto literal (com `@x` e `{expr}` interpolados) | qualquer lugar |
| `nome` sem aspas | identificador: classe, handler, enum (`center`, `fill`), símbolo (`+`) | atributos de nome |
| `@x` | lê a chave `x`; dentro de texto, atalho de `{@x}` | qualquer lugar |
| `{expr}` | expressão dentro de um texto | qualquer texto |
| `:attr = chave` | **liga** o widget à chave, nos dois sentidos. `:attr = @x` / `"{x}"` é a ligação **dinâmica** (o nome da chave vem de `x`) | widgets de valor |
| valor começando por `@`, `!`, `(`, `[` ou `t(`, `len(`… | expressão | qualquer atributo (exceto `on_*` e `:attr`) |
| `on_x = nome(args)` | ação com argumentos | atributos de ação |

### Expressões

```text
expr    := cond ? expr : expr      (o `: expr` é opcional; falso dá "")
         | a || b | a && b | !a
         | a == b | a != b | a < b | a <= b | a > b | a >= b
         | item in lista | x is empty | x is not empty
         | @chave | chave | "texto" | 'texto' | 12 | true | false | null
         | [a, b, c] | (expr) | função(args, nome = expr)
funções := t num date len lower upper trim json
```

- **Tipos.** O contexto guarda texto; o tipo é inferido na operação. `==` compara texto
  igual ou, se os dois lados são número, o número (`"1.0" == 1`). `<`/`>` só com
  número (texto dá falso). Falso: ausente, `""`, `false`, `0`, `no`/`não`, `off`, `null`,
  `[]`, `{}` — todo o resto é verdadeiro, inclusive um texto qualquer.
- **`is empty`**: ausente, `""`, `null`, `[]`, `{}`.
- **`in`**: lista literal, array JSON do contexto ou texto separado por vírgula/espaço.
- **`{chave|padrão}`**: o padrão (texto livre) quando a chave está ausente.
- **Texto entre aspas interpola** também dentro da expressão: `if @atual == "@alvo"` compara
  com o valor de `alvo`; `"v@versao"` junta. `@@` é o `@` literal.

### Condicional e laço — duas formas por conceito

| | Curta (um nó) | Bloco |
|---|---|---|
| Condição | `button("Salvar", if = @sujo && !@ocupado)` | `if expr { … } else if expr { … } else { … }` |
| Senão de um nó | `text("B", else)` **logo depois** do irmão com `if =` (senão é erro) | o `else` do bloco |
| Laço | `PickerRow(each = @servicos as s, title = @s.name)` | `each @servicos as s (key = id, fallback = Vazio) { … }` |

`if =` na raiz de um componente vale (a raiz é embrulhada num `Fragment`).

### Classe e atributo condicionais

```gv
button(class = nav-item, class:nav-item--active = @view in @alvo, on_click = @acao)
svg(src = @maximizada ? "restaurar.svg" : "maximizar.svg")
```

### Ações

```gv
on_click = salvar                       // handler sem argumento
on_click = exportar(@item.key, true)    // handler(chave, true) — tipados no Luau
on_click = window.resize("nw")          // embutido: vira window:resize:nw
on_click = copiar:@id                   // forma antiga (um argumento texto) — depreciada
```

O argumento literal chega com o tipo (`true` é booleano, `3` é número); o que vem de uma
chave chega como texto.

### i18n

`text(t("comum.salvar"))`, `button(t("svc.nome", nome = @n), …)`; dentro de um texto
composto, `"{t('k')}"`.

### Props

```gv
props {
  prop(name = rotulo, type = string)
  prop(name = ativo, type = bool, default = false)
  prop(name = itens, type = list, default = [])
}
```

Tipos: `string`, `number`, `bool`, `list`, `object`, `action`, `component`. Literal do
tipo errado no uso é erro; valor do contexto do tipo errado avisa uma vez. Prop sem
`default` é obrigatória; prop não declarada é erro.

### Botão ocupado

`app(busy_buttons = true)`: o botão fica desabilitado enquanto o handler do próprio
`on_click` está suspenso (num `fetch`, num diálogo). A chave é `__busy.<handler>`.
`busy = false` desliga num nó; `busy = @x` usa outra condição.


## A linguagem das frentes (`TODO.md`, 0.170)

### O `:` decide o que o valor é

```gv
textinput(::value = @form.nome)      // vínculo nos dois sentidos: a CHAVE
badge(:text = @ativo ? "on" : "off") // uma via: o atributo recebe a expressão
text(:count = @{n} + 1)              //   (e só aqui o valor é expressão)
text("oi", class = [btn, btn--lg, @extra])   // lista de classes
row(size = [1280, 820])              // par numérico como lista
row(:class.ativa = @view == @alvo)   // classe condicional
text("{@total ?? "nenhum"}")       // `??` no lugar de `{total|nenhum}`
```

Sem `:` o valor é literal ou interpolação (`"texto com @{n}"`), como sempre. A
forma antiga de ligação (`:value = nome`, com o nome nu) continua valendo e é
avisada — `glacier fmt --migrate` escreve `::value = @nome`.

### Ação é sempre chamada

```gv
button("Salvar", on_click = salvar())              // a forma canônica
button("Exportar", on_click = exportar(@item.key, true))
button("_", on_click = window.minimize())          // embutidos, mesma regra
button("Ir", on_click = "app:ir(@destino)")        // escapa do dono do componente
```

`on_click = salvar` e `window:minimize` são depreciados (aviso com `arquivo:linha`,
o `fmt --migrate` reescreve, a `feat!` torna erro). O `glacier check` lê as
assinaturas `--!strict` dos `.luau` da árvore e acusa handler inexistente (com o
nome mais próximo), aridade errada e argumento do tipo errado.

### `state` e o global explícito

```gv
component(name = Cartao) {
  props {
    prop(name = titulo, type = string)
    prop(name = on_abrir, type = action, default = "")
  }
  state {                       // local ao componente, tipado, com default
    aberto = false,
    toques = 0,
  }
  column(if = @aberto) { text("{@app.contagem}") }   // @app. é o global, dito
}
```

`state` declara a chave, o tipo e o valor inicial; o widget (`::checked = @aberto`)
e o script do componente escrevem nela. A separação por **instância** ainda não
existe (dois `Cartao` compartilham a chave) — é o próximo passo do `TODO.md`.

### Verdade com tipo

`app(truthy = typed)`: falso é `false`, `0`, `null`, `""`, `[]` e `{}`; **todo
texto é verdadeiro**, inclusive `"não"`, `"no"` e `"off"`. Sem a marca, vale a
regra textual de sempre. O `glacier check` lista os testes que mudam de valor com
a marca, e o `fmt --migrate` só a liga quando essa lista está vazia.

### Botão ocupado

`app(busy_buttons = true, busy_release = toast)`: o botão de um handler suspenso
fica desabilitado, um segundo disparo da mesma ação é descartado, o `type = submit`
é ocupado pelo `on_submit` do form, e o primeiro `toast` da corrotina (ou um
`busy.release()` explícito) libera antes de ela terminar. `spinner(if =
@__busy.salvar)` lê o bool tipado. `busy = false` num nó desliga a guarda dele.

## `.gss`

### Seletores

`.classe`, `#id`, `tag`, `*`, compostos (`row.a`, `.a.b`), `>` (filho), espaço
(descendente), vírgula (lista), `:hover :focus :active :disabled :invalid` (só no
último), `:first-child :last-child :only-child :nth-child(an+b)`, `:not(composto, …)`,
`:is(a, b)`.

Fora, com o motivo no erro: `+`/`~` e `:estado` num ancestral (o estado vem do widget
no desenho), `[attr]` (use classe condicional no `.gv`), `::pseudo`.

### Aninhamento

```gss
.btn {
  padding: 8 15;
  &:hover    { background: var(--hover); }   /* .btn:hover */
  &--primary { background: var(--green); }   /* .btn--primary (BEM) */
  .icone     { size: 12; }                   /* .btn .icone */
}
```

### Propriedades

Layout: `width height padding spacing align_x align_y max_width max_height`.
Caixa: `background gradient border_radius border_width border_color box_shadow opacity`.
Texto: `color size font font_family bold font_weight font_style line_height text_align`.
Outros: `cursor transition display` (e `hidden`, depreciado).

- Nomes em `snake_case` (os mesmos dos atributos do `.gv`); variáveis `--com-hifen` e
  features de `@media` com hífen (sintaxe do CSS).
- `font: mono` é a mono do app (`app(mono_font = "…")`) ou, sem ela, a do sistema.
- `bold`/`font_weight` mudam só o peso da família da regra.
- `line_height: 1.4` (fator) ou `20px`. `box_shadow: x y desfoque cor`.
- `letter_spacing` é erro: o iced 0.14 não o desenha.

### Cor

`app(gss_color = css)`: `background` é o fundo e `color` a cor do conteúdo em **todo**
widget. Sem ele (`legacy`, o padrão por enquanto), o `color` de `button`, `select`,
`comboedit` e `menu` é o fundo e o rótulo é `text_color`. `glacier fmt --migrate-color`
converte e liga o `css`.

### Tema

```gss
:root { --bg: #0A0E14; --tx: #E6EDF3; }
@theme { background: var(--bg); text: var(--tx); primary: #58A6FF; success: #3FB950; danger: #F85149; }
```

O `@theme` vence um `link(rel = theme)`, que é ignorado com aviso.

### Modo estrito

`app(gss_strict = true)` (e `glacier check --strict`): propriedade desconhecida é erro.

## Ferramentas

- `glacier check [pasta] [--strict]` — formas depreciadas, erros de parse, classes
  usadas e não definidas (e o contrário), fundo igual ao do tema.
- `glacier fmt [pasta] --migrate` — reescreve as formas depreciadas (idempotente);
  `--migrate-color` passa à cor do CSS; `--check` só lista.
- `glacier test --layout [LxA] [--strict]` — mede o layout sem janela a cada `:render()`
  e acusa nó com largura ou altura zero.
