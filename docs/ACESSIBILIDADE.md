# Acessibilidade

Fase 2 do `PLANO_SUBSTITUIBILIDADE.md`. **Parcial**: a árvore chega ao AT-SPI e o teclado
alcança os botões, mas o critério da fase (o exemplo "navegável só com teclado e **lido por
Orca**") não foi verificado com o Orca de verdade. Leia "O que NÃO está feito".

## Atributos no markup

Três atributos universais, em qualquer tag (como `tooltip=`):

```gv
button(on_click = fechar, aria_label = "Fechar janela", text = "✕")
container(role = button, on_click = abrir, aria_label = "Abrir o menu")
image(src = "enfeite.png", aria_hidden = true)
```

| Atributo | Efeito |
|---|---|
| `role` | o papel ARIA do nó (`button`, `link`, `checkbox`, `heading`, `list`, `dialog`, `alert`, `region`… — a lista está em `a11y::PAPEIS`). `role="none"` tira o nó da árvore. Valor desconhecido: ignorado, com aviso no stderr |
| `aria_label` | o nome que o leitor de tela lê, no lugar do texto do nó. Indispensável num botão só com ícone |
| `aria_hidden="true"` | decorativo: fora da árvore (e seus filhos junto) |

No `.gv`: `button(text = "✕", on_click = fechar, aria_label = "Fechar janela")`.

## A árvore (feature `a11y`, só Linux)

```toml
glacier-ui = { version = "0.146", features = ["a11y"] }
```

A cada reavaliação da tela, o motor monta a árvore de acessibilidade **da árvore avaliada** —
papel pela tag (`button`, `checkbox`, `textbox`, `slider`, `progressbar`, `heading`, `table`…),
nome pelo texto, estado pelo contexto (marcado, valor) — e a entrega ao AT-SPI pelo
`accesskit_unix`. O `prose` vira parágrafos, títulos com nível, citações e listas.
`GLACIER_A11Y=0` desliga a ponte em tempo de execução.

**Por que o AT-SPI direto, e não o `accesskit_winit`.** O `iced` 0.14 e o `winit` 0.30 não
expõem o AccessKit, e o `iced` é o dono do laço de eventos (o `accesskit_winit` precisa do
`ActiveEventLoop`). O `accesskit_unix::Adapter` não depende do `winit`. Custo: só Linux, e só
a janela do processo.

O adaptador só pede a árvore quando um cliente de acessibilidade está conectado (o
barramento `org.a11y.Bus` com `IsEnabled`); sem leitor de tela ligado o custo é uma
construção de árvore por reavaliação, que a feature desligada não paga.

### Medido

Com o exemplo `i18n` (`--features a11y`), lendo pelo barramento AT-SPI da sessão
(`busctl` no `org.a11y.Bus`, já que esta máquina não tem `pyatspi`):

- o app aparece como um nó `application` (papel 75) → `frame` (23, a janela "i18n") → os
  textos como `label` (29) e os botões como `push button` (43), com os nomes certos
  (`Services dashboard`, `Hello, Ana!`, `Add`, `Remove`…);
- **ação:** `org.a11y.atspi.Action.DoAction(0)` no botão "Add" devolveu `true`, o motor
  executou o `on_click`, reavaliou e **a árvore publicada passou de "1 active service" para
  "3 active services"** (duas chamadas): o ciclo leitor → app → árvore nova fecha.

Para repetir: ligue a acessibilidade na sessão (`gsettings set org.gnome.desktop.interface
toolkit-accessibility true`, ou a propriedade `IsEnabled` do `org.a11y.Status`) e inspecione
com o Accerciser ou o Orca.

## Teclado

O `button` agora entra na ordem de **Tab / Shift+Tab**, mostra um **anel de foco** de 2 px na
cor de destaque do tema, e **Enter** ou **Espaço** disparam a mesma ação do clique
(`on_click`, `navigate_to`, envio de formulário…). Um botão `disabled` ou sem ação fica
fora da ordem. Medido numa janela real (Xvfb, teclas por XTEST): Tab põe o anel em "Add", Enter
duas vezes sobe a contagem para 3, Tab move o anel para "Remove", Espaço desce para 2.
Feito em `src/focus_button.rs`, no padrão do `src/link_focus.rs` (links do `prose`).

Já tinham teclado: `textinput`/`textarea` (nativos do `iced`), links do `prose`.

## Contraste (WCAG AA)

O `.gss` avisa (stderr, com arquivo:linha) quando uma regra tem `color` e `background`
opacos que falham no AA: **4,5:1** para texto normal, **3:1** para texto grande (24 px, ou 18,66
px em negrito). Num `button` o par é `text_color` sobre `color` (o `color` do botão é o
fundo). Vale também para regras que usam `var(--x)` declarado no **mesmo** arquivo.

Primeiro achado real, no `app.gss` do rustploy: `.env__del` — `#DA3633` sobre `#21262D` dá
3,30:1.

Não mede: gradiente, cor com transparência (não se sabe o que há por baixo), `var()` vinda de
**outro** sheet, e o texto que herda `color` de um pai cujo fundo está em outra regra. É um
alarme, não uma garantia: a ausência de aviso não prova que a tela passa.

## O que NÃO está feito

- **Orca não foi testado.** A árvore foi lida pelo D-Bus do AT-SPI, não por um leitor de
  tela falando. O critério "lido por Orca" da fase fica **em aberto**.
- **Foco e geometria.** A árvore diz sempre "o foco está na janela": o foco de teclado do
  `iced` (qual botão tem o anel) não chega ao AT-SPI, então o Orca não anuncia a troca de
  foco ao apertar Tab. Os nós também não têm **posição/tamanho**: leitura por "onde está o
  mouse" e a ampliação (lupa) não funcionam.
- **Só o botão ganhou teclado.** `checkbox`, `toggle`, `radio`, `slider`, `select`,
  as abas, a árvore, a tabela, o calendário: **não foram auditados** e o `iced` não os torna
  focáveis por si. A lista de "Navegação por teclado completa nos builtins" segue aberta.
- **Só Linux, uma janela.** Sem NVDA/UIA (Windows) nem VoiceOver (macOS); e um app com várias
  janelas publica só a tela da janela ativa de cada reavaliação num único adaptador.
- **Sem texto selecionado, sem live regions**, sem `aria-describedby`/`labelledby`, sem
  relação `label → campo` (um `textinput` é nomeado pelo `placeholder` ou por `aria_label`).
- **Links do `prose`** não viram nós `link` na árvore (o parágrafo é um texto só).
- **Ação do leitor de tela** só `Click`, em botões (o que tem `on_click`/`on_press`); `Focus`,
  `SetValue`, `Increment` não.
- **Web (wasm):** nada.
