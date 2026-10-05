# `GlacierDaemon`: o mínimo de Rust

O `GlacierDaemon` é o programa que abre as janelas. Ele tem uma API em Rust
(`.title(…)`, `.main(|motor| …)`, `.tray(…)`), mas **quase tudo o que ela faz
também se escreve no markup** — no manifesto do app. A regra deste projeto:

> **Rust só para o que o markup não expressa.** O resto vai para o `app(...)`, as
> `screen(...)`, a `tray` e o `resources`.

## 0. O app é a raiz, e as telas são filhos dele

O arquivo que o daemon abre tem como raiz o **`app`** — o que é do aplicativo —, e
as telas são filhas dele (planejamento em `PLANO_APP_TELAS.md`):

```
app(
  id = meu_app, single_instance = true, remember_geometry = true,
  size = "980 640", min_size = "560 420", icon = "views/assets/icone.png",
) {
  resources {
    link(rel = theme, href = "views/styles/theme.json")
    link(rel = stylesheet, href = "views/styles/app.gss")
    component(name = Rotulo) { … }                    // global: vale em todas as telas
    link(rel = import, href = "components/nav_item.gvb", as = NavItem)
    tray(icon = "views/assets/icone.png") { … }
  }

  screen(name = home, initial = true, title = "Início") {   // inline
    column { … }
  }

  screen(name = ajustes, title = "Ajustes", src = "views/ajustes.gvb")  // em arquivo
  screen(name = relogio, title = "Relógio")   // sem corpo: um `impl Component` em Rust
}
```

| No `app` (vale para o app inteiro) | Na `screen` (vale para ela) |
|---|---|
| `id`, `single_instance`, `remember_geometry`, `antialiasing`, `toast_period`, `reload_period`, `font`, `application_id` | `name`, `initial`, `title`, `src` |
| a **janela principal**: `size`, `min_size`, `max_size`, `fixed_size`, `resizable`, `decorations`, `icon` | o layout |
| `resources` global: `link` de tema/`.gss`/dados, `component`, `import`, `dialog`, `font` e a `tray` | `resources` local: `script`, `style(scoped)`, `link`, `component` |

- **A `screen` é só conteúdo.** `size`, `icon` & cia. numa `screen` sob o `app` são
  erro de parse; o tamanho de uma janela **filha** vai na chamada:
  `open_window("sobre", { size = "420 300" })` (o nome é o de uma tela). A filha
  herda o `icon` do app.
- **`initial = true`**: no máximo uma. Sem nenhuma, vale a primeira declarada.
- **`name`** é o de `navigate_to`/`open_window`. Com `src`, vale o nome do arquivo
  sem extensão. Os arquivos de `src` são os de sempre (raiz `screen`/`component`).
- **Nomes de componente são do app inteiro** (um espaço de nomes só): o mesmo nome
  em duas telas, ou numa tela e no `resources` do app, é erro de parse.
- `app` e `tray` **só existem na raiz**: dentro do `resources` de uma tela são erro.
  Um arquivo com raiz `screen` continua sendo uma tela válida (o app de uma tela só
  pode seguir sem `app`, e sem bandeja nem `id`).
- `.gv`/`.gva` (XML) e `.gvb` usam a mesma estrutura (`<app><screen/></app>`).

O programa mais curto possível é este, e é o que a maioria dos apps precisa:

```rust
use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new().run()                      // abre views/app.gvb (ou .gva, .gv)
}

// ou, com outro arquivo:
//     GlacierDaemon::new().main_template("views/painel.gvb").run()
```

O resto desta página responde a duas perguntas: **o que cada chamada faz** e **onde
isso se escreve no markup**.

---

## 1. Os métodos do `GlacierDaemon`

