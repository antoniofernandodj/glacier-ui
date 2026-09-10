# Onde a glacier-ui está, em relação ao Qt e ao Vue + Vuetify

Documento de avaliação, escrito sobre a **0.101.0**. A pergunta que ele responde
é estreita de propósito:

> Abstraindo o fato de que quem pinta é o `iced` — o motor já compete com o Qt
> na **capacidade de desenhar telas complexas**? E com o Vue + Vuetify?

A resposta curta é: **no catálogo, sim; na tela complexa, ainda não.** As duas
metades dessa frase medem coisas diferentes, e o resto do documento é a
separação delas.

## A resposta em uma tabela

| Eixo | vs. Qt Widgets | vs. Vue + Vuetify |
|---|---|---|
| Catálogo de widgets | **par** | **par**, e à frente no que é de desktop |
| Layout | quase par | atrás |
| Vocabulário de estilo | **muito atrás** | **muito atrás** |
| Estado e reatividade | atrás | **muito atrás** |
| Model/view (tabela, árvore) | atrás | atrás |
| Desenho custom | atrás | par (ninguém pinta à mão) |
| Extensão pelo app | **muito atrás** | atrás |
| Infra (a11y, i18n, RTL, impressão) | **ausente** | **ausente** |
| Custo por tela | par | à frente |

## 1. O catálogo: aqui a comparação já se ganhou

O motor publica hoje **69 tags primitivas** (`NodeType::tag_name`, em
`src/parser.rs`) e **33 builtins registrados** (`builtin_components`, em
`src/builtins/mod.rs`) — cerca de **cem tags** disponíveis sem o app configurar
nada.

O `PLANO_WIDGETS.md` catalogou 124 widgets distintos do Qt e marcou, na revisão
mais recente, **~119 como ✅ (95%)**. Os quatro ⬜ que sobram — `TextBrowser`,
`Shader`, `Q3D`, `PrintDialog` — saíram da fila **por decisão escrita**, não por
bloqueio: são rich text, wgpu/WGSL, 3D e integração com spooler de impressão. Os
🟡 restantes (`ScrollBar` avulso, `QStackedLayout`, `QScroller`) são 🟡 de
propósito.

> A tabela da §2 do `PLANO_WIDGETS.md` está **desatualizada em duas linhas**:
> `FontSelect` e `FontDialog` continuam ⬜ lá, mas existem em
> `src/builtins/font_select.rs` e `font_dialog.rs` desde a 0.98 (Onda 10). A
> contagem acima já corrige isso.

E o catálogo tem peças que o Vuetify **não tem**, porque não são de web: `dock`,
`mdiarea`/`mdisubwindow`, `splitter`, `statusbar`, `menubar` nativo, bandeja do
sistema, `gauge`, `dial`, `lcdnumber`, `tumbler`, `sizegrip`, `rubberband`,
`shortcutinput`, `wizard`, os diálogos de arquivo/cor/fonte do SO. Numa
comparação por catálogo, a glacier-ui não está atrás do Vuetify — está num eixo
diferente e mais próximo do Qt.

**Conclusão do eixo:** por catálogo, a resposta à pergunta é sim. O que segue
diz por que isso não basta.

## 2. Por que "catálogo" e "tela complexa" não são a mesma medida

Uma tela complexa não é uma tela com muitos widgets. É uma tela em que:

1. os widgets precisam ficar **arranjados** de um jeito específico;
2. eles precisam ficar **parecidos com um produto**, não com um kit;
3. o estado deles precisa **conviver** — dez instâncias do mesmo painel, cada
   uma com a sua expansão, a sua seleção, o seu filtro;
4. alguém precisa poder **desenhar o que não existe no catálogo**.

O catálogo cobre o item 0 (ter a peça). Os quatro acima é onde o Qt e o Vue
ganham hoje, e cada um por um motivo diferente.

## 3. Layout

O motor tem `column`, `row`, `stack`, `grid` (com medição bidimensional real, em
`src/grid.rs`), `flow`, `scrollable` com **virtualização** (`virtualize=`),
`splitter`, `dock`, `mdiarea` e overlays ancorados (`src/anchored.rs`). É um
conjunto sério.

O que falta, em relação ao `QLayout`:

- **medidas**: `parse_length` (`src/widget.rs:1058`) aceita `fill`, `shrink`,
  `fill N` e um número em px. **Não há porcentagem** e **não há `min-width`** no
  nó (só `max_width`/`max_height`, via `.gss`);
- **size policies**: o Qt tem `Preferred`/`Expanding`/`MinimumExpanding`/
  `Ignored` com `stretch` por eixo. `fill N` cobre o caso comum de esticamento,
  e só ele;
