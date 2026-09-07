//! Diálogos modais estilo `QMessageBox`: informação, aviso, erro, confirmação
//! e pergunta, além de uma variante totalmente customizável.
//!
//! Ao contrário do resto do glacier-ui (declarado em XML e cacheado como
//! árvore de template), um diálogo é transiente e disparado por código — um
//! [`Component::update`](crate::Component::update) pede
//! [`Context::show_dialog`](crate::Context::show_dialog) com um
//! [`DialogSpec`], o motor o sobrepõe (overlay) à tela atual em
//! [`crate::GlacierUI::render_current`], e o clique num botão chega de volta
//! ao `update()` do mesmo componente como uma ação comum — a mesma rota de
//! um `<Button on_click="...">`.
//!
//! ```ignore
//! ctx.show_dialog(DialogSpec::question("Excluir?", "Essa ação não pode ser desfeita.")
//!     .with_button(DialogButton::no("cancelar"))
//!     .with_button(DialogButton::yes("confirmar_exclusao")));
//! ```

use iced::widget::{Space, button, column, container, mouse_area, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

use crate::widget::EngineMessage;

/// O ícone mostrado ao lado do título, escolhendo também a cor de destaque —
/// o mesmo papel do `QMessageBox::Icon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogIcon {
    Information,
    Warning,
    Error,
    Question,
    /// Sem ícone (diálogos totalmente customizados).
    None,
}

impl DialogIcon {
    fn glyph(self) -> &'static str {
        match self {
            DialogIcon::Information => "ℹ",
            DialogIcon::Warning => "⚠",
            DialogIcon::Error => "✕",
            DialogIcon::Question => "?",
            DialogIcon::None => "",
        }
    }

    /// Cor de destaque do ícone/título, lida da paleta estendida do tema ativo.
    fn color(self, palette: &iced::theme::palette::Extended) -> Color {
        match self {
            DialogIcon::Information => palette.primary.base.color,
            DialogIcon::Warning => palette.warning.base.color,
            DialogIcon::Error => palette.danger.base.color,
            DialogIcon::Question => palette.primary.base.color,
            DialogIcon::None => palette.background.base.text,
        }
    }
}

/// Ação sentinela do botão **confirmar** de um `confirm()` suspensivo da camada
/// Lua (ver [`crate::luau`] e [`crate::component::DialogAction::ShowResumable`]):
/// o motor a reconhece e retoma a corrotina suspensa com `true` em vez de
/// despachá-la como ação normal. O botão **cancelar** usa [`CONFIRM_NO`] (→
/// `false`). Prefixadas com NUL para jamais colidir com um nome de ação real.
pub(crate) const CONFIRM_YES: &str = "\0glacier:confirm:yes";
/// Par de [`CONFIRM_YES`] para o botão cancelar (retoma a corrotina com `false`).
pub(crate) const CONFIRM_NO: &str = "\0glacier:confirm:no";

/// Ação sentinela do botão que **só fecha** — o `Fechar` que um
/// `<dialog name="…">` sem `buttons` ganha, e o que um `Cancelar::` sem ação
/// escrita vira. O motor a reconhece, fecha o diálogo e **não roteia nada**:
/// sem ela, um botão de cancelar despacharia uma ação fantasma que nenhum
/// componente trata, e um dia alguém escreveria um `update` chamado "cancelar"
/// que passaria a disparar sozinho.
///
/// Prefixada com NUL pelo mesmo motivo dos dois acima: nunca colidir com um
/// nome de ação de verdade.
pub const DIALOG_CLOSE: &str = "\0glacier:dialog:close";

/// O prefixo das chaves de contexto que um diálogo usa como rascunho — o
/// `__dialog.value` de um `prompt{}`, o `__dialog.progresso` de um
/// `ProgressDialog`.
///
/// # Por que um prefixo, e por que ele é limpo no fechamento
///
/// O corpo de um diálogo é avaliado no contexto do **app**, então as chaves que
/// ele escreve são chaves do app. Isso é o que faz um diálogo com campo
/// dispensar estado por instância — o valor mora no contexto como o de qualquer
/// `<textinput>` — e é também a única armadilha real da forma: dois diálogos
/// que usem `nome` colidem, mesmo sendo os dois singletons.
///
/// O prefixo separa esse rascunho do resto, e o motor apaga tudo que começa com
/// ele quando o diálogo fecha ([`crate::GlacierUI::limpa_rascunho_de_dialogo`]).
/// Sem a limpeza, a segunda abertura do mesmo diálogo já viria preenchida com a
/// resposta anterior — o bug que ninguém reporta e todo mundo estranha.
pub const DIALOG_KEY_PREFIX: &str = "__dialog.";

