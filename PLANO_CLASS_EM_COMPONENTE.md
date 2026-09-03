# Plano: `class` num componente para de mentir, e o `SpinBox` deixa de engolir props

> **Status: implementado na 0.69.0.**
>
> Origem: a adoção dos widgets da 0.63–0.68 no `rustploy` esbarrou nos dois
> itens abaixo ao trocar seis `<input>` de texto por `<spinbox>` numa aba de
> formulário. O primeiro item é um bug de verdade; o segundo é o que o uso
> concreto pediu.

---

## O problema, em duas camadas

### Camada 1 (o bug): `class` numa tag de componente não faz nada, e não avisa

```gv
<spinbox value="qtd" class="campo_num" />
```

```gss
.campo_num { background: #123456 }
```

Hoje isto **não pinta nada**. Não erra, não avisa, não aparece em log: a classe
é lida pelo parser (é atributo genérico de nó, `parser.rs`), viaja no mapa de
props do `NodeType::Component`, e depois ninguém a usa. O `background` da raiz
expandida sai `None`.

Isso vale para **todo** componente — os builtins da lib (`card`, `groupbox`,
`badge`, `tabbar`, `toolbar`…) e os do app. E é a pior forma de falhar: a mesma
família do seletor por vírgula no GSS e da auto-referência que dava `SIGABRT`
sem mensagem antes da 0.68. Quem escreve `class` num componente tem toda a razão
de esperar que funcione — é o que funciona em qualquer outra tag do motor.

O detalhe que torna isso irônico: o motor **já sabe** fazer isso. O seletor de
tag de componente (`spinbox { }`, minúsculo, item 12 do `PLANO_GSS_LIMITACOES`)
resolve o estilo **no escopo do uso** e o entrega à raiz avaliada do template
como `underlay`. A infraestrutura existe inteira; falta a classe entrar nela.

### Camada 2: um builtin não tem como repassar nada ao widget que ele monta

O `<SpinBox>` monta um `<TextInput>` por dentro e lhe passa quatro coisas:
`value`, `onChange`, `placeholder` e `width`. Nada mais atravessa. Então, para
quem usa:

- **não dá para estilizar o campo** — a classe do app não chega nele;
- **o campo fica fora do `<Form>`** — sem `form_control` ele não tem id de foco
  estável e **engole o Enter**: não submete nem avança.

A segunda é a que dói. **Correção sobre a primeira leitura deste plano:** eu
havia escrito "não entra na cadeia de tabulação", e isso está errado. A
travessia por Tab é um listener **global** do motor (`focus_next`, `lib.rs`),
que percorre todo widget focável sem olhar para `formControl` — o campo do
`SpinBox` já era alcançado por ela. O que falta é o **Enter**, e num formulário
de seis campos numéricos são seis buracos no fluxo de teclado.

---

## O que este plano decide

### Decisão 1 — `class` no uso aplica na raiz expandida, ACIMA das classes do template

A pergunta difícil não é "aplicar?", é "em que degrau?". A escada de
especificidade documentada hoje é:

```
underlay de tag-de-componente  <  tag builtin  <  classe  <  id  <  inline
```

A classe escrita no **uso** poderia entrar em dois lugares, e a escolha muda o
que o usuário consegue fazer:

| Opção | Efeito |
|---|---|
| Junto do `underlay` (embaixo de tudo) | Nunca quebra a aparência do builtin — e **nunca deixa o chamador mudar nada** que o template já tenha declarado por classe. Inútil para o caso comum ("deixe este card vermelho"). |
| **Acima das classes do template, abaixo dos atributos inline dele** ✅ | O chamador consegue redefinir o que o autor do componente deixou como *padrão* (uma classe), e não consegue passar por cima do que o autor cravou *explicitamente* (um atributo inline). |

Escolhemos a segunda. A regra em uma frase, e é a que vai para o README:

> **A classe escrita no uso de um componente vence as classes do template dele,
> e perde para os atributos inline do template.**

É a mesma intuição do CSS: classe do autor é default, `style=""` é decisão. E é
a que faz `<card class="destaque">` funcionar sem que o `card-surface` do
template precise sumir.

A escada final:

```
underlay de tag-de-componente  <  tag builtin  <  classe do template  <
classe do USO  <  id do template  <  inline do template
```

`id` continua acima da classe do uso: um `id` é mais específico que uma classe
em qualquer lugar do motor, e inverter isso aqui criaria uma exceção para
decorar.

**O que isto NÃO é:** não é a classe atravessando para dentro do template. Ela
aplica **só na raiz** expandida. Estilizar um nó específico lá dentro continua
sendo responsabilidade do componente — e é o que a Decisão 2 resolve para o
caso do `SpinBox`.

### Decisão 2 — o repasse ao campo interno é uma prop EXPLÍCITA, não `class`

Com a Decisão 1, `class` num `<spinbox>` passa a significar "estilize o widget
inteiro" (a `Row`: campo + degraus). Isso é o que o Qt faz com um `QSpinBox`, e
é o significado certo.

Só que o `rustploy` quer a outra coisa: estilizar **o campo de dentro**. As duas
são legítimas e são diferentes, então colapsá-las no mesmo nome seria criar a
ambiguidade que este plano existe para matar. Props novas, com nome que diz o
alvo:

```gv
<spinbox value="qtd" min="1" max="9"
         class="moldura"          <!-- a Row inteira -->
         field_class="campo_num"  <!-- só o <TextInput> de dentro -->
         form_control="qtd" />    <!-- só o <TextInput> de dentro -->
```

`form_control` não precisa de prefixo `field_`: só existe um nó focável dentro
do `SpinBox`, então não há ambiguidade a desfazer.

E vale repetir o que ele **não** é: não é o Tab. O motor tem um listener global
(`lib.rs`) que transforma Tab em `focus_next`, e ele já percorria o campo do
`SpinBox`. `form_control` liga o Enter.

---

## Por que isto é barato (medido, não estimado)

Um protótipo antes de escrever o plano confirmou as duas partes:

1. **O repasse do `SpinBox` é edição de template, zero motor.** `class` e
   `form_control` são atributos **genéricos de nó** (`parser.rs:1443` e `:1505`,
   lidos para qualquer tag, não por tag), e o parser encaminha *todo* atributo
   de uma tag desconhecida como prop. Então `class="{field_class}"` e
   `formControl="{form_control}"` no `<TextInput>` do template já bastam.

2. **A `<Form>` enxerga o campo que veio de dentro de um builtin.** Não era
   óbvio e era o risco real do item. A hidratação da `<Form>` roda sobre
   `children_eval` (`eval.rs:2145`) — ou seja, **depois** da expansão de
   componente. Um `form_control` que só passa a existir após a expansão é
   encontrado normalmente. Verificado: o campo recebeu `form_submit_action` e o
   `form_next_focus` apontando para o controle seguinte — que é para onde o
   Enter avança.

A Decisão 1 é um `overlay` simétrico ao `underlay` que já existe: mais um par de
parâmetros em `eval_owned`, resolvido no escopo do uso na expansão e mesclado
logo depois do `resolve_classes` da raiz.

---

## Implementação

### 1. `overlay` no `eval_owned` (`src/eval.rs`)

- Dois parâmetros novos, gêmeos dos de `underlay`: `overlay: Option<&StyleRule>`
  e `overlay_states: Option<&StateStyles>`.
- No bloco que monta `(style, state_styles)`: depois do `base.merge_from(&resolve_classes(...))`
  e do `states.merge_from(&resolve_state_classes(...))`, mesclar o overlay por
  cima. Assim ele vence a classe do template e continua perdendo para o inline,
  que é aplicado no `match` por campo mais abaixo.
- Os 6 call-sites que não expandem componente passam `None, None`.

### 2. Resolver a classe do uso na expansão (`src/eval.rs`)

No braço `if let Some((name, props)) = reference`, ao lado do bloco que já monta
`underlay_rule` a partir do seletor de tag:

- se `node.class` (ou `node.id`) existir, interpolar e resolver com
  `resolve_classes`/`resolve_state_classes` usando `styles.active(scope)` — o
  **escopo do uso**, não o do componente, porque a classe foi escrita lá;
- passar o resultado como `overlay` na chamada `eval_owned` que avalia o
  template.

**Cuidado com o cache.** O cache de componente é indexado pelo contexto; duas
instâncias do mesmo componente com classes diferentes têm de render nós
diferentes. A chave precisa incluir a classe/id do uso, ou o segundo uso
recebe o estilo do primeiro. É o mesmo tipo de armadilha que fez o uso *com
conteúdo de slot* ficar fora do cache na 0.65.

### 3. `SpinBox` (`src/builtins/spin_box.rs`)

Nas duas formas (`stacked` e `inline`), no `<TextInput>`:

```xml
class="{field_class}"
formControl="{form_control}"
```

Documentar as duas props novas na docstring, junto do porquê de `field_class`
não se chamar `class`.

### 4. Testes (`tests/engine_tests.rs`)

- `class` num componente do app pinta a raiz expandida;
- `class` num builtin (`spinbox`) idem;
- classe do uso **vence** classe do template, e **perde** para inline do template;
- duas instâncias com classes diferentes na mesma tela não se contaminam (o
  teste do cache);
- `SpinBox` com `field_class` + `form_control` dentro de uma `<Form>`: o campo
  recebe o estilo, o `form_submit_action` e o `form_next_focus`.

### 5. Extensão de VS Code (`editors/vscode-gv`)

- `class` deixa de ser "atributo de nó comum" e ganha nota na referência: num
  componente, aplica na raiz expandida.
- `<SpinBox>` ganha `field_class` e `form_control` na tabela de props.

---

## O que fica de fora, de propósito

- **`class` atravessando para dentro do template** (algo como `::part()` do CSS).
  Estilizar um nó interno arbitrário continua sendo decisão do componente, que
  expõe uma prop com nome próprio quando quiser — a Decisão 2. Um seletor que
  fura a fronteira do componente é uma porta muito maior, e nada hoje pede.
- **O mesmo repasse nos outros builtins.** Só o `SpinBox` tem um caso concreto
  (um campo focável, dentro de um formulário, com estilo do app). Quando o
  segundo aparecer, o padrão já estará estabelecido por ele.
- **`class` numa tag desconhecida que não é componente** continua sendo o erro
  de componente não registrado que já é.
