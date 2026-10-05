# `GlacierDaemon`: o mínimo de Rust

O `GlacierDaemon` é o programa que abre as janelas. Ele tem uma API em Rust
(`.title(…)`, `.main(|motor| …)`, `.tray(…)`), mas **quase tudo o que ela faz
também se escreve no markup** — no cabeçalho do template. A regra deste projeto:

> **Rust só para o que o markup não expressa.** O resto vai para o `<screen>`, o
> `<app>`, a `<tray>` e o `<resources>`.

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
| `.main_template("views/app.gvb")` | diz qual arquivo é a tela principal | **é a forma markup.** Sem ela, o `run` abre `./views/app.*` ou `app.*` |
| `.title("Meu app")` | título da janela principal | `<screen title="Meu app">` — **vence** o `.title` |
| `.main_size(900.0, 600.0)` | tamanho inicial | `<screen size="900 600">` — **vence** o `.main_size` |
| `.main_window(Settings)` | `decorations`, `icon`, `min_size`, `resizable`… | `<screen decorations="false" icon="…" min_size="…" max_size="…" fixed_size="…" resizable="false">`. Posição inicial, `platform_specific` e `exit_on_close_request` só em Rust |
| `.child_window(f)` | ajusta as `Settings` das janelas abertas por `open_window` | cada janela filha tem o próprio `<screen>`, com os mesmos atributos |
| `.single_instance("meu-app")` | uma segunda execução foca a primeira e sai | `<app id="meu-app" single_instance="true" />` |
| `.remember_window_geometry(true)` | grava o tamanho/posição ao fechar e restaura ao abrir | `<app id="meu-app" remember_geometry="true" />` |
| `.storage_dir(dir)` | onde o global `storage` e a geometria gravam | sai do `id` do `<app>` (`~/.local/share/<id>`). Um diretório próprio só em Rust |
| `.tray(TrayConfig)` + `.on_tray(f)` | ícone de bandeja e menu | `<tray>` com `<item>`, `<check>` e `<separator>`; as ações `tray:open`, `tray:quit` e `notifications:toggle` o runner trata sozinho, o resto vai para o `<script>` |
| `.style(style::FUSION)` | estilo visual embutido da janela | um botão com `on_click="style:fusion"` troca em tempo de execução; o estilo **inicial** só em Rust |
| `.font(bytes)` / `.font_named("Inter", bytes)` / `.default_font(f)` | embute uma fonte no binário | só em Rust (são bytes do binário). Depois de registrada, `font="Inter"` no markup e `font_family: Inter` no `.gss` a usam |
| `.antialiasing(false)` | liga/desliga o MSAA | só em Rust |
| `.reload_period(d)` / `.toast_period(d)` | período do hot-reload e da expiração de toasts | só em Rust |
| `.assets(fonte)` | embute templates/estilos no binário (web, standalone) | só em Rust |
| `.lua_extension(f)` | expõe funções Rust como globais do `<script>` | só em Rust |
| `.on_message(f)` / `.on_close(f)` | observa cada mensagem despachada / o fechamento da principal | só em Rust |
| `.main(\|motor\| …)` | configura o motor à mão | quase sempre dispensável: ver a seção 2 |
| `.run()` | sobe o loop | — |

### Duas regras que valem para todas

1. **O `<screen>` vence o builder.** `.title(…)` e `.main_size(…)` só fazem efeito
   quando o template não diz nada. Escreva no `<screen>` e apague a chamada.
2. **`<app>` e `<tray>` só são lidos do template principal**, e só quando ele vem
   do padrão ou de `.main_template(…)`. Um `.main(|motor| …)` escrito à mão não diz
   ao daemon qual é o template principal, então o `<app>` e a `<tray>` dele são
   ignorados. O builder em Rust (`.single_instance`, `.tray`…) **vence** o
   markup onde é usado.


**Um app com bandeja não precisa de `.main`.** A `<tray>` só é lida com
`.main_template(…)` (ou o padrão), então prefira a lógica da tela no `<script>` Luau
(`notify(…)`, `toast(…)`, `ctx`) a um `impl Component` só para disparar uma
notificação: o exemplo `bandeja` é isso, sem uma linha de Rust além do
`main_template`.

---

## 2. As chamadas do motor dentro de `.main(|motor| …)`

Quem escreve `.main(|motor| …)` recebe o `GlacierUI` (o motor) e o configura. Quase
tudo o que se faz ali tem uma forma no markup:

| Chamada no `.main` | O que faz | No markup |
|---|---|---|
| `motor.register_component("app", "views/app.gvb")` + `motor.set_initial_screen("app")` | registra o template e o torna a tela inicial | `.main_template("views/app.gvb")` |
| `motor.register_component("detalhe", "views/detalhe.gvb")` | registra outro template | `<link rel="import" href="views/detalhe.gvb" />` no `<resources>` (o nome é o do arquivo; `as="detalhe"` o troca) |
| `motor.load_stylesheet("views/app.gss")` | carrega uma folha global | `<link rel="stylesheet" href="views/app.gss" />` |
| `motor.define_data("paises", json)` | publica um JSON no contexto | `<link rel="data" as="paises" href="views/paises.json" />` |
| `motor.define_data("qtd", "1")` | valor inicial de uma chave | `<script>` com `function init() ctx.qtd = "1" end` |
| `motor.register(Box::new(MeuComponente))` | liga um `impl Component` (lógica em Rust) | **só em Rust.** É o motivo legítimo de ter um `.main` |

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

é, por inteiro, isto (com o `title` indo para o `<screen title="…">`):

```rust
GlacierDaemon::new().main_template("views/console.gvb").run()
```

Uma tela **com** `impl Component` em Rust continua precisando do `.main`, mas só dele:

```rust
GlacierDaemon::new()
    .main(|motor| {
        let _ = motor.register(Box::new(Contador::new()));
        motor.set_initial_screen("contador");
    })
    .run()
```

O título, o tamanho, o ícone e as folhas de estilo ficam no template do componente
(`<screen title=… size=…>`, `<link rel="stylesheet">`).

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

**Depois** — o Rust diz só qual arquivo abre, e o cabeçalho do template diz o resto:

```rust
GlacierDaemon::new().main_template("views/app.gvb").run()
```

```
screen(title = "Glacier - Bandeja", size = "460 300") {
  resources {
    app(id = bandeja, single_instance = true, remember_geometry = true)
    link(rel = stylesheet, href = "views/app.gss")
  }
  …
}
```

---

## 4. O que o markup não expressa (e por isso fica em Rust)

- lógica em Rust (`impl Component`, `.on_message`, `.on_close`);
- pontes para código nativo (`.lua_extension`);
- recursos embutidos no binário (`.font`, `.assets`);
- ajustes finos de renderização (`.antialiasing`, `.reload_period`, `.toast_period`);
- o estilo embutido inicial (`.style`) e a posição inicial da janela.

Se você se pegar escrevendo Rust para outra coisa, provavelmente já existe a forma
em markup: procure primeiro no `<screen>`, no `<app>`, na `<tray>` e no
`<resources>`.