/// A chave onde um `prompt{}` guarda o que o usuário está digitando, e de onde
/// o aceite lê a resposta. Ver [`DIALOG_KEY_PREFIX`].
pub const DIALOG_VALUE_KEY: &str = "__dialog.value";

/// O papel de um botão, usado só para escolher seu estilo visual (destaque
/// para a ação principal, tom neutro para cancelar, tom de perigo para ações
/// destrutivas) — não afeta o roteamento, que é sempre por `action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonRole {
    /// Ação principal/afirmativa (OK, Yes, Save) — estilo em destaque (cor primária).
    Accept,
    /// Ação neutra ou de cancelamento (Cancel, No, Close) — estilo discreto.
    Neutral,
    /// Ação destrutiva (Discard, Abort) — estilo de perigo.
    Destructive,
}

/// Um botão do diálogo: rótulo mostrado, ação despachada ao `update()` do
/// componente dono da tela quando clicado (a mesma convenção de
/// `on_click="..."` num `<Button>`), e papel visual.
#[derive(Debug, Clone)]
pub struct DialogButton {
    pub label: String,
    pub action: String,
    pub role: ButtonRole,
}

impl DialogButton {
    /// Um botão com rótulo, ação e papel explícitos.
    pub fn new(label: impl Into<String>, action: impl Into<String>, role: ButtonRole) -> Self {
        Self {
            label: label.into(),
            action: action.into(),
            role,
        }
    }

    /// Atalhos para os botões padrão de um `QMessageBox`. O rótulo é fixo em
    /// inglês (como os `StandardButton` do Qt); use [`DialogButton::new`]
    /// para rótulos localizados.
    pub fn ok(action: impl Into<String>) -> Self {
        Self::new("OK", action, ButtonRole::Accept)
    }
    pub fn yes(action: impl Into<String>) -> Self {
        Self::new("Yes", action, ButtonRole::Accept)
    }
    pub fn no(action: impl Into<String>) -> Self {
        Self::new("No", action, ButtonRole::Neutral)
    }
    pub fn cancel(action: impl Into<String>) -> Self {
        Self::new("Cancel", action, ButtonRole::Neutral)
    }
    pub fn save(action: impl Into<String>) -> Self {
        Self::new("Save", action, ButtonRole::Accept)
    }
    pub fn discard(action: impl Into<String>) -> Self {
        Self::new("Discard", action, ButtonRole::Destructive)
    }
    pub fn retry(action: impl Into<String>) -> Self {
        Self::new("Retry", action, ButtonRole::Accept)
    }
    pub fn close(action: impl Into<String>) -> Self {
        Self::new("Close", action, ButtonRole::Neutral)
    }
}

/// A especificação de um diálogo modal: ícone, título, mensagem, um texto de
/// detalhe opcional (colapsável no Qt; aqui sempre visível, num bloco
/// destacado — ver [`DialogSpec::with_detail`]) e os botões disponíveis.
///
/// Construída com um dos construtores de conveniência
/// ([`DialogSpec::information`], [`DialogSpec::warning`],
/// [`DialogSpec::error`], [`DialogSpec::question`], [`DialogSpec::confirm`])
/// ou do zero com [`DialogSpec::new`], e ajustada com os métodos builder
/// (`with_*`/`dismissible`).
#[derive(Debug, Clone)]
pub struct DialogSpec {
    pub icon: DialogIcon,
    pub title: String,
    pub message: String,
    pub detail: Option<String>,
    pub buttons: Vec<DialogButton>,
    /// Se `true`, clicar no fundo escurecido fecha o diálogo sem despachar
    /// nenhuma ação (`EngineMessage::DialogDismiss`). Diálogos de erro e de
    /// pergunta/confirmação nascem com isso desligado — o usuário precisa
    /// escolher um botão explicitamente, como no Qt (`exec()` só retorna com
    /// um `StandardButton`).
    pub dismissible: bool,
    /// O **corpo em markup** do diálogo: o nome de um template já avaliado
    /// (componente, tela ou um `<dialog name="…">` do `<resources>`), montado
    /// entre a mensagem e os botões por [`crate::GlacierUI::render_current`].
    ///
    /// É o que separa um `QMessageBox` de um `QDialog`. Sem ele o diálogo é o
    /// que sempre foi — ícone, título, mensagem, botões, tudo em Rust. Com ele
    /// o cartão vira uma **moldura em volta de uma tela**: o conteúdo passa
    /// pelo mesmo `render_node` de qualquer outro nó, é estilizável pelo
    /// `.gss`, e o que o usuário digita mora numa chave de contexto como em
    /// qualquer `<textinput>` — que é por que um diálogo com campo **não**
    /// exige estado por instância (o diálogo é singleton; nunca há uma segunda
    /// instância com que colidir).
    pub body: Option<String>,
}

