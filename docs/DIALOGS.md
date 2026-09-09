# Diálogos modais (`src/dialogs.rs`)

Módulo estilo `QMessageBox` do Qt: diálogos de informação, aviso, erro,
pergunta e confirmação, sobrepostos à tela ativa. Publicado a partir da
versão `0.4.9`; ganhou **corpo em markup** na `0.94` (Onda 8), que é o que o
transformou também num `QDialog`.

## Por que ele era diferente do resto do glacier-ui — e por que não é mais

Até a `0.93` este documento abria assim, e estava certo:

> O resto do framework é declarativo — a UI vem de um template XML, cacheada
> como árvore (`UiNode`) e reavaliada a cada mudança de contexto. Um diálogo não
> segue esse caminho: ele é transiente e construído inteiramente em Rust
> (`src/dialogs.rs`), **sem markup**.

Isso vale enquanto todo diálogo é uma **caixa de mensagem**: ícone, título,
texto, botões — e os cinco construtores de conveniência dão conta. Deixa de
valer no instante em que um diálogo precisa de um **campo**. O
`QInputDialog::getText` é um `QLineEdit` dentro de um cartão, e o motor já sabe
desenhar `<textinput>`, estilizá-lo pelo `.gss` e ligá-lo a uma chave; escrever
um segundo caminho de render, em Rust, para cada diálogo com conteúdo seria
escrever o motor duas vezes.

Na `0.94` o `DialogSpec` ganhou um `body`, e o diálogo deixou de ser uma tela
paralela para virar **uma moldura em volta de uma tela**. O que continua
verdadeiro é a outra metade: ele é transiente, é **singleton** (`dialog:
Option<DialogSpec>` no motor — um campo, não um mapa), e não faz parte de tela
nenhuma.

Esse singleton não é detalhe de implementação: é o que faz um diálogo com campo
**não precisar de estado por instância**. O que o usuário digita mora numa chave
de contexto comum, como em qualquer `<textinput>`, e não há como duas instâncias
do mesmo diálogo disputarem essa chave — não existe uma segunda.

## API

### `DialogSpec` — a especificação do diálogo

Campos: `icon` (`DialogIcon::Information|Warning|Error|Question|None`),
`title`, `message`, `detail: Option<String>`, `buttons: Vec<DialogButton>`,
`dismissible: bool` (clicar fora fecha sem despachar ação) e
`body: Option<String>` — o **nome de um template** montado entre a mensagem e os
botões (0.94; ver a seção seguinte). Um diálogo com corpo nasce mais largo (480
em vez de 380: 380 é a largura de uma frase, não de um formulário) e esconde a
linha da mensagem quando ela está vazia.

Construtores de conveniência, cada um já com os botões e o `dismissible`
corretos (o mesmo papel dos métodos estáticos de `QMessageBox`):

| Construtor | Ícone | Botões | `dismissible` |
|---|---|---|---|
| `DialogSpec::information(title, msg)` | Information | OK | `true` |
| `DialogSpec::warning(title, msg)` | Warning | OK | `true` |
| `DialogSpec::error(title, msg)` | Error | OK | `false` |
| `DialogSpec::question(title, msg)` | Question | No, Yes | `false` |
| `DialogSpec::confirm(title, msg)` | Question | Cancel, OK | `false` |

Builders: `.with_button(DialogButton)`, `.with_detail(texto)` (bloco de
detalhe destacado, tipo `QMessageBox::setDetailedText`), `.dismissible(bool)`.

### `DialogButton` — um botão

`label`, `action` (string roteada ao `Component::update` do dono da tela
quando clicado — mesma convenção de `on_click="..."` num `<Button>`), `role`
(`ButtonRole::Accept|Neutral|Destructive`, só controla a cor). Atalhos:
`ok`, `yes`, `no`, `cancel`, `save`, `discard`, `retry`, `close`.

### Disparando e fechando

De dentro de um `Component::update`:

```rust
fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
    match action {
        "excluir" => ctx.show_dialog(
            DialogSpec::confirm("Excluir projeto", "Essa ação não pode ser desfeita.")
                .with_detail("3 serviços e 2 deployments associados serão removidos.")
                .with_button(DialogButton::discard("excluir_confirmado")),
        ),
        "excluir_confirmado" => { /* ação já veio do botão do diálogo */ }
        _ => {}
    }
}
```

