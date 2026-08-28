# Plano: um cabeçalho para o `.gv` (metadados de janela + recursos)

> **Status: implementado** em 2026-08-28, na 0.59.0. Este documento ficou como o
> registro de *por que* a sintaxe é essa — as três opções que estavam em jogo
> (seção 4) e as regras de convivência que foram escolhidas (seção 6).
>
> **O que foi decidido:** o **desenho B** (raiz que envolve tudo, com bloco de
> recursos), com as tags chamadas **`<screen>`** e **`<resources>`** — e os
> apelidos `<tela>`/`<recursos>`, como o resto do vocabulário de tags. O tamanho
> ficou em `size="960 700"`, e o template ganha do builder Rust.
>
> Onde está a documentação de uso: `README.md`, seção "Cabeçalho da tela".

---

## 1. O incômodo

Um `.gv` hoje é uma lista de coisas soltas no mesmo nível. Este é o exemplo
`examples/controle_externo/controle_externo.gv` depois de mover o estilo para um
bloco `<style>`:

```xml
<style scoped="true">
    .tela { padding: 28; width: fill; height: fill; background: #0D1117; }
    …
</style>

<container class="tela">
    …
</container>

<script src="controle_externo.luau"></script>
```

Duas coisas incomodam nisso:

**(a) O `<style>` fica lado a lado com os widgets.** Ele não é um widget, não
desenha nada, não tem posição na tela — mas ocupa o mesmo nível de indentação
que o `<container>` que de fato aparece na janela. O mesmo vale para
`<link rel="theme">`, `<import>` e `<script>`. Quem abre o arquivo não tem uma
fronteira visual entre "o que a tela precisa para existir" e "o que a tela
mostra"; precisa ler tag por tag para descobrir.

**(b) O arquivo não fala da própria janela.** Título e tamanho da janela só
existem em Rust, no builder:

```rust
GlacierDaemon::new()
    .title("Glacier - Controle externo")
    .main_size(1000.0, 700.0)
```

Então o arquivo que descreve a tela não sabe como se chama nem de que tamanho
nasce. Para mudar o título de uma tela você sai do `.gv`, abre o `main.rs`,
recompila — e perde o hot-reload no caminho. Pior: se o app tem várias telas na
mesma janela (`navigate_to`), só existe **um** título para todas, porque ele foi
decidido uma vez no boot.

Isso já aparece de outra forma nas janelas-filhas: quem abre uma janela pelo Luau
tem que repetir os metadados no ponto da chamada, longe do arquivo que eles
descrevem:

```lua
open_window({ file = "telas/detalhe.gv", title = "Detalhe", width = 400, height = 300 })
```

O título e o tamanho de `detalhe.gv` estão escritos em *quem chama*, não em
`detalhe.gv`. Se três telas abrem essa mesma janela, a informação está triplicada.

---

## 2. Como funciona hoje, por baixo

Vale entender o mecanismo atual porque a proposta encaixa nele em vez de
substituí-lo.

Quando o parser lê um `.gv` (`UiNode::parse_xml_with_source`, em
`src/parser.rs:1559`), ele envolve o arquivo inteiro numa raiz sintética
invisível — é isso que permite ter várias tags no topo sem um elemento raiz de
verdade. Depois ele separa os filhos dessa raiz em dois grupos:

- **declarações** — `<import>`, `<link>` e `<style>`;
- **layout** — todo o resto.

O layout vira a raiz (ou um `Fragment`, se houver mais de um), e as declarações
são penduradas como filhas dela. Na hora de avaliar (`src/eval.rs`) elas são
descartadas, então não desenham nada; quem as consome é o motor, em
`load_imports` e `process_links` (`src/lib.rs`).

O `<script>` nem chega ao XML: `eval::strip_script` o recorta do texto antes do
parse, preservando a contagem de linhas.

Ou seja: **o conceito de "declaração que viaja junto com o template mas não
desenha" já existe e já funciona.** O que falta é (a) um lugar visível no arquivo
para agrupá-las e (b) uma declaração nova que descreva a janela.

