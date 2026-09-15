//! Runner **multi-janela** do Glacier, sobre o modelo `iced::daemon`.
//!
//! No iced 0.14 múltiplas janelas exigem o `daemon` (não `application`), porque
//! só ele tem `view`/`title` indexados por [`window::Id`]. O [`GlacierDaemon`]
//! mantém **um [`GlacierUI`] por janela** (`windows`), cada um independente:
//! contexto, telas, componentes e estado isolados. Abrir uma janela nova (via
//! [`crate::Context::open_window`] no Rust ou `open_window(...)` na Lua) sobe um
//! motor fresco que carrega aquela fonte do zero.
//!
//! Uso típico no `main` de um app:
//!
//! ```ignore
//! fn main() -> iced::Result {
//!     GlacierDaemon::new()
//!         .title("Meu app")
//!         .main(|motor| {
//!             motor.register(Box::new(MinhaTela::new())).unwrap();
//!             motor.set_initial_screen("minha_tela");
//!         })
//!         .run()
//! }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use iced::window;
use iced::{Element, Font, Point, Size, Subscription, Task};

use crate::asset_source::{AssetSource, DiskAssets};
use crate::component::{WindowSource, WindowSpec};
use crate::parser::{TrayItemDecl, TrayItemKind};
use crate::tray::{TrayActions, TrayConfig, TrayHandle, TrayItem, TrayMsg, TrayRequest};
use crate::{EngineMessage, GlacierUI};

/// A geometria de uma janela no momento em que ela vai fechar — o que um app
/// precisa para reabrir onde parou. Entregue ao gancho de
/// [`GlacierDaemon::on_close`].
///
/// `position` é `None` no Wayland: o protocolo simplesmente não expõe a posição
/// da janela ao cliente. Não é um bug a corrigir; é para o app decidir o que
/// fazer (na prática, só persistir o tamanho).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowGeometry {
    pub size: Size,
    pub position: Option<Point>,
}

/// Ajuste das `window::Settings` de uma janela-filha, recebendo o [`WindowSpec`]
/// que a pediu. Ver [`GlacierDaemon::child_window`].
type ChildSettingsHook = Rc<dyn Fn(&WindowSpec, &mut window::Settings)>;
/// Observador de cada mensagem despachada na janela principal, com o motor no
/// estado resultante. Ver [`GlacierDaemon::on_message`].
type MessageHook = Rc<dyn Fn(&EngineMessage, &GlacierUI)>;
/// Gancho de fechamento da janela principal, com a geometria dela. Ver
/// [`GlacierDaemon::on_close`].
type CloseHook = Rc<dyn Fn(&GlacierUI, WindowGeometry)>;
/// Configuração do motor da janela principal (registra componentes, tela
/// inicial, …). É um `Rc` para o `Runtime` guardá-la e **reabrir** a principal
/// depois que ela fecha (o clique "Open Rustploy" da bandeja). Ver
/// [`GlacierDaemon::main`].
type SetupHook = Rc<dyn Fn(&mut GlacierUI)>;

/// Templates que o `run` procura, nesta ordem, quando o app não chamou
/// [`GlacierDaemon::main`] nem [`GlacierDaemon::main_template`].
const DEFAULT_MAIN_TEMPLATES: [&str; 2] = ["./views/app.gv", "app.gv"];

/// O `setup` de [`GlacierDaemon::main_template`]: registra `path` com o nome do
/// arquivo sem extensão e o torna a tela inicial.
fn template_setup(path: String) -> impl Fn(&mut GlacierUI) + 'static {
    let name = std::path::Path::new(&path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());
    move |motor| {
        if let Err(erro) = motor.register_component(&name, &path) {
            eprintln!("{erro}");
        }
        motor.set_initial_screen(&name);
    }
}

/// Gancho de clique num item do menu da bandeja: recebe o `id` do item e um
/// [`TrayActions`] para pedir ações (abrir a principal, sair, mudar rótulo). Ver
/// [`GlacierDaemon::on_tray`].
type TrayHook = Rc<dyn Fn(&str, &mut TrayActions)>;

/// Construtor/runner do app multi-janela. Ver [módulo](self).
pub struct GlacierDaemon {
    /// Título da janela principal (e default das demais que não trazem um).
    title: String,
    /// `window::Settings` da janela principal. Começa no default do iced com o
    /// tamanho de [`GlacierDaemon::main_size`]; um app que precise de mais
    /// (borderless, ícone, `min_size`, geometria restaurada) troca o bloco
    /// inteiro com [`GlacierDaemon::main_window`].
    main_settings: window::Settings,
    /// Ajuste opcional das `window::Settings` de cada janela-filha, aplicado
    /// sobre o default do daemon. Ver [`GlacierDaemon::child_window`].
    child_settings: Option<ChildSettingsHook>,
    /// Configura o motor da janela principal (registra componentes, define a
    /// tela inicial, carrega `.gss`, …). Rodado na inicialização e de novo a
    /// cada reabertura da principal pela bandeja. `None` enquanto o app não
    /// chamou `main` nem `main_template`; o `run` então usa um dos
    /// `DEFAULT_MAIN_TEMPLATES`.
    setup: Option<SetupHook>,
    /// Fontes embutidas a registrar no runtime do iced (bytes de `.ttf`/`.otf`).
    fonts: Vec<&'static [u8]>,
    /// Fonte padrão de todas as janelas, quando o app embute a sua.
    default_font: Option<Font>,
    /// Observador rodado depois de cada `dispatch` na janela principal — é o
    /// gancho de persistência (ver [`GlacierDaemon::on_message`]).
    on_message: Option<MessageHook>,
    /// Gancho de fechamento da janela principal (ver [`GlacierDaemon::on_close`]).
    on_close: Option<CloseHook>,
    /// Período do tick de hot-reload (checagem de arquivos alterados).
    reload_period: Duration,
    /// Período do tick de expiração de toasts.
    toast_period: Duration,
    /// Raiz opcional onde o global `storage` (persistência local em JSON) grava
    /// seus arquivos, aplicada a todos os motores. Sem isto, `storage` grava
    /// relativo ao diretório do script — inviável quando os assets moram num
    /// diretório read-only. Ver [`GlacierDaemon::storage_dir`].
    storage_dir: Option<PathBuf>,
    /// Persistir automaticamente a geometria (tamanho/posição) da janela
    /// principal, reabrindo-a onde parou. Requer [`GlacierDaemon::storage_dir`]
    /// (é lá que o arquivo mora). Ver [`GlacierDaemon::remember_window_geometry`].
    remember_geometry: bool,
    /// Configuração da bandeja (ícone + menu), quando o app quer sobreviver à
    /// última janela. Ver [`GlacierDaemon::tray`].
    tray_config: Option<TrayConfig>,
    /// Gancho de clique nos itens da bandeja. Ver [`GlacierDaemon::on_tray`].
    on_tray: Option<TrayHook>,
    /// De onde os motores desta aplicação leem seus assets (templates, estilos,
    /// scripts Luau, binários). Default [`DiskAssets`]; um app standalone injeta
    /// uma fonte embutida via [`GlacierDaemon::assets`]. Aplicada a **todos** os
    /// motores (principal, reabertura pela bandeja e janelas-filhas).
    assets: Arc<dyn AssetSource>,
    /// Estilo builtin default de **todos** os motores (ver [`crate::style`]),
    /// aplicado antes do `setup` de cada janela — para que um `<link
    /// rel="theme">` ou `.gss` do app continue vencendo. Ver
    /// [`GlacierDaemon::style`].
    style: Option<crate::style::Style>,
    /// Liga o antialiasing (MSAAx4) do renderer do iced. Ver
    /// [`GlacierDaemon::antialiasing`].
    antialiasing: bool,
    /// `app_id` da trava de instância única, quando ligada. Ver
    /// [`GlacierDaemon::single_instance`].
    single_instance_id: Option<String>,
    /// O template registrado por [`GlacierDaemon::main_template`], quando foi
    /// essa a forma do `setup`. É o que deixa o `run` ler o `<app>`/`<tray>`
    /// dele antes do boot; um `setup` escrito à mão com [`GlacierDaemon::main`]
    /// não diz qual template abre, e aí só o builder configura o aplicativo.
    main_template_path: Option<String>,
}

/// Força o `winit`/GDK a escolher X11 antes de qualquer janela nascer —
/// única forma de a feature `webview` funcionar no Linux. O `wry` não
/// suporta Wayland nativo nesta plataforma (a doc dele é explícita: "Linux
/// (X11 Only)"), e o `winit` decide o backend uma vez só pro `EventLoop`
/// inteiro, na primeira janela — forçar depois, só quando um `<script>`
/// finalmente pede uma `WindowSpec::webview`, seria tarde demais.
///
/// Duas pegadinhas, as duas resolvidas aqui:
/// - `WINIT_UNIX_BACKEND` (a env var clássica) foi removida no winit 0.29;
///   desde então o `winit` só olha se `WAYLAND_DISPLAY`/`WAYLAND_SOCKET`
///   estão setadas — daí `remove_var("WAYLAND_DISPLAY")`.
/// - Isso sozinho não basta: o GDK por baixo do `wry` ainda recai em Wayland,
///   porque `wl_display_connect(NULL)` tenta o socket `wayland-0` por padrão
///   mesmo sem a variável — daí `GDK_BACKEND=x11` também.
///
/// Condicionado a `DISPLAY` já estar setado: numa sessão Wayland "pura", sem
/// XWayland (rara — algumas distros mínimas/embarcadas), remover
/// `WAYLAND_DISPLAY` sem ter `DISPLAY` pra cair de volta quebraria o app
/// INTEIRO — o `winit` recusa abrir QUALQUER janela sem um dos dois, mesmo
/// pra quem nunca chamaria `open_window({webview_url=...})`. Nesse caso não
/// mexemos em nada: o app segue em Wayland normal, e só a webview continua
/// indisponível — a mesma limitação de sempre do `wry` ali, sem piorar nada.
#[cfg(all(target_os = "linux", feature = "webview"))]
fn forcar_x11_para_webview() {
    if std::env::var_os("DISPLAY").is_some() {
        // SAFETY: chamado no primeiríssimo passo de `GlacierDaemon::run`,
        // antes de qualquer thread adicional (winit/GTK) existir — não há
        // outra thread lendo/escrevendo o ambiente ao mesmo tempo.
        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
            std::env::set_var("GDK_BACKEND", "x11");
        }
    }
}

#[cfg(not(all(target_os = "linux", feature = "webview")))]
fn forcar_x11_para_webview() {}

impl GlacierDaemon {
    /// Novo runner. Sem [`GlacierDaemon::main`] nem
    /// [`GlacierDaemon::main_template`], o [`GlacierDaemon::run`] abre
    /// `./views/app.gv` na janela principal ou, se ele não existir, `app.gv`.
    pub fn new() -> Self {
        Self {
            title: "Glacier".to_string(),
            main_settings: window::Settings {
                size: Size::new(1024.0, 768.0),
                ..window::Settings::default()
            },
            child_settings: None,
            setup: None,
            fonts: Vec::new(),
            default_font: None,
            on_message: None,
            on_close: None,
            reload_period: Duration::from_millis(500),
            toast_period: Duration::from_millis(400),
            storage_dir: None,
            remember_geometry: false,
            tray_config: None,
            on_tray: None,
            assets: Arc::new(DiskAssets),
            style: None,
            antialiasing: true,
            single_instance_id: None,
            main_template_path: None,
        }
    }