- **`QFormLayout`** e alinhamento por **baseline** não existem;
- **`margin` não existe**. Só `padding` (no pai) e `spacing` (entre irmãos).
  Afastar *um* filho dos outros exige um `<space>` ou um container extra — e
  container extra, neste motor, é caixa pintada a mais (ver §7).

Contra o Vue, a distância é maior: não há `flex-grow`/`flex-basis`,
`position: absolute`, `z-index`, `grid-template-areas` nem `gap` por eixo.

## 4. Estilo — o gargalo real

Esta é a seção que responde à pergunta.

O `.gss` reconhece **21 propriedades** (o `StyleRule` de `src/stylesheet.rs`;
o `match` de chaves fica perto da linha 1126): `width`, `height`, `padding`, `spacing`, `align_x`, `align_y`,
`background`, `border_radius`, `border_width`, `border_color`, `color`,
`text_color`, `size`, `bold`, `font_family`, `gradient`, `text_align`, `cursor`,
`max_width`/`max_height`, `hidden`/`display`.

O que **não** existe, e cada ausência é uma decisão de tela que não dá para
tomar:

| Ausente | Consequência prática |
|---|---|
| `margin` | espaçamento assimétrico vira wrapper |
| borda **por lado** | uma linha só embaixo (a régua de um cabeçalho) exige um `<rule>` irmão |
| raio **por canto** | um card com o topo arredondado e a base reta não é escrevível |
| `box-shadow` | está escrito na própria tabela do plano: "o `UiNode` não tem campo de sombra" — é por isso que o `QFrame` saiu sem `Raised`/`Sunken` |
| `opacity` | não há esmaecer nada |
| `transform` / `rotate` | não há girar um rótulo |
| `transition` | **não há animação declarativa** (ver §6) |
| `line_height`, `letter_spacing`, `italic`, pesos de fonte | tipografia é `size` + `bold` |
| `overflow` | recorte só onde o `scrollable` já dá |

E os **seletores** são de um nível só: `.classe`, `#id`, `Tag`, mais quatro
pseudo-estados (`:hover`, `:focus`, `:active`, `:disabled`) e `@media` com
quatro features de tamanho. **Não há** descendente (`.card .titulo`), filho
(`>`), composto (`.a.b`), `:nth-child`, atributo, `::before`/`::after` nem
sub-controle. O `@media` também não tem `prefers-color-scheme`.

A comparação é direta e desfavorável nas duas direções:

- o **Qt Style Sheets** tem margem, borda por lado, raio por canto, gradientes,
  `image`/`border-image` de 9 fatias, sub-controles (`QComboBox::drop-down`),
  estados (`:checked`, `:!enabled`, `:first-child`) e — o degrau acima —
  `QStyle`/`QProxyStyle`, que reescreve a pintura de **qualquer** primitiva sem
  tocar no widget;
- o **CSS** que o Vuetify assume é o box model inteiro, mais flex, grid,
  transições, `z-index` e um seletor arbitrário.

O `.gss` compensa parte disso com coisas que o CSS não tem de graça: a escada de
precedência bem definida (tag de componente < tag builtin < classe do template <
classe do uso < id < inline), as props `*_class` que todo builtin expõe desde a
0.89, o aviso com sugestão em propriedade desconhecida. São boas decisões dentro
de um vocabulário pequeno. O vocabulário continua pequeno.

**Se um único item desta comparação fosse escolhido para trabalhar, é este.** As
21 propriedades são o motivo pelo qual uma tela glacier-ui e uma tela Qt com o
mesmo catálogo não saem parecidas.

## 5. Estado, reatividade e composição

O contexto do motor é `HashMap<String, String>` (`ContextMap`, em
`src/lib.rs:458`): **plano, global, e de strings**. Toda a reatividade é: mudou
uma chave → reavalia a árvore → o `{placeholder}` vira o texto novo.

Isso tem três consequências para telas complexas:

1. **Não há estado por instância.** É o assunto mais discutido do
   `PLANO_WIDGETS.md`, e a marca `●` da tabela existe só para ele. Duas
   instâncias do mesmo componente numa tela compartilham as chaves que ele usa,
   a menos que o app passe nomes de chave diferentes por prop. O motor
   contornou isso treze vezes com invenções boas (o **conjunto nomeado** via
   `contains`, o valor no nome de chave que o markup escolhe, o `Program::State`
   do `iced` para o arrasto) — mas cada contorno é o app fazendo a contabilidade
   de nomes que o Vue faz sozinho.
