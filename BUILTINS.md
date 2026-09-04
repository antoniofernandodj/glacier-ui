# Widgets embutidos (`src/builtins/`)

Biblioteca de componentes que a própria `glacier-ui` registra sozinha, para
ficarem disponíveis por tag em **qualquer** template sem o app configurar nada —
o objetivo de longo prazo é uma biblioteca vasta de widgets, no espírito do Qt.

Este documento é o guia prático de **como estender** essa biblioteca. As
garantias e restrições do motor por trás dela estão documentadas nas docstrings
de `src/builtins/mod.rs`; aqui o foco é o passo a passo.

## Três níveis de "componente" no glacier-ui

| | Onde vive | Precisa registrar? | Disponível como |
|---|---|---|---|
| **Primitiva** | `src/widget.rs` + `src/parser.rs` | não | `<Button/>`, `<Text/>`, … |
| **Builtin** | `src/builtins/` (um arquivo por widget) | não (a lib registra) | `<Badge/>`, `<SpinBox/>`, `<GroupBox/>` e afins |
| **Componente do app** | código/arquivos do app, ou o `<resources>` de um template | sim (`register`/`import`), exceto a forma declarada | `<PerfilCard/>` |

Este documento cobre só o nível **Builtin**. Para o passo a passo de uma
**Primitiva** nova — inclusive uma armadilha real do motor (`Length::Fill` vs.
o wrap genérico de background/borda) — ver [`PRIMITIVAS.md`](PRIMITIVAS.md).

Um componente do app tem **três** formas de existir, e a terceira dispensa
arquivo: `<component name="X">…</component>` dentro do `<resources>` de um
template declara o componente ali mesmo (ver `README.md`). Ela usa a mesma casca
de um `.gv` — `<props>` e layout — e serve às peças pequenas de uma tela só; um
builtin continua sendo o caminho para o que a **lib** publica a todos os apps.

### Como saber se o widget é builtin ou primitiva

A pergunta se responde antes de escrever a primeira linha, e este projeto a
errou dez vezes — sempre para o mesmo lado, o de escrever (ou adiar) um builtin
que não tinha como funcionar. Os quatro sinais que já apareceram:

1. **O template precisaria ler uma chave cujo *nome* vem de uma prop.** É a
   indireção `{{value}}`, que o interpolador não tem. Foi o que reclassificou
   `TimePicker` → `<timeedit>` (0.68), `Calendar` (0.84) e `MaskedInput` (0.85).
2. **A repetição é dirigida por um *número*, não por uma coleção.** O `for-each`
   lê uma chave com um array JSON; a janela `4 5 6` de uma paginação e as cinco
   estrelas de uma nota não existem em array nenhum — são derivadas. Foi o que
   reclassificou `Pagination` e `Rating` (0.85).
3. **O widget precisa de um evento que o markup não expõe.** O motor dá
   `on_press`, `on_double_click`, `cursor` e `tooltip` a qualquer nó, mas não um
   `on_enter`. Foi o segundo motivo do `Rating`, e é o que a pré-visualização no
   hover exige.
4. **O widget precisa de uma camada, ou de medir os irmãos.** Um painel que sai
   do fluxo e se posiciona contra a janela é um `iced::advanced::Overlay`
   (`<popover>`, `<popup>`, `<autocomplete>`, 0.92); uma coluna cuja largura é o
   máximo das células dela precisa de uma medição bidimensional (`<grid>`,
   `<tableview>`, 0.92). Nenhuma das duas é composição de nós.

A regra prática, que o [`PRIMITIVAS.md`](PRIMITIVAS.md) já registrava: **um
builtin que só funcionaria se o interpolador (ou o markup) tivesse mais uma
capacidade é, quase sempre, uma primitiva mal classificada.**

E a recíproca, que a Onda 5 acrescentou: **um builtin adiado por um habilitador
merece uma releitura antes de virar uma rodada.** O `Tabs` completo esperava
"estado por instância" desde a 0.65; o que faltava era **interpolar o nome de um
slot**, uma linha no `eval.rs`. O `Drawer` esperava "a animação do motor"; o que
faltava era um eixo no `<reveal>`. Os dois viraram builtins na 0.92, sem nada de
extraordinário.