Do lado da janela, o caminho também já está aberto. No boot do daemon
(`src/daemon.rs:437`) a ordem é:

```rust
setup(&mut engine);                        // registra componentes, define a tela inicial
let (id, open) = window::open(main_settings.clone());   // só então a janela abre
```

O `setup` roda **antes** de a janela existir. Quer dizer que, nesse ponto, o
motor já sabe qual é a tela inicial e já tem o template dela parseado — dá para
perguntar a ele "essa tela pede algum título/tamanho?" e ajustar
`main_settings`/`main_title` antes do `window::open`. Sem gambiarra, sem abrir a
janela e redimensionar depois (que causaria aquele "pulo" visível).

---

## 3. O que queremos poder escrever

Metas, em ordem de importância:

1. As declarações (estilo, script, tema, imports) ficam num lugar só,
   visualmente separadas do layout.
2. O `.gv` pode declarar título e tamanho da própria janela.
3. Cada tela pode ter seu próprio título, e ele acompanha a navegação.
4. Uma janela aberta por `open_window({ file = ... })` herda os metadados do
   arquivo, sem precisar repeti-los na chamada.
5. **Nada disso quebra os 56 arquivos `.gv` que já existem** (35 no glacier-ui,
   21 no rustploy). O formato de hoje continua válido para sempre; o cabeçalho é
   opcional.

---

## 4. Três desenhos de sintaxe

Os três resolvem o mesmo problema. A diferença é quanto aninhamento custam e
quão explícita fica a fronteira. Os nomes das tags abaixo são provisórios — a
seção 8 trata disso à parte.

### Desenho A — cabeçalho irmão do layout

O bloco no topo carrega os metadados e engole as declarações. O layout continua
na raiz, com a indentação que tem hoje.

```xml
<screen title="Controle externo" size="960 700">
    <style scoped="true">
        .tela { padding: 28; background: #0D1117; }
    </style>
    <script src="controle_externo.luau" />
</screen>

<container class="tela">
    <column class="conteudo">
        <text class="titulo" content="Controle externo" />
    </column>
</container>
```

**A favor:** zero indentação extra; a migração de um arquivo existente é recortar
e colar o `<style>` para dentro do bloco; é a menor mudança possível no parser
(mais uma declaração, do mesmo tipo das que já existem).

**Contra:** o nome da tag mente um pouco. Um `<screen>` que *não* contém a tela é
estranho de ler. Isso se resolve escolhendo um nome que não prometa conteúdo
(`<setup>`, `<manifest>`, `<ficha>`), mas aí some a semelhança com o WPF.

### Desenho B — raiz que envolve tudo, com bloco de recursos (o mais WPF)

```xml
<screen title="Controle externo" size="960 700">
    <resources>
        <style scoped="true">
            .tela { padding: 28; background: #0D1117; }
        </style>
        <script src="controle_externo.luau" />
    </resources>

    <container class="tela">
        <column class="conteudo">
            <text class="titulo" content="Controle externo" />
        </column>
    </container>
</screen>
```

É o `<Window Title="…"><Window.Resources>…</Window.Resources>…</Window>` do WPF,
sem o ponto no nome. A raiz é a janela; dentro dela, primeiro o que a tela
precisa, depois o que a tela é.

**A favor:** um único elemento raiz, como todo XML "normal"; a fronteira é
explícita; os metadados ficam onde a intuição de quem vem de WPF/XAML procura.

**Contra:** todo o layout ganha um nível de indentação. Migrar um arquivo
existente é reindentar o arquivo inteiro (o `git diff` fica barulhento, ainda que
o resultado seja melhor).

### Desenho C — raiz com as duas seções nomeadas

```xml
<screen title="Controle externo" size="960 700">
    <resources>
        <style scoped="true"> … </style>
        <script src="controle_externo.luau" />
    </resources>

    <view>
        <container class="tela"> … </container>
    </view>
</screen>
```