2. **Não há expressão no template.** A interpolação é substituição de texto.
   Não há aritmética, concatenação condicional nem chamada de função no markup;
   o controle de fluxo é `if`/`else-if`/`else` com `equals`, `notEquals`,
   `one_of`, `contains`, `empty`/`not_empty`. Qualquer coisa derivada é
   calculada no Luau ou no Rust e **gravada de volta numa chave** — o
   equivalente a não ter `computed`, só `watch` + variável.
3. **Não há tipo.** Uma lista é uma string com JSON dentro. O motor cacheia o
   parse (a 0.76 fez isso, e foi o que deu os 4,46 ms de acerto de cache em 2000
   linhas), mas o modelo mental continua sendo texto.

Contra o **Vue**, é a maior distância do documento inteiro: `ref`/`reactive`,
`computed`, `watch`, estado por componente, `provide`/`inject`, `v-model` em
componente, slots com escopo, eventos — nada disso tem análogo. Contra o **Qt**,
a distância é menor mas existe: um `QWidget` é um objeto com campos, e sinais e
slots são tipados.

O que o motor **tem** e é bom: props com default inline (`{prop|valor}`),
`spread=`, `<slot/>` nomeado com nome interpolável, componentes declarados no
próprio `<resources>`, `<template>` sem nó extra na árvore, `<form>` reativo, e
uma camada Luau completa (`fetch` suspensivo, timers, `storage`, `sse`,
`websocket`, `require`, extensão por função Rust). A camada de comportamento não
é o problema; a de **estado** é.

## 6. Animação

Não há `transition` no `.gss`. O que existe é um **padrão de implementação**
documentado no `ANIMACOES.md` — estado no `tree::State`, transição detectada no
`diff()`, loop de quadros auto-sustentado, interpolação no `draw()` — aplicado
caso a caso em Rust: `src/animated_toggler.rs`, `src/spinner.rs`,
`src/reveal.rs`. O `<Skeleton>` saiu **sem** pulsação exatamente por esse custo,
e está escrito na linha dele.

Qt tem `QPropertyAnimation` e, no QML, `Behavior`/`Transition` declarativos. CSS
tem `transition` e `@keyframes` numa linha. Aqui, animar algo novo é uma tarefa
de Rust com quatro peças.

## 7. Desenho custom e extensão pelo app

Duas afirmações separadas, e a segunda é a mais importante.

**Desenho custom** existe, e é declarativo: a Onda 13 abriu o `src/canvas.rs`
num vocabulário de formas — `<canvas>` com `<path>`, `<arc>`, `<circle>`,
`<rect>`, `<line>`, `<polyline>`, `<polygon>` e `<shapetext>`, com
`fill`/`stroke`/`stroke-width` saindo do `.gss`. Ficou de fora, por escrito: o
comando `A` de arco elíptico no `<path>`, `on_click`/hover **por forma**, e
animação por tique. Comparado ao `paintEvent` do Qt e ao `QGraphicsScene`
(itens com evento, transformação e Z), é bem menos.

**A extensão pelo app é o degrau que falta.** Um `Component` registrado por um
app é `name` + `template` (`Template::File` ou `Template::Inline`) + `update` —
ou seja, **composição de tags existentes**. Não há hook de pintura, nem de
medição, nem de evento bruto. Uma primitiva nova mexe em três arquivos do motor
(`parser.rs`, `eval.rs`, `widget.rs`), como o `PRIMITIVAS.md` descreve, o que
significa que **um widget genuinamente novo é um PR no motor, não uma crate do
app**.

É uma diferença categórica:

- no **Qt**, `class MeuWidget : public QWidget` com `paintEvent` é a primeira
  aula do tutorial;
- no **Vue**, um componente de terceiro é um `npm install`, e o ecossistema
  inteiro depende disso.

Enquanto esse degrau não existir, o catálogo da glacier-ui é exatamente o que a
lib publica — **não há terceiro widget possível**. Para "desenhar telas
complexas", esse é o segundo item mais caro depois do vocabulário de estilo.

## 8. Model/view

A §2.4 do plano fechou em 10/11, e `tableview` tem mais do que o nome sugere:
`items`, `columns`, seleção simples ou múltipla, **ordenação** (`sort=`) e
**largura de coluna arrastável** (`widths=`), sobre a medição de
`src/grid.rs`. Com `virtualize=` no `scrollable`, lista longa não é problema.

O que o Qt tem e não há aqui: `QAbstractItemModel` como interface (o modelo é
sempre um array JSON numa chave), proxies de **ordenação e filtro**
encadeáveis, **delegates** (um widget arbitrário por célula, e edição na
célula), cabeçalhos com seções móveis, `span`, e o mesmo modelo servindo três
views ao mesmo tempo.