    /// Define o estilo visual default do app (ver [`crate::style`]) — o análogo
    /// do `QApplication::setStyle` do Qt. Aplicado a **todas** as janelas
    /// (principal, reabertura pela bandeja e filhas de `open_window`), antes do
    /// `setup` de cada uma, então qualquer tema/`.gss` do próprio app vence.
    ///
    /// ```no_run
    /// use glacier_ui::{style, GlacierDaemon};
    /// GlacierDaemon::new().style(style::FUSION_DARK);
    /// ```
    pub fn style(mut self, style: crate::style::Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Liga/desliga o antialiasing (MSAAx4) do renderer do iced. Default
    /// `true` (mesmo default do iced), preservando o comportamento atual dos
    /// consumidores existentes.
    ///
    /// Custa performance real — e num fallback puramente por software (sem
    /// GPU compatível, ver [`crate::asset_source`] e os logs de
    /// `iced_wgpu::window::compositor` para diagnosticar isso numa máquina
    /// específica), o custo é multiplicado várias vezes por não ter
    /// aceleração de hardware para absorvê-lo. Um app que sabe que vai rodar
    /// em hardware modesto/antigo, ou que não depende de antialiasing pra
    /// legibilidade (a maioria das telas de formulário/lista não desenha
    /// `canvas` com curvas), ganha desligando isto.
    pub fn antialiasing(mut self, enabled: bool) -> Self {
        self.antialiasing = enabled;
        self
    }

    /// Define a fonte de assets de **todos** os motores da aplicação — o que
    /// permite um binário standalone que carrega templates/estilos/scripts/
    /// binários embutidos e não lê nada do disco. Tipicamente injetada só em
    /// release (`#[cfg(not(debug_assertions))]`), deixando o dev com o default
    /// [`DiskAssets`] (disco + hot-reload). Ver [`crate::asset_source`].
    pub fn assets(mut self, assets: Arc<dyn AssetSource>) -> Self {
        self.assets = assets;
        self
    }

    /// Registra uma **extensão da camada Lua**: funções Rust que passam a
    /// existir como globais no `<script>` de qualquer componente, em qualquer
    /// janela. É a ponte para acoplar ao app um cliente de banco, um cofre de
    /// segredos, um SDK — coisas que o motor não traz e não deveria trazer.
    ///
    /// Encaminha para [`crate::luau::register_lua_extension`] (config de
    /// processo); chame antes de [`Self::run`]. Um closure basta:
    ///
    /// ```no_run
    /// use glacier_ui::{GlacierDaemon, mlua};
    /// GlacierDaemon::new().lua_extension(|lua: &mlua::Lua| {
    ///     lua.globals().set("app_nome", "exemplo")
    /// });
    /// ```
    ///
    /// Ver o exemplo `sqlite_crud` para uma ponte completa — um cliente SQLite
    /// com `connect` / `execute` / `query` / `begin` / `commit` / `close` — e
    /// um mini-CRUD que a usa inteiramente do `<script>`.
    pub fn lua_extension(self, ext: impl crate::luau::LuaExtension) -> Self {
        crate::luau::register_lua_extension(ext);
        self
    }

    /// Define o título da janela principal (encadeável).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Define o tamanho inicial da janela principal (encadeável).
    pub fn main_size(mut self, width: f32, height: f32) -> Self {
        self.main_settings.size = Size::new(width, height);
        self
    }

    /// Substitui as `window::Settings` da janela principal — o escape hatch para
    /// tudo que o builder não nomeia: `decorations: false` (titlebar própria),
    /// `icon`, `min_size`, `position` restaurada, `platform_specific`.
    ///
    /// Um app com titlebar custom também vai querer `exit_on_close_request:
    /// false`, para que o pedido de fechar da WM passe por
    /// [`GlacierDaemon::on_close`] antes de a janela sumir.
    pub fn main_window(mut self, settings: window::Settings) -> Self {
        self.main_settings = settings;
        self
    }

    /// Ajusta as `window::Settings` de cada janela-filha (as abertas por
    /// `open_window(...)`), recebendo o [`WindowSpec`] que a pediu. Sem isto,
    /// filhas nascem com o default do iced — o que destoa num app borderless,
    /// onde elas precisam do mesmo `decorations: false` da principal.
    pub fn child_window(
        mut self,
        f: impl Fn(&WindowSpec, &mut window::Settings) + 'static,
    ) -> Self {
        self.child_settings = Some(Rc::new(f));
        self
    }

    /// Embute uma fonte (bytes de um `.ttf`/`.otf`) no binário e a registra no
    /// iced. Encadeável — chame uma vez por peso (regular, bold, …).
    pub fn font(mut self, bytes: &'static [u8]) -> Self {
        self.fonts.push(bytes);
        self
    }

    /// Como [`GlacierDaemon::font`], mas também **dá um nome de família** aos
    /// bytes — o habilitador da Onda 10 do `PLANO_WIDGETS.md`.
    ///
    /// Sem o nome, o `iced` carrega a fonte mas nada no markup tem como pedi-la;
    /// com ele, `font="Inter"` no `.gv` e `font_family: Inter` no `.gss` passam
    /// a resolvê-la pelo caminho que já existe (`font_for` → [`crate::fonts`]).
    /// A chave de contexto `__fonts` (semeada em
    /// [`GlacierUI::set_initial_screen`](crate::GlacierUI::set_initial_screen))
    /// passa a listá-la, então `<fontselect>` / `<combo items="__fonts">` a
    /// mostram sem configuração.
    ///
    /// Encadeável; chame uma vez por peso. Para usá-la como padrão de todas as
    /// janelas, combine com [`GlacierDaemon::default_font`] —
    /// `crate::fonts::register_family(nome)` devolve a [`iced::Font`] a passar
    /// para lá.
    pub fn font_named(mut self, family: &str, bytes: &'static [u8]) -> Self {
        self.fonts.push(bytes);
        crate::fonts::register_family(family);
        self
    }

    /// Define a fonte padrão de todas as janelas (tipicamente uma embutida com
    /// [`GlacierDaemon::font`]).
    pub fn default_font(mut self, font: Font) -> Self {
        self.default_font = Some(font);
        self
    }

    /// Registra o `setup` da janela principal: recebe o [`GlacierUI`] dela para
    /// registrar componentes, definir a tela inicial, carregar estilos, etc.
    pub fn main(mut self, setup: impl Fn(&mut GlacierUI) + 'static) -> Self {
        self.setup = Some(Rc::new(setup));
        self.main_template_path = None;
        self
    }

    /// Atalho para o `setup` mais comum: registra o `.gv` em `path` como
    /// componente, com o nome do arquivo sem extensão (`views/app.gv` → `app`),
    /// e o torna a tela inicial. Equivale a
    ///
    /// ```no_run
    /// # use glacier_ui::GlacierDaemon;
    /// GlacierDaemon::new().main(|motor| {
    ///     if let Err(erro) = motor.register_component("app", "app.gv") {
    ///         eprintln!("{erro}");
    ///     }
    ///     motor.set_initial_screen("app");
    /// });
    /// ```
    ///
    /// Substitui o `setup`, como [`GlacierDaemon::main`]: quem precisa de mais
    /// que isso (chamar `load_stylesheet`, registrar outros componentes) usa o
    /// `.main`.
    ///
    /// Um erro de registro é impresso e não encerra o app, porque o `run` só
    /// devolve `iced::Result` e não tem como carregar um erro do motor.
    ///
    /// Sem nenhuma chamada a `main` ou `main_template`, o `run` faz o mesmo com
    /// `./views/app.gv` ou, se ele não existir, `app.gv`.
    pub fn main_template(self, path: impl Into<String>) -> Self {
        let path = path.into();
        let mut daemon = self.main(template_setup(path.clone()));
        daemon.main_template_path = Some(path);
        daemon
    }

    /// Habilita um **ícone de bandeja** (system tray) — e, com ele, um app que
    /// **sobrevive à última janela**: com bandeja configurada, fechar a última
    /// janela não encerra mais o app; ele recolhe para a bandeja, e só o gancho
    /// [`GlacierDaemon::on_tray`] (via `quit()`) o encerra. Sem bandeja, o
    /// comportamento é o de sempre (encerra na última janela).
    ///
    /// A bandeja sobe numa thread própria (ver [`crate::tray`]); em plataformas
    /// sem suporte (macOS, ou build sem a feature `tray`) isto é ignorado e o app
    /// volta a encerrar na última janela.
    pub fn tray(mut self, config: TrayConfig) -> Self {
        self.tray_config = Some(config);
        self
    }

    /// Gancho de clique nos itens do menu da bandeja: recebe o `id` do item e um
    /// [`TrayActions`] para pedir `open_main()`, `quit()` ou atualizar o menu
    /// (`set_label`/`set_checked`). Sem `on_tray`, os cliques não fazem nada.
    pub fn on_tray(mut self, f: impl Fn(&str, &mut TrayActions) + 'static) -> Self {
        self.on_tray = Some(Rc::new(f));
        self
    }

    /// Garante uma única instância do app rodando por `app_id`: uma segunda
    /// tentativa de lançamento sinaliza a primeira — que reabre/foca a janela
    /// principal, o mesmo caminho do "Open" da bandeja — e [`GlacierDaemon::run`]
    /// retorna de imediato **sem** construir motor nem abrir janela nenhuma
    /// nessa segunda tentativa.
    ///
    /// A trava é um `TcpListener` em loopback numa porta derivada do `app_id`
    /// (ver [`crate::single_instance`]); `app_id` deve ser estável e
    /// razoavelmente único no processo do usuário — o mesmo `application_id` já
    /// usado em `PlatformSpecific` serve bem.
    ///
    /// Sem bandeja configurada, "reabrir" é só focar a janela já aberta — o app
    /// nunca chega a recolher pra lugar nenhum sem [`GlacierDaemon::tray`].
    pub fn single_instance(mut self, app_id: impl Into<String>) -> Self {
        self.single_instance_id = Some(app_id.into());
        self
    }

    /// Período do tick de hot-reload (checagem de arquivos alterados em disco).
    /// Padrão: 500ms.
    pub fn reload_period(mut self, period: Duration) -> Self {
        self.reload_period = period;
        self
    }

    /// Período do tick que expira toasts. Padrão: 400ms — mais curto deixa a
    /// expiração mais pontual, ao custo de acordar o loop mais vezes.
    pub fn toast_period(mut self, period: Duration) -> Self {
        self.toast_period = period;
        self
    }

    /// Observa cada mensagem já **despachada** na janela principal, com o motor
    /// no estado resultante. É o gancho de persistência: a camada Luau não tem
    /// I/O de arquivo, então salvar preferências (um "lembrar meu login") passa
    /// por aqui — o script grava no contexto, e o app lê o contexto e persiste.
    ///
    /// Roda **depois** do dispatch, de propósito: o interesse é o estado novo,
    /// não o velho.
    pub fn on_message(mut self, f: impl Fn(&EngineMessage, &GlacierUI) + 'static) -> Self {
        self.on_message = Some(Rc::new(f));
        self
    }

    /// Roda antes de a janela principal fechar, com a geometria dela — para
    /// persistir tamanho/posição e reabrir onde parou.
    ///
    /// A geometria é **consultada na hora** (uma ida ao runtime do iced), não
    /// acumulada de eventos `Resized`/`Moved`. A diferença é prática: durante o
    /// handshake de configuração do xdg-shell no Wayland chega um `Resized`
    /// espúrio com o `min_size` da janela, e um valor rastreado de eventos
    /// nasce envenenado com o mínimo antes de o usuário tocar em nada.
    /// Perguntar "qual é o tamanho agora?" no instante de fechar não tem essa
    /// janela de obsolescência.
    ///
    /// Só dispara se a janela principal tiver `exit_on_close_request: false`
    /// (ver [`GlacierDaemon::main_window`]) — senão o iced a fecha sozinho, sem
    /// passar por aqui.
    pub fn on_close(mut self, f: impl Fn(&GlacierUI, WindowGeometry) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// Define o diretório onde o global `storage` (persistência local em JSON,
    /// análoga a `localStorage`) grava seus arquivos — aplicado a todas as
    /// janelas do app. Passe um diretório gravável pelo usuário (ex.: o data dir
    /// do XDG). Sem isto, `storage` grava em `.glacier-storage/` relativo ao
    /// diretório do script, o que falha silenciosamente quando os assets moram
    /// num caminho read-only (um app empacotado rodando de `/usr/share`).
    pub fn storage_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.storage_dir = Some(dir.into());
        self
    }

    /// Liga a persistência automática da geometria da janela principal: o
    /// tamanho (e a posição, onde a plataforma a expõe) é gravado ao fechar e
    /// restaurado ao abrir, de modo que o app reabre onde parou. Sem isto, a
    /// principal sempre nasce com o tamanho de [`GlacierDaemon::main_window`].
    ///
    /// O arquivo mora sob [`GlacierDaemon::storage_dir`] (`window-geometry.json`);
    /// **sem** um `storage_dir` definido não há onde persistir e a opção é um
    /// no-op. O tamanho restaurado é sempre respeitado contra o `min_size` das
    /// `window::Settings` (nunca abre menor que o mínimo). No Wayland a posição
    /// não é restaurável (o protocolo não a expõe ao cliente), então lá só o
    /// tamanho volta.
    ///
    /// Substitui o padrão antigo de o app fazer isso à mão via
    /// [`GlacierDaemon::on_close`] + `window::Settings` montadas na inicialização.
    pub fn remember_window_geometry(mut self, enabled: bool) -> Self {
        self.remember_geometry = enabled;
        self
    }

    /// Sobe o daemon e roda o loop do iced até a última janela fechar.
    pub fn run(self) -> iced::Result {
        // Antes de QUALQUER coisa gráfica: o backend do `winit` (X11 vs
        // Wayland) é escolhido uma vez só, pro processo inteiro, na primeira
        // janela — inclusive a principal, que pode nascer bem antes de
        // qualquer `<script>` pedir uma `WindowSpec::webview`. Chegar tarde
        // demais pra forçar X11 (só quando alguém abre a webview) não
        // funciona: a essa altura o `EventLoop` já escolheu Wayland.
        forcar_x11_para_webview();

        // O `<app>`/`<tray>` do template principal, lidos ANTES de tudo: a
        // instância única decide aqui mesmo se o processo segue, e o diretório
        // de dados precisa existir antes do primeiro motor. Só dá para saber
        // qual é o template principal quando o `setup` é o padrão ou veio de
        // `main_template`; um `.main(|motor| …)` escrito à mão não diz.
        let manifest_path: Option<String> = match (&self.setup, &self.main_template_path) {
            (None, _) => Some(
                DEFAULT_MAIN_TEMPLATES
                    .iter()
                    .find(|p| self.assets.exists(p))
                    .unwrap_or(&DEFAULT_MAIN_TEMPLATES[0])
                    .to_string(),
            ),
            (Some(_), path) => path.clone(),
        };
        // Um erro de parse aqui é silencioso de propósito: o registro do mesmo
        // arquivo, logo depois, o reporta com o nome do componente.
        let (app_meta, tray_meta, screen_title) = manifest_path
            .as_deref()
            .and_then(|p| Some((p, self.assets.read_to_string(p).ok()?)))
            .and_then(|(p, conteudo)| crate::app_manifest(p, &conteudo).ok())
            .unwrap_or_default();

        // Checagem de instância única ANTES de qualquer coisa (winit, GPU,
        // motor) — uma segunda tentativa só precisa pingar a primeira e sair,
        // não vale gastar nada além disso. Ver [`GlacierDaemon::single_instance`].
        // O builder vence o `<app single_instance>`.
        let single_instance_id = self.single_instance_id.clone().or_else(|| {
            app_meta
                .as_ref()
                .filter(|app| app.single_instance)
                .map(|app| app.id.clone())
        });
        if let Some(app_id) = &single_instance_id
            && matches!(
                crate::single_instance::acquire(app_id),
                crate::single_instance::Lock::Secondary
            )
        {
            return Ok(());
        }

        let GlacierDaemon {
            title,
            main_settings,
            child_settings,
            setup,
            fonts,
            default_font,
            on_message,
            on_close,
            reload_period,
            toast_period,
            storage_dir,
            remember_geometry,
            tray_config,
            on_tray,
            assets,
            style,
            antialiasing,
            single_instance_id: _,
            main_template_path: _,
        } = self;

        // O que o builder não disse, o `<app>` diz. O builder vence sempre: é
        // código explícito, e o markup é o padrão de quando nada foi dito.
        let storage_dir = storage_dir.or_else(|| app_meta.as_ref().map(|app| app_data_dir(&app.id)));
        let remember_geometry =
            remember_geometry || app_meta.as_ref().is_some_and(|app| app.remember_geometry);

        // A `<tray>` do template, quando o builder não configurou uma. O ícone
        // é lido já, pela fonte de assets; sem ele não há bandeja — e o aviso
        // diz qual arquivo faltou, em vez de o app encerrar na última janela
        // sem explicação.
        let tray_markup: Option<(Vec<u8>, String, Vec<TrayItemDecl>)> = match (&tray_config, tray_meta) {
            (None, Some(meta)) => match assets.read_bytes(&meta.icon) {
                Ok(icon) => Some((
                    icon.into_owned(),
                    meta.tooltip
                        .clone()
                        .or_else(|| screen_title.clone())
                        .unwrap_or_else(|| title.clone()),
                    meta.items,
                )),
                Err(erro) => {
                    eprintln!("<tray icon=\"{}\">: não consegui ler o ícone: {erro}", meta.icon);
                    None
                }
            },
            _ => None,
        };
        let main_title = title.clone();

        // Sem `main` nem `main_template`: a principal é o primeiro template
        // padrão que a fonte de assets tiver. Sem nenhum, fica o primeiro, e o
        // erro do registro diz qual arquivo faltou.
        let setup: SetupHook = setup.unwrap_or_else(|| {
            let path = DEFAULT_MAIN_TEMPLATES
                .iter()
                .find(|p| assets.exists(p))
                .unwrap_or(&DEFAULT_MAIN_TEMPLATES[0]);
            Rc::new(template_setup(path.to_string()))
        });

        // Diretório onde a geometria da principal é persistida (só quando o app
        // ligou `remember_window_geometry` E definiu um `storage_dir` — é lá que
        // o arquivo mora). Guardado para o `Runtime` gravar ao fechar.
        let geometry_dir = remember_geometry.then(|| storage_dir.clone()).flatten();

        // A geometria salva, lida uma vez. Ela é aplicada lá dentro do `boot`,
        // **depois** do `<screen>` do template: o tamanho declarado no arquivo é
        // o de primeira abertura, enquanto a geometria lembrada é uma decisão que
        // o usuário tomou arrastando a janela — e essa ganha. (Aplicar depois
        // também deixa o `min-size` do template valer no clamp.)
        let saved_geometry = geometry_dir.as_ref().and_then(|dir| load_geometry(dir));

        // Semeia a raiz do `storage` ANTES de qualquer motor ser construído (o
        // `boot` abaixo e cada janela-filha em `build_engine` instalam o global
        // `storage` no registro do componente, lendo esta raiz já definida).
        if let Some(dir) = storage_dir {
            crate::luau::set_storage_root(dir);
        }

        // `boot` do iced: constrói o motor principal via `setup` e abre a janela
        // inicial. `window::open` devolve o `Id` de imediato, então já inserimos
        // o motor em `windows` com essa chave (o daemon não abre janela sozinho),
        // e guardamos esse `Id` como o da principal — ver `Runtime::main_id`.
        let boot = move || {
            let mut engine = GlacierUI::new().with_asset_source(assets.clone());
            apply_style(&mut engine, style.as_ref());
            setup(&mut engine);
            // O `setup` já registrou os componentes e definiu a tela inicial, e a
            // janela ainda não existe — esta é a única janela de tempo em que o
            // `<screen>` do template pode decidir título e tamanho **sem** o pulo
            // visível de abrir num tamanho e redimensionar depois.
            let mut main_settings = main_settings.clone();
            let mut main_title = main_title.clone();
            let base_title = main_title.clone();
            let initial_screen = engine.current_screen_name().map(str::to_string);
            let effective_size = resolve_main_window(
                engine.current_screen_meta(),
                saved_geometry.as_ref(),
                &mut main_settings,
                &mut main_title,
            );
            let sized_by_screen = initial_screen.zip(effective_size);
            if let Some(meta) = engine.current_screen_meta() {
                apply_window_icon(meta, &mut main_settings, assets.as_ref());
            }
            // A geometria só é consultada se o fechamento pedido pelo sistema
            // (Alt+F4, o × da moldura) passar pelo daemon — com
            // `exit_on_close_request: true` o iced fecha sozinho e ninguém fica
            // sabendo o tamanho. Então, quando alguém precisa dela, o pedido vem
            // para cá (ver `Runtime::close`), sem o app ter de lembrar disso.
            if geometry_dir.is_some() || on_close.is_some() {
                main_settings.exit_on_close_request = false;
            }
            let (id, open) = window::open(main_settings.clone());
            let mut rt = Runtime::new(
                reload_period,
                toast_period,
                id,
                setup.clone(),
                main_settings.clone(),
                main_title.clone(),
                assets.clone(),
            );
            rt.child_settings = child_settings.clone();
            rt.on_message = on_message.clone();
            rt.on_close = on_close.clone();
            rt.geometry_dir = geometry_dir.clone();
            rt.style = style;
            // Sobe a bandeja (thread própria) uma vez, no boot. Só a
            // configuração é `move`d para cá; a thread devolve a alça de
            // comandos, guardada para as atualizações de menu e o shutdown.
            if let Some(cfg) = tray_config.clone() {
                rt.tray = crate::tray::spawn(cfg);
                rt.on_tray = on_tray.clone();
            } else if let Some((icon, tooltip, items)) = tray_markup.clone() {
                // A `<tray>` do template: os rótulos com `{chave}` já saem
                // resolvidos contra o motor recém-montado, e daí em diante
                // `Runtime::sync_tray` manda à bandeja só o que mudar.
                let mut binds = TrayBindings::new(items);
                let cfg = binds.config(icon, tooltip, &engine);
                rt.tray = crate::tray::spawn(cfg);
                rt.on_tray = on_tray.clone();
                rt.tray_markup = Some(binds);
            }
            rt.titles.insert(id, main_title.clone());
            rt.base_titles.insert(id, base_title);
            if let Some(entry) = sized_by_screen {
                rt.sized_by.insert(id, entry);
            }
            rt.windows.insert(id, engine);
            // Consulta a preferência clara/escura do SO uma vez no boot — a
            // subscription `theme_changes` só emite em MUDANÇAS. Só afeta o
            // tema se o app não fixou um estilo (ver `GlacierUI::theme`).
            let probe = iced::system::theme().map(|mode| {
                DaemonMessage::TickAll(EngineMessage::SystemAppearanceChanged(
                    mode_to_appearance(mode),
                ))
            });
            (rt, Task::batch([open.map(DaemonMessage::Opened), probe]))
        };

        let mut app = iced::daemon(boot, Runtime::update, Runtime::view)
            .title(Runtime::title)
            .theme(Runtime::theme)
            .subscription(Runtime::subscription)
            .antialiasing(antialiasing);
        for bytes in fonts {
            app = app.font(bytes);
        }
        if let Some(font) = default_font {
            app = app.default_font(font);
        }
        app.run()
    }
}

impl Default for GlacierDaemon {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Persistência da geometria da janela principal (opt-in via
// `GlacierDaemon::remember_window_geometry`)
// ---------------------------------------------------------------------------

/// Nome do arquivo (sob o `storage_dir`) onde a geometria da principal é
/// gravada. Um arquivo por app — um daemon tem uma única janela principal.
const GEOMETRY_FILE: &str = "window-geometry.json";

/// Geometria lida do disco. `position` fica `None` quando não foi gravada (ex.:
/// Wayland, que nunca reporta a posição), caso em que só o tamanho é restaurado.
struct SavedGeometry {
    size: Size,
    position: Option<Point>,
}

/// Nunca abre a janela menor que o `min_size` das `window::Settings`: uma
/// geometria salva com um valor abaixo do mínimo (ou um `min_size` que cresceu
/// entre versões) não deve nascer espremida.
fn clamp_to_min(size: Size, min: Option<Size>) -> Size {
    match min {
        Some(min) => Size::new(size.width.max(min.width), size.height.max(min.height)),
        None => size,
    }
}

/// O par do [`clamp_to_min`] para o `max_size` (e o `fixed_size`, que o fixa):
/// uma geometria salva maior que o máximo declarado não abre a janela além dele.
fn clamp_to_max(size: Size, max: Option<Size>) -> Size {
    match max {
        Some(max) => Size::new(size.width.min(max.width), size.height.min(max.height)),
        None => size,
    }
}

/// Lê a geometria persistida sob `dir`, ou `None` se ausente/corrompida (é
/// "best effort" — nunca deve impedir o app de abrir). Exige ao menos
/// `width`/`height`; `x`/`y` são opcionais.
fn load_geometry(dir: &std::path::Path) -> Option<SavedGeometry> {
    let content = std::fs::read_to_string(dir.join(GEOMETRY_FILE)).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let width = v.get("width")?.as_f64()? as f32;
    let height = v.get("height")?.as_f64()? as f32;
    let x = v.get("x").and_then(serde_json::Value::as_f64);
    let y = v.get("y").and_then(serde_json::Value::as_f64);
    let position = match (x, y) {
        (Some(x), Some(y)) => Some(Point::new(x as f32, y as f32)),
        _ => None,
    };
    Some(SavedGeometry {
        size: Size::new(width, height),
        position,
    })
}

/// Grava a geometria sob `dir` (criando o diretório se preciso). Falhas de I/O
/// são logadas, não propagadas — não devem impedir a janela de fechar.
fn save_geometry(dir: &std::path::Path, size: Size, position: Option<Point>) {
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!(
            "[glacier-ui] geometria: falha ao criar '{}': {e}",
            dir.display()
        );
        return;
    }
    let json = serde_json::json!({
        "width": size.width,
        "height": size.height,
        "x": position.map(|p| p.x),
        "y": position.map(|p| p.y),
    });
    let path = dir.join(GEOMETRY_FILE);
    match serde_json::to_string_pretty(&json) {
        Ok(s) => {
            if let Err(e) = std::fs::write(&path, s) {
                eprintln!(
                    "[glacier-ui] geometria: falha ao gravar '{}': {e}",
                    path.display()
                );
            }
        }
        Err(e) => eprintln!("[glacier-ui] geometria: falha ao serializar: {e}"),
    }
}

/// Mensagem do daemon. Roteia eventos para o motor da janela certa. Note que
/// nenhuma variante carrega um [`WindowSpec`]/`Box<dyn Component>`: a janela é
/// materializada de imediato em `update` (com o `Id` síncrono de `window::open`),
/// então a mensagem só precisa carregar tipos `Clone`.
#[derive(Debug, Clone)]
pub enum DaemonMessage {
    /// Um [`EngineMessage`] destinado ao motor da janela `id`.
    Ui { id: window::Id, msg: EngineMessage },
    /// Uma janela terminou de abrir (retorno de `window::open`). Só informativo.
    Opened(window::Id),
    /// Uma janela foi fechada (via `window::close_events`): remove o motor e,
    /// se era a última, encerra o app.
    Closed(window::Id),
    /// A OS/WM pediu para fechar uma janela (`window::close_requests`, ANTES do
    /// fechamento). Na principal, dá a chance de [`GlacierDaemon::on_close`]
    /// rodar com a geometria; nas demais, fecha direto.
    CloseRequested(window::Id),
    /// A geometria consultada em resposta a um `CloseRequested` da principal
    /// chegou: entrega-a ao gancho `on_close` e então fecha a janela.
    CloseWithGeometry(window::Id, Size, Option<Point>),
    /// Tick periódico aplicado a **todas** as janelas (hot-reload, expiração de
    /// toasts) — cada motor checa os próprios arquivos/toasts.
    TickAll(EngineMessage),
    /// Um evento da bandeja (clique de menu ou no ícone). Ver [`crate::tray`].
    Tray(TrayMsg),
    /// Uma segunda tentativa de lançar o app pingou esta instância. Ver
    /// [`crate::single_instance`] / [`GlacierDaemon::single_instance`].
    ActivateRequested,
    /// Uma mensagem injetada de fora do loop do iced, por outra thread do
    /// próprio app (servidor local, watcher, integração com o SO). Vai sempre
    /// para o motor da janela PRINCIPAL. Ver [`crate::external`].
    External(EngineMessage),
    /// Uma janela de webview (`webview_ids`) foi redimensionada — a `wry`
    /// não segue o tamanho da janela sozinha sob X11 (ver `crate::webview`),
    /// então o daemon reposiciona manualmente. Ignorado para qualquer `Id`
    /// fora de `webview_ids` (uma janela comum já resolve tamanho pelo
    /// próprio layout do `iced`).
    WebviewResized(window::Id, Size),
    /// Tick para avançar o loop do GTK (ver `crate::webview::pump`) — só
    /// registrado enquanto houver alguma janela de webview aberta. Sem
    /// carga: o `update` só precisa saber que é hora de bombear.
    WebviewPumpGtk,
}

/// Converte o `iced::theme::Mode` (preferência do SO) no enum do motor, sem
/// vazar o tipo do iced pelo [`EngineMessage`].
fn mode_to_appearance(mode: iced::theme::Mode) -> crate::widget::SystemAppearance {
    match mode {
        iced::theme::Mode::Light => crate::widget::SystemAppearance::Light,
        iced::theme::Mode::Dark => crate::widget::SystemAppearance::Dark,
        iced::theme::Mode::None => crate::widget::SystemAppearance::Unknown,
    }
}

/// Estado do daemon: um motor por janela + seus títulos.
struct Runtime {
    windows: HashMap<window::Id, GlacierUI>,
    titles: HashMap<window::Id, String>,
    /// O título de cada janela **sem** o que o `<screen>` diz: o do builder, para
    /// a principal, ou o do `WindowSpec`/nome do arquivo, para as filhas. É para
    /// cá que o título volta ao navegar para uma tela que não declara `title`.
    base_titles: HashMap<window::Id, String>,
    /// Por janela, qual tela ditou o tamanho atual e qual foi o valor aplicado.
    /// Serve para o hot-reload: salvar o arquivo só redimensiona a janela quando
    /// o número escrito no `<screen>` mudou de fato — senão cada `Ctrl+S`
    /// desfaria o arrasto que o usuário acabou de fazer no canto da janela.
    sized_by: HashMap<window::Id, (String, (f32, f32))>,
    /// `Id` da janela principal, conhecido já no `boot` (`window::open` o
    /// devolve síncrono). Tê-lo em mãos evita um round-trip `window::latest()`
    /// por ação de janela — e no Wayland esse adiamento **quebra** o arrasto:
    /// o compositor exige que `window::drag` seja pedido com o serial do
    /// pointer-grab ainda vivo, e um round-trip o perde, fazendo o
    /// `onPress="window:drag"` da titlebar custom virar um no-op silencioso.
    main_id: window::Id,
    child_settings: Option<ChildSettingsHook>,
    on_message: Option<MessageHook>,
    on_close: Option<CloseHook>,
    /// Diretório onde a geometria da principal é persistida, `Some` quando o app
    /// ligou [`GlacierDaemon::remember_window_geometry`] com um `storage_dir`.
    /// Gravado ao fechar (ver `DaemonMessage::CloseWithGeometry`).
    geometry_dir: Option<PathBuf>,
    reload_period: Duration,
    toast_period: Duration,
    /// Alça da bandeja, `Some` quando ela subiu. Enquanto for `Some`, o app
    /// **não** encerra ao fechar a última janela (recolhe para a bandeja).
    tray: Option<TrayHandle>,
    /// Gancho de clique dos itens da bandeja.
    on_tray: Option<TrayHook>,
    /// A bandeja declarada numa `<tray>` do template principal, quando foi de
    /// lá que ela veio: a ação de cada item e os rótulos já enviados.
    tray_markup: Option<TrayBindings>,
    /// O `setup`/`settings`/`título` da principal, guardados para **reabri-la**
    /// (o "Open Rustploy" da bandeja) idêntica à do boot.
    main_setup: SetupHook,
    main_settings: window::Settings,
    main_title: String,
    /// A janela principal está **na tela** (`true`) ou **recolhida na bandeja**
    /// (`false`)? Alternado no ramo `DaemonMessage::Closed` de
    /// [`Runtime::update`] (recolhe) e em [`Runtime::open_main`] (reabre).
    ///
    /// Quando recolhida, a janela do SO foi destruída (no Wayland esconder é
    /// impossível — a única forma de sumir de verdade é destruir), mas o
    /// **motor** dela continua vivo em `windows`, sob o `main_id` já morto: assim
    /// o SSE segue conectado e o login intacto, e as notificações de deploy
    /// continuam chegando mesmo sem janela. O "Open Rustploy" religa esse mesmo
    /// motor numa janela nova (ver [`Runtime::open_main`]).
    main_shown: bool,
    /// Fonte de assets herdada do [`GlacierDaemon`], injetada em cada motor novo
    /// (reabertura da principal e janelas-filhas).
    assets: Arc<dyn AssetSource>,
    /// Estilo builtin herdado de [`GlacierDaemon::style`], aplicado a cada
    /// motor novo (reabertura da principal e janelas-filhas).
    style: Option<crate::style::Style>,
    /// Janelas de `open_window({ webview_url = ... })` cujo `window::open` já
    /// devolveu o `Id`, mas cuja `wry::WebView` ainda não foi criada — drenado
    /// no primeiro `DaemonMessage::Opened(id)` que bater aqui. Guarda a URL e
    /// o tamanho lógico inicial: não há como consultar o tamanho real da
    /// janela a partir do handle cru que `iced::window::run` entrega (ver
    /// `crate::webview`). Só existe com a feature `webview` — sem ela,
    /// `open_webview_child` nunca teria o que inserir aqui.
    #[cfg(feature = "webview")]
    pending_webviews: HashMap<window::Id, (String, (f32, f32))>,
    /// Todo `window::Id` que é uma janela de webview PURA (sem `GlacierUI`
    /// atrás — não roda `.gv`/`<script>`). `view`/`title`/`theme` já degradam
    /// graciosamente para um `Id` ausente de `windows`, mas o daemon precisa
    /// saber quais IDs são estes para (a) rotear `window::resize_events` só a
    /// eles — as demais janelas resolvem tamanho pelo próprio layout do
    /// `iced` — e (b) não contar uma webview fechada como "ainda tem janela
    /// de verdade aberta" ao decidir encerrar o app (ver `DaemonMessage::Closed`).
    webview_ids: std::collections::HashSet<window::Id>,
}

impl Runtime {
    fn new(
        reload_period: Duration,
        toast_period: Duration,
        main_id: window::Id,
        main_setup: SetupHook,
        main_settings: window::Settings,
        main_title: String,
        assets: Arc<dyn AssetSource>,
    ) -> Self {
        Self {
            windows: HashMap::default(),
            titles: HashMap::default(),
            base_titles: HashMap::default(),
            sized_by: HashMap::default(),
            main_id,
            child_settings: None,
            on_message: None,
            on_close: None,
            geometry_dir: None,
            reload_period,
            toast_period,
            tray: None,
            on_tray: None,
            tray_markup: None,
            main_setup,
            main_settings,
            main_title,
            main_shown: true,
            assets,
            style: None,
            #[cfg(feature = "webview")]
            pending_webviews: HashMap::default(),
            webview_ids: std::collections::HashSet::default(),
        }
    }