**A favor:** simetria total, nenhuma ambiguidade sobre o que é o quê, e um lugar
óbvio para colocar coisas futuras (transições de tela, por exemplo).

**Contra:** dois níveis de indentação a mais no layout e uma tag a mais para
digitar em todo arquivo, sem informação nova — o `<view>` não diz nada que "não
ser uma declaração" já não dissesse.

### Recomendação

**Desenho B.** É o que você descreveu (WPF com resources), paga um preço de
indentação só uma vez por arquivo, e não precisa inventar um nome que disfarce
uma tag mentirosa como o A precisa. O C cobra uma segunda tag para comprar
simetria que o B já entrega na prática, já que "tudo que não é declaração é
layout" é a regra que o parser **já** aplica hoje.

Se a indentação extra incomodar mais do que o `<style>` solto, o A é uma escolha
legítima — e os dois podem coexistir depois, porque o parser distingue os casos
sem ambiguidade (um `<screen>` com filhos de layout é o B; sem eles, é o A).

---

## 5. O que vai dentro do cabeçalho

Atributos da tag de tela, primeira leva:

| Atributo | Exemplo | Efeito |
|---|---|---|
| `title` | `title="Detalhe do serviço"` | título da janela; acompanha a navegação |
| `size` | `size="960 700"` | tamanho inicial, largura e altura |
| `min-size` | `min-size="640 480"` | tamanho mínimo |
| `resizable` | `resizable="false"` | se a janela pode ser redimensionada |

`size="960 700"` segue o formato que o `padding` já usa (números separados por
espaço) em vez de `width`/`height` separados, justamente porque `width`/`height`
já querem dizer outra coisa no vocabulário do glacier (`fill`, `shrink`,
`fill 2`). Dois significados para o mesmo nome de atributo, em tags diferentes,
é o tipo de coisa que custa caro depois.

Ficam de fora nesta primeira leva (dá para acrescentar sem quebrar nada):
`icon`, `decorations`, `position`, `theme`.

Dentro do bloco de recursos entram as declarações que já existem: `<style>`,
`<link>`, `<import>`, `<script>`.

---

## 6. As regras de convivência

Estas são as decisões que evitam surpresa em uso real:

**O formato antigo continua válido.** Um `.gv` sem cabeçalho funciona
exatamente como hoje, para sempre. Nenhum dos 56 arquivos existentes precisa
mudar; migrar é opcional e arquivo a arquivo.

**Quem ganha, o template ou o Rust?** Proposta: **o template ganha.** O builder
(`.title()`, `.main_size()`) passa a ser o valor usado quando o `.gv` não diz
nada. O raciocínio: hoje nenhum arquivo tem cabeçalho, então nada muda de
comportamento; a partir do momento em que alguém escreve `title=` no `.gv`, foi
uma decisão explícita e recente, e é o arquivo que a pessoa tem aberto na frente
e recarrega a quente.

**Título muda ao navegar; tamanho não.** Se a janela navega de `lista.gv` para
`detalhe.gv`, o título passa a ser o de `detalhe.gv`. O tamanho, não — quem
decide o tamanho é quem abre a janela, uma vez. Uma tela que redimensiona a
janela por baixo do usuário no meio do uso seria hostil.

**Hot-reload: o título recarrega, o tamanho só se você mudou o número.** Salvar o
arquivo atualiza o título na hora. O tamanho só é reaplicado se o valor escrito
no arquivo mudou desde o último parse — senão, cada `Ctrl+S` desfaria o
redimensionamento que você acabou de fazer com o mouse.

**Componente importado ignora os metadados de janela.** Um `.gv` importado por
outro (`<import>`) é um pedaço de tela, não uma janela; `title`/`size` ali não
têm a quem se aplicar e são ignorados em silêncio. O mesmo arquivo aberto por
`open_window({ file = ... })` usa os metadados normalmente.

**A chamada explícita ainda ganha do arquivo.** `open_window({ file = "x.gv",
title = "Outro" })` continua mandando: quem abre a janela sabe do contexto que o
arquivo não sabe (ex.: "Editando *nginx*"). O cabeçalho é o padrão, não uma
camisa de força.