| Chamada em Rust | O que faz | No markup |
|---|---|---|
| `GlacierDaemon::new()` | cria o daemon | — |
| `.main_template("views/app.gvb")` | diz qual arquivo é o manifesto (ou a tela) principal | **é a forma markup.** Sem ela, o `run` abre `./views/app.*` ou `app.*` |
| `.title("Meu app")` | título da janela principal | `screen(title = …)`, a da tela ativa — **vence** o `.title` |
| `.main_size(900.0, 600.0)` | tamanho inicial | `app(size = "900 600")` — **vence** o `.main_size` |
| `.main_window(Settings)` | `decorations`, `icon`, `min_size`, `resizable`… e o `application_id` (Linux): `app(application_id = meu-app)` | `app(decorations = false, icon = "…", min_size = "…", max_size = "…", fixed_size = "…", resizable = false)`. Posição inicial, `platform_specific` e `exit_on_close_request` só em Rust |
| `.child_window(f)` | ajusta as `Settings` das janelas abertas por `open_window` | o tamanho e a moldura de cada filha vão em `open_window("nome", { size = "…", decorations = false })`; o resto (`platform_specific`…) só em Rust |
| `.single_instance("meu-app")` | uma segunda execução foca a primeira e sai | `app(id = meu_app, single_instance = true)` |
| `.remember_window_geometry(true)` | grava o tamanho/posição ao fechar e restaura ao abrir | `app(id = meu_app, remember_geometry = true)` |
| `.storage_dir(dir)` | onde o global `storage` e a geometria gravam | sai do `id` do `app` (`~/.local/share/<id>`). Um diretório próprio só em Rust |
| `.tray(TrayConfig)` + `.on_tray(f)` | ícone de bandeja e menu | `tray` com `item`, `check` e `separator` no `resources` do `app`; as ações `tray:open`, `tray:quit` e `notifications:toggle` o runner trata sozinho, o resto vai para o `<script>` |
| `.style(style::FUSION)` | estilo visual embutido da janela | um botão com `on_click="style:fusion"` troca em tempo de execução; o estilo **inicial** só em Rust |
| `.font(bytes)` / `.font_named("Inter", bytes)` / `.default_font(f)` | embute uma fonte no binário | `font(src = "assets/fonts/Inter.ttf", family = Inter)` no `resources` do `app`, e `app(font = Inter)` para torná-la a padrão. O arquivo é lido pela fonte de assets (`.assets(…)` a embute). `font="Inter"` no markup e `font_family: Inter` no `.gss` a usam |
| `.antialiasing(false)` | liga/desliga o MSAA | `app(antialiasing = false)` |
| `.reload_period(d)` / `.toast_period(d)` | período do hot-reload e da expiração de toasts | `app(reload_period = 500, toast_period = 250)` (milissegundos) |
| `.assets(fonte)` | embute templates/estilos no binário (web, standalone) | só em Rust |
| `.lua_extension(f)` | expõe funções Rust como globais do `<script>` | só em Rust |
| `.on_message(f)` / `.on_close(f)` | observa cada mensagem despachada / o fechamento da principal | só em Rust |
| `.main(\|motor\| …)` | configura o motor à mão | quase sempre dispensável: ver a seção 2 |
| `.run()` | sobe o loop | — |

### Duas regras que valem para todas

1. **O markup vence o builder.** `.title(…)` e `.main_size(…)` só fazem efeito
   quando o manifesto não diz nada. Escreva no `app`/`screen` e apague a chamada.
   Inversamente, o builder em Rust (`.single_instance`, `.tray`…) **vence** o
   markup onde é usado.
2. **`app` e `tray` são lidos do manifesto principal**, o que vem do padrão
   (`views/app.*`) ou de `.main_template(…)`. O `.main(|motor| …)` **não** desliga
   mais essa leitura: ele roda **depois** do manifesto e só registra o que tem
   lógica em Rust. Sem `.main_template`, o `.main` ainda é o dono da principal,
   a não ser que o `views/app.*` padrão seja um manifesto (raiz `app`).

**Um app com bandeja não precisa de `.main`.** Prefira a lógica da tela no
`<script>` Luau (`notify(…)`, `toast(…)`, `ctx`) a um `impl Component` só para
disparar uma notificação: o exemplo `bandeja` é isso, sem uma linha de Rust além
do `main_template`.

---

