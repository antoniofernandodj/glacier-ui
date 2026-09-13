//! Webview nativa embutida numa janela — atrás da feature `webview` (ver o
//! comentário da dependência `wry` no `Cargo.toml` para as limitações de
//! plataforma, em especial o requisito de X11 no Linux).
//!
//! Uma janela de webview não é uma tag do motor: ela não roda `.gv`/`<script>`
//! nenhum, é a `wry::WebView` cobrindo a área inteira da janela, sozinha — ver
//! `WindowSource::WebView` em `component.rs` e o tratamento especial que
//! `daemon.rs` dá a esses `window::Id` (sem `GlacierUI` nenhum atrás deles).
//! Uso típico, da camada Lua: `open_window({ webview_url = "https://…" })`.
//!
//! **Incompatível com a feature `tray`, no Linux** — as duas exigem GTK
//! inicializado em threads diferentes (a bandeja tem a própria, esta módulo
//! precisa da principal) e GTK só aceita uma inicialização por processo. Ver
//! o comentário equivalente no topo de `crate::tray`.
//!
//! ## Por que uma `thread_local`
//!
//! `wry::WebView` não é `Send`: é um wrapper fino sobre objetos nativos
//! (GTK/WebKit no Linux, WebView2 no Windows, WKWebView no macOS) que só podem
//! ser tocados na thread que os criou. `iced::window::run` — o único jeito de
//! obter um handle de janela nesta versão do `iced` (ver o módulo `daemon`) —
//! exige que o closure devolva algo `Send + 'static`, porque só o QUE ele
//! devolve atravessa a fronteira de thread; o closure em si roda sempre na
//! thread da janela. Por isso a `WebView` nunca sai daqui: fica presa numa
//! `thread_local!`, e todo acesso a partir do `daemon` passa por
//! `iced::window::run(id, ...)` de novo, ainda que só para "rode isto na
//! thread certa" — nem sempre precisando do handle em si (ver `resize`).
use std::cell::RefCell;
use std::collections::HashMap;

use iced::window;

thread_local! {
    static WEBVIEWS: RefCell<HashMap<window::Id, wry::WebView>> = RefCell::new(HashMap::new());
}

fn bounds_for(size: (f32, f32)) -> wry::Rect {
    wry::Rect {
        position: wry::dpi::LogicalPosition::new(0.0_f64, 0.0_f64).into(),
        size: wry::dpi::LogicalSize::new(size.0 as f64, size.1 as f64).into(),
    }
}

/// Cria a webview da janela `id`, carregando `url`, do tamanho lógico
/// `size` — o mesmo que a `window::Settings` da janela pediu: não há como
/// consultar o tamanho real a partir do `&dyn Window` cru que
/// `iced::window::run` entrega (só os handles de janela/display, nada de
/// geometria). Chamada de dentro de `iced::window::run` (ver `daemon.rs`),
/// logo já na thread dona da janela.
///
/// Uma segunda chamada para o mesmo `id` é ignorada (`entry().or_insert_with`)
/// — não deveria acontecer (o daemon drena `pending_webviews` antes de
/// disparar a criação), mas uma janela recriando a webview por engano vazaria
/// a antiga silenciosamente, então a segurança é barata.
pub(crate) fn create(id: window::Id, window: &dyn window::Window, url: &str, size: (f32, f32)) {
    ensure_gtk_init();
    WEBVIEWS.with(|slots| {
        if slots.borrow().contains_key(&id) {
            return;
        }
        let builder = wry::WebViewBuilder::new();
        let builder = if let Some(html) = url.strip_prefix("html:") {
            builder.with_html(html)
        } else {
            builder.with_url(url)
        };
        let built = builder
            .with_bounds(bounds_for(size))
            // O IFrame Player do YouTube pede autoplay (mudo, por política dos
            // navegadores) para o próprio player iniciar sem gesto do usuário;
            // sem isto o WebKit embutido bloqueia e a tela fica preta parada
            // no logo do player.
            .with_autoplay(true)
            // O UA default do WebKitGTK identifica a engine de um jeito que
            // alguns sites (o YouTube incluso) tratam como cliente suspeito/
            // não-navegador e recusam a embutir. Um UA de Chrome comum evita
            // esse bloqueio sem mudar nada do motor de renderização real.
            .with_user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            )
            // `build::<W: HasWindowHandle>` exige `W: Sized` (bound implícito)
            // — `dyn Window` não é `Sized`, então passamos `&window`
            // (`&&dyn Window`): agora `W = &dyn Window`, que É `Sized` (um
            // ponteiro largo tem tamanho fixo) e implementa `HasWindowHandle`
            // via `impl<T: HasWindowHandle + ?Sized> HasWindowHandle for &T`
            // do próprio `raw-window-handle`.
            .build(&window);
        match built {
            Ok(webview) => {
                slots.borrow_mut().insert(id, webview);
            }
            Err(e) => {
                eprintln!("webview: falha ao criar ({url}): {e}");
            }
        }
    });
}