`Context::show_dialog`/`close_dialog` funcionam como `Context::navigate_to`/
`navigate_back`: só marcam a intenção; o motor aplica depois que `update()`
retorna (`route_to_owner`, em `lib.rs`). Fora de um `Component` (direto no
host app), o equivalente é `GlacierUI::show_dialog`/`close_dialog`.

### Renderização

`GlacierUI::render_current` já sobrepõe o diálogo ativo automaticamente —
nenhuma mudança no código do host app:

```rust
Ok(match &self.dialog {
    Some(spec) => {
        // O corpo em markup (0.94): o `DialogSpec` guarda o NOME de um
        // template, e quem o monta é o mesmo `render` de qualquer tela.
        let body = spec.body.as_deref().and_then(|n| self.render(n).ok());
        iced::widget::stack![screen, dialogs::overlay(spec, &self.theme(), body)].into()
    }
    None => screen,
})
```

Clicar num botão do diálogo despacha `EngineMessage::DialogButton(action)`;
o `dispatch()` fecha o diálogo e roteia `action` pro `update()` do
componente dono da tela ativa, exatamente como um `UiClick` comum.

## O corpo em markup (`0.94`)

### `<dialog name="…">` — o `QDialog`

Uma declaração do `<resources>`, ao lado do `<component name="…">` — e por um
bom motivo: é o mesmo objeto, um template registrado sob um nome. O que ele tem
a mais é a moldura.

```xml
<screen>
    <resources>
        <dialog name="editar_servico"
                title="Editar serviço"
                buttons="Cancelar::|Salvar:salvar_servico:accept">
            <Column spacing="10" width="fill">
                <TextInput value="__dialog.nome" placeholder="api-gateway" width="fill" />
                <SpinBox value="__dialog.replicas" min="1" max="32" />
            </Column>
        </dialog>
    </resources>

    <Button text="Editar" on_click="dialog:editar_servico" />
</screen>
```

Nada desenha onde a tag está escrita: a declaração viaja pendurada na raiz, como
`<screen>` e `<props>`. O que a faz aparecer é a ação `dialog:nome`.

**Atributos:** `name` (obrigatório), `title`, `message`, `icon`
(`information`/`warning`/`error`/`question`), `buttons` e `dismissible`.
Qualquer outro é erro posicionado, com a linha e a coluna.

**`buttons`** é uma lista compacta, `Rótulo:ação:papel` separados por `|`:

- o **rótulo** sai do primeiro `:`; o **papel**, quando é uma palavra-chave
  conhecida, do último `:`; o que sobra no meio é a ação **inteira, `:` e
  tudo** — é o que deixa `Voltar:dialog:editar:neutral` encadear para
  `dialog:editar` (ver `botao_de` em `src/dialogs.rs`);
- ação vazia (`Cancelar::`, `Cancelar:` ou `Cancelar`) → o botão só fecha, e
  **não despacha nada**;
- papel ausente → `neutral` se a ação for vazia, `accept` se não for;
- papéis: `accept`, `neutral`, `destructive` (e os equivalentes em pt-BR);
- sem o atributo → um `Fechar` que só fecha.

O botão que só fecha usa a sentinela `dialogs::DIALOG_CLOSE`, e ela existe para
que um "Cancelar" não despache uma ação fantasma — no dia em que o app escrevesse
um `update` chamado `cancelar`, ele passaria a disparar sozinho.

### A ação `dialog:`

A terceira família de prefixos de ação, depois do `app:` (0.63) e do `::` de
dono. `dialog:nome` abre; `dialog:close` (ou `dialog:fechar`) fecha. Vale em
qualquer rota que produza uma ação — `<button on_click>`, `<menuitem>`, e o botão
de outro diálogo, que é como um modal encadeia noutro.

Um nome que não existe é **ignorado em silêncio**: a alternativa seria derrubar o
app por um erro de digitação. O lugar de apontar isso é a validação de template,
que enxerga a linha.

### Do lado Rust