impl DialogSpec {
    /// Um diálogo em branco, sem botões (adicione com [`DialogSpec::with_button`]).
    pub fn new(icon: DialogIcon, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            icon,
            title: title.into(),
            message: message.into(),
            detail: None,
            buttons: Vec::new(),
            dismissible: true,
            body: None,
        }
    }

    /// `QMessageBox::information` — um único botão OK.
    pub fn information(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DialogIcon::Information, title, message).with_button(DialogButton::ok("ok"))
    }

    /// `QMessageBox::warning` — um único botão OK.
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DialogIcon::Warning, title, message).with_button(DialogButton::ok("ok"))
    }

    /// `QMessageBox::critical` — um único botão OK; não dispensável clicando
    /// fora, o usuário precisa reconhecer o erro explicitamente.
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DialogIcon::Error, title, message)
            .with_button(DialogButton::ok("ok"))
            .dismissible(false)
    }

    /// `QMessageBox::question` — botões Yes/No.
    pub fn question(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DialogIcon::Question, title, message)
            .with_button(DialogButton::no("no"))
            .with_button(DialogButton::yes("yes"))
            .dismissible(false)
    }

    /// Confirmação genérica de uma ação — botões Cancel/OK.
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DialogIcon::Question, title, message)
            .with_button(DialogButton::cancel("cancel"))
            .with_button(DialogButton::ok("ok"))
            .dismissible(false)
    }

    /// Adiciona um botão, na ordem em que aparecem da esquerda pra direita.
    pub fn with_button(mut self, button: DialogButton) -> Self {
        self.buttons.push(button);
        self
    }

    /// Anexa um texto de detalhe (`QMessageBox::setDetailedText`), mostrado
    /// num bloco destacado abaixo da mensagem principal.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Define se clicar fora do cartão do diálogo o fecha sem despachar ação.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Anexa um **corpo em markup** — ver [`DialogSpec::body`]. O argumento é o
    /// *nome* de um template já registrado, não o markup em si: o motor já sabe
    /// montar um template por nome ([`crate::GlacierUI::render`]), e passar o
    /// nome mantém o `DialogSpec` `Clone` e barato de guardar.
    pub fn with_body(mut self, template: impl Into<String>) -> Self {
        self.body = Some(template.into());
        self
    }

    /// Monta a especificação a partir de um `<dialog name="…">` declarado no
    /// markup (ver [`crate::parser::DialogMeta`]).
    ///
    /// O corpo já sai apontado para o próprio nome da declaração, porque são a
    /// mesma coisa: o `<dialog name="editar">` registra o conteúdo dele como o
    /// template `editar`, e o `DialogSpec` só precisa dizer "monte `editar`
    /// aqui dentro".
    pub fn from_meta(meta: &crate::parser::DialogMeta) -> Self {
        let mut spec = Self::new(
            icone_de(meta.icon.as_deref()),
            meta.title.clone().unwrap_or_default(),
            meta.message.clone().unwrap_or_default(),
        )
        .with_body(meta.name.clone());

        match meta.buttons.as_deref().map(str::trim) {
            // Escrito e não-vazio: a lista manda, inclusive quando ela pede
            // botão nenhum (`buttons=""` é um diálogo que só o fundo fecha —
            // útil para um progresso que não dá para cancelar).
            Some(lista) if !lista.is_empty() => {
                for b in lista.split('|') {
                    if let Some(botao) = botao_de(b) {
                        spec = spec.with_button(botao);
                    }
                }
            }
            Some(_) => {}
            // Sem `buttons`: um `Fechar` que só fecha. O par OK/Cancelar do Qt
            // não cabe aqui — lá o `accept()` tem significado próprio, aqui um
            // "OK" sem ação escrita faria exatamente o que o "Cancelar" faz, e
            // dois botões idênticos com rótulos diferentes é pior do que um.
            None => {
                spec = spec.with_button(DialogButton::new(
                    "Fechar",
                    DIALOG_CLOSE,
                    ButtonRole::Neutral,
                ));
            }
        }

        // Um diálogo com corpo é dispensável por padrão, como qualquer
        // `DialogSpec::new` — mas quem escreve o markup pode fechar a saída
        // (`dismissible="false"`) quando a escolha tem de ser explícita.
        if let Some(d) = meta.dismissible {
            spec = spec.dismissible(d);
        }
        spec
    }
}