/// Reposiciona a webview de `id` para cobrir a janela inteira em `size`
/// (tamanho lógico) — chamada a cada `window::resize_events()` da janela (ver
/// `daemon.rs`). Ao contrário de Windows/macOS, o `wry` não redimensiona
/// sozinho sob X11 (ver a doc de `WebViewBuilder::build`), daí este primitivo
/// existir. Sem efeito se a webview ainda não foi criada — a corrida entre
/// `Opened` e o primeiro `Resized` é inofensiva, porque `create` já recebe o
/// tamanho inicial certo.
pub(crate) fn resize(id: window::Id, size: (f32, f32)) {
    WEBVIEWS.with(|slots| {
        if let Some(webview) = slots.borrow().get(&id)
            && let Err(e) = webview.set_bounds(bounds_for(size))
        {
            eprintln!("webview: falha ao redimensionar: {e}");
        }
    });
}

/// Destrói a webview de `id` — chamada quando a janela fecha
/// (`DaemonMessage::Closed`, ver `daemon.rs`). Sem isto o `wry::WebView`
/// ficaria pendurado na `thread_local` (e o processo/objetos nativos dela
/// junto) para sempre depois da janela do SO já ter sumido.
pub(crate) fn destroy(id: window::Id) {
    WEBVIEWS.with(|slots| {
        slots.borrow_mut().remove(&id);
    });
}

/// `gtk::init()` — exigido pelo `wry` no Linux mesmo passando só um handle
/// X11 (ver a doc de `WebViewBuilder::build`: "Panics on Linux, if `gtk::init`
/// was not called in this thread"), porque por baixo ainda é WebKitGTK.
/// `Once` porque uma segunda chamada é indefinida — e como `create` só roda
/// dentro de `iced::window::run` (sempre a mesma thread, a da janela/GUI),
/// isto acontece uma vez só, na thread certa, na primeira webview aberta.
///
/// Nas outras plataformas é vazio: `WebView2`/`WKWebView` não têm essa
/// exigência, e a dependência `gtk` nem compila fora do Linux (é por-alvo,
/// ver `Cargo.toml`).
#[cfg(target_os = "linux")]
static GTK_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(target_os = "linux")]
fn ensure_gtk_init() {
    GTK_INIT.call_once(|| {
        if let Err(e) = gtk::init() {
            eprintln!("webview: gtk::init falhou: {e}");
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn ensure_gtk_init() {}

/// Avança o loop do GTK "por fora" — o `wry` roda sobre WebKitGTK no Linux,
/// que precisa que ALGUÉM chame `gtk::main_iteration_do` periodicamente para
/// processar seus próprios eventos (pintura da página, JS, rede…); sem isto a
/// janela abre em branco e nunca atualiza, porque o loop do `iced`/`winit` não
/// sabe nada sobre GTK. Chamado a cada tick de `DaemonMessage::WebviewPumpGtk`
/// (ver `daemon.rs`), só enquanto houver alguma janela de webview aberta.
///
/// `events_pending` evita bloquear: processa o que já chegou e devolve na
/// hora, em vez de esperar o próximo evento — que travaria o `update` do
/// `iced` (e com ele a janela inteira) até o WebKit ter algo nas mãos.
///
/// **A corrida com `ensure_gtk_init`:** `webview_ids` (o que decide se este
/// tick é sequer agendado, ver `daemon.rs`) ganha a entrada da janela assim
/// que `open_window` é pedido — SÍNCRONO — mas o `gtk::init()` de verdade só
/// roda depois, dentro de `iced::window::run` (assíncrono, quando a janela
/// termina de abrir). Um tick de 16ms cabe fácil nessa janela de tempo, e
/// `gtk::events_pending` **entra em pânico** se chamado antes do `init` —
/// daí o `is_completed()` aqui: sem ele, abrir a primeira webview tinha uma
/// chance real de derrubar o app antes mesmo dela aparecer.
#[cfg(target_os = "linux")]
pub(crate) fn pump() {
    if !GTK_INIT.is_completed() {
        return;
    }
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn pump() {}