    fn update(&mut self, message: DaemonMessage) -> Task<DaemonMessage> {
        match message {
            DaemonMessage::Ui { id, msg } => self.route(id, msg),
            // Sempre na principal — inclusive quando ela está recolhida na
            // bandeja, caso em que o motor segue vivo sob o `main_id` e só a
            // janela sumiu. É o que mantém um app de bandeja dirigível de fora.
            DaemonMessage::External(msg) => self.route(self.main_id, msg),
            DaemonMessage::Opened(id) => {
                // Só uma janela de `open_window({ webview_url = ... })` chega
                // aqui com uma entrada em `pending_webviews` — nunca populada
                // sem a feature `webview` (ver `open_webview_child`). A
                // criação de verdade só pode acontecer dentro de
                // `iced::window::run`: é o único lugar com acesso ao handle
                // nativo da janela (ver `crate::webview::create`).
                #[cfg(feature = "webview")]
                if let Some((url, size)) = self.pending_webviews.remove(&id) {
                    return iced::window::run(id, move |w| {
                        crate::webview::create(id, w, &url, size);
                    })
                    .discard();
                }
                #[cfg(not(feature = "webview"))]
                let _ = id;
                Task::none()
            }
            DaemonMessage::WebviewResized(id, size) => {
                #[cfg(feature = "webview")]
                if self.webview_ids.contains(&id) {
                    let wh = (size.width, size.height);
                    return iced::window::run(id, move |_w| {
                        crate::webview::resize(id, wh);
                    })
                    .discard();
                }
                #[cfg(not(feature = "webview"))]
                let _ = (id, size);
                Task::none()
            }
            DaemonMessage::WebviewPumpGtk => {
                #[cfg(feature = "webview")]
                crate::webview::pump();
                Task::none()
            }
            DaemonMessage::Closed(id) => {
                // A janela PRINCIPAL fechando com bandeja: não encerra nem
                // descarta o motor — **destaca-o** (headless), mantendo SSE +
                // login vivos para as notificações continuarem chegando. Só o
                // título sai; o motor fica em `windows` sob o `main_id` morto.
                if id == self.main_id && self.tray.is_some() {
                    self.titles.remove(&id);
                    self.base_titles.remove(&id);
                    self.sized_by.remove(&id);
                    self.main_shown = false;
                    return Task::none();
                }
                // Demais janelas (filhas), ou sem bandeja: remove o motor. Sem
                // bandeja, a última janela fechada encerra o app (como sempre).
                self.windows.remove(&id);
                self.titles.remove(&id);
                self.base_titles.remove(&id);
                self.sized_by.remove(&id);
                // Uma janela de webview não tem motor em `windows` (por isso
                // precisa da limpeza própria), mas ainda conta como "janela de
                // verdade aberta" para não encerrar o app debaixo dela.
                if self.webview_ids.remove(&id) {
                    #[cfg(feature = "webview")]
                    crate::webview::destroy(id);
                }
                if self.windows.is_empty() && self.webview_ids.is_empty() && self.tray.is_none() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            // A WM pediu para fechar (Alt+F4, botão da barra, fim de sessão).
            DaemonMessage::CloseRequested(id) => self.close(id),
            DaemonMessage::CloseWithGeometry(id, size, position) => {
                if let (Some(hook), Some(engine)) = (&self.on_close, self.windows.get(&id)) {
                    hook(engine, WindowGeometry { size, position });
                }
                // Se é a principal, lembra a geometria atual: se ela for recolher
                // para a bandeja, a reabertura ("Open Rustploy") nasce do mesmo
                // tamanho/posição em vez do valor do boot. (Posição é `None` no
                // Wayland — lá só o tamanho é lembrado.)
                if id == self.main_id {
                    self.main_settings.size = size;
                    if let Some(p) = position {
                        self.main_settings.position = window::Position::Specific(p);
                    }
                    // Persistência nativa da geometria (opt-in via
                    // `remember_window_geometry`): grava para o próximo boot
                    // reabrir aqui. Best-effort — falha de I/O não impede fechar.
                    if let Some(dir) = &self.geometry_dir {
                        save_geometry(dir, size, position);
                    }
                }
                window::close(id)
            }
            DaemonMessage::TickAll(msg) => {
                // Aplica o tick a cada janela (clonando a mensagem por janela).
                let ids: Vec<window::Id> = self.windows.keys().copied().collect();
                let tasks: Vec<_> = ids
                    .into_iter()
                    .map(|id| self.route(id, msg.clone()))
                    .collect();
                Task::batch(tasks)
            }
            DaemonMessage::Tray(msg) => self.on_tray(msg),
            DaemonMessage::ActivateRequested => self.open_main(),
        }
    }

    /// Trata um evento da bandeja. O gancho `on_tray` roda com um
    /// [`TrayActions`] e só **registra** a intenção (abrir a principal / sair);
    /// aqui a traduzimos numa `Task`. O borrow imutável do gancho/handle termina
    /// antes de mexermos em `self` (abrir janela), por isso o `request` é
    /// extraído primeiro.
    fn on_tray(&mut self, msg: TrayMsg) -> Task<DaemonMessage> {
        let request = match msg {
            // Clique esquerdo no ícone (Windows): reabre a principal direto.
            TrayMsg::IconLeftClick => Some(TrayRequest::OpenMain),
            TrayMsg::Menu(id) => match (&self.on_tray, &self.tray) {
                (Some(hook), Some(handle)) => {
                    let mut actions = TrayActions::new(handle);
                    hook(&id, &mut actions);
                    actions.request
                }
                // Sem gancho em Rust, quem decide é o `on_click` do item na
                // `<tray>` do template.
                (None, Some(_)) => {
                    let acao = self
                        .tray_markup
                        .as_ref()
                        .and_then(|binds| binds.action(&id))
                        .map(str::to_string);
                    match acao.as_deref() {
                        Some("tray:open") => Some(TrayRequest::OpenMain),
                        Some("tray:quit") => Some(TrayRequest::Quit),
                        Some("notifications:toggle") => {
                            crate::tray::set_notifications_enabled(
                                !crate::tray::notifications_enabled(),
                            );
                            self.sync_tray();
                            None
                        }
                        // Qualquer outra ação vai ao script/`update` da tela
                        // principal — cujo motor segue vivo mesmo com a janela
                        // recolhida na bandeja (ver `main_shown`).
                        Some(outra) => {
                            let main_id = self.main_id;
                            return self.route(main_id, EngineMessage::UiClick(outra.to_string()));
                        }
                        None => None,
                    }
                }
                _ => None,
            },
        };
        match request {
            Some(TrayRequest::OpenMain) => self.open_main(),
            Some(TrayRequest::Quit) => {
                if let Some(tray) = &self.tray {
                    tray.shutdown();
                }
                iced::exit()
            }
            None => Task::none(),
        }
    }

    /// Reavalia os rótulos e marcações da `<tray>` do template contra o motor da
    /// principal e manda à bandeja o que mudou. No-op sem bandeja em markup.
    fn sync_tray(&mut self) {
        let (Some(handle), Some(binds)) = (&self.tray, &mut self.tray_markup) else {
            return;
        };
        if let Some(engine) = self.windows.get(&self.main_id) {
            binds.sync(handle, engine);
        }
    }

    /// Reabre (ou foca, se já visível) a janela principal — o "Open Rustploy" da
    /// bandeja.
    ///
    /// - **Já visível** (`main_shown`): traz pra frente. `gain_focus` (via
    ///   `focus_window` do winit) rouba o foco de verdade no X11 (manda
    ///   `_NET_ACTIVE_WINDOW`), mas é **no-op no Wayland nativo** — o protocolo
    ///   não deixa um cliente ativar a janela de outro à força, de propósito
    ///   (mesma classe de restrição do `window:drag`, já documentada no
    ///   projeto). Por isso soma `request_user_attention(Critical)`: no
    ///   Wayland o winit implementa isso via `xdg_activation_v1` — o cliente
    ///   pede um token pra própria superfície e se auto-ativa —, que os
    ///   compositores (Mutter, KWin) honram; no X11 vira `XUrgencyHint`
    ///   (inofensivo, já que `gain_focus` ali já resolve sozinho). As duas
    ///   tasks somadas cobrem os três casos (X11 ativa, Wayland ativa via
    ///   xdg_activation, e o fallback nas plataformas sem nenhum dos dois é só
    ///   o pisca de atenção nativo do SO).
    /// - **Recolhida na bandeja**: o motor foi destacado e continua vivo em
    ///   `windows` sob o `main_id` morto (ver o campo `main_shown`).
    ///   Aqui ele é **religado** numa janela nova — preservando login e a sessão
    ///   SSE — e o `main_id` migra para o id da janela nova. (O recipe do SSE
    ///   inclui o id da janela, então a migração provoca um breve reconnect do
    ///   stream; irrelevante, pois é justamente o momento da reabertura.)
    /// - **Sem motor retido** (partida a frio, ex.: nunca houve principal): um
    ///   motor novo é construído via o `setup` guardado.
    fn open_main(&mut self) -> Task<DaemonMessage> {
        if self.main_shown {
            return Task::batch([
                window::gain_focus(self.main_id),
                window::request_user_attention(self.main_id, Some(window::UserAttention::Critical)),
            ]);
        }
        // Reusa o motor destacado (login + SSE preservados) ou, se não houver,
        // constrói do zero.
        let engine = self.windows.remove(&self.main_id).unwrap_or_else(|| {
            let mut e = GlacierUI::new().with_asset_source(self.assets.clone());
            apply_style(&mut e, self.style.as_ref());
            (self.main_setup)(&mut e);
            e
        });
        let (id, open) = window::open(self.main_settings.clone());
        self.main_id = id;
        self.main_shown = true;
        self.titles.insert(id, self.main_title.clone());
        self.base_titles.insert(id, self.main_title.clone());
        self.windows.insert(id, engine);
        open.map(DaemonMessage::Opened)
    }

    /// Fecha a janela `id`. Na principal, quando algo precisa da geometria ao
    /// fechar — um gancho `on_close` OU a persistência nativa ligada por
    /// [`GlacierDaemon::remember_window_geometry`] —, primeiro **consulta** a
    /// geometria de verdade e só fecha depois de entregá-la (ver
    /// `DaemonMessage::CloseWithGeometry`). Sem nenhum dos dois, fecha direto.
    fn close(&mut self, id: window::Id) -> Task<DaemonMessage> {
        if id != self.main_id || !self.needs_geometry_on_close() {
            return window::close(id);
        }
        window::size(id).then(move |size| {
            window::position(id)
                .map(move |position| DaemonMessage::CloseWithGeometry(id, size, position))
        })
    }

    /// Se o fechamento da principal precisa **consultar** a geometria antes de
    /// fechar: quando há um gancho `on_close` OU a persistência nativa está
    /// ligada ([`GlacierDaemon::remember_window_geometry`], que semeia o
    /// `geometry_dir`). Sem nenhum dos dois, fechar não precisa da geometria.
    fn needs_geometry_on_close(&self) -> bool {
        self.on_close.is_some() || self.geometry_dir.is_some()
    }

    /// Despacha `msg` ao motor da janela `id` e, em seguida, abre quaisquer
    /// janelas que aquele motor tenha pedido durante o `dispatch`.
    fn route(&mut self, id: window::Id, msg: EngineMessage) -> Task<DaemonMessage> {
        // Controles de janela da titlebar custom (`window:drag`, `window:close`,
        // `window:resize:se`, …) são tratados AQUI, contra o `Id` da janela em
        // roteamento, e não dentro do motor — que, sem saber em qual janela vive,
        // teria de resolvê-lo via `window::latest()` e perderia o pointer-grab
        // serial no Wayland (ver `Runtime::main_id`). O `close` ainda passa por
        // `Runtime::close`, para o gancho `on_close` poder salvar a geometria.
        if let EngineMessage::UiClick(action) = &msg
            && let Some(cmd) = action.strip_prefix("window:")
        {
            return match cmd {
                "close" => self.close(id),
                _ => window_control(id, cmd),
            };
        }

        // 1. despacha ao motor da janela (borrow escopado)
        let ui_task = match self.windows.get_mut(&id) {
            Some(engine) => engine
                .dispatch(&msg)
                .map(move |m| DaemonMessage::Ui { id, msg: m }),
            None => return Task::none(),
        };

        // Observador de persistência: depois do dispatch (o interesse é o estado
        // resultante), e só na principal — é lá que vive o formulário cujo
        // estado o app quer guardar.
        if id == self.main_id
            && let (Some(hook), Some(engine)) = (&self.on_message, self.windows.get(&id))
        {
            // Com `GLACIER_PERF`, o gancho tem parcela própria: ele roda na
            // thread da UI, e um que bloqueie (um lock disputado, um I/O
            // síncrono) trava o quadro sem aparecer no render nem no dispatch.
            if crate::perf::ligado() {
                let t0 = std::time::Instant::now();
                hook(&msg, engine);
                crate::perf::anota_app(t0.elapsed());
            } else {
                hook(&msg, engine);
            }
        }

        let mut tasks = vec![ui_task];

        // Um rótulo da `<tray>` com `{chave}` pode ter mudado com esta ação.
        if id == self.main_id {
            self.sync_tray();
        }

        // 2. drena os pedidos de janela nova desse mesmo motor e abre cada um
        let pending = self
            .windows
            .get_mut(&id)
            .map(|e| e.take_pending_windows())
            .unwrap_or_default();
        for spec in pending {
            tasks.push(self.open_child(spec));
        }

        // 3. drena os broadcasts desse motor e entrega às OUTRAS janelas
        let broadcasts = self
            .windows
            .get_mut(&id)
            .map(|e| e.take_pending_broadcasts())
            .unwrap_or_default();
        if !broadcasts.is_empty() {
            let others: Vec<window::Id> =
                self.windows.keys().copied().filter(|w| *w != id).collect();
            for b in &broadcasts {
                for &oid in &others {
                    if let Some(engine) = self.windows.get_mut(&oid) {
                        tasks.push(
                            engine
                                .deliver_broadcast(&b.event, &b.payload)
                                .map(move |m| DaemonMessage::Ui { id: oid, msg: m }),
                        );
                    }
                }
            }
        }

        // 4. se o motor pediu para fechar a própria janela (`close_window()` na
        // Lua), fecha — pela mesma porta do botão da titlebar, para o gancho
        // `on_close` da principal também valer aqui.
        if self
            .windows
            .get_mut(&id)
            .map(|e| e.take_close_requested())
            .unwrap_or(false)
        {
            tasks.push(self.close(id));
        }

        // 5. a tela pode ter mudado (navegação) ou o arquivo pode ter sido salvo
        //    (hot-reload): reacerta o que a janela mostra do que a tela declara.
        tasks.push(self.sync_window_meta(id));

        Task::batch(tasks)
    }

    /// Reacerta a janela `id` com o que a **tela ativa** declara no `<screen>`.
    ///
    /// O título acompanha a navegação: entrar numa tela que declara `title` troca
    /// o da janela, e sair dela para uma que não declara nada devolve o título
    /// base (o do builder, ou o de quem abriu a janela).
    ///
    /// O tamanho é outra história — quem decide o tamanho de uma janela é quem a
    /// abre, uma vez. Navegar não redimensiona (seria hostil no meio do uso), e
    /// o hot-reload só redimensiona quando o número mudou no arquivo, e só para a
    /// tela que ditou o tamanho desta janela: sem isso, cada `Ctrl+S` desfaria o
    /// arrasto que o usuário acabou de dar no canto da janela.
    fn sync_window_meta(&mut self, id: window::Id) -> Task<DaemonMessage> {
        // A principal recolhida na bandeja mantém o motor vivo sob um `Id` morto
        // (ver `main_shown`), e continua recebendo ticks. Sem esta guarda, o
        // primeiro tick devolveria o título dela ao mapa que o `Closed` acabou de
        // limpar — título de janela que não está na tela.
        if !self.titles.contains_key(&id) {
            return Task::none();
        }
        let Some(engine) = self.windows.get(&id) else {
            return Task::none();
        };
        let screen = engine.current_screen_name().map(str::to_string);
        let meta = engine.current_screen_meta().cloned();

        let base = self.base_titles.get(&id).cloned();
        if let Some(wanted) = meta.as_ref().and_then(|m| m.title.clone()).or(base)
            && self.titles.get(&id) != Some(&wanted)
        {
            self.titles.insert(id, wanted);
        }

        let (Some(screen), Some(size)) = (screen, meta.and_then(|m| m.effective_size())) else {
            return Task::none();
        };
        match self.sized_by.get(&id) {
            Some((owner, applied)) if *owner == screen && *applied != size => {
                self.sized_by.insert(id, (screen, size));
                window::resize(id, Size::new(size.0, size.1))
            }
            _ => Task::none(),
        }
    }

    /// Materializa um [`WindowSpec`] numa janela nova: constrói um motor fresco,
    /// abre a janela (o `Id` vem síncrono) e registra motor + título.
    fn open_child(&mut self, spec: WindowSpec) -> Task<DaemonMessage> {
        // Uma webview não tem motor nenhum por trás — é uma janela nativa
        // pura (ver `WindowSource::WebView`), então segue por um caminho bem
        // mais curto, à parte do resto desta função (que é toda sobre montar
        // um `GlacierUI`).
        if let WindowSource::WebView(url) = spec.source {
            return self.open_webview_child(url, spec.title, spec.size, spec.resizable);
        }
        // O motor primeiro: é ele que sabe o que o `<screen>` do arquivo declara,
        // e a janela ainda não abriu — mesma janela de tempo que o boot usa para
        // a principal. Assim `open_window({ file = "detalhe.gv" })` herda título e
        // tamanho de `detalhe.gv`, em vez de exigir que quem chama os repita.
        // O hook `child_window` roda depois de o motor existir (é dele que vem o
        // meta), mas `build_engine` já terá consumido o `source` — um
        // `Component` é um Box que precisa ser movido para dentro do motor. O
        // eco recria fielmente as duas formas clonáveis, que são as que um app
        // olha para decidir a aparência da filha; um `Component` vira o `Named`
        // do próprio nome.
        let echo_source = match &spec.source {
            WindowSource::File(path) => WindowSource::File(path.clone()),
            WindowSource::Named(name) => WindowSource::Named(name.clone()),
            WindowSource::Component(comp) => WindowSource::Named(comp.name().to_string()),
            // Inalcançável: já retornamos acima para este caso.
            WindowSource::WebView(url) => WindowSource::WebView(url.clone()),
        };
        let WindowSpec {
            source,
            title,
            size,
            resizable,
            data,
        } = spec;
        let (engine, fallback_title) =
            build_engine(source, &data, self.assets.clone(), self.style.as_ref());
        let meta = engine.current_screen_meta().cloned().unwrap_or_default();
        let screen = engine.current_screen_name().map(str::to_string);

        // Quem abre a janela ainda manda: sabe do contexto que o arquivo não sabe
        // ("Editando nginx"). O `<screen>` é o padrão de quando não disseram nada.
        let (w, h) = size.or(meta.size).unwrap_or((640.0, 480.0));
        let mut settings = window::Settings {
            size: Size::new(w, h),
            resizable: resizable && meta.resizable.unwrap_or(true),
            min_size: meta.min_size.map(|(w, h)| Size::new(w, h)),
            ..window::Settings::default()
        };
        // `max_size`, `fixed_size`, `decorations` e `icon` do `<screen>` da
        // filha: cada janela declara a própria moldura, como a principal.
        apply_window_bounds(&meta, &mut settings);
        apply_window_icon(&meta, &mut settings, self.assets.as_ref());
        // O app tem a última palavra sobre a aparência da filha (ex.: também
        // borderless, num app com titlebar própria).
        if let Some(f) = &self.child_settings {
            let echo = WindowSpec {
                source: echo_source,
                title: title.clone(),
                size,
                resizable,
                data,
            };
            f(&echo, &mut settings);
        }

        let (id, open) = window::open(settings);
        let base_title = title.unwrap_or(fallback_title);
        self.titles
            .insert(id, meta.title.clone().unwrap_or_else(|| base_title.clone()));
        self.base_titles.insert(id, base_title);
        if let Some(entry) = screen.zip(size.or(meta.effective_size())) {
            self.sized_by.insert(id, entry);
        }
        self.windows.insert(id, engine);
        open.map(DaemonMessage::Opened)
    }

    /// A metade de [`Runtime::open_child`] para `WindowSource::WebView`: abre
    /// a janela do SO sem nenhum `GlacierUI` atrás dela. A `wry::WebView` em
    /// si só é criada depois, quando `window::open` avisar que a janela
    /// terminou de abrir (`DaemonMessage::Opened`) — é só então que
    /// `iced::window::run` consegue um handle nativo para ela (ver
    /// `crate::webview::create`).
    fn open_webview_child(
        &mut self,
        url: String,
        title: Option<String>,
        size: Option<(f32, f32)>,
        resizable: bool,
    ) -> Task<DaemonMessage> {
        let (w, h) = size.unwrap_or((960.0, 600.0));
        let mut settings = window::Settings {
            size: Size::new(w, h),
            resizable,
            ..window::Settings::default()
        };
        if let Some(f) = &self.child_settings {
            let echo = WindowSpec {
                source: WindowSource::WebView(url.clone()),
                title: title.clone(),
                size,
                resizable,
                data: Vec::new(),
            };
            f(&echo, &mut settings);
        }

        let (id, open) = window::open(settings);
        let base_title = title.unwrap_or_default();
        self.titles.insert(id, base_title.clone());
        self.base_titles.insert(id, base_title);

        #[cfg(feature = "webview")]
        {
            self.webview_ids.insert(id);
            self.pending_webviews.insert(id, (url, (w, h)));
        }
        #[cfg(not(feature = "webview"))]
        {
            eprintln!(
                "open_window: `webview_url` pedido ({url}), mas o glacier-ui foi \
                 compilado sem `--features webview` — a janela abriu sem conteúdo."
            );
        }

        open.map(DaemonMessage::Opened)
    }

    fn view(&self, id: window::Id) -> Element<'_, DaemonMessage> {
        match self.windows.get(&id) {
            Some(engine) => match engine.render_current() {
                Ok(elem) => elem.map(move |msg| DaemonMessage::Ui { id, msg }),
                Err(e) => iced::widget::text(format!("Erro ao renderizar: {e}"))
                    .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                    .into(),
            },
            None => iced::widget::text("").into(),
        }
    }