/// Lê o `icon=` de um `<dialog>`. Desconhecido vira [`DialogIcon::None`] em vez
/// de erro: um ícone errado não é motivo para o diálogo não abrir.
fn icone_de(nome: Option<&str>) -> DialogIcon {
    match nome
        .map(str::trim)
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "information" | "info" | "informacao" | "informação" => DialogIcon::Information,
        "warning" | "warn" | "aviso" => DialogIcon::Warning,
        "error" | "critical" | "erro" => DialogIcon::Error,
        "question" | "pergunta" => DialogIcon::Question,
        _ => DialogIcon::None,
    }
}

/// Lê um botão da lista compacta do `<dialog buttons="…">`, na forma
/// `Rótulo:acao:papel` — os dois últimos campos opcionais.
///
/// Ação vazia (`Cancelar::`) vira [`DIALOG_CLOSE`]: o botão fecha e não despacha
/// nada. Papel ausente é `neutral`, com uma exceção que vale a magia — se a
/// ação está escrita, o papel default é `accept`, porque um botão que *faz*
/// alguma coisa é a ação principal em quase todo diálogo, e escrever
/// `:accept` em cada um seria ruído.
fn botao_de(bruto: &str) -> Option<DialogButton> {
    let mut campos = bruto.split(':');
    let label = campos.next()?.trim();
    if label.is_empty() {
        return None;
    }
    let acao = campos.next().unwrap_or("").trim();
    let papel = campos.next().unwrap_or("").trim().to_lowercase();
    let role = match papel.as_str() {
        "accept" | "aceitar" | "principal" => ButtonRole::Accept,
        "neutral" | "neutro" | "cancel" | "cancelar" => ButtonRole::Neutral,
        "destructive" | "destrutivo" | "perigo" | "danger" => ButtonRole::Destructive,
        _ if acao.is_empty() => ButtonRole::Neutral,
        _ => ButtonRole::Accept,
    };
    let acao = if acao.is_empty() { DIALOG_CLOSE } else { acao };
    Some(DialogButton::new(label, acao, role))
}