Um **builtin** é um componente comum (`impl Component`) — a única diferença é que
a lib o registra em `GlacierUI::new()`, então ele não exige `register()` do app.
Uma tag desconhecida no XML vira uma referência de componente resolvida pelo
nome; como o builtin já está registrado, `<Badge/>` "simplesmente funciona".

## Passo a passo: adicionar um widget

### 1. Escreva o `impl Component` num arquivo de `src/builtins/`

```rust
/// Um separador horizontal fino — divide seções de uma coluna.
///
/// Props (opcionais, com default inline):
/// - `divider_color`  — cor. Default: `#313244`.
/// - `divider_height` — espessura em px (numérico). Default: `1`.
struct Divider;

impl Component for Divider {
    fn name(&self) -> &str {
        "Divider"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Container
                    background="{divider_color|#313244}"
                    width="fill"
                    height="{divider_height|1}"
                />"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {}
}
```

### 2. Registre-o na lista

`builtin_components()` (em `src/builtins/mod.rs`) é o **único** ponto que o motor
lê. Declare o `mod` e some o widget na lista:

```rust
pub fn builtin_components() -> Vec<Box<dyn Component>> {
    vec![
        Box::new(Badge),
        Box::new(Divider), // <-- novo
    ]
}
```

Pronto. `<Divider/>` já está disponível em qualquer template. Nada mais na lib
precisa mudar.

### 3. Teste (ver a seção _Testando_)

## O contrato `Component`

Definido em `src/component.rs`. Para um widget apresentacional você só implementa
`name`, `template` e um `update` vazio; o resto tem default.

| Método | Obrigatório | Papel |
|---|---|---|
| `name(&self) -> &str` | sim | Nome único = a tag usada no XML (`"Badge"` → `<Badge/>`). |
| `template(&self) -> Template` | sim | A UI. Em builtin, **sempre** `Template::Inline` (ver abaixo). |
| `update(&mut self, action, value, ctx)` | sim | Reage a ações da própria UI. Vazio se apresentacional. |
| `init(&mut self, ctx)` | não | Semeia estado inicial. **Evite em builtins** (ver restrições). |
| `children(&self) -> Vec<Box<dyn Component>>` | não | Subcomponentes próprios, registrados em cascata. |
| `on_form_submit(&mut self, action, ctx)` | não | Trata o `onSubmit` de um `<Form>`. |

### Por que `Template::Inline` e não `Template::File`

O XML de um builtin é **compilado no binário** (uma string `Inline`), nunca lido
de disco. Isso torna o parse determinístico: se falhar, é bug da lib, não do app.
Graças a isso `register_builtins` pode usar `expect`/`panic`, e `GlacierUI::new()`
continua infalível (devolve `Self`, não `Result`). Um builtin com
`Template::File` reintroduziria I/O que pode falhar em runtime — não use.

Dica de sintaxe: como o XML tem `#` (cores) logo após aspas (`background="#..."`),
use raw string com `##` quando houver cores literais: `r##"..."##`. Com só
placeholders (`{badge_bg|#...}`) o `#` está "dentro" do template e `r#"..."#`
basta — mas `r##` sempre funciona.

## Props: passar valores por instância

Os atributos de `<Badge foo="x"/>` viram **props**: são mesclados num clone do
contexto *só daquela instância* durante a avaliação. No template, `{foo}` resolve
para o valor da prop.

```xml
<Badge badge_text="Novo" badge_bg="#A6E3A1" />
```
```rust
// no template do Badge:
content="{badge_text}"  background="{badge_bg}"
```

Convenção: **prefixe** os nomes de prop com o do widget (`badge_*`, `divider_*`).
Não é obrigatório, mas evita confusão com chaves de contexto do app.

### Defaults por instância: `{prop|default}`

Um placeholder pode declarar um default inline após `|`. Se a prop não for
passada, usa-se o texto depois do `|`:

```xml
background="{badge_bg|#89B4FA}"   <!-- sem a prop badge_bg, fica #89B4FA -->
```

Isso é o jeito **correto** de dar defaults a um builtin — não semeie defaults no
contexto global via `init()` (ver restrições). Espaços em volta da chave e do
default são aparados: `{ badge_bg | #89B4FA }` funciona.

### Atributos numéricos também aceitam props

Atributos numéricos — `size`, `spacing`, `border_radius`, `border_width`,
`max_width`, `max_height` — aceitam `{prop}` (e `{prop|default}`), igual aos de
string. O motor detecta o `{`, adia a conversão e resolve para `f32` na
avaliação (mecanismo em `src/parser.rs`, enum `NumAttr`).

```xml
<Text content="{badge_text|Badge}" size="{badge_size|13}" />
```

Se o valor resolvido não for um número (ex.: prop omitida e sem default), o
atributo fica sem valor — herda o default do widget nativo do `iced`.

### Precedência de um campo

Do mais forte ao mais fraco:

1. Valor templado (`{…}` já resolvido) — **desde que resolva para algo**
2. Literal inline no atributo (`padding="4 10"`)
3. Valor herdado de uma classe `.gss` (`class="..."`)

A ressalva do degrau 1 é o que faz prop e classe conviverem no mesmo campo. Um
`background="{bg}"` cuja prop não veio resolve para vazio, e vazio **não é um
valor**: o campo cai para a classe. Antes da 0.89 ele vencia mesmo vazio, e o
efeito era um widget sem fundo nenhum — o `<Frame shape="filled">` chegou a ter
o braço inteiro duplicado (um com o atributo, outro sem) só para contornar isso,
e o `<Avatar>` foi o único builtin sem folha `<style>` pela mesma razão. Os dois
voltaram à forma simples.

O corolário para quem escreve um builtin: **o default de uma cor vai numa
classe da lib, não num `{prop|#aabbcc}`**. Um default inline resolve sempre, e
resolvendo sempre ele vence toda classe — inclusive a que o app injetou.

## Deixar o app estilizar o que está DENTRO do widget

`class` no uso de um componente aplica na **raiz expandida** e só nela (0.69).
É o certo — `<card class="destaque">` deve pintar o cartão, não o subtítulo —
mas deixa sem resposta o pedido mais comum de quem adota a biblioteca: *"quero
este item de lista com a cor do meu tema"*.

A resposta é uma **prop por nó**, com nome que diz o alvo, encaminhada à `class`
daquele nó no template:

```rust
// no template do builtin
r#"<Button class="listview-item {item_class}" …>
       <Text class="{label_class}" content="{it.label}" />
   </Button>"#
```

```xml
<!-- no uso -->
<listview items="servicos" value="qual" selected="{qual}"
          item_class="linha" selected_class="linha_ativa" />
```

Três regras que a biblioteca inteira segue desde a 0.89:

1. **A injetada vem por último na lista de classes.** Classes resolvem da
   esquerda para a direita, então `class="listview-item {item_class}"` deixa o
   app redefinir o que quiser do default e herdar o resto — inclusive os
   `:hover`, que continuam valendo se ele não declarar os seus.
2. **Um par base/refinamento se escreve na ordem em que se aplica.**
   `class="listview-item listview-item-sel {item_class} {selected_class}"`:
   `selected_class` vence `item_class` porque está à direita.
3. **Nó de raiz não ganha prop.** O `class` do uso já o alcança, e uma prop a
   mais ali seria um segundo jeito de fazer a mesma coisa. É por isso que o
   `<toolbutton>` — cuja raiz *é* o `<Button>` — não tem `button_class`.

Um nó sem classe própria da lib recebe `class="{alguma_class}"` puro. Isso não
custa nada quando ninguém injeta: o eval interpola a classe **antes** de decidir
buscar folhas, então um `class` que resolve para vazio devolve o nó ao caminho
barato, sem o `Vec` de folhas ativas nem as varreduras de regra.

O nome da prop segue o alvo, não o tipo: `field_class`, `item_class`,
`title_class`, `bar_class`, `head_class`. O sufixo é sempre `_class`.

### A armadilha: uma classe não substitui uma prop do widget

Injetar classe resolve **estilo**. O que o widget documenta como **prop**
continua prop — e trocar uma pela outra falha em silêncio, porque a precedência
está do lado do template:

```gv
<groupbox title="Rede" class="col_larga" />   <!-- NÃO: .col_larga { width: 440 } é ignorada -->
<groupbox title="Rede" width="440" />          <!-- sim -->
```

O template do `<GroupBox>` escreve `width="{width|fill}"` inline. Um default
inline **resolve sempre**, e um valor inline vence toda classe — inclusive a do
uso. A classe não pinta nada, e o `width` cai no default `fill`.

O estrago passa do cosmético quando o default é `fill`: dois filhos `fill`
dentro de uma `<row>` sem largura colapsam para zero, e a seção inteira some da
tela — sem erro, sem aviso, com a árvore avaliada intacta. Foi o que aconteceu
com a aba "Accordion + ToolBox" do `examples/onda4` ao migrar os estilos para o
`.gss`; `tests/engine_tests.rs` tem o teste que o pega agora.

As três props de geometria que hoje se comportam assim, e que portanto ficam no
markup: `width` do `<GroupBox>`/`<Frame>`/`<Card>`, `height` do `<ListView>`
(`{height|240}`) e `width` do `<SpinBox>` (`{width|72}`, que é o campo interno —
uma classe no uso pinta a `Row` de fora e deixa o campo em 72).

Numa **primitiva** o problema não existe: não há template no caminho, então
`<maskedinput class="campo">` com `.campo { width: 200 }` funciona.

## Grafia da tag: `<GroupBox/>` e `<groupbox/>`

Todo builtin é publicado sob **dois** nomes: o canônico e o mesmo em minúsculas
coladas. É a convenção que as primitivas já tinham (`<textinput/>`,
`<progressbar/>`, `<contextmenu/>`); o `snake_case` no motor é a convenção dos
apelidos em **português** (`entrada_texto`, `barra_progresso`).

Você não faz nada para isso: `register_builtins` registra a lista de
[`builtin_components`] e, em seguida, os apelidos que `builtin_aliases` deriva
dela. Vale saber **por que** o segundo registro é necessário, porque o mecanismo
difere do das primitivas: uma primitiva casa num `match` de tags que lista as
grafias à mão, enquanto um builtin resolve por `parsed_templates.get(name)` —
igualdade exata de string. Sem o alias, `<groupbox/>` seria `UnknownComponent`.

O alias é uma **instância própria** no mapa de componentes, e isso importa para
quem escreve um widget com comportamento: a ação carrega o nome pelo qual a tag
foi resolvida, então `<tabbar/>` produz `tabbar::pick:…` e é a instância do
alias que recebe esse `update`. Como todo builtin da lib é sem estado (o estado
mora em chaves que o app nomeia), as duas instâncias serem separadas não muda
nada — mas um builtin que **guardasse** estado em `self` teria dois estados
independentes, um por grafia. É mais uma razão para não guardar.

## Espaço de nomes e override

Builtins compartilham o espaço de nomes dos componentes do app. Para não
"sequestrar" um nome, **uma definição explícita do app vence o builtin** de mesmo
nome:

- `register(Box::new(MeuBadge))` ou `register_component("Badge", …)` inserem por
  cima.
- `<import name="Badge" from="…"/>` e `<link rel="import" href="…" as="Badge"/>`
  também sobrescrevem (os dois guardas abrem exceção para nomes que ainda são
  builtin). O caminho do `<link>` só ganhou essa exceção na 0.66 — até então ele
  era engolido em silêncio, o que ninguém notava enquanto nenhum builtin tinha
  um nome que um app fosse querer.

As duas grafias entram no conjunto de nomes builtin, então a regra vale igual
para `Badge` e `badge` — mas **uma de cada vez**: registrar `Badge` não desativa
`badge`. Um app que queira mesmo cobrir o nome inteiro registra as duas.

Ou seja: escolha nomes bons, mas saiba que o app sempre pode substituir. Evite
colidir com **primitivas** (`Button`, `Text`, `Column`, `Row`, `Container`,
`Image`, `Svg`, `Checkbox`, `Toggle`, `Select`, `Form`, `Rule`, …) — essas são
resolvidas antes e não são sobrescrevíveis por um componente.

## Restrição importante: contexto global único

O estado escrito com `ctx.set` (em `update` ou `init`) vai para **um** contexto
global — não há estado por instância. Consequências práticas:

- ✅ **Widgets apresentacionais / prop-driven** (Badge, Divider): recebem
  tudo por prop, não guardam estado. Podem ser usados N vezes na mesma tela sem
  colisão.
- ✅ **Recipientes** (GroupBox, Frame, Card, ToolBar, StatusBar): o que eles
  mostram nem é deles — é o conteúdo que quem usa escreve entre as tags, via
  `<slot/>` (seção adiante). Sem estado nenhum, usáveis N vezes.
- ✅ **Widgets com comportamento cujo valor o app nomeia** (SpinBox): a chave de
  contexto entra por prop e a ação a carrega — ver a seção adiante. Também são
  usáveis N vezes.
- ✅ **Widgets que ESCOLHEM conteúdo por uma chave nomeada** (`Tabs`, 0.92): a
  aba ativa é uma chave do app, e o nome do slot **interpola** contra ela
  (`<slot name="{active}"/>`). Duas instâncias com chaves diferentes não se
  veem. Vale saber o preço: o conteúdo de **todos** os slots é avaliado, porque
  a partição acontece uma vez na fronteira do componente; só o escolhido é
  renderizado.
- ⚠️ **Widgets com estado** (um contador): duas instâncias na mesma tela
  **compartilhariam** o estado e colidiriam. Não há isolamento por instância
  ainda — e a lista de widgets que de fato precisam disso encolheu para quatro
  (ver a §3 do `PLANO_WIDGETS.md`).
- ❌ **Não** semeie defaults com `init()` num builtin: isso polui o contexto
  global com as chaves do widget. Use `{prop|default}` no template.

Enquanto o motor não tiver estado por instância, um builtin pode ter
comportamento — mas todo valor que ele guarda tem de morar numa chave nomeada
por quem o usa.

## Widget com comportamento: a chave vem por prop, os parâmetros vêm na ação

A restrição acima diz que builtin não pode guardar estado. Ela **não** diz que
builtin não pode ter comportamento — e a diferença é o `SpinBox`
(`src/builtins/spin_box.rs`), que soma, subtrai e satura em Rust.

O truque tem duas partes:

1. **O valor mora numa chave que o app nomeia.** O widget não inventa chave
   nenhuma: recebe o nome dela por prop (`<SpinBox value="qtd"/>`). Como as
   chaves são do app, duas instâncias são independentes — o contexto continua
   global, mas ninguém disputa a mesma posição.

2. **A ação carrega os parâmetros da instância.** O `update` recebe só
   `(action, value, ctx)` — ele **não** enxerga as props de quem o disparou.
   Então o template escreve os parâmetros dentro da própria ação:

   ```xml
   <Button text="▲" on_click="inc:{value}|{min|0}|{max|100}|{step|1}" />
   ```

   O eval interpola (`inc:qtd|1|99|1`) e prefixa o dono
   (`namespace_action` → `SpinBox::inc:qtd|1|99|1`). O motor quebra no `::`,
   encontra `SpinBox` no mapa de componentes — builtin entra lá igual a
   componente de app — e chama o `update` **do widget**, não o da tela.

   ```rust
   fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
       let (op, payload) = action.split_once(':')?;      // "inc", "qtd|1|99|1"
       let mut campos = payload.split('|');
       let chave = campos.next()?;                        // a chave do app
       // …lê ctx.get(chave), calcula, ctx.set(chave, novo)
   }
   ```

   Cuidado com o `|`: **dentro** de `{…}` ele separa o default inline
   (`{min|0}`), **fora** é literal e separa os campos do payload. E não comece a
   ação com `clipboard:`/`open:`/`window:`/`style:` — esses prefixos são globais
   e escapam do namespacing (`BUILTIN_ACTION_PREFIXES`).

## Widget que delega: `app:` na frente da ação

O padrão acima é para o widget que **age**. O oposto é o widget que **delega** —
o `<ToolButton/>` recebe `on_click` por prop e só repassa ao botão interno.

A armadilha: o `namespace_action` prefixa **toda** ação com o dono, então
`on_click="{on_click}"` vira `ToolButton::salvar`, o motor acha o `ToolButton`
no mapa de componentes e chama o `update` **dele** — que não conhece ação nenhuma
do app. O handler nunca roda e não há erro: o botão só não faz nada.

O escape é o prefixo `app:`, que sai no lugar do prefixo de dono:

```xml
<Button text="{text}" on_click="app:{on_click}" />
<TextInput value="{value}" onChange="app:{on_change}" />
```

`app:` significa **a tela atual** (é onde o `dispatch` cai sem dono), não o
componente intermediário que porventura tenha usado o widget — delegar de
componente para componente ainda depende de um `ctx.dispatch` que o motor não
tem. Do lado Rust a limitação continua: um `update` não consegue despachar outra
ação, então um builtin que trata o `on_change` para si (como o `SpinBox`) não
tem como *também* repassá-lo — um `<TextInput>` só tem um `onChange`.

## Widget que embrulha conteúdo: o `<slot/>`

As duas seções acima são sobre widgets que recebem **valores**. Um recipiente
recebe outra coisa: **markup**. `<GroupBox>` não tem prop nenhuma que descreva o
que ele mostra — o que ele mostra é o que se escreve entre as tags dele.

```xml
<GroupBox title="Rede">
    <Checkbox label="Usar proxy" checked="proxy" />
    <Button text="Salvar" on_click="salvar" />
</GroupBox>
```

Escreva `<slot/>` no template do builtin onde esse conteúdo deve entrar:

```xml
<Container class="groupbox-frame" padding="{padding|12}">
    <Column spacing="{spacing|8}">
        <slot/>
    </Column>
</Container>
```

Até a 0.64 isso não existia: `NodeType::Component` carregava só props e os
filhos do uso eram **descartados** na expansão, o que mantinha toda a família
dos recipientes fora do nível Builtin (era o item 2 dos habilitadores de motor
do `PLANO_WIDGETS.md`).

### A regra que importa: o conteúdo é de quem o escreveu

O conteúdo do slot é avaliado no **contexto** e com o **dono** de quem usou o
widget — antes de qualquer prop entrar em cena. Consequências práticas:

- `on_click="salvar"` lá em cima chega no `update` da **tela**, não no do
  `GroupBox`. **Não** se escreve `app:` no conteúdo — esse prefixo é para o
  outro caso, o da ação recebida por prop (seção anterior).
- `{host}` dentro do conteúdo lê o contexto da tela, e **não** enxerga as props
  do recipiente (`{title}` ali dentro não resolve para o título do GroupBox).
- Um recipiente dentro de outro funciona sem cerimônia: cada `<slot/>` recebe o
  conteúdo do **seu** uso.

### Conteúdo de reserva

Os filhos escritos dentro do próprio `<slot>` aparecem quando quem usou não
escreveu nada. Esses são do componente — avaliam no contexto dele e **enxergam**
as props da instância:

```xml
<slot><Text content="Nada em {title}" /></slot>
```

### Mais de um buraco: o slot nomeado

Um widget com duas regiões distintas — corpo e rodapé, título e ações —
declara um `<slot name="…"/>` por região, e quem usa etiqueta o conteúdo com o
atributo `slot`:

```xml
<!-- no template do builtin -->
<column>
    <slot/>                       <!-- o anônimo: tudo que não foi etiquetado -->
    <rule />
    <row><slot name="footer"/></row>
</column>
```
```xml
<!-- no uso -->
<card title="Servidor">
    <text content="uptime 31 dias" />
    <template slot="footer">
        <button text="Reiniciar" on_click="reiniciar" />
    </template>
</card>
```

`<template slot="…">` agrupa vários nós; para um nó só, o atributo direto
(`<button slot="footer" …/>`) evita o embrulho. Vários blocos com o mesmo nome
se concatenam na ordem em que foram escritos, e o conteúdo anônimo preserva a
ordem de documento mesmo quando um bloco nomeado é escrito no meio dele.

### Decorar um slot opcional: `{slot_<nome>}`

Um rodapé quer uma linha divisória **acima dele**, e só quando existe rodapé.
O template não consegue perguntar isso sozinho: o nome do slot não é uma prop, e
o conteúdo nem chega ao interpolador. Por isso o motor semeia, na fronteira do
componente, um marcador por slot nomeado preenchido:

```xml
<template if="{slot_footer|false}" equals="true">
    <rule />
    <row><slot name="footer"/></row>
</template>
```

Ele entra na camada **depois** das props, então uma prop escrita à mão com o
mesmo nome vence. Não existe marcador para o slot anônimo.

### Limites

- **Nome fixo, resolvido no template.** `<slot name="{aba}"/>` — nome vindo do
  contexto — ainda não existe. É o que separa o `<tabbar>` de hoje de um
  `QTabWidget` inteiro, cuja página visível depende do valor de uma chave.