    fn title(&self, id: window::Id) -> String {
        self.titles
            .get(&id)
            .cloned()
            .unwrap_or_else(|| "Glacier".to_string())
    }

    fn theme(&self, id: window::Id) -> iced::Theme {
        self.windows
            .get(&id)
            .map(|e| e.theme())
            .unwrap_or(iced::Theme::Dark)
    }

    fn subscription(&self) -> Subscription<DaemonMessage> {
        // Listeners globais de evento, registrados UMA vez no daemon: usam o
        // `window::Id` que o callback recebe para rotear ao motor certo. Se cada
        // motor os registrasse, o iced fundiria os recipes idênticos num só.
        let mut subs = vec![
            iced::event::listen_with(|e, s, id| {
                crate::drag_end_from_event(e, s, id).map(|msg| DaemonMessage::Ui { id, msg })
            }),
            iced::event::listen_with(|e, s, id| {
                crate::timeedit_key_from_event(e, s, id).map(|msg| DaemonMessage::Ui { id, msg })
            }),
            iced::event::listen_with(|e, s, id| {
                crate::tab_focus_from_event(e, s, id).map(|msg| DaemonMessage::Ui { id, msg })
            }),
            iced::event::listen_with(|e, s, id| {
                crate::viewport_from_event(e, s, id).map(|msg| DaemonMessage::Ui { id, msg })
            }),
            iced::event::listen_with(|e, s, id| {
                crate::menu_escape_from_event(e, s, id).map(|msg| DaemonMessage::Ui { id, msg })
            }),
            // O sexto, e o habilitador B da Onda 9: `<shortcut>` e
            // `<shortcutinput>` são o MESMO listener em dois modos, e quem
            // decide qual é o contexto (ver `crate::keys`).
            iced::event::listen_with(|e, s, id| {
                crate::shortcut_from_event(e, s, id).map(|msg| DaemonMessage::Ui { id, msg })
            }),
            window::close_events().map(DaemonMessage::Closed),
            // A `wry` não redimensiona a webview sozinha sob X11 (ver
            // `crate::webview::resize`); esta é a fonte que aciona esse
            // reposicionamento. Registrada sempre (mesmo sem nenhuma janela
            // de webview aberta) porque é barata e evita alternar a lista de
            // subscriptions a cada `open_window`/fechamento — o handler no
            // `update` já descarta qualquer `Id` fora de `webview_ids`.
            window::resize_events().map(|(id, size)| DaemonMessage::WebviewResized(id, size)),
            // O pedido de fechar da WM (Alt+F4, botão da barra, logout) — chega
            // ANTES do fechamento, que é o único momento em que ainda dá para
            // consultar a geometria da janela para o gancho `on_close`. Só tem
            // efeito se a janela declarar `exit_on_close_request: false`.
            window::close_requests().map(DaemonMessage::CloseRequested),
            // Preferência clara/escura do SO: quando o app não fixou um estilo,
            // é ela que decide `Theme::Light`/`Theme::Dark` (ver
            // `GlacierUI::theme`). Registrada uma vez no daemon, como os demais
            // listeners globais — cada mudança vai para TODAS as janelas.
            iced::system::theme_changes()
                .map(|mode| DaemonMessage::TickAll(EngineMessage::SystemAppearanceChanged(
                    mode_to_appearance(mode),
                ))),
        ];

        // O movimento do mouse só é escutado quando alguma janela tem menu em
        // jogo. Cada evento vira mensagem, e no `iced` cada mensagem vira um
        // quadro: escutar sempre fazia o app redesenhar ~100 vezes por segundo
        // enquanto o cursor atravessava a tela, sem nada ter mudado. Ver
        // `GlacierUI::precisa_do_cursor`.
        if self.windows.values().any(|e| e.precisa_do_cursor()) {
            subs.push(iced::event::listen_with(|e, s, id| {
                crate::cursor_from_event(e, s, id).map(|msg| DaemonMessage::Ui { id, msg })
            }));
        }

        // Bombeia o loop do GTK (ver `crate::webview::pump`) só enquanto
        // alguma janela de webview estiver aberta — mesma economia condicional
        // do cursor acima. 16ms ~ um quadro a 60Hz: rápido o bastante para a
        // página parecer viva (scroll, vídeo, JS), sem gastar CPU à toa quando
        // nenhuma webview existe.
        if !self.webview_ids.is_empty() {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(16))
                    .map(|_| DaemonMessage::WebviewPumpGtk),
            );
        }

