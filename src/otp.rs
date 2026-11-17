//! `otpinput`: o campo de código de uso único — N caixinhas de um caractere,
//! o `OtpInput` do gpui-kit (o código do SMS, do e-mail, do app autenticador).
//!
//! ```gv
//! otpinput(:value = codigo, length = 6, on_complete = verificar)
//! ```
//!
//! # Por que um widget próprio, e não N `textinput`
//!
//! Com N campos de um caractere, o que faz o OTP ser bom de usar — o foco que
//! **avança** sozinho a cada dígito, o Backspace que **volta**, e o **colar**
//! que espalha o código inteiro pelas caixas — seria um vaivém de foco entre
//! widgets que o motor não comanda (um `update` não move foco). Aqui é **um**
//! widget, com **um** valor: as caixas são só o desenho dele. Digitar acrescenta
//! no fim, Backspace tira o último, Ctrl+V cola. Não há cursor no meio, de
//! propósito: num código de 6 dígitos, corrigir é apagar e redigitar.
//!
//! # Foco
//!
//! O estado implementa [`Focusable`], então o widget entra na ordem de Tab do
//! motor como um campo qualquer. Clicar nele foca; clicar fora ou Esc tira o
//! foco. Focado, a caixa da vez ganha a borda de destaque do tema.
//!
//! # O valor
//!
//! O contrato do `textinput`: sem `on_change`, o widget grava a chave sozinho;
//! com ele, delega ao app (que grava). Quando a última caixa é preenchida,
//! `on_complete` recebe o código inteiro — **depois** da gravação, então o
//! handler já lê a chave no estado novo.

use iced::advanced::widget::operation::{Focusable, Operation};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::keyboard::{self, Key, key::Named};
use iced::{Border, Element, Event, Length, Pixels, Point, Rectangle, Size};

/// Quais caracteres uma caixa aceita.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modo {
    /// Só `0`–`9`. O default: é o que um SMS e um app autenticador mandam.
    #[default]
    Digitos,
    /// Letras e dígitos, letras em **maiúscula** (códigos de recuperação,
    /// `A7K2Q9`). Digitar minúscula grava maiúscula.
    Alfanumerico,
}

impl Modo {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "alnum" | "alphanumeric" | "text" => Modo::Alfanumerico,
            _ => Modo::Digitos,
        }
    }

    fn aceita(self, c: char) -> Option<char> {
        match self {
            Modo::Digitos => c.is_ascii_digit().then_some(c),
            Modo::Alfanumerico => c.is_ascii_alphanumeric().then(|| c.to_ascii_uppercase()),
        }
    }
}

/// O que sobra de `entrada` depois do filtro do `modo`, sem passar de
/// `comprimento` — o caminho do **colar**. Separadores que um código colado
/// costuma trazer (`123 456`, `123-456`) somem pelo filtro, não por regra
/// especial.
pub fn filtra(entrada: &str, modo: Modo, comprimento: usize) -> String {
    entrada
        .chars()
        .filter_map(|c| modo.aceita(c))
        .take(comprimento)
        .collect()
}

/// O valor novo depois de digitar `texto` sobre `atual`: acrescenta no fim o
/// que o modo aceita, até `comprimento`. `None` = nada mudou (tecla recusada
/// ou campo cheio) e não há o que publicar.
pub fn acrescenta(atual: &str, texto: &str, modo: Modo, comprimento: usize) -> Option<String> {
    let atual = filtra(atual, modo, comprimento);
    let resto = comprimento.saturating_sub(atual.chars().count());
    let novos = filtra(texto, modo, resto);
    (!novos.is_empty()).then(|| atual + &novos)
}

pub struct OtpInput<'a, Message> {
    valor: String,
    comprimento: usize,
    modo: Modo,
    secreto: bool,
    lado: f32,
    espaco: f32,
    /// Como o valor novo vira mensagem (gravar a chave, ou delegar).
    ao_mudar: Box<dyn Fn(String) -> Message + 'a>,
    /// Publicada quando a última caixa é preenchida, com o código inteiro.
    ao_completar: Option<Box<dyn Fn(String) -> Message + 'a>>,
}

impl<'a, Message> OtpInput<'a, Message> {
    pub fn new(
        valor: &str,
        comprimento: usize,
        modo: Modo,
        ao_mudar: impl Fn(String) -> Message + 'a,
    ) -> Self {
        let comprimento = comprimento.clamp(1, 12);
        Self {
            valor: filtra(valor, modo, comprimento),
            comprimento,
            modo,
            secreto: false,
            lado: 44.0,
            espaco: 8.0,
            ao_mudar: Box::new(ao_mudar),
            ao_completar: None,
        }
    }

    pub fn secreto(mut self, s: bool) -> Self {
        self.secreto = s;
        self
    }

    /// Largura de cada caixa (a altura é 1,2 × isso).
    pub fn lado(mut self, l: f32) -> Self {
        self.lado = l.clamp(20.0, 120.0);
        self
    }

    pub fn espaco(mut self, e: f32) -> Self {
        self.espaco = e.max(0.0);
        self
    }

    pub fn ao_completar(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.ao_completar = Some(Box::new(f));
        self
    }