---

## 7. O que muda no código

Nada aqui é grande; o trabalho é distribuído.

**`src/parser.rs`** — reconhecer a tag de tela (e a de recursos, no desenho B/C)
e produzir uma declaração nova, `NodeType::Screen { title, size, min_size,
resizable }`, que viaja como filha da raiz igual a `Style`/`Link` fazem hoje. No
desenho B, os filhos de layout do `<screen>` viram a raiz real. A raiz sintética
(`FRAGMENT_OPEN`) continua existindo para o formato antigo.

**`src/eval.rs`** — descartar a declaração na avaliação, como já faz com
`Style`/`Link` (é uma linha no `matches!`). O `strip_script` não muda: ele acha o
`<script>` por texto, esteja ele aninhado ou não.

**`src/lib.rs`** — guardar os metadados por componente (`screen_meta:
HashMap<String, ScreenMeta>`) preenchidos em `register_one`, e expor
`GlacierUI::current_screen_meta()` (a tela ativa) e `screen_meta(name)`.

**`src/daemon.rs`** — três pontos:
- no `boot` (`:437`), depois de `setup(&mut engine)` e **antes** do
  `window::open`, sobrepor `main_title`/`main_settings.size` com o que a tela
  inicial declarou;
- em `open_window` (`:953`), usar os metadados do arquivo como fallback do
  `WindowSpec`, no lugar do `fallback_title` atual;
- no update, depois de uma navegação ou de um hot-reload, atualizar
  `self.titles` para o da tela ativa (o `Runtime::title` já lê desse mapa, então
  o iced pega sozinho).

**Testes** — parser (as duas formas produzem o mesmo layout; metadados lidos),
daemon (título da tela inicial vence o do builder; navegação troca o título), e
migrar um exemplo de verdade para o novo formato.

**Tooling e docs** — `editors/vscode-gv/syntaxes/glacier-view.tmLanguage.json` e
`editors/vscode-gv/references/glacier-view.md` (destaque das tags novas),
`README.md` (seção de sintaxe do template), `CHANGELOG.md`.

**Versão** — é uma feature aditiva: bump de minor no glacier-ui, publicação, e só
então (se quiser) migrar os `.gv` do rustploy, que hoje repetem título e tamanho
no Rust.

---

## 8. As decisões, e o que ficou de fora

Decidido: **desenho B**, tags `<screen>` e `<resources>` (com `<tela>`/`<recursos>`
de apelido), `size="960 700"`, template ganhando do builder.

Ficou de fora desta primeira leva, e cabe sem quebrar nada quando fizer falta:

- os atributos `icon`, `decorations`, `position` e `theme` no `<screen>`;
- `min-size`/`resizable` reaplicados no **hot-reload** (hoje só valem na abertura
  da janela; só `title` e `size` são reacertados a quente);
- um diagnóstico para um widget escrito por engano dentro do `<resources>` —
  hoje ele é tratado como declaração e simplesmente não desenha, em silêncio.

## 9. Onde isto ficou no código

| O quê | Onde |
|---|---|
| `ScreenMeta` e o parse do `<screen>`/`<resources>` | `src/parser.rs` (`ScreenMeta`, `NodeType::Screen`/`Resources`, achatamento em `parse_xml_with_source`) |
| descarte na avaliação | `src/eval.rs`, `src/widget.rs` |
| guarda dos metadados por componente | `src/lib.rs` (`record_screen_meta`, `screen_meta`, `current_screen_meta`, `current_screen_name`) |
| aplicação na janela | `src/daemon.rs` (`apply_screen_meta` no boot, `open_child`, `sync_window_meta` no `route`) |
| testes | `src/parser.rs::screen_tests`, `src/daemon.rs::tests` |
| documentação de uso | `README.md` ("Cabeçalho da tela"), `editors/vscode-gv/references/glacier-view.md` |
| exemplo migrado | `examples/controle_externo/controle_externo.gv` |
