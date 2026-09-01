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
| **Componente do app** | código/arquivos do app | sim (`register`/`import`) | `<PerfilCard/>` |

Este documento cobre só o nível **Builtin**. Para o passo a passo de uma
**Primitiva** nova — inclusive uma armadilha real do motor (`Length::Fill` vs.
o wrap genérico de background/borda) — ver [`PRIMITIVAS.md`](PRIMITIVAS.md).

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

1. Valor templado (`{…}` já resolvido)
2. Literal inline no atributo (`padding="4 10"`)
3. Valor herdado de uma classe `.gss` (`class="..."`)

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
- ⚠️ **Widgets com estado** (um contador, um accordion aberto/fechado): duas
  instâncias na mesma tela **compartilhariam** o estado e colidiriam. Não há
  isolamento por instância ainda.
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
o `<TimePicker/>` recebe `on_pick` por prop e só repassa ao botão interno.

A armadilha: o `namespace_action` prefixa **toda** ação com o dono, então
`on_click="{on_pick}"` vira `TimePicker::abrir_modal`, o motor acha o
`TimePicker` no mapa de componentes e chama o `update` **dele** — que não conhece
ação nenhuma do app. O handler nunca roda e não há erro: o botão só não faz nada.

O escape é o prefixo `app:`, que sai no lugar do prefixo de dono:

```xml
<Button text="{pick_icon|⏰}" on_click="app:{on_pick}" />
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

### Limites

- **Um slot, sem nome.** `<slot name="footer"/>` não existe, então um widget com
  dois buracos (cartão com corpo e rodapé, `QTabWidget` com uma página por aba)
  ainda não é construtível como builtin. É o degrau seguinte.
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
