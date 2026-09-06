# Solução de problemas

Problemas conhecidos, do lado do ambiente (GPU/driver/SO), que não são bug do
`glacier-ui` mas afetam qualquer app construído com ele (é o `iced`/`wgpu` por
baixo). Cada entrada tem o sintoma, a causa e a correção.

---

## Corrupção visual (listras/ruído) ao redimensionar a janela

### Sintoma

Ao redimensionar a janela de um app `glacier-ui` (ou qualquer app `iced`/
`wgpu`), a área da janela fica coberta de listras diagonais/ruído gráfico —
não é um glitch de um frame só, o conteúdo real (widgets, texto) some por
trás do ruído até soltar o mouse.

### Causa

GPUs Intel integradas antigas (Ivy Bridge/Haswell — ~2012-2014, ex.: **HD
Graphics 2500/4000**) rodam num driver Vulkan da Mesa (`ANV`) que a própria
Mesa marca como **incompleto** para essas gerações. Ao abrir qualquer
superfície Vulkan nessas GPUs, o stderr mostra algo como:

```
MESA-INTEL: warning: Ivy Bridge Vulkan support is incomplete
```

`wgpu` (o backend de renderização do `iced`) tenta Vulkan primeiro por
padrão. A parte mais frágil de qualquer implementação Vulkan é a
**recriação da swapchain durante um resize ao vivo** — exatamente onde a
corrupção aparece. O driver **OpenGL** da Mesa para essas mesmas GPUs, em
contraste, é maduro (décadas de uso) e não sofre disso.

Diagnóstico: rode o app uma vez sem nada e note se o warning acima aparece no
terminal; depois rode de novo com `WGPU_BACKEND=gl` (ver correção) — se o
warning some e o ruído no resize também some, é este o problema.

### Correção

Force o `wgpu` a usar o backend OpenGL em vez de Vulkan — não muda nada no
código do app, é uma variável de ambiente que o `wgpu` já respeita
nativamente:

```bash
WGPU_BACKEND=gl cargo run --example galeria_estilos   # teste pontual
```

Para não precisar setar toda vez, o pulo do gato é que **um terminal sozinho
não é suficiente**: apps abertos via ícone/launcher (entradas `.desktop`) não
herdam o ambiente de um shell interativo. São necessárias até três camadas,
cada uma cobrindo uma forma diferente de abrir o app — configure as que se
aplicam ao seu fluxo:

| Onde | Cobre | Como |
|---|---|---|
| Shell interativo (terminal) | `cargo run`, apps abertos por linha de comando | `export WGPU_BACKEND=gl` no `.bashrc`/`.zshrc` |
| Sessão do systemd (qualquer DE, se a distro usa systemd) | Todo processo da sessão gráfica, inclusive `.desktop` | Arquivo `KEY=value` (sem `export`) em `~/.config/environment.d/algum-nome.conf` |
| Sessão do KDE Plasma especificamente | Reforço do caso acima; roda antes do `startplasma-wayland`/`-x11` subir a sessão | Script `.sh` executável com `export WGPU_BACKEND=gl` em `~/.config/plasma-workspace/env/algum-nome.sh` |

As duas últimas **não são retroativas**: só valem depois de um **logout/login
completo (ou reboot)** — travar/destravar a tela não reinicia o
`systemd --user` nem o Plasma, então não é suficiente pra testar.

Verificação (depois do logout/login):

```bash
systemctl --user show-environment | grep WGPU_BACKEND
# esperado: WGPU_BACKEND=gl
```

### Isto é específico de GPU

Só se aplica a GPUs cujo driver Vulkan é realmente incompleto (Intel
Ivy Bridge/Haswell é o caso conhecido). Numa GPU diferente, corrupção
visual no resize indicaria outra coisa — confirme o warning do driver
(`MESA-INTEL: ... Vulkan support is incomplete`, ou o equivalente do seu
driver) no stderr antes de aplicar esta correção.

---

## Painel flutuante que escurece até ficar preto

### Sintoma

Um painel com **sombra** que vive numa camada por cima da tela — o painel de
sugestões do `<autocomplete>`, um `<dialog>`, um toast — vai ficando
progressivamente mais escuro a cada quadro, até virar um retângulo preto. O
detalhe que denuncia o mecanismo: **as linhas por onde o cursor passa voltam ao
normal** e as outras continuam escurecendo. Num `<autocomplete>` filtrado até
sobrar uma opção só, o efeito **não** aparece — a única linha é sempre a
realçada, e a realçada desenha fundo opaco todo quadro.

### Causa

A mesma GPU/driver da seção acima (Intel Ivy Bridge/Haswell no Vulkan da Mesa).
O corpo do container é repintado a cada quadro, mas o **quad da sombra** —
preto translúcido — não é apagado junto: ele se soma sobre o que já estava ali.
Onze quadros de preto a 0,35 já são um painel praticamente preto. Onde algo
opaco é redesenhado por cima (a linha realçada, a linha sob o cursor), a conta
zera; onde nada opaco é redesenhado, ela acumula.

Diagnóstico (dá para fechar em dois minutos, sem RenderDoc): troque a cor da
sombra por vermelho e o fundo do painel por verde. Se o vermelho aparecer
**sobre** o verde em faixas de intensidades diferentes, é isto. Depois rode o
mesmo binário com `WGPU_BACKEND=gl` — no OpenGL a mesma sombra renderiza
correta e estável.

### Correção

`WGPU_BACKEND=gl`, exatamente como na seção anterior (as três camadas de
ambiente valem igual).

Do lado do motor, nada mais depende disso: os três painéis que traziam sombra —
o do `<autocomplete>` (`src/widget.rs`), o cartão do `<dialog>` (`src/dialogs.rs`)
e o cartão do toast (`src/toasts.rs`) — perderam a `shadow` e ganharam a elevação
por um **fundo opaco mais claro** (`background.weak`) mais a borda que cada um já
tinha. Opacos, e por isso imunes ao acúmulo em qualquer driver.

É a regra prática que vale para qualquer painel próprio: numa camada que flutua,
prefira elevar com **cor de fundo** a elevar com sombra.