        // Cada tick força um redraw da tela inteira em TODAS as janelas (é
        // como o loop do iced funciona: qualquer Message processada reconstrói
        // a árvore de widgets via `view()`, ver `iced_winit`). Num fallback
        // puramente por software (sem GPU compatível) isso é um custo real, e
        // repetido para nada quando o tick não tinha trabalho a fazer — então
        // cada ticker só entra quando pode genuinamente ter efeito.
        if self
            .windows
            .values()
            .any(|engine| engine.assets.supports_reload())
        {
            subs.push(
                iced::time::every(self.reload_period)
                    .map(|_| DaemonMessage::TickAll(EngineMessage::FileChanged(String::new()))),
            );
        }
        if self
            .windows
            .values()
            .any(|engine| !engine.toasts.is_empty())
        {
            subs.push(
                iced::time::every(self.toast_period)
                    .map(|_| DaemonMessage::TickAll(EngineMessage::ToastTick)),
            );
        }

        // O relógio do `<delaybutton>` (Onda 9), e só enquanto um estiver
        // apertado — a mesma economia condicional do cursor logo acima. 30ms dá
        // um anel liso o bastante para o olho e um trigésimo do custo de pedir
        // um quadro por vsync.
        if self
            .windows
            .values()
            .any(|engine| engine.precisa_do_relogio())
        {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(30))
                    .map(|_| DaemonMessage::TickAll(EngineMessage::HoldTick)),
            );
        }

        // `GLACIER_PERF_STRESS`: pede um quadro por vsync para medir a
        // CAPACIDADE do app, não a demanda. Sem isto o relatório mede o quanto
        // ele ficou parado esperando evento — o que já se leu como travamento
        // mais de uma vez. O `ToastTick` é o portador porque, sem toast na tela,
        // ele é um no-op: força o quadro sem alterar estado nenhum.
        if crate::perf::estresse() {
            subs.push(
                iced::window::frames().map(|_| DaemonMessage::TickAll(EngineMessage::ToastTick)),
            );
        }

        // Eventos da bandeja (cliques de menu/ícone), só quando ela subiu. Drena
        // os canais globais do `tray-icon` (ver [`crate::tray::event_stream`]).
        if self.tray.is_some() {
            subs.push(iced::Subscription::run(crate::tray::event_stream).map(DaemonMessage::Tray));
        }

        // Ações injetadas por outra thread do app. Só registrada quando
        // alguém pediu um `external::sender()` — quem não usa não paga o poll.
        if crate::external::is_active() {
            subs.push(
                iced::Subscription::run(crate::external::event_stream).map(DaemonMessage::External),
            );
        }

        // Ping de uma segunda tentativa de lançamento. Só registrada quando
        // este processo detém a trava (ver [`crate::single_instance`]).
        if crate::single_instance::has_lock() {
            subs.push(
                iced::Subscription::run(crate::single_instance::event_stream)
                    .map(|_| DaemonMessage::ActivateRequested),
            );
        }

        // Subscriptions por-motor (streams `sse`/`websocket`, `Component::subscription`):
        // marcadas com o `id` da janela. Streams já vêm isolados por `engine_id`.
        // `Subscription::map` exige um closure não-capturante; para embutir o
        // `id` da janela usamos `.with(id)` (que emite `(id, msg)`) e um map sem
        // captura.
        for (id, engine) in &self.windows {
            subs.push(
                engine
                    .subscription()
                    .with(*id)
                    .map(|(id, msg)| DaemonMessage::Ui { id, msg }),
            );
        }
        Subscription::batch(subs)
    }
}