## 2. As chamadas do motor dentro de `.main(|motor| …)`

Quem escreve `.main(|motor| …)` recebe o `GlacierUI` (o motor) e o configura. Quase
tudo o que se faz ali tem uma forma no markup:

| Chamada no `.main` | O que faz | No markup |
|---|---|---|
| `motor.register_component("app", "views/app.gvb")` + `motor.set_initial_screen("app")` | registra o template e o torna a tela inicial | `.main_template("views/app.gvb")` |
| `motor.register_component("detalhe", "views/detalhe.gvb")` | registra outra tela | `screen(name = detalhe, src = "views/detalhe.gvb")` no `app` (ou `link(rel = import, …)` num `resources`) |
| `motor.load_stylesheet("views/app.gss")` | carrega uma folha global | `<link rel="stylesheet" href="views/app.gss" />` |
| `motor.define_data("paises", json)` | publica um JSON no contexto | `<link rel="data" as="paises" href="views/paises.json" />` |
| `motor.define_data("qtd", "1")` | valor inicial de uma chave | `<script>` com `function init() ctx.qtd = "1" end` |
| `motor.register(Box::new(MeuComponente))` | liga um `impl Component` (lógica em Rust) | **só em Rust.** No manifesto, a tela aparece como `screen(name = x)` **sem corpo e sem `src`**, e o `.main` registra o componente com esse nome. Sem registro, o app não sobe e o erro cita o nome |

Uma tela **sem lógica em Rust** — a lógica mora no `<script>` Luau do template — não
precisa de `.main`. Esta:

```rust
GlacierDaemon::new()
    .title("Glacier - Console")
    .main(|motor| {
        if let Err(e) = motor.register_component("console", "views/console.gvb") {
            eprintln!("Erro ao registrar a tela: {e}");
        }
        motor.set_initial_screen("console");
    })
    .run()
```

é, por inteiro, isto (com o `title` indo para o `screen(title = …)`):

```rust
GlacierDaemon::new().main_template("views/console.gvb").run()
```

Uma tela **com** `impl Component` em Rust precisa do `.main`, mas só dele:

```rust
GlacierDaemon::new()
    .main(|motor| {
        let _ = motor.register(Box::new(Relogio));   // casa com `screen(name = relogio)`
    })
    .run()
```

A tela inicial, o título, o tamanho, o ícone e as folhas de estilo ficam no
manifesto (`app(size = …)`, `screen(name = relogio, title = …)`,
`link(rel = stylesheet, …)`). Exemplo: `examples/gvb/app_telas`.

---

## 3. Antes e depois

**Antes** — o Rust dizia o que o template já sabia dizer:

```rust
GlacierDaemon::new()
    .title("Glacier - Bandeja")
    .main_size(460.0, 300.0)
    .single_instance("bandeja")
    .remember_window_geometry(true)
    .main(|motor| {
        motor.load_stylesheet("views/app.gss").ok();
        motor.register_component("app", "views/app.gvb").ok();
        motor.set_initial_screen("app");
    })
    .run()
```

**Depois** — o Rust diz só qual arquivo abre, e o manifesto diz o resto:

```rust
GlacierDaemon::new().main_template("views/app.gvb").run()
```

```
app(id = bandeja, single_instance = true, remember_geometry = true, size = "460 300") {
  resources {
    link(rel = stylesheet, href = "views/app.gss")
  }

  screen(name = app, title = "Glacier - Bandeja") {
    …
  }
}
```

---

## 4. O que o markup não expressa (e por isso fica em Rust)

- lógica em Rust (`impl Component`, `.on_message`, `.on_close`);
- pontes para código nativo (`.lua_extension`);
- os bytes embutidos no binário (`.font(include_bytes!(…))`, `.assets`) — a fonte lida de um arquivo é `font(src = …)`;
- o estilo embutido inicial (`.style`) e a posição inicial da janela.

Se você se pegar escrevendo Rust para outra coisa, provavelmente já existe a forma
em markup: procure primeiro no `app`, na `screen`, na `tray` e no
`resources`.
