# AGENTS.md

Instruções para quem trabalha neste projeto — pessoa ou agente. O que está aqui
foi medido, não suposto: cada número veio de um app real numa GPU integrada de
2012 (Intel HD 2500), que é onde as diferenças aparecem. Numa máquina moderna
boa parte disto some — o que não muda é a ordem de grandeza entre as causas.

## O que custa num quadro, em ordem

Antes de otimizar qualquer coisa, saiba onde o tempo vai. Numa tela de **111 nós
com 20 caixas pintadas**, janela de 900×720:

| | por quadro |
|---|---|
| Pintar as caixas | **~45 ms** |
| Layout, texto e o resto do `iced` | ~50 ms |
| O motor glacier montar a árvore | **0,07 ms** |

O motor é seiscentas vezes mais barato que a pintura. **Quase nunca é ele.** Uma
tela de 300 nós sem fundo nenhum roda mais rápido que uma de 111 nós pintada.

## As três regras de estilo que mais pagam

Toda caixa com fundo, borda ou canto arredondado vira um retângulo que a GPU
sombreia **por pixel**. O custo é da **área**, não da quantidade — e as camadas
se somam onde se sobrepõem.

### 1. Não pinte a janela inteira

O tema já pinta o fundo. Um `background` de mesma cor num nó `width: fill;
height: fill` é uma camada redobrada em **cada pixel da tela**, invisível e cara.

```gss
/* não */
.tela    { width: fill; height: fill; background: var(--bg); }
.conteudo { width: fill; height: fill; background: var(--bg); }

/* sim — a cor vive no theme.json, e o resto herda */
.tela    { width: fill; height: fill; }
.conteudo { width: fill; height: fill; }
```

Num app real isto apareceu **nove vezes**, incluindo duas camadas empilhadas no
mesmo arquivo. Removê-las não mudou um pixel na tela e foi o maior ganho isolado.

Se o fundo precisar diferir do tema, **mude o tema** — não pinte por cima.

### 2. Menos camadas sobrepostas

Um `groupbox` dentro de um `frame` dentro de um container pintado são três
passadas na mesma área. Escolha uma. Antes de acrescentar um fundo, pergunte o
que já está pintado ali embaixo.

### 3. Canto arredondado só nas caixas pequenas

Arredondar exige matemática de distância em cada pixel do retângulo — inclusive
no meio dele, longe dos cantos. Numa caixa que ocupa a tela, isso é caro e o
detalhe de 7px quase não se nota; num crachá ou botão, é barato e faz a
aparência. Guarde o `border-radius` para as caixas pequenas.

## Como medir, em vez de adivinhar

Três variáveis de ambiente, todas sem custo quando desligadas:

```sh
GLACIER_PERF=1 ./app                      # relatório por segundo
GLACIER_PERF=1 GLACIER_PERF_STRESS=1 ./app  # mede CAPACIDADE, não demanda
GLACIER_PERF=1 GLACIER_PERF_STRESS=1 GLACIER_NO_PAINT=1 ./app  # sem pintura
```

O relatório reparte o quadro em quatro:

```
render 0.43 méd | dispatch 1.20/quadro (14 msgs) | app 0.05 | resto 15.6ms (90.7%)
```

| Parcela grande | Onde mexer |
|---|---|
| `render` | o motor monta nós demais → `virtualize` numa coluna dentro de `<scrollable>` |
| `dispatch` | tratamento de mensagem: `update`, Luau, reavaliação |
| `app` | seus ganchos (`on_message`) — um lock disputado aqui trava a UI |
| `resto`, com árvore pequena | `iced`/GPU: layout, texto, **pintura** |

**Duas armadilhas de leitura**, ambas já custaram diagnósticos errados:

- **Sempre use `GLACIER_PERF_STRESS` para julgar velocidade.** Sem ele, um app
  orientado a evento fica ocioso entre eventos e o `intervalo` medido é a
  espera, não o custo. Um app parado já apareceu como "quadro de 19 segundos".
- **Compare com pintura e sem.** Se `NO_PAINT` acelerar muito, o gargalo é
  rasterização e a saída são as três regras acima — não otimizar código.

O procedimento que resolve em dois minutos: rode com `STRESS`, anote o
`intervalo méd`; rode de novo com `NO_PAINT` junto; compare.

## Listas longas

Uma lista que não cabe na tela entrega ao `iced` itens que ninguém vê, e ele
mede e desenha todos. `virtualize` monta só os visíveis:

```xml
<scrollable height="fill">
  <column spacing="12" virtualize="300">   <!-- 300 = altura de CADA item -->
    <ForEach items="servicos" var="s"> … </ForEach>
  </column>
</scrollable>
```

A altura é **declarada**, não medida (medir exigiria o layout, que é o trabalho
a evitar). A coluna precisa ser filha direta do `<scrollable>`. Errar a altura
desalinha a barra de rolagem, não quebra a tela.

Só vale para listas que **não cabem** na tela: com poucos itens ela não age, de
propósito.

## Convenções deste projeto

- **Templates são `.gv`**; folhas de estilo, `.gss`. Não existe `.kdl`, `.iss`
  nem `.rss` — se vir menção a esses, é documentação velha.
- **Scripts são Luau**, em `views/scripts/`. Os tipos estão em `glacier.d.luau`;
  um nome listado em `globals` no `.luaurc` vira `any` e anula o tipo.
- **Um valor que o app nomeia não precisa de estado por instância.** A chave
  entra por prop e a ação carrega a chave — é como `SpinBox`, `TabBar` e
  `Pagination` funcionam sem colidir entre instâncias.
