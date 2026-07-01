//! Componentes que encapsulam UI (template XML) + comportamento + estado próprio.
//!
//! Em vez de o app registrar a UI (`register_component`) e tratar o comportamento
//! à parte no seu `update()`, um [`Component`] junta os dois num único tipo que o
//! motor registra de uma vez via [`crate::GlacierUI::register`].

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Um efeito assíncrono que um componente solicita durante o `update`.
///
/// O motor o transforma num [`iced::Task`]; quando o future completa, seus
/// pares `(chave, valor)` são mesclados no contexto (via
/// [`crate::EngineMessage::ContextPatch`]) e a UI é reavaliada. É a peça que
/// deixa um componente disparar I/O (rede, disco, timers) e refletir o
/// resultado no estado sem bloquear a thread de UI.
pub enum Effect {
    /// Executa um future e mescla o `Vec<(chave, valor)>` resultante no contexto.
    Perform(Pin<Box<dyn Future<Output = Vec<(String, String)>> + Send>>),
}

/// De onde vem o XML de um componente.
pub enum Template {
    /// Caminho em disco — mantém o hot-reload do motor.
    File(String),
    /// XML embutido no binário.
    Inline(String),
}

/// Pedido de navegação feito por um componente, aplicado pelo motor depois.
pub enum Nav {
    To(String),
    Back,
}

/// Uma variável de contexto nomeada: agrupa a chave e o valor num único valor,
/// aplicado de uma vez com [`Context::set_var`]. Útil para declarar defaults de
/// forma legível em vez de repetir a chave string solta.
pub struct ContextVar {
    key: String,
    value: String,
}

impl ContextVar {
    /// Cria uma variável com sua chave e valor.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self { key: key.into(), value: value.into() }
    }

    /// A chave (nome) da variável.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// O valor da variável.
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Acesso restrito ao estado do motor entregue ao componente durante
/// `init`/`update`. Expõe só o necessário (ler/escrever dados e pedir
/// navegação), evitando o conflito de borrow que existiria ao passar o
/// `GlacierUI` inteiro.
pub struct Context<'a> {
    pub(crate) data: &'a mut HashMap<String, String>,
    pub(crate) nav: Option<Nav>,
    pub(crate) effects: Vec<Effect>,
}

impl<'a> Context<'a> {
    /// Lê um valor do contexto de estado.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Define/atualiza um valor do contexto de estado (visível aos templates).
    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        self.data.insert(key.to_string(), value.into());
    }

    /// Aplica uma [`ContextVar`] (chave + valor) ao contexto.
    pub fn set_var(&mut self, var: &ContextVar) {
        self.data.insert(var.key.clone(), var.value.clone());
    }

    /// Pede ao motor para navegar para outra tela após o `update`.
    pub fn navigate_to(&mut self, screen: &str) {
        self.nav = Some(Nav::To(screen.to_string()));
    }

    /// Pede ao motor para voltar à tela anterior após o `update`.
    pub fn navigate_back(&mut self) {
        self.nav = Some(Nav::Back);
    }

    /// Agenda um efeito assíncrono: o `future` roda no executor do `iced` e,
    /// ao completar, seus pares `(chave, valor)` são mesclados no contexto e a
    /// UI é reavaliada. Use para rede, disco e qualquer I/O sem bloquear a UI.
    ///
    /// ```ignore
    /// fn update(&mut self, action: &str, _v: Option<&str>, ctx: &mut Context) {
    ///     if action == "load" {
    ///         ctx.perform(async {
    ///             let body = fetch().await;
    ///             vec![("status".into(), "ok".into()), ("body".into(), body)]
    ///         });
    ///     }
    /// }
    /// ```
    pub fn perform<F>(&mut self, future: F)
    where
        F: Future<Output = Vec<(String, String)>> + Send + 'static,
    {
        self.effects.push(Effect::Perform(Box::pin(future)));
    }

    /// Agenda um efeito que produz um único par `(chave, valor)`.
    pub fn perform_one<F>(&mut self, future: F)
    where
        F: Future<Output = (String, String)> + Send + 'static,
    {
        self.effects.push(Effect::Perform(Box::pin(async move {
            vec![future.await]
        })));
    }
}

/// Encapsula a UI, o comportamento e o estado próprio de um componente.
pub trait Component {
    /// Nome único, usado para registrar o template e rotear as ações.
    fn name(&self) -> &str;

    /// A UI deste componente.
    fn template(&self) -> Template;

    /// Semeia o contexto com o estado inicial (opcional).
    fn init(&mut self, _ctx: &mut Context) {}

    /// Sub-componentes que este componente possui. Ao registrar o pai, o motor
    /// registra cada filho em cascata (template + `init`), e as ações vindas da
    /// UI de um filho (referenciado por `<Component name="...">`) são roteadas
    /// para o `update` do próprio filho.
    ///
    /// Padrão: sem filhos.
    fn children(&self) -> Vec<Box<dyn Component>> {
        Vec::new()
    }

    /// Reage a uma ação vinda da sua própria UI.
    ///
    /// `value` vem preenchido em inputs (`UiInputChanged`); é `None` em
    /// cliques (`UiClick`).
    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context);

    /// Reage ao `onSubmit` de um `<Form>` (veja [`crate::forms::Form`]). Ao
    /// contrário de `update` — que recebe todo o resto (cliques, `onChange`,
    /// drag-and-drop, ...) — Enter num `formControl` ou um botão de submit
    /// dentro de um `<Form>` chegam aqui, não em `update`: a atualização de
    /// cada campo e a submissão do formulário nunca competem pelo mesmo
    /// `match`. `action` é a string do `onSubmit` (já sem o namespace do
    /// componente). Padrão: no-op — componentes sem formulário não precisam
    /// implementar. Um jeito comum de implementar é só delegar pra closure
    /// registrada via `FormBuilder::on_submit`:
    /// ```ignore
    /// fn on_form_submit(&mut self, _action: &str, ctx: &mut Context) {
    ///     self.form.submit(ctx);
    /// }
    /// ```
    fn on_form_submit(&mut self, _action: &str, _ctx: &mut Context) {}

    /// Fontes contínuas de eventos externos (sockets, timers, watchers) que
    /// alimentam o contexto. Mapeie cada stream para
    /// [`crate::EngineMessage::ContextPatch`] e o motor mesclará os pares no
    /// contexto e reavaliará a UI a cada item. O motor agrega as subscriptions
    /// de todos os componentes registrados em [`crate::GlacierUI::subscription`].
    ///
    /// Padrão: nenhuma subscription.
    fn subscription(&self) -> iced::Subscription<crate::EngineMessage> {
        iced::Subscription::none()
    }
}