  A saída, quando o conteúdo varia por seção, é **uma tag por seção** em vez de
  uma coleção: `<accordion>` + `<accordionitem>`, `<toolbox>` + `<toolboxitem>`
  (0.85). Cada item tem o seu `<slot/>` anônimo, e o conteúdo continua sendo de
  quem escreveu a tela. É a mesma forma que o Qt usa —
  `QToolBox::addItem(widget, "Título")` também recebe uma seção por chamada.
- **Um uso com conteúdo não entra no cache de componente.** As dependências do
  conteúdo pertencem ao quadro de quem chamou, e uma entrada de cache não teria
  como perceber que ele mudou. Custo desprezível — são os containers da tela —,
  mesma exceção que uma lista reordenável já tinha.

## Testando

Um teste de integração exercita o caminho completo (parse → builtin
auto-registrado → árvore avaliada) sem GUI. Registre **só** uma tela que usa a
tag e verifique a árvore em `motor.evaluated_templates`:

```rust
#[test]
fn test_divider_disponivel_sem_registro() {
    let mut motor = GlacierUI::new();
    std::fs::create_dir_all("templates").ok();
    let tela = "templates/test_divider.gv";
    std::fs::write(tela, r##"<Column><Divider divider_color="#f00" /></Column>"##).unwrap();

    motor.register_component("tela", tela).unwrap(); // NÃO registra Divider

    let ev = motor.evaluated_templates.get("tela").unwrap();
    assert_eq!(ev.children[0].background.as_deref(), Some("#f00"));
    // default inline preservado onde a prop foi omitida:
    // assert_eq!(ev.children[0].height.as_deref(), Some("1"));

    std::fs::remove_file(tela).ok();
}
```

Veja `tests/engine_tests.rs`:
- `test_builtin_badge_disponivel_sem_registro` — disponibilidade sem registro + defaults + não-poluição do contexto.
- `test_slot_conteudo_do_uso_pertence_a_quem_escreveu` — a garantia central do `<slot/>`.
- `test_slot_reserva_quando_o_uso_nao_passa_nada` — o conteúdo de reserva.
- `test_builtins_onda2_disponiveis_sem_registro` — os quatro recipientes embrulhando conteúdo.
- `test_builtin_tabbar_escreve_a_chave_do_app` — a barra de abas pelo padrão do `SpinBox`.
- `test_atributo_numerico_templado` — prop num atributo numérico.
- `test_template_default_inline` — a sintaxe `{prop|default}`.

## Checklist

- [ ] `name()` único, sem colidir com primitivas.
- [ ] `Template::Inline` (nunca `File`).
- [ ] Props prefixadas (`widget_*`) com default inline `{prop|default}`.
- [ ] Sem `init()` semeando contexto global; sem estado se for usável N vezes.
- [ ] Adicionado a `builtin_components()`.
- [ ] Docstring no `struct` listando as props e seus defaults.
- [ ] Teste de disponibilidade-sem-registro em `tests/engine_tests.rs`.
- [ ] Se embrulha conteúdo: `<slot/>` no template, e um teste de que a ação de
      dentro chega na tela sem o prefixo do dono.

## Referência: o `Badge`

`Badge` é o exemplo canônico — uma "pílula" de rótulo, puramente apresentacional,
com props string e numérica, todas com default inline. Veja o código-fonte em
`src/builtins/badge.rs` e o exemplo executável em `examples/builtins/` (`cargo run
--example builtins`).