`DialogSpec::with_body(nome)` recebe o **nome** de um template já registrado, não
o markup — o motor já sabe montar um template por nome (`GlacierUI::render`), e
passar o nome mantém o `DialogSpec` `Clone` e barato de guardar.

```rust
ctx.set("__dialog.nome", "api-gateway".to_string());
ctx.show_dialog(
    DialogSpec::new(DialogIcon::None, "Editar serviço", "")
        .with_body("editar_servico")
        .with_button(DialogButton::new("Cancelar", DIALOG_CLOSE, ButtonRole::Neutral))
        .with_button(DialogButton::new("Salvar", "salvar_servico", ButtonRole::Accept)),
);
```

Os dois caminhos chegam ao mesmo lugar: o `<dialog>` do `.gv` registra o corpo
dele como um template comum, e o Rust o monta pelo nome.

### As chaves `__dialog.*`, e por que elas são apagadas

O corpo é avaliado no **contexto do app**, então as chaves que ele escreve são
chaves do app. É o que faz o diálogo com campo dispensar estado por instância — e
é também a única armadilha real da forma: dois diálogos que usem `nome` colidem,
mesmo sendo os dois singletons.

A convenção é o prefixo `__dialog.` (`dialogs::DIALOG_KEY_PREFIX`), e o motor
apaga **tudo** que começa com ele quando o diálogo fecha. Sem essa limpeza, a
segunda abertura do mesmo diálogo já viria preenchida com a resposta da primeira.

A limpeza roda **depois** de a resposta ter sido lida, nunca antes: quem lê é o
`DialogButton`, e ele lê da chave que ela apaga.

### O rascunho de um campo que valida

O `pick_color{}` usa **duas** chaves, e o padrão vale para qualquer campo cujo
texto em digitação não seja um valor válido:

- `__dialog.value` — o valor **cometido** (a cor, em `#rrggbb`);
- `__dialog.value__hex` — o **texto** que o campo mostra e edita.

O campo edita o rascunho; a cor só é cometida quando o texto vira uma cor
inteira (`color_picker::hex_completo`). Sem isso, a roda leria `#f`, `#ff`,
`#ff8` como branco e piscaria a cada tecla.

O caminho de volta é da própria roda: ela reescreve as **duas** chaves a cada
gesto, que é o que faz o campo segui-la.

## Os diálogos suspensivos da camada Luau

Quatro funções do prelúdio param a corrotina no diálogo e voltam com a resposta,
o que deixa o fluxo linear — a mesma aparência síncrona que o `fetch` tem.

| Função | Devolve | Corpo |
|---|---|---|
| `confirm{}` | `boolean` | nenhum (a caixa de mensagem de sempre) |
| `prompt{}` | `string` ou `nil` | `__InputDialog` |
| `pick_color{}` | `"#rrggbb"` ou `nil` | `__ColorDialog` |
| `progress{}` | **não suspende** | `__ProgressDialog` |

```lua
local nome = prompt({ title = "Renomear", label = "Novo nome", value = atual })
if nome then ctx.servico = nome end
```

O `nil` da desistência é a convenção que o `open_file()` já usava, e ela separa
"não respondeu" de "respondeu vazio" (que chega como `""`).

`prompt{}` aceita `kind`: `text` (default), `int`, `double`, `item` — as quatro
variantes estáticas do `QInputDialog`, que aqui são **um** diálogo com corpos
diferentes. `pick_color{}` entra pela mesma porta: é um `prompt` cujo campo é uma
roda.

O `progress{}` é o único que não suspende, porque ele acompanha um trabalho que
continua rodando — suspender a corrotina ali pararia justamente o que ele mostra.
Atualiza-se com `progress_set(valor, rotulo)` e fecha-se com `progress_close()`;
o cancelamento é uma **ação comum** (`on_cancel="…"`), não um mecanismo novo.

## Exemplo

O corpo em markup e os suspensivos da Onda 8: `cargo run --example onda8`
(o `<dialog>` declarativo, o wizard, a roda de cor) e
`cargo run --example onda8_luau` (`prompt`, `progress`, `pick_color`,
`confirm`).

## Bug encontrado e corrigido: hover e clique vazando pro que está atrás do modal

### Sintoma

Com o diálogo aberto, passar o mouse sobre um botão da tela por trás dele
mostrava o cursor de mãozinha (`Pointer`) do botão de baixo, como se o
diálogo nem estivesse ali.