    fn altura(&self) -> f32 {
        (self.lado * 1.2).round()
    }

    fn caixa(&self, origem: Point, i: usize) -> Rectangle {
        Rectangle {
            x: origem.x + i as f32 * (self.lado + self.espaco),
            y: origem.y,
            width: self.lado,
            height: self.altura(),
        }
    }

    /// Publica o valor novo e, se ele encheu as caixas, o `on_complete`.
    fn publica(&self, novo: String, shell: &mut Shell<'_, Message>) {
        let completo = novo.chars().count() == self.comprimento;
        shell.publish((self.ao_mudar)(novo.clone()));
        if completo && let Some(f) = &self.ao_completar {
            shell.publish(f(novo));
        }
    }
}

#[derive(Debug, Default)]
struct Estado {
    focado: bool,
}

impl Focusable for Estado {
    fn is_focused(&self) -> bool {
        self.focado
    }
    fn focus(&mut self) {
        self.focado = true;
    }
    fn unfocus(&mut self) {
        self.focado = false;
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, iced::Theme, iced::Renderer> for OtpInput<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Estado>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Estado::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let n = self.comprimento as f32;
        let largura = n * self.lado + (n - 1.0) * self.espaco;
        layout::Node::new(limits.resolve(Length::Shrink, Length::Shrink, Size::new(largura, self.altura())))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let estado = tree.state.downcast_mut::<Estado>();
        operation.focusable(None, layout.bounds(), estado);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let estado = tree.state.downcast_mut::<Estado>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(iced::touch::Event::FingerPressed { .. }) => {
                let dentro = cursor.is_over(layout.bounds());
                if dentro != estado.focado {
                    estado.focado = dentro;
                    shell.request_redraw();
                }
                if dentro {
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key, modifiers, text, ..
            }) if estado.focado => {
                match key.as_ref() {
                    Key::Named(Named::Backspace) => {
                        let mut novo = self.valor.clone();
                        if novo.pop().is_some() {
                            shell.publish((self.ao_mudar)(novo));
                        }
                        shell.capture_event();
                    }
                    Key::Named(Named::Escape) => {
                        estado.focado = false;
                        shell.request_redraw();
                        shell.capture_event();
                    }
                    // Tab é do motor (a travessia de foco): não consumir.
                    Key::Named(Named::Tab) => {}
                    Key::Character("v") if modifiers.command() => {
                        if let Some(colado) = clipboard.read(iced::advanced::clipboard::Kind::Standard) {
                            // Colar SUBSTITUI: é o código inteiro que chegou.
                            let novo = filtra(&colado, self.modo, self.comprimento);
                            if !novo.is_empty() && novo != self.valor {
                                self.publica(novo, shell);
                            }
                        }
                        shell.capture_event();
                    }
                    _ if modifiers.command() || modifiers.alt() => {}
                    _ => {
                        if let Some(t) = text
                            && let Some(novo) = acrescenta(&self.valor, t, self.modo, self.comprimento)
                        {
                            self.publica(novo, shell);
                        }
                        // Focado, a tecla de texto é nossa, aceita ou não: um
                        // atalho de uma letra só não deve disparar enquanto se
                        // digita um código.
                        if text.is_some() {
                            shell.capture_event();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::text::Renderer as _;

        let focado = tree.state.downcast_ref::<Estado>().focado;
        let pal = theme.extended_palette();
        let origem = layout.bounds().position();
        let chars: Vec<char> = self.valor.chars().collect();
        // A caixa da vez: a primeira vazia, ou a última com o campo cheio.
        let da_vez = chars.len().min(self.comprimento - 1);

        for i in 0..self.comprimento {
            let b = self.caixa(origem, i);
            let ativa = focado && i == da_vez;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: b,
                    border: Border {
                        color: if ativa {
                            pal.primary.strong.color
                        } else {
                            pal.background.strong.color
                        },
                        width: if ativa { 2.0 } else { 1.0 },
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                },
                pal.background.base.color,
            );
            if let Some(c) = chars.get(i) {
                let conteudo = if self.secreto { '•' } else { *c };
                renderer.fill_text(
                    iced::advanced::Text {
                        content: conteudo.to_string(),
                        bounds: b.size(),
                        size: Pixels((self.lado * 0.5).round()),
                        line_height: iced::advanced::text::LineHeight::default(),
                        font: iced::Font::MONOSPACE,
                        align_x: iced::advanced::text::Alignment::Center,
                        align_y: iced::alignment::Vertical::Center,
                        shaping: iced::advanced::text::Shaping::Basic,
                        wrapping: iced::advanced::text::Wrapping::None,
                    },
                    b.center(),
                    pal.background.base.text,
                    b,
                );
            } else if ativa {
                // O cursor: um traço na caixa da vez, para dizer "é aqui".
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: b.center_x() - 1.0,
                            y: b.y + b.height * 0.25,
                            width: 2.0,
                            height: b.height * 0.5,
                        },
                        ..Default::default()
                    },
                    pal.primary.strong.color,
                );
            }
        }
    }
}

impl<'a, Message: Clone + 'a> From<OtpInput<'a, Message>> for Element<'a, Message> {
    fn from(w: OtpInput<'a, Message>) -> Self {
        Element::new(w)
    }
}