O Vuetify tem um `v-data-table` com paginação de servidor, linhas expansíveis,
agrupamento e slot por coluna. A distância para os dois é a mesma e tem a mesma
causa: **falta o widget por célula**, que por sua vez esbarra na §7.

## 9. O que não existe em nenhuma quantidade

Quatro ausências que não são "atrás", são "zero", e que qualquer app de porte
vai encontrar:

- **acessibilidade** — nenhuma ocorrência no repositório. Sem árvore de
  acessibilidade, sem papéis, sem rótulo para leitor de tela. Qt tem
  `QAccessible`; o Vuetify emite ARIA por componente;
- **internacionalização** — não há catálogo de tradução, plural nem formatação
  por locale. Só `date`, no Luau, sobre strings ISO;
- **RTL** — nenhuma ocorrência. Layout invertido não é suportado;
- **ordem de foco declarável** — Tab e Shift+Tab funcionam em toda a tela: um
  listener global (`tab_focus_from_event`, em `lib.rs:4015`) vira
  `focus_next`/`focus_previous` do `iced`, porque os inputs dele não avançam
  sozinhos. O que não existe é o **`tabindex`**: a ordem é a da árvore, e não há
  como declarar outra, nem tirar um nó da sequência.

Some-se a impressão (`PrintDialog`, fora por decisão) e não há framework de
undo/redo.

## 10. Custo e escala — onde o motor está bem

Os números do `tests/perf_arvore.rs` (`--release`, do CHANGELOG da 0.76):

| N linhas | acerto de cache | reavaliação completa | memória |
|---|---|---|---|
| 100 | 97 µs | 1,09 ms | 0,7 MB |
| 500 | 545 µs | 6,15 ms | 4,0 MB |
| 2000 | 4,46 ms | 31,8 ms | 15,2 MB |

Ler isso com honestidade: **31,8 ms para reavaliar uma lista de 2000 linhas é um
quadro perdido**. Mas é o caso em que a lista inteira mudou; a mudança que não
toca a lista custa 4,46 ms, e com `virtualize=` a árvore renderizada nem chega
lá. Um app Vuetify com 2000 linhas sem virtual scroller se sai pior.

E o número que o `PRIMITIVAS.md` mediu vale repetir, porque contradiz a
intuição: numa Intel HD 2500, com 111 nós e 20 caixas pintadas, o **render do
motor custa 0,07 ms** e a **pintura custa ~45 ms**. O gargalo de uma tela
glacier-ui não é o motor nem o número de widgets — é a **área pintada**. Isso é
uma vantagem estrutural sobre o Vuetify (que é DOM + CSS + layout do browser) e
uma paridade com o Qt.

Somem-se: binário único sem runtime, hot-reload, 515 testes, extensão do VS
Code, multi-janela, bandeja, instância única.

## 11. Veredito

**Contra o Qt.** Por catálogo de widgets, a glacier-ui **compete** — 95% do
catálogo que ela mesma enumerou, com quase cem tags e as peças de desktop que
importam. Para *desenhar telas complexas*, ainda **não compete**, e as razões
são três, em ordem de custo: (1) o vocabulário de estilo de 21 propriedades sem
margem, sombra nem borda por lado, contra o Qt Style Sheets mais `QStyle`; (2) a
impossibilidade de um app criar um widget pintado novo; (3) o estado global e
plano, que faz cada instância repetida virar contabilidade de nomes.

**Contra o Vue + Vuetify.** O catálogo é **par ou melhor** no que é de desktop.
O resto é uma distância maior do que contra o Qt, e por um motivo só: o Vuetify
herda **CSS inteiro** e a **reatividade do Vue** de graça, que são exatamente as
duas coisas mais fracas aqui. Em compensação, a glacier-ui ganha no que o
browser não dá: um binário, sem runtime, com pintura de GPU e um custo de
avaliação medido em microssegundos.

**A leitura mais útil deste documento**: o projeto passou nove ondas resolvendo
o eixo em que já estava ganhando (mais widgets) e chegou ao fim dele — a §6.2 do
plano registra que "não sobra linha que alguém tenha decidido fazer e não tenha
feito". O próximo eixo não é mais uma onda de catálogo. É, na ordem:

1. **o vocabulário de estilo** — `margin`, borda por lado, raio por canto,
   sombra, `opacity`, e seletor descendente;
2. **o ponto de extensão do app** — um `Component` que possa pintar, para que
   exista um terceiro widget no mundo;
3. **estado com escopo** — o `●` da tabela, resolvido de vez em vez de
   contornado;
4. **`transition` no `.gss`** — o `ANIMACOES.md` já tem o padrão; falta a
   declaração;
5. **acessibilidade e i18n** — as duas que hoje são zero.