/// Renderiza o diálogo como um overlay completo: um fundo semitransparente
/// cobrindo toda a área disponível (clicável para dispensar, se
/// `spec.dismissible`) com o cartão do diálogo centralizado por cima. Chame
/// [`crate::GlacierUI::render_current`] normalmente — ele já empilha isto por
/// cima da tela ativa quando há um diálogo em exibição.
///
/// O `body` é o corpo em markup já montado (ver [`DialogSpec::body`]), que
/// entra entre a mensagem e a fileira de botões. `None` dá o diálogo clássico,
/// só com texto — e é o caminho dos cinco construtores de `QMessageBox`.
///
/// **Cuidado ao mexer aqui:** o `body` fica na camada de cima do `stack!`,
/// acima do fundo escurecido. O fundo captura hover *e* clique de propósito
/// (ver o comentário do `backdrop_area` e o `DIALOGS.md`), e inverter a ordem
/// das camadas faz o corpo parar de receber input — o sintoma é um campo que
/// simplesmente não pega foco, sem erro nenhum.
pub fn overlay<'a>(
    spec: &'a DialogSpec,
    theme: &iced::Theme,
    body: Option<Element<'a, EngineMessage>>,
) -> Element<'a, EngineMessage> {
    let palette = theme.extended_palette();

    let mut header = row![].spacing(10).align_y(Alignment::Center);
    if spec.icon != DialogIcon::None {
        header = header.push(
            text(spec.icon.glyph())
                .size(22)
                .color(spec.icon.color(palette)),
        );
    }
    header = header.push(text(spec.title.as_str()).size(18));

    // Um diálogo com corpo costuma não ter mensagem — o conteúdo já diz o que
    // ele pede. Um `text("")` não é invisível: ele ocupa a altura de uma linha
    // mais o `spacing`, e o buraco entre o título e o formulário denuncia.
    let mut card = column![header].spacing(14);
    if !spec.message.is_empty() {
        card = card.push(text(spec.message.as_str()).size(14));
    }

    if let Some(detail) = &spec.detail {
        let detail_bg = palette.background.weak.color;
        let detail_text = palette.background.weak.text;
        card = card.push(
            container(text(detail.as_str()).size(12).color(detail_text))
                .padding(8)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme| container::Style {
                    background: Some(Background::Color(detail_bg)),
                    border: Border {
                        radius: iced::border::Radius::new(4.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                }),
        );
    }

    // O corpo em markup entra **depois** do detalhe e **antes** dos botões: a
    // ordem do `QDialog` (mensagem, conteúdo, caixa de botões), e a única em
    // que os botões continuam sendo a última coisa que a leitura encontra.
    if let Some(body) = body {
        card = card.push(container(body).width(Length::Fill));
    }

    let mut buttons = row![Space::new().width(Length::Fill)].spacing(8);
    for b in &spec.buttons {
        buttons = buttons.push(dialog_button(b, palette));
    }
    card = card.push(buttons);

    // Elevação por fundo opaco, não por `shadow` — ver `TROUBLESHOOTING.md`.
    // Nos drivers Vulkan incompletos (Intel Ivy Bridge/Haswell na Mesa) o quad
    // de uma sombra translúcida não é apagado junto com o corpo do container:
    // ele se soma quadro após quadro até o cartão virar um retângulo preto. Um
    // tom mais claro que o fundo da tela, mais a borda, dizem a mesma coisa
    // ("isto flutua") com cores opacas, que não acumulam em driver nenhum.
    let card_bg = palette.background.weak.color;
    let card_border = palette.background.strong.color;
    // 380 é a largura de uma frase; um formulário não cabe nela. Com corpo, o
    // cartão vai a 480 — ainda estreito o bastante para ler como modal, largo o
    // bastante para um `<form>` de duas colunas não quebrar.
    let largura = if spec.body.is_some() { 480.0 } else { 380.0 };
    let card_box = container(card)
        .width(Length::Fixed(largura))
        .padding(20)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(Background::Color(card_bg)),
            border: Border {
                radius: iced::border::Radius::new(8.0),
                width: 1.0,
                color: card_border,
            },
            ..Default::default()
        });

    let centered = container(card_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    let backdrop = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
            ..Default::default()
        });

    // `Idle` (not `None`) is deliberate: `iced::widget::Stack` picks the
    // topmost layer whose `mouse_interaction()` isn't `Interaction::None` —
    // `None` doesn't mean "the idle/arrow cursor", it means "no opinion,
    // check the layer underneath". Reporting `None` here (the default,
    // if `.interaction()` is never called) let hover state leak through the
    // backdrop to whatever button sat at the same screen position on the
    // base screen below, showing its hand cursor right through the modal.
    //
    // `on_press` is always attached, even for non-dismissible dialogs, for
    // the same reason: `MouseArea::update` only calls `shell.capture_event()`
    // when it actually has a press handler — without one, a click on the
    // backdrop wouldn't just fail to close the dialog, it would fall through
    // the stack and land on whatever's underneath (a real click-through, not
    // just a cosmetic hover leak). `dispatch()` already checks `dismissible`
    // before honoring `DialogDismiss`, so attaching it unconditionally here
    // only affects event capture, not whether clicking outside closes it.
    let backdrop_area = mouse_area(backdrop)
        .interaction(iced::mouse::Interaction::Idle)
        .on_press(EngineMessage::DialogDismiss);

    iced::widget::stack![Element::from(backdrop_area), centered].into()
}

/// Renderiza um botão do diálogo, colorido pelo seu [`ButtonRole`].
fn dialog_button<'a>(
    b: &'a DialogButton,
    palette: &iced::theme::palette::Extended,
) -> Element<'a, EngineMessage> {
    let base = match b.role {
        ButtonRole::Accept => palette.primary.base.color,
        ButtonRole::Neutral => palette.background.strong.color,
        ButtonRole::Destructive => palette.danger.base.color,
    };
    let text_color = match b.role {
        ButtonRole::Neutral => palette.background.base.text,
        _ => Color::WHITE,
    };

    button(text(b.label.as_str()).color(text_color))
        .padding([8, 16])
        .style(move |_theme: &iced::Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Color { a: 0.85, ..base },
                button::Status::Pressed => Color { a: 0.7, ..base },
                _ => base,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color,
                border: Border {
                    radius: iced::border::Radius::new(6.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .on_press(EngineMessage::DialogButton(b.action.clone()))
        .into()
}