/// Constrói um [`GlacierUI`] novo para uma janela a partir da sua fonte, e
/// devolve também o título de fallback (nome do componente). `Named` já deve ter
/// sido resolvido para `File` no motor de origem (ver `run_on_owner`). `data`
/// (pares `open_window({ data = ... })`) é semeado no contexto **antes** de
/// registrar o componente, para que seu `init` já enxergue os valores.
/// Sobrepõe às `window::Settings`/título da janela o que o `<screen>` do
/// template declarou.
///
/// **O template ganha do builder.** Enquanto nenhum `.gv` tinha cabeçalho, o
/// builder era o único lugar onde essa informação podia estar; a partir do
/// momento em que alguém escreve `title=`/`size=` no arquivo, foi uma decisão
/// explícita e recente — e é o arquivo que está aberto na frente da pessoa e
/// recarrega a quente. Um campo não declarado (`None`) não opina: o valor do
/// builder fica.
/// Decide com que título e tamanho a janela principal nasce, resolvendo as três
/// camadas que opinam sobre isso — da mais fraca para a mais forte:
///
/// 1. o **builder** (`GlacierDaemon::title`/`main_size`/`main_window`), que é o
///    default de quem escreveu o app;
/// 2. o **`<screen>` do template**, que é o default de quem escreveu a tela;
/// 3. a **geometria lembrada** (`remember_window_geometry`), que não é default
///    nenhum: é o tamanho em que o usuário deixou a janela da última vez.
///
/// A ordem importa de verdade num app que lembra geometria: se o `size` do
/// arquivo viesse por último, toda abertura desfaria o redimensionamento do
/// usuário — o arquivo mandaria mais que a pessoa usando o programa. Aplicar o
/// `<screen>` primeiro também faz o `min-size` declarado valer no clamp da
/// geometria salva.
///
/// Devolve o tamanho que de fato ficou, quando alguém opinou — é o valor que o
/// `sized_by` do runtime guarda para saber se um hot-reload deve redimensionar.
fn resolve_main_window(
    meta: Option<&crate::ScreenMeta>,
    saved: Option<&SavedGeometry>,
    settings: &mut window::Settings,
    title: &mut String,
) -> Option<(f32, f32)> {
    let mut size = None;
    if let Some(meta) = meta {
        apply_screen_meta(meta, settings, title);
        size = meta.effective_size();
    }
    if let Some(saved) = saved {
        settings.size = clamp_to_max(clamp_to_min(saved.size, settings.min_size), settings.max_size);
        if let Some(p) = saved.position {
            settings.position = window::Position::Specific(p);
        }
        size = Some((settings.size.width, settings.size.height));
    }
    size
}

fn apply_screen_meta(
    meta: &crate::ScreenMeta,
    settings: &mut window::Settings,
    title: &mut String,
) {
    if let Some(t) = &meta.title {
        *title = t.clone();
    }
    if let Some((w, h)) = meta.size {
        settings.size = Size::new(w, h);
    }
    if let Some((w, h)) = meta.min_size {
        settings.min_size = Some(Size::new(w, h));
    }
    if let Some(r) = meta.resizable {
        settings.resizable = r;
    }
    apply_window_bounds(meta, settings);
}

/// O que o `<screen>` diz da janela além de título, tamanho e mínimo:
/// `max_size`, `fixed_size` e `decorations`. Partilhado pela principal
/// ([`apply_screen_meta`]) e pelas filhas (`Runtime::open_child`), onde o
/// tamanho segue outra regra — quem abre a janela pode pedir um.
///
/// O `fixed_size` vem por último e manda: é tamanho, mínimo e máximo ao mesmo
/// tempo, sem redimensionamento. O parser já recusa escrevê-lo ao lado dos
/// outros; aqui ele vence também o que veio de quem abriu a janela.
fn apply_window_bounds(meta: &crate::ScreenMeta, settings: &mut window::Settings) {
    if let Some((w, h)) = meta.max_size {
        settings.max_size = Some(Size::new(w, h));
    }
    if let Some((w, h)) = meta.fixed_size {
        let fixo = Size::new(w, h);
        settings.size = fixo;
        settings.min_size = Some(fixo);
        settings.max_size = Some(fixo);
        settings.resizable = false;
    }
    if let Some(d) = meta.decorations {
        settings.decorations = d;
    }
}

/// `icon="…"` do `<screen>`: lido pela fonte de assets e decodificado. Um
/// arquivo que falta ou não decodifica avisa no terminal e deixa a janela sem
/// ícone — nunca a impede de abrir.
fn apply_window_icon(
    meta: &crate::ScreenMeta,
    settings: &mut window::Settings,
    assets: &dyn AssetSource,
) {
    let Some(path) = &meta.icon else {
        return;
    };
    match assets.read_bytes(path) {
        Ok(bytes) => match window::icon::from_file_data(&bytes, None) {
            Ok(icon) => settings.icon = Some(icon),
            Err(erro) => eprintln!("<screen icon=\"{path}\">: não consegui decodificar o ícone: {erro}"),
        },
        Err(erro) => eprintln!("<screen icon=\"{path}\">: não consegui ler o ícone: {erro}"),
    }
}