### Causa raiz

O `overlay()` empilha duas camadas com `iced::widget::stack![backdrop,
centered]` — `backdrop` é o fundo escurecido que cobre a tela inteira
(`container(Space::new())` com `background: rgba(0,0,0,0.55)`), `centered`
é o cartão do diálogo.

`iced::widget::Stack` decide qual cursor mostrar chamando
`mouse_interaction()` de cada camada, **de cima pra baixo**, e usa a
primeira que devolver algo diferente de `Interaction::None`
(`iced_widget-0.14.2/src/stack.rs`):

```rust
self.children.iter().rev()
    .map(|child| child.mouse_interaction(...))
    .find(|&interaction| interaction != mouse::Interaction::None)
    .unwrap_or_default()
```

O `backdrop` era só um `mouse_area(container(Space::new()))` sem
`.interaction(...)` explícito. Sem esse método, `MouseArea::mouse_interaction`
delega pro conteúdo (`iced_widget-0.14.2/src/mouse_area.rs`):

```rust
match (self.interaction, content_interaction) {
    (Some(interaction), mouse::Interaction::None) if cursor.is_over(layout.bounds()) => interaction,
    _ => content_interaction,
}
```

`Space`/`Container` não têm opinião sobre cursor — devolvem
`Interaction::None`. Para o `Stack`, `None` **não** significa "cursor
padrão"; significa "não sei, pergunta pra camada de baixo". Resultado: ele
ignorava o backdrop inteiro e ia direto checar a tela por trás do modal,
achava o botão sob o cursor e usava o `Pointer` dele.

### Correção

Dar ao `backdrop` uma opinião própria e explícita, usando `Interaction::Idle`
— uma variante do enum `iced_core::mouse::Interaction` **distinta** de
`None`, que existe justamente para dizer "cursor padrão, mas de propósito"
(em vez de "não sei"):

```rust
let backdrop_area = mouse_area(backdrop)
    .interaction(iced::mouse::Interaction::Idle)
    .on_press(EngineMessage::DialogDismiss);
```

### Bug irmão descoberto no processo: clique (não só hover) vazando

Ao investigar o `mouse_interaction`, o `update()` do `MouseArea`
(`iced_widget-0.14.2/src/mouse_area.rs`) revelou um segundo problema, mais
sério: `shell.capture_event()` só é chamado se houver um handler de
`on_press` registrado. Diálogos não-dismissíveis (`error`, `question`,
`confirm`) não tinham `on_press` no backdrop antes da correção — então um
clique nele **não era capturado**: o evento vazava pro `Stack` continuar
procurando na camada de baixo e **acionava de verdade** o botão da tela por
trás do modal, na mesma posição de tela. Não era só cosmético — dava pra
clicar "através" de um diálogo de erro/confirmação.

A correção do hover já resolveu os dois: `on_press(EngineMessage::DialogDismiss)`
ficou **sempre** anexado ao backdrop, mesmo quando `dismissible: false`. O
`dispatch()` (em `lib.rs`) já decidia, de antes, se `DialogDismiss` realmente
fecha o diálogo com base em `spec.dismissible`:

```rust
EngineMessage::DialogDismiss => {
    if self.dialog.as_ref().is_some_and(|d| d.dismissible) {
        self.dialog = None;
    }
    return iced::Task::none();
}
```

Ou seja: anexar `on_press` sempre só muda se o evento é **capturado** (para
de vazar pra camada de baixo); se ele realmente fecha o diálogo continua
sendo decidido só por `dismissible`, sem mudança de comportamento visível
pro usuário em diálogos não-dismissíveis.

### Lição para o resto do framework

Qualquer widget futuro que precise "bloquear" uma área por cima de outra
camada de um `Stack` (outro tipo de overlay, um tooltip, etc.) precisa dos
dois cuidados juntos: `.interaction(Interaction::Idle)` (não deixar
`mouse_interaction` cair em `None`) **e** um `on_press`/`on_release` sempre
presente (não deixar o `update()` deixar de capturar o evento) — só cobrir
visualmente uma área não basta para bloqueá-la nas duas frentes de input do
`iced`.