/// O diretório de dados do app `id`, onde moram a geometria lembrada e o global
/// `storage`: `$XDG_DATA_HOME/<id>`, `%APPDATA%\<id>`,
/// `~/Library/Application Support/<id>` ou `~/.local/share/<id>` — o mesmo lugar
/// que os templates do `glacier new` calculavam à mão no `main.rs`.
fn app_data_dir(id: &str) -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                let home = PathBuf::from(home);
                if cfg!(target_os = "macos") {
                    home.join("Library/Application Support")
                } else {
                    home.join(".local/share")
                }
            })
        })
        .unwrap_or_else(|| PathBuf::from("."))
        .join(id)
}

/// A bandeja declarada numa `<tray>` do template principal: o que cada item faz
/// e os últimos rótulos/marcações enviados à thread da bandeja, para só mandar
/// o que mudou.
struct TrayBindings {
    items: Vec<TrayItemDecl>,
    labels: HashMap<String, String>,
    checked: HashMap<String, bool>,
}

impl TrayBindings {
    fn new(items: Vec<TrayItemDecl>) -> Self {
        Self {
            items,
            labels: HashMap::new(),
            checked: HashMap::new(),
        }
    }

    /// A ação do item `id`, se ele tiver uma.
    fn action(&self, id: &str) -> Option<&str> {
        self.items.iter().find(|item| item.id == id)?.on_click.as_deref()
    }

    /// O `TrayConfig` inicial, com rótulos e marcações já resolvidos contra o
    /// motor da principal — e guardados, para o `sync` saber o que mudou.
    fn config(&mut self, icon: Vec<u8>, tooltip: String, engine: &GlacierUI) -> TrayConfig {
        let mut items = Vec::with_capacity(self.items.len());
        for item in &self.items {
            match item.kind {
                TrayItemKind::Separator => items.push(TrayItem::separator()),
                TrayItemKind::Item => {
                    let rotulo = tray_text(&item.label, engine);
                    self.labels.insert(item.id.clone(), rotulo.clone());
                    items.push(TrayItem::button(&item.id, rotulo));
                }
                TrayItemKind::Check => {
                    let rotulo = tray_text(&item.label, engine);
                    let marcado = item
                        .checked
                        .as_deref()
                        .is_some_and(|c| truthy(&tray_text(c, engine)));
                    self.labels.insert(item.id.clone(), rotulo.clone());
                    self.checked.insert(item.id.clone(), marcado);
                    items.push(TrayItem::check(&item.id, rotulo, marcado));
                }
            }
        }
        TrayConfig {
            icon,
            tooltip,
            items,
        }
    }

    /// Reavalia rótulos e marcações contra o motor e manda à bandeja só o que
    /// mudou desde a última vez.
    fn sync(&mut self, handle: &TrayHandle, engine: &GlacierUI) {
        let acoes = TrayActions::new(handle);
        for item in &self.items {
            if item.kind == TrayItemKind::Separator {
                continue;
            }
            let rotulo = tray_text(&item.label, engine);
            if self.labels.get(&item.id) != Some(&rotulo) {
                acoes.set_label(&item.id, rotulo.clone());
                self.labels.insert(item.id.clone(), rotulo);
            }
            if let Some(c) = &item.checked {
                let marcado = truthy(&tray_text(c, engine));
                if self.checked.get(&item.id) != Some(&marcado) {
                    acoes.set_checked(&item.id, marcado);
                    self.checked.insert(item.id.clone(), marcado);
                }
            }
        }
    }
}

/// O texto de um item da `<tray>`, interpolado no contexto da janela principal.
/// `{__notifications}` é a chave do motor para o interruptor global das
/// notificações (`notifications:toggle`), que não mora no contexto de motor
/// nenhum.
fn tray_text(template: &str, engine: &GlacierUI) -> String {
    let template = template.replace(
        "{__notifications}",
        if crate::tray::notifications_enabled() {
            "true"
        } else {
            "false"
        },
    );
    crate::eval::process_template(&template, engine.context())
}

/// O mesmo teste do `if="{chave}"` sem comparador: vazio, `false` e `0` são
/// falsos.
fn truthy(valor: &str) -> bool {
    let v = valor.trim();
    !(v.is_empty() || v.eq_ignore_ascii_case("false") || v == "0")
}

fn build_engine(
    source: WindowSource,
    data: &[(String, String)],
    assets: Arc<dyn AssetSource>,
    style: Option<&crate::style::Style>,
) -> (GlacierUI, String) {
    let mut engine = GlacierUI::new().with_asset_source(assets);
    apply_style(&mut engine, style);
    for (k, v) in data {
        engine.define_data(k, v);
    }
    let title = match source {
        WindowSource::Component(comp) => {
            let name = comp.name().to_string();
            if let Err(e) = engine.register(comp) {
                eprintln!("open_window: falha ao registrar componente: {e}");
            }
            engine.set_initial_screen(&name);
            name
        }
        WindowSource::File(path) => {
            let name = file_stem(&path);
            if let Err(e) = engine.register_component(&name, &path) {
                eprintln!("open_window: falha ao carregar '{path}': {e}");
            }
            engine.set_initial_screen(&name);
            name
        }
        WindowSource::Named(name) => {
            // Não deveria acontecer: `run_on_owner` resolve `Named` para `File`.
            eprintln!("open_window: fonte 'Named({name})' não resolvida; janela vazia");
            name
        }
        WindowSource::WebView(url) => {
            // Não deveria acontecer: `Runtime::open_child` desvia `WebView`
            // para `open_webview_child` antes de chamar `build_engine` — uma
            // webview não tem motor nenhum por trás.
            eprintln!("open_window: 'WebView' chegou a build_engine ({url}); janela vazia");
            url
        }
    };
    (engine, title)
}

/// Aplica o estilo builtin de [`GlacierDaemon::style`] (quando houver) a um
/// motor recém-construído — antes do `setup`/registro da janela, para que tema
/// e `.gss` do app continuem vencendo. Falha é logada, não propagada: um estilo
/// inválido não deve impedir a janela de abrir.
fn apply_style(engine: &mut GlacierUI, style: Option<&crate::style::Style>) {
    if let Some(s) = style
        && let Err(e) = engine.set_style(s)
    {
        eprintln!("[glacier-ui] estilo '{}': falha ao aplicar: {e}", s.name);
    }
}

/// Nome de componente derivado do caminho de um arquivo (o stem, sem extensão).
fn file_stem(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("janela")
        .to_string()
}

/// Traduz uma ação `window:<cmd>` da titlebar custom na `Task` do iced
/// correspondente, dirigida ao `Id` **conhecido** da janela — não ao que um
/// `window::latest()` devolveria depois. Ver [`Runtime::main_id`].
fn window_control(id: window::Id, cmd: &str) -> Task<DaemonMessage> {
    if let Some(dir) = cmd.strip_prefix("resize:") {
        return match resize_direction(dir) {
            Some(d) => window::drag_resize(id, d),
            None => Task::none(),
        };
    }
    match cmd {
        "minimize" => window::minimize(id, true),
        "maximize" | "toggle_maximize" => window::toggle_maximize(id),
        "drag" => window::drag(id),
        _ => Task::none(),
    }
}

/// Direção de um puxador de redimensionamento (`window:resize:se`, …). Aceita as
/// abreviações de bússola e os nomes por extenso.
fn resize_direction(s: &str) -> Option<window::Direction> {
    use window::Direction::*;
    Some(match s.trim().to_ascii_lowercase().as_str() {
        "n" | "north" | "top" => North,
        "s" | "south" | "bottom" => South,
        "e" | "east" | "right" => East,
        "w" | "west" | "left" => West,
        "ne" | "northeast" | "north-east" => NorthEast,
        "nw" | "northwest" | "north-west" => NorthWest,
        "se" | "southeast" | "south-east" => SouthEast,
        "sw" | "southwest" | "south-west" => SouthWest,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineMessage;
    use crate::component::{Component, Context, Template};

    /// Componente de teste: cada ação pede uma janela nova de tipo diferente.
    struct Abridor;
    impl Component for Abridor {
        fn name(&self) -> &str {
            "abridor"
        }
        fn template(&self) -> Template {
            Template::Inline("<Text content=\"x\" />".to_string())
        }
        fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
            match action {
                "rust" => ctx.open_window_component(Box::new(Abridor)),
                "arquivo" => ctx.open_window(
                    WindowSpec::file("examples/janelas_glacier/detalhe.gv").title("D"),
                ),
                "nomeado" => ctx.open_window(WindowSpec::named("detalhe")),
                _ => {}
            }
        }
    }

    #[test]
    fn open_window_component_vira_pending_window() {
        let mut motor = GlacierUI::new();
        motor.register(Box::new(Abridor)).unwrap();
        motor.set_initial_screen("abridor");

        // Antes de qualquer ação, nada pendente.
        assert!(motor.take_pending_windows().is_empty());

        // A ação Rust deve enfileirar uma janela com fonte Component.
        let _ = motor.dispatch(&EngineMessage::UiClick("rust".into()));
        let pending = motor.take_pending_windows();
        assert_eq!(pending.len(), 1);
        assert!(matches!(pending[0].source, WindowSource::Component(_)));

        // A ação de arquivo enfileira uma janela File com título.
        let _ = motor.dispatch(&EngineMessage::UiClick("arquivo".into()));
        let pending = motor.take_pending_windows();
        assert_eq!(pending.len(), 1);
        assert!(matches!(&pending[0].source, WindowSource::File(p) if p.ends_with("detalhe.gv")));
        assert_eq!(pending[0].title.as_deref(), Some("D"));
    }

    #[test]
    fn open_window_named_resolve_para_arquivo() {
        let mut motor = GlacierUI::new();
        // Registra "detalhe" como componente de arquivo; a resolução Named→File
        // acontece na drenagem do Context (ver `run_on_owner`).
        motor
            .register_component("detalhe", "examples/janelas_glacier/detalhe.gv")
            .unwrap();
        motor.register(Box::new(Abridor)).unwrap();
        motor.set_initial_screen("abridor");

        let _ = motor.dispatch(&EngineMessage::UiClick("nomeado".into()));
        let pending = motor.take_pending_windows();
        assert_eq!(pending.len(), 1);
        match &pending[0].source {
            WindowSource::File(p) => assert_eq!(p, "examples/janelas_glacier/detalhe.gv"),
            _ => panic!("Named deveria ter sido resolvido para File"),
        }
    }

    #[test]
    fn build_engine_de_arquivo_usa_stem_como_titulo() {
        let (engine, title) = build_engine(
            WindowSource::File("examples/janelas_glacier/detalhe.gv".into()),
            &[],
            Arc::new(DiskAssets),
            None,
        );
        assert_eq!(title, "detalhe");
        // O motor da nova janela renderiza a tela carregada sem erro.
        assert!(engine.render_current().is_ok());
    }

    #[test]
    fn build_engine_semeia_data_no_contexto() {
        let (engine, _) = build_engine(
            WindowSource::File("examples/janelas_glacier/detalhe.gv".into()),
            &[
                ("url".into(), "http://x".into()),
                ("token".into(), "abc".into()),
            ],
            Arc::new(DiskAssets),
            None,
        );
        assert_eq!(engine.get_data("url").map(String::as_str), Some("http://x"));
        assert_eq!(engine.get_data("token").map(String::as_str), Some("abc"));
    }

    /// Emissor: uma ação envia um broadcast. Receptor: registra o que recebe.
    struct Emissor;
    impl Component for Emissor {
        fn name(&self) -> &str {
            "emissor"
        }
        fn template(&self) -> Template {
            Template::Inline("<Text content=\"x\" />".to_string())
        }
        fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
            match action {
                "enviar" => ctx.broadcast("ping", "{\"v\":\"1\"}"),
                "fechar" => ctx.close_window(),
                _ => {}
            }
        }
    }
    struct Receptor;
    impl Component for Receptor {
        fn name(&self) -> &str {
            "receptor"
        }
        fn template(&self) -> Template {
            Template::Inline("<Text content=\"x\" />".to_string())
        }
        fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {}
        fn on_broadcast(&mut self, event: &str, payload: &str, ctx: &mut Context) {
            ctx.set("rx", format!("{event}:{payload}"));
        }
    }

    #[test]
    fn broadcast_de_um_motor_chega_no_on_broadcast_de_outro() {
        // Motor emissor: a ação enfileira um broadcast pendente.
        let mut a = GlacierUI::new();
        a.register(Box::new(Emissor)).unwrap();
        a.set_initial_screen("emissor");
        assert!(a.take_pending_broadcasts().is_empty());
        let _ = a.dispatch(&EngineMessage::UiClick("enviar".into()));
        let msgs = a.take_pending_broadcasts();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].event, "ping");

        // Motor receptor: `deliver_broadcast` chama seu `on_broadcast`.
        let mut b = GlacierUI::new();
        b.register(Box::new(Receptor)).unwrap();
        b.set_initial_screen("receptor");
        let _ = b.deliver_broadcast(&msgs[0].event, &msgs[0].payload);
        assert_eq!(
            b.get_data("rx").map(String::as_str),
            Some("ping:{\"v\":\"1\"}")
        );
    }

    #[test]
    fn close_window_vira_take_close_requested() {
        let mut a = GlacierUI::new();
        a.register(Box::new(Emissor)).unwrap();
        a.set_initial_screen("emissor");
        assert!(!a.take_close_requested());
        let _ = a.dispatch(&EngineMessage::UiClick("fechar".into()));
        assert!(a.take_close_requested());
        // Consumido: não persiste.
        assert!(!a.take_close_requested());
    }

    /// Monta um `Runtime` como o `boot` faria: uma janela principal já com um
    /// motor, opcionalmente com bandeja. Os `Task` retornados por `open`/`update`
    /// não são poll-ados (não há loop iced no teste) — só inspecionamos o estado.
    fn runtime_de_teste(com_bandeja: bool) -> (Runtime, window::Id) {
        let settings = window::Settings::default();
        let (main_id, _open) = window::open(settings.clone());
        let mut rt = Runtime::new(
            Duration::from_millis(500),
            Duration::from_millis(400),
            main_id,
            Rc::new(|_| {}),
            settings,
            "T".to_string(),
            Arc::new(DiskAssets),
        );
        if com_bandeja {
            rt.tray = Some(crate::tray::TrayHandle::for_test());
        }
        rt.windows.insert(main_id, GlacierUI::new());
        rt.titles.insert(main_id, "T".to_string());
        (rt, main_id)
    }

    /// Uma tela de teste com markup inline — o atalho para registrar um
    /// `<screen>` sem tocar no disco.
    struct TelaInline(&'static str, String);
    impl Component for TelaInline {
        fn name(&self) -> &str {
            self.0
        }
        fn template(&self) -> Template {
            Template::Inline(self.1.clone())
        }
        fn update(&mut self, _action: &str, _v: Option<&str>, _ctx: &mut Context) {}
    }

    fn tela(nome: &'static str, markup: &str) -> Box<TelaInline> {
        Box::new(TelaInline(nome, markup.to_string()))
    }

    /// O `<screen title>` da tela ativa manda na barra da janela, e a navegação
    /// leva o título junto — inclusive de volta ao título base quando a tela
    /// seguinte não declara nenhum.
    #[test]
    fn titulo_segue_a_tela_ativa() {
        let (mut rt, id) = runtime_de_teste(false);
        let mut motor = GlacierUI::new();
        motor
            .register(tela(
                "lista",
                r#"<screen title="Lista"><column /></screen>"#,
            ))
            .unwrap();
        motor
            .register(tela(
                "detalhe",
                r#"<screen title="Detalhe"><column /></screen>"#,
            ))
            .unwrap();
        motor.register(tela("anonima", "<column />")).unwrap();
        motor.set_initial_screen("lista");
        rt.windows.insert(id, motor);
        rt.base_titles.insert(id, "T".to_string());

        let _ = rt.sync_window_meta(id);
        assert_eq!(rt.title(id), "Lista");

        rt.windows.get_mut(&id).unwrap().navigate_to("detalhe");
        let _ = rt.sync_window_meta(id);
        assert_eq!(rt.title(id), "Detalhe");

        rt.windows.get_mut(&id).unwrap().navigate_to("anonima");
        let _ = rt.sync_window_meta(id);
        assert_eq!(
            rt.title(id),
            "T",
            "tela sem título devolve o título de quem abriu a janela"
        );
    }

    /// Apagar o `title=` do arquivo e salvar (o hot-reload re-registra o
    /// componente) tem de apagar o título também — e não deixar o valor velho
    /// pendurado no motor.
    #[test]
    fn hot_reload_que_apaga_o_titulo_devolve_o_base() {
        let (mut rt, id) = runtime_de_teste(false);
        let mut motor = GlacierUI::new();
        motor
            .register(tela(
                "lista",
                r#"<screen title="Lista"><column /></screen>"#,
            ))
            .unwrap();
        motor.set_initial_screen("lista");
        rt.windows.insert(id, motor);
        rt.base_titles.insert(id, "T".to_string());
        let _ = rt.sync_window_meta(id);
        assert_eq!(rt.title(id), "Lista");

        // O "arquivo salvo" sem cabeçalho nenhum.
        rt.windows
            .get_mut(&id)
            .unwrap()
            .register(tela("lista", "<column />"))
            .unwrap();
        let _ = rt.sync_window_meta(id);
        assert_eq!(rt.title(id), "T");
    }

    /// Tamanho é de quem abre a janela, não da tela: navegar nunca redimensiona,
    /// e o hot-reload só redimensiona quando o número mudou no arquivo.
    #[test]
    fn tamanho_so_muda_quando_o_numero_muda_no_arquivo() {
        let (mut rt, id) = runtime_de_teste(false);
        let mut motor = GlacierUI::new();
        motor
            .register(tela(
                "lista",
                r#"<screen title="L" size="800 600"><column /></screen>"#,
            ))
            .unwrap();
        motor
            .register(tela(
                "outra",
                r#"<screen title="O" size="1200 900"><column /></screen>"#,
            ))
            .unwrap();
        motor.set_initial_screen("lista");
        rt.windows.insert(id, motor);
        rt.base_titles.insert(id, "T".to_string());
        rt.sized_by
            .insert(id, ("lista".to_string(), (800.0, 600.0)));

        // Mesma tela, mesmo número: nada a fazer.
        let _ = rt.sync_window_meta(id);
        assert_eq!(rt.sized_by[&id], ("lista".to_string(), (800.0, 600.0)));

        // Navegar para uma tela maior NÃO redimensiona a janela.
        rt.windows.get_mut(&id).unwrap().navigate_to("outra");
        let _ = rt.sync_window_meta(id);
        assert_eq!(
            rt.sized_by[&id],
            ("lista".to_string(), (800.0, 600.0)),
            "quem ditou o tamanho foi 'lista'; navegar não muda isso"
        );

        // Já editar o `size` da tela que ditou o tamanho, sim.
        rt.windows.get_mut(&id).unwrap().navigate_to("lista");
        rt.windows
            .get_mut(&id)
            .unwrap()
            .register(tela(
                "lista",
                r#"<screen title="L" size="900 640"><column /></screen>"#,
            ))
            .unwrap();
        let _ = rt.sync_window_meta(id);
        assert_eq!(rt.sized_by[&id], ("lista".to_string(), (900.0, 640.0)));
    }

    /// A geometria que o usuário deixou ganha do `size` do template — senão
    /// abrir o app desfaria o redimensionamento dele a cada boot.
    #[test]
    fn geometria_lembrada_ganha_do_size_do_template() {
        let meta = crate::ScreenMeta {
            title: Some("do template".into()),
            size: Some((1000.0, 700.0)),
            min_size: Some((480.0, 360.0)),
            resizable: None,
            ..Default::default()
        };
        let saved = SavedGeometry {
            size: Size::new(1440.0, 900.0),
            position: None,
        };

        let mut settings = window::Settings::default();
        let mut title = "do builder".to_string();
        let size = resolve_main_window(Some(&meta), Some(&saved), &mut settings, &mut title);

        assert_eq!(settings.size, Size::new(1440.0, 900.0));
        assert_eq!(size, Some((1440.0, 900.0)));
        assert_eq!(title, "do template", "o título segue vindo do template");
        assert_eq!(
            settings.min_size,
            Some(Size::new(480.0, 360.0)),
            "o min-size do template continua valendo"
        );

        // Sem geometria salva (primeira abertura), o template manda no tamanho.
        let mut settings = window::Settings::default();
        let mut title = "do builder".to_string();
        let size = resolve_main_window(Some(&meta), None, &mut settings, &mut title);
        assert_eq!(settings.size, Size::new(1000.0, 700.0));
        assert_eq!(size, Some((1000.0, 700.0)));
    }

    /// Uma geometria salva menor que o `min-size` declarado no template não
    /// abre a janela espremida.
    #[test]
    fn geometria_lembrada_respeita_o_min_size_do_template() {
        let meta = crate::ScreenMeta {
            min_size: Some((800.0, 600.0)),
            ..Default::default()
        };
        let saved = SavedGeometry {
            size: Size::new(400.0, 300.0),
            position: None,
        };
        let mut settings = window::Settings::default();
        let mut title = String::new();
        resolve_main_window(Some(&meta), Some(&saved), &mut settings, &mut title);
        assert_eq!(settings.size, Size::new(800.0, 600.0));
    }

    /// O template ganha do builder — e um campo que ele não declara não opina.
    #[test]
    fn apply_screen_meta_sobrepoe_so_o_declarado() {
        let mut settings = window::Settings {
            size: Size::new(640.0, 480.0),
            resizable: true,
            ..window::Settings::default()
        };
        let mut title = "do builder".to_string();

        apply_screen_meta(
            &crate::ScreenMeta {
                title: Some("do template".into()),
                size: Some((1000.0, 700.0)),
                min_size: None,
                resizable: None,
                ..Default::default()
            },
            &mut settings,
            &mut title,
        );

        assert_eq!(title, "do template");
        assert_eq!(settings.size, Size::new(1000.0, 700.0));
        assert!(settings.resizable, "não declarado, então o builder fica");
        assert!(settings.min_size.is_none());
    }

    #[test]
    fn remember_geometry_consulta_geometria_ao_fechar_sem_on_close() {
        // Regressão: `remember_window_geometry` (que semeia o `geometry_dir`) tem
        // de fazer o fechamento CONSULTAR a geometria mesmo SEM um gancho
        // `on_close` — senão o `CloseWithGeometry` (que grava o arquivo) nunca
        // dispara e a geometria não é persistida.
        let (mut rt, _main_id) = runtime_de_teste(false);
        assert!(
            !rt.needs_geometry_on_close(),
            "sem on_close nem geometry_dir, fechar não precisa da geometria"
        );
        rt.geometry_dir = Some(std::env::temp_dir());
        assert!(
            rt.needs_geometry_on_close(),
            "com geometry_dir (persistência nativa), fechar deve consultar a geometria"
        );
    }

    #[test]
    fn com_bandeja_fechar_principal_destaca_o_motor_e_reabre_reusando_o_mesmo() {
        let (mut rt, main_id) = runtime_de_teste(true);
        // Marca o motor para reconhecê-lo depois da migração.
        rt.windows
            .get_mut(&main_id)
            .unwrap()
            .define_data("marca", "vivo");
        assert!(rt.main_shown);

        // Fechar a principal com bandeja: destaca (motor segue vivo/headless),
        // não encerra, tira só o título.
        let _ = rt.update(DaemonMessage::Closed(main_id));
        assert!(!rt.main_shown);
        assert!(
            rt.windows.contains_key(&main_id),
            "o motor deve continuar vivo sob o id morto (SSE + login preservados)"
        );
        assert!(!rt.titles.contains_key(&main_id));

        // "Open Rustploy": religa o MESMO motor numa janela nova; main_id migra.
        let _ = rt.open_main();
        assert!(rt.main_shown);
        assert_ne!(rt.main_id, main_id, "deve migrar para uma janela nova");
        assert!(
            !rt.windows.contains_key(&main_id),
            "o id antigo (janela morta) deve sair do mapa"
        );
        let migrado = rt.windows.get(&rt.main_id).expect("motor na janela nova");
        assert_eq!(
            migrado.get_data("marca").map(String::as_str),
            Some("vivo"),
            "deve ser o MESMO motor retido, não um recém-construído"
        );
    }

    #[test]
    fn sem_bandeja_fechar_principal_remove_o_motor() {
        let (mut rt, main_id) = runtime_de_teste(false);
        // Sem bandeja o comportamento é o clássico: fecha e remove o motor (e o
        // app encerraria — o `iced::exit()` retornado não é inspecionável aqui).
        let _ = rt.update(DaemonMessage::Closed(main_id));
        assert!(!rt.windows.contains_key(&main_id));
        assert!(rt.main_shown, "sem destacamento, a flag não muda");
    }

    #[test]
    fn com_bandeja_fechar_janela_filha_nao_destaca_a_principal() {
        let (mut rt, main_id) = runtime_de_teste(true);
        // Uma janela-filha qualquer.
        let (filha_id, _) = window::open(window::Settings::default());
        rt.windows.insert(filha_id, GlacierUI::new());

        let _ = rt.update(DaemonMessage::Closed(filha_id));
        // A filha some; a principal e sua flag ficam intactas.
        assert!(!rt.windows.contains_key(&filha_id));
        assert!(rt.windows.contains_key(&main_id));
        assert!(rt.main_shown);
    }

    #[test]
    fn geometria_round_trip_com_e_sem_posicao() {
        let dir = std::env::temp_dir().join(format!("glacier_geom_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        // Ausente → None (não impede boot).
        assert!(load_geometry(&dir).is_none());

        // Com posição (X11): tamanho e posição voltam.
        save_geometry(&dir, Size::new(900.0, 640.0), Some(Point::new(30.0, 50.0)));
        let g = load_geometry(&dir).expect("deveria ler");
        assert_eq!(g.size, Size::new(900.0, 640.0));
        assert_eq!(g.position, Some(Point::new(30.0, 50.0)));

        // Sem posição (Wayland): só o tamanho volta, posição fica None.
        save_geometry(&dir, Size::new(700.0, 500.0), None);
        let g = load_geometry(&dir).expect("deveria ler");
        assert_eq!(g.size, Size::new(700.0, 500.0));
        assert_eq!(g.position, None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn geometria_respeita_min_size() {
        // Um tamanho salvo abaixo do mínimo (ou um min_size que cresceu entre
        // versões) nunca deve abrir a janela espremida.
        let clamped = clamp_to_min(Size::new(200.0, 100.0), Some(Size::new(480.0, 680.0)));
        assert_eq!(clamped, Size::new(480.0, 680.0));
        // Acima do mínimo passa intacto; sem mínimo, também.
        assert_eq!(
            clamp_to_min(Size::new(900.0, 700.0), Some(Size::new(480.0, 680.0))),
            Size::new(900.0, 700.0)
        );
        assert_eq!(
            clamp_to_min(Size::new(200.0, 100.0), None),
            Size::new(200.0, 100.0)
        );
    }
}
