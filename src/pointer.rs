//! Os quatro widgets da Onda 9 que **desenham o próprio arrasto**:
//! `<rangeslider>`, `<tumbler>`, `<delaybutton>` e `<rubberband>`.
//!
//! # Por que estes não passam pelo [`crate::grip`]
//!
//! Um `canvas::Program` do `iced` ganha um `State` por instância de graça, e é
//! ali que mora o "estou arrastando" — foi assim no `<dial>` da Onda 7 e é
//! assim aqui. O [`crate::grip`] existe para o arrasto que acontece *entre*
//! filhos, onde não há `Program` nenhum (o `<splitter>` e o `<swipeview>`, em
//! [`crate::panes`]).
//!
//! O que os quatro compartilham com aqueles dois é o que importa, e é a tese da
//! onda: **o que o arrasto move é sempre um valor que o app nomeia**. Nenhum
//! deles guarda o valor no `State` — se guardasse, dois na mesma tela até
//! funcionariam (o `iced` dá um `State` por widget), mas o app não conseguiria
//! ler nem escrever o que a pessoa mexeu, que é o ponto todo.
//!
//! # O `<delaybutton>` é o de fora, e vale dizer por quê
//!
//! Nele o que anda é o **tempo**, não o pixel: segurar não move nada, só deixa
//! o relógio correr. Por isso ele é o único dos quatro que precisa de um
//! *ticker*, e o único cujo progresso mora numa chave global do motor
//! ([`HOLD_CONTEXT`]) em vez de no `State` — quem faz o tempo andar é o
//! `GlacierUI::update`, do lado de fora do `canvas`.

use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, mouse};

use crate::ContextMap;
use crate::canvas::{anel, cor_ou, fracao, rotulo};
use crate::gauges::{casas_do_passo, escreve, no_degrau, valor_de};
use crate::widget::EngineMessage;

/// O `<delaybutton>` apertado agora: `acao|inicio_ms|duracao_ms`.
///
/// **Um por app**, como todo estado de ponteiro deste motor (`__grip`,
/// `__colgrip` antes dele, `__timeedit`): não se segura dois botões ao mesmo
/// tempo. A fração decorrida não é guardada — ela é recalculada do relógio a
/// cada tick, que é o que evita ter duas fontes de verdade para a mesma coisa.
pub(crate) const HOLD_CONTEXT: &str = "__hold";

/// O retângulo do `<rubberband>` em curso: `x0,y0,x1,y1` em coordenadas do
/// próprio widget. Some ao soltar. Mesma família do `__cal_hover` do
/// `<calendar>` — um por tela, e por isso sem identidade no nome.
pub(crate) const BAND_CONTEXT: &str = "__band";

/// Um valor que o markup escreveu ou que a chave guarda, com o texto exato de
/// volta. Serve às duas pontas do `<rangeslider>`.
fn le(context: &ContextMap, chave: &str, padrao: f64) -> f64 {
    valor_de(context, chave, padrao)
}

/// A mensagem que grava um par chave→valor, ou delega numa ação. O par de
/// sempre: vazio = o widget grava sozinho, preenchido = o app decide.
fn grava(chave: &str, texto: String, acao: &str) -> EngineMessage {
    if acao.is_empty() {
        EngineMessage::ContextPatch(vec![(chave.to_string(), texto)])
    } else {
        EngineMessage::UiInputChanged {
            action: acao.to_string(),
            value: texto,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// <rangeslider> — a faixa com duas pontas
// ─────────────────────────────────────────────────────────────────────────────

/// Qual das duas pontas está presa. É **todo** o estado do widget, e é por isso
/// que o `●` do catálogo nunca valeu aqui: o par de valores mora em duas chaves
/// nomeadas, exatamente como no `<daterangepicker range>` da Onda 3.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Ponta {
    #[default]
    Nenhuma,
    Inicio,
    Fim,
}

#[derive(Debug, Default)]
pub struct EstadoFaixa {
    presa: Ponta,
}

pub struct ProgramaFaixa {
    pub inicio: f64,
    pub fim: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub cor: String,
    pub readonly: bool,
    pub chave_inicio: String,
    pub chave_fim: String,
    pub on_change: String,
    pub on_release: String,
}

/// Altura do trilho e raio do cursor, em pixels. Fixos porque o widget declara
/// só o **comprimento**: a espessura de um slider é vocabulário do tema, não do
/// dado.
const TRILHO: f32 = 6.0;
const CURSOR: f32 = 8.0;

impl ProgramaFaixa {
    /// A fração horizontal do cursor dentro da barra útil (descontadas as
    /// bordas de meio cursor de cada lado, senão as pontas seriam inalcançáveis).
    fn fracao_no_cursor(bounds: Rectangle, cursor: mouse::Cursor) -> Option<f32> {
        let p = cursor.position_in(bounds)?;
        let util = (bounds.width - CURSOR * 2.0).max(1.0);
        Some(((p.x - CURSOR) / util).clamp(0.0, 1.0))
    }

    fn valor_em(&self, f: f32) -> f64 {
        no_degrau(
            self.min + (self.max - self.min) * f as f64,
            self.min,
            self.max,
            self.step,
        )
    }

    /// Qual ponta o clique pegou: a mais perto, com desempate no **fim**.
    ///
    /// O desempate não é arbitrário. Com as duas pontas juntas no mesmo pixel
    /// (uma faixa de largura zero, que é o estado inicial de um filtro), pegar
    /// o início trava — só dá para arrastar para trás, e a faixa nunca abre.
    /// Pegando o fim, o primeiro arrasto para a direita já abre a faixa.
    fn ponta_mais_perto(&self, v: f64) -> Ponta {
        if (v - self.inicio).abs() < (v - self.fim).abs() {
            Ponta::Inicio
        } else {
            Ponta::Fim
        }
    }

    fn mensagens(&self, ponta: Ponta, novo: f64) -> Option<EngineMessage> {
        let casas = casas_do_passo(self.step);
        // As pontas não se cruzam: empurrar o início além do fim vira uma faixa
        // invertida, que não quer dizer nada. Elas encostam e param.
        let (chave, valor) = match ponta {
            Ponta::Inicio => (&self.chave_inicio, novo.min(self.fim)),
            Ponta::Fim => (&self.chave_fim, novo.max(self.inicio)),
            Ponta::Nenhuma => return None,
        };
        let atual = match ponta {
            Ponta::Inicio => self.inicio,
            _ => self.fim,
        };
        if (valor - atual).abs() < f64::EPSILON {
            return None;
        }
        Some(grava(chave, escreve(valor, casas), &self.on_change))
    }
}

impl canvas::Program<EngineMessage> for ProgramaFaixa {
    type State = EstadoFaixa;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<EngineMessage>> {
        if self.readonly {
            return None;
        }
        use iced::Event;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let f = Self::fracao_no_cursor(bounds, cursor)?;
                let v = self.valor_em(f);
                let ponta = self.ponta_mais_perto(v);
                state.presa = ponta;
                match self.mensagens(ponta, v) {
                    Some(m) => Some(canvas::Action::publish(m).and_capture()),
                    None => Some(canvas::Action::capture()),
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.presa != Ponta::Nenhuma => {
                let f = Self::fracao_no_cursor(bounds, cursor)?;
                let v = self.valor_em(f);
                // Sem mensagem quando o valor não mudou: arrastar dentro do
                // mesmo degrau publicaria uma reavaliação por pixel.
                match self.mensagens(state.presa, v) {
                    Some(m) => Some(canvas::Action::publish(m).and_capture()),
                    None => Some(canvas::Action::capture()),
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.presa != Ponta::Nenhuma =>
            {
                state.presa = Ponta::Nenhuma;
                if self.on_release.is_empty() {
                    Some(canvas::Action::capture())
                } else {
                    // O valor que o `on_release` carrega é a faixa inteira, e
                    // não a ponta: quem escuta o soltar quer o intervalo.
                    let casas = casas_do_passo(self.step);
                    Some(
                        canvas::Action::publish(EngineMessage::UiInputChanged {
                            action: self.on_release.clone(),
                            value: format!(
                                "{},{}",
                                escreve(self.inicio, casas),
                                escreve(self.fim, casas)
                            ),
                        })
                        .and_capture(),
                    )
                }
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let pal = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let cor = cor_ou(&self.cor, pal.primary.base.color);
        let y = bounds.height / 2.0;
        let util = (bounds.width - CURSOR * 2.0).max(1.0);
        let x = |v: f64| CURSOR + util * fracao(v, self.min, self.max);

        let trilho = |frame: &mut Frame, x0: f32, x1: f32, c: Color| {
            frame.fill(
                &Path::rounded_rectangle(
                    Point::new(x0, y - TRILHO / 2.0),
                    Size::new((x1 - x0).max(0.0), TRILHO),
                    (TRILHO / 2.0).into(),
                ),
                c,
            );
        };
        trilho(
            &mut frame,
            CURSOR,
            CURSOR + util,
            pal.background.strong.color,
        );
        trilho(&mut frame, x(self.inicio), x(self.fim), cor);

        for v in [self.inicio, self.fim] {
            let c = Point::new(x(v), y);
            frame.fill(&Path::circle(c, CURSOR), pal.background.base.color);
            frame.stroke(
                &Path::circle(c, CURSOR),
                Stroke::default().with_color(cor).with_width(2.5),
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !self.readonly && cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

/// `<rangeslider>` — ver [`crate::parser::NodeType::RangeSlider`].
#[allow(clippy::too_many_arguments)]
pub fn render_range_slider<'a>(
    context: &'a ContextMap,
    start_var: &'a str,
    end_var: &'a str,
    min: f32,
    max: f32,
    step: f32,
    size: f32,
    color: &'a str,
    on_change: &'a str,
    on_release: &'a str,
    readonly: bool,
) -> Element<'a, EngineMessage> {
    let (min, max) = (min as f64, max as f64);
    // Sem chave semeada, a faixa nasce inteira — que é o que um filtro "sem
    // filtro" quer dizer, e o que faz o widget aparecer certo antes do primeiro
    // clique sem precisar de `init` nenhum.
    let inicio = le(context, start_var, min).clamp(min, max);
    let fim = le(context, end_var, max).clamp(min, max);

    Canvas::new(ProgramaFaixa {
        inicio: inicio.min(fim),
        fim: fim.max(inicio),
        min,
        max,
        step: step as f64,
        cor: color.to_string(),
        readonly,
        chave_inicio: start_var.to_string(),
        chave_fim: end_var.to_string(),
        on_change: on_change.to_string(),
        on_release: on_release.to_string(),
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(CURSOR * 2.0 + 8.0))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// <tumbler> — o <dial> desenrolado numa linha
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct EstadoRoleta {
    arrastando: bool,
    /// Onde o dedo encostou, em `y` local. O deslocamento desde aqui é o que
    /// anda a roleta — e ele é reancorado a cada item que passa, senão
    /// arrastar devagar pularia vários de uma vez.
    origem: f32,
}

pub struct ProgramaRoleta {
    pub itens: Vec<String>,
    pub indice: usize,
    pub visiveis: usize,
    pub altura_item: f32,
    pub chave: String,
    pub on_change: String,
}

impl ProgramaRoleta {
    fn anda(&self, degraus: i64) -> Option<EngineMessage> {
        if self.itens.is_empty() || degraus == 0 {
            return None;
        }
        // A roleta **vira dentro de si** — o `wrapping` do `QAbstractSpinBox`,
        // a mesma decisão que as seções do `<timeedit>` tomaram na 0.68. É o
        // que faz "dezembro → janeiro" acontecer sem sair da roda.
        let n = self.itens.len() as i64;
        let novo = (self.indice as i64 + degraus).rem_euclid(n) as usize;
        (novo != self.indice).then(|| grava(&self.chave, self.itens[novo].clone(), &self.on_change))
    }
}

impl canvas::Program<EngineMessage> for ProgramaRoleta {
    type State = EstadoRoleta;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<EngineMessage>> {
        use iced::Event;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                state.arrastando = true;
                state.origem = p.y;
                Some(canvas::Action::capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.arrastando => {
                let p = cursor.position_in(bounds)?;
                let degraus = ((state.origem - p.y) / self.altura_item) as i64;
                if degraus == 0 {
                    return Some(canvas::Action::capture());
                }
                // Reancora no item que acabou de passar. Sem isto, o `as i64`
                // trunca sempre a partir do zero original e a roleta anda em
                // saltos crescentes.
                state.origem -= degraus as f32 * self.altura_item;
                match self.anda(degraus) {
                    Some(m) => Some(canvas::Action::publish(m).and_capture()),
                    None => Some(canvas::Action::capture()),
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.arrastando => {
                state.arrastando = false;
                Some(canvas::Action::capture())
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let linhas = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 40.0,
                };
                // A roda desce a lista quando gira para baixo, que é o
                // contrário do sinal que o iced entrega.
                let m = self.anda(-linhas.round() as i64)?;
                Some(canvas::Action::publish(m).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let pal = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());
        if self.itens.is_empty() {
            return vec![frame.into_geometry()];
        }

        let meio = self.visiveis as i64 / 2;
        let centro_y = bounds.height / 2.0;

        // A faixa do meio: é ela que diz qual item está escolhido, e é a única
        // pintura do widget que não é texto.
        frame.fill(
            &Path::rounded_rectangle(
                Point::new(2.0, centro_y - self.altura_item / 2.0),
                Size::new((bounds.width - 4.0).max(1.0), self.altura_item),
                4.0.into(),
            ),
            pal.primary.weak.color,
        );

        let n = self.itens.len() as i64;
        for d in -meio..=meio {
            let i = (self.indice as i64 + d).rem_euclid(n) as usize;
            let y = centro_y + d as f32 * self.altura_item;
            // Quem está longe do meio desbota — é o que dá a impressão de
            // cilindro sem desenhar cilindro nenhum.
            let cor = if d == 0 {
                pal.primary.weak.text
            } else {
                let f = 1.0 - (d.abs() as f32 / (meio.max(1) as f32 + 0.6));
                let base = pal.background.base.text;
                Color {
                    a: base.a * f.clamp(0.15, 1.0),
                    ..base
                }
            };
            let mut t: Text = rotulo(
                self.itens[i].clone(),
                Point::new(bounds.width / 2.0, y),
                if d == 0 { 15.0 } else { 13.0 },
                cor,
            );
            t.align_x = iced::alignment::Horizontal::Center.into();
            frame.fill_text(t);
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Lê a coleção de um `<tumbler>`: chave do contexto, JSON escrito no atributo,
/// ou lista separada por vírgula. As três formas que todo `items=` do motor
/// aceita desde o `<menu>`.
pub(crate) fn lista_de(context: &ContextMap, bruto: &str) -> Vec<String> {
    let texto = context.get(bruto).map(String::as_str).unwrap_or(bruto);
    if let Ok(serde_json::Value::Array(arr)) = serde_json::from_str::<serde_json::Value>(texto) {
        return arr
            .iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Object(o) => o
                    .get("label")
                    .or_else(|| o.get("rotulo"))
                    .or_else(|| o.get("id"))
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_else(|| v.to_string()),
                outro => outro.to_string(),
            })
            .collect();
    }
    texto
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// `<tumbler>` — ver [`crate::parser::NodeType::Tumbler`].
pub fn render_tumbler<'a>(
    context: &'a ContextMap,
    value_var: &'a str,
    items: &'a str,
    visible: usize,
    row: f32,
    size: f32,
    on_change: &'a str,
) -> Element<'a, EngineMessage> {
    let itens = lista_de(context, items);
    let atual = context.get(value_var).map(String::as_str).unwrap_or("");
    // O valor manda, não o índice: a chave guarda o **texto** escolhido, então
    // uma coleção reordenada não move a escolha de lugar.
    let indice = itens.iter().position(|i| i == atual).unwrap_or(0);

    Canvas::new(ProgramaRoleta {
        indice,
        visiveis: visible,
        altura_item: row,
        chave: value_var.to_string(),
        on_change: on_change.to_string(),
        itens,
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(row * visible as f32))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// <delaybutton> — o botão que se segura
// ─────────────────────────────────────────────────────────────────────────────

pub struct ProgramaDemora {
    pub texto: String,
    pub acao: String,
    /// `[0, 1]` — quanto do tempo já correu, lido de [`HOLD_CONTEXT`].
    pub fracao: f32,
    pub duracao: f32,
    pub cor: String,
}

impl canvas::Program<EngineMessage> for ProgramaDemora {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<EngineMessage>> {
        use iced::Event;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(bounds) =>
            {
                Some(
                    canvas::Action::publish(EngineMessage::HoldStart {
                        action: self.acao.clone(),
                        duracao: self.duracao,
                    })
                    .and_capture(),
                )
            }
            // Soltar **antes do fim desiste**, e é o ponto todo do widget: é um
            // botão para a ação que não se quer por engano. Quem termina o
            // tempo é o motor, não este evento.
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(canvas::Action::publish(EngineMessage::HoldEnd).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let pal = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let centro = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let raio = (bounds.width.min(bounds.height) / 2.0 - 2.0).max(1.0);
        let cor = cor_ou(&self.cor, pal.danger.base.color);
        let espessura = (raio * 0.16).clamp(3.0, 12.0);

        // O anel começa em cima e dá a volta inteira — um relógio, e não os 270°
        // do `<dial>`: aqui não há faixa útil nem abertura, o que se mostra é
        // tempo, e tempo fecha o círculo.
        frame.fill(
            &anel(centro, raio, espessura, -90.0, 360.0),
            pal.background.strong.color,
        );
        if self.fracao > 0.0 {
            frame.fill(
                &anel(centro, raio, espessura, -90.0, 360.0 * self.fracao),
                cor,
            );
        }

        let corpo = raio - espessura - 4.0;
        if corpo > 4.0 {
            frame.fill(&Path::circle(centro, corpo), pal.background.weak.color);
        }
        if !self.texto.is_empty() {
            let mut t: Text = rotulo(
                self.texto.clone(),
                centro,
                (corpo * 0.34).clamp(9.0, 16.0),
                pal.background.base.text,
            );
            t.align_x = iced::alignment::Horizontal::Center.into();
            frame.fill_text(t);
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

/// `<delaybutton>` — ver [`crate::parser::NodeType::DelayButton`].
pub fn render_delay_button<'a>(
    context: &'a ContextMap,
    text: &'a str,
    action: &'a str,
    delay: f32,
    size: f32,
    color: &'a str,
) -> Element<'a, EngineMessage> {
    // A fração é do apertar em curso, e só dele: um `<delaybutton>` que não é o
    // apertado desenha o anel vazio, mesmo que outro esteja enchendo.
    let fracao = context
        .get(HOLD_CONTEXT)
        .and_then(|h| h.split('|').next().map(str::to_string))
        .filter(|a| a == action)
        .and(context.get(crate::pointer::HOLD_FRAC_CONTEXT))
        .and_then(|f| f.parse::<f32>().ok())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);

    Canvas::new(ProgramaDemora {
        texto: text.to_string(),
        acao: action.to_string(),
        fracao,
        duracao: delay,
        cor: color.to_string(),
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

/// A fração já decorrida do apertar em curso, `"0"` a `"1"`. Separada do
/// [`HOLD_CONTEXT`] porque só ela muda a cada tick: um `<delaybutton>` que
/// dependesse da chave inteira reavaliaria a árvore por causa da ação e da
/// duração, que não mudaram.
pub(crate) const HOLD_FRAC_CONTEXT: &str = "__hold_frac";

// ─────────────────────────────────────────────────────────────────────────────
// <rubberband> — o retângulo de seleção
// ─────────────────────────────────────────────────────────────────────────────

/// Um retângulo da coleção de um `<rubberband>`, em coordenadas do widget.
#[derive(Debug, Clone)]
pub struct Alvo {
    pub id: String,
    pub r: Rectangle,
}

#[derive(Debug, Default)]
pub struct EstadoLaco {
    origem: Option<Point>,
}

pub struct ProgramaLaco {
    pub alvos: Vec<Alvo>,
    pub chave_selecao: String,
    pub on_select: String,
    pub cor: String,
    /// O retângulo em curso, lido de [`BAND_CONTEXT`] — o desenho vem do
    /// contexto e não do `State` porque quem precisa vê-lo é a **tela** (os
    /// itens tocados se acendem enquanto o laço passa), e a tela lê contexto.
    pub atual: Option<Rectangle>,
    /// Quem já está selecionado, lido da chave. O widget **desenha os alvos**
    /// que conhece, e não só a faixa.
    ///
    /// Não é o que o `QRubberBand` faz — lá a faixa é um widget solto e quem
    /// desenha o conteúdo é a view por baixo. Aqui não há por baixo: o motor
    /// não tem `<stack>` no markup, então um laço que só desenhasse a faixa
    /// pediria para arrastar sobre o vazio. Como o widget **já tem** a
    /// geometria dos alvos (é com ela que decide o que tocou), desenhá-los é
    /// de graça — e é o que torna o widget demonstrável sozinho.
    pub selecionados: Vec<String>,
}

impl ProgramaLaco {
    /// Os `id` que o retângulo toca, como conjunto nomeado — a mesma grafia
    /// separada por vírgula que o `<listview mode="multi">` guarda desde a 0.85,
    /// e que o `contains` do condicional lê.
    fn tocados(&self, r: Rectangle) -> String {
        self.alvos
            .iter()
            .filter(|a| a.r.intersects(&r))
            .map(|a| a.id.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }

    fn patch(&self, r: Rectangle) -> Vec<(String, String)> {
        let mut p = vec![(
            BAND_CONTEXT.to_string(),
            format!("{},{},{},{}", r.x, r.y, r.width, r.height),
        )];
        if !self.chave_selecao.is_empty() {
            p.push((self.chave_selecao.clone(), self.tocados(r)));
        }
        p
    }
}

/// O retângulo entre dois pontos, em qualquer ordem de arrasto — arrastar para
/// cima e para a esquerda tem de dar um retângulo positivo.
fn entre(a: Point, b: Point) -> Rectangle {
    Rectangle {
        x: a.x.min(b.x),
        y: a.y.min(b.y),
        width: (b.x - a.x).abs(),
        height: (b.y - a.y).abs(),
    }
}

impl canvas::Program<EngineMessage> for ProgramaLaco {
    type State = EstadoLaco;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<EngineMessage>> {
        use iced::Event;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                state.origem = Some(p);
                Some(
                    canvas::Action::publish(EngineMessage::ContextPatch(self.patch(entre(p, p))))
                        .and_capture(),
                )
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let origem = state.origem?;
                let p = cursor.position_in(bounds)?;
                Some(
                    canvas::Action::publish(EngineMessage::ContextPatch(
                        self.patch(entre(origem, p)),
                    ))
                    .and_capture(),
                )
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let origem = state.origem.take()?;
                let p = cursor.position_in(bounds).unwrap_or(origem);
                let r = entre(origem, p);
                // O laço some ao soltar; a seleção que ele fez, não. São duas
                // coisas diferentes e é por isso que moram em chaves diferentes.
                let mut patch = self.patch(r);
                patch[0].1 = String::new();
                let msg = if self.on_select.is_empty() {
                    EngineMessage::ContextPatch(patch)
                } else {
                    EngineMessage::PatchThen {
                        patch,
                        inner: Box::new(EngineMessage::UiInputChanged {
                            action: self.on_select.clone(),
                            value: format!("{},{},{},{}", r.x, r.y, r.width, r.height),
                        }),
                    }
                };
                Some(canvas::Action::publish(msg).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let pal = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let cor = cor_ou(&self.cor, pal.primary.base.color);

        // Os alvos primeiro, a faixa por cima — e a faixa é translúcida, para
        // que o que ela cobre continue legível enquanto se arrasta.
        for alvo in &self.alvos {
            let marcado = self.selecionados.contains(&alvo.id);
            let caminho = Path::rounded_rectangle(
                Point::new(alvo.r.x, alvo.r.y),
                Size::new(alvo.r.width, alvo.r.height),
                6.0.into(),
            );
            frame.fill(
                &caminho,
                if marcado {
                    Color { a: 0.30, ..cor }
                } else {
                    pal.background.weak.color
                },
            );
            frame.stroke(
                &caminho,
                Stroke::default()
                    .with_color(if marcado {
                        cor
                    } else {
                        pal.background.strong.color
                    })
                    .with_width(if marcado { 2.0 } else { 1.0 }),
            );
            let mut t: Text = rotulo(
                alvo.id.clone(),
                Point::new(
                    alvo.r.x + alvo.r.width / 2.0,
                    alvo.r.y + alvo.r.height / 2.0,
                ),
                12.0,
                pal.background.base.text,
            );
            t.align_x = iced::alignment::Horizontal::Center.into();
            frame.fill_text(t);
        }

        if let Some(r) = self.atual {
            let caminho = Path::rectangle(Point::new(r.x, r.y), Size::new(r.width, r.height));
            frame.fill(&caminho, Color { a: 0.18, ..cor });
            frame.stroke(&caminho, Stroke::default().with_color(cor).with_width(1.0));
        }
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Lê a coleção de retângulos de um `<rubberband>`.
fn alvos_de(context: &ContextMap, bruto: &str) -> Vec<Alvo> {
    let texto = context.get(bruto).map(String::as_str).unwrap_or(bruto);
    let Ok(serde_json::Value::Array(arr)) = serde_json::from_str::<serde_json::Value>(texto) else {
        return Vec::new();
    };
    arr.iter()
        .enumerate()
        .filter_map(|(i, v)| {
            let o = v.as_object()?;
            let num = |n: &[&str]| -> f32 {
                n.iter()
                    .find_map(|k| o.get(*k))
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0) as f32
            };
            Some(Alvo {
                id: o
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| i.to_string()),
                r: Rectangle {
                    x: num(&["x"]),
                    y: num(&["y"]),
                    width: num(&["w", "width", "largura"]),
                    height: num(&["h", "height", "altura"]),
                },
            })
        })
        .collect()
}

/// `<rubberband>` — ver [`crate::parser::NodeType::RubberBand`].
pub fn render_rubber_band<'a>(
    node: &'a crate::parser::UiNode,
    context: &'a ContextMap,
    items: &'a str,
    selection_var: &'a str,
    on_select: &'a str,
    color: &'a str,
) -> Element<'a, EngineMessage> {
    let atual = context
        .get(BAND_CONTEXT)
        .filter(|s| !s.is_empty())
        .and_then(|s| {
            let n: Vec<f32> = s.split(',').filter_map(|v| v.parse().ok()).collect();
            match n[..] {
                [x, y, w, h] => Some(Rectangle {
                    x,
                    y,
                    width: w,
                    height: h,
                }),
                _ => None,
            }
        });

    // Sem `width`/`height` o laço ocupa o que lhe derem — ele é uma superfície,
    // não um controle: um `Shrink` aqui daria um retângulo de zero pixel em que
    // não há como arrastar nada.
    let medida = |m: &Option<String>| match m {
        Some(_) => crate::widget::parse_length(m),
        None => Length::Fill,
    };
    let (largura, altura) = (medida(&node.width), medida(&node.height));

    Canvas::new(ProgramaLaco {
        alvos: alvos_de(context, items),
        selecionados: context
            .get(selection_var)
            .map(|s| {
                s.split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        chave_selecao: selection_var.to_string(),
        on_select: on_select.to_string(),
        cor: color.to_string(),
        atual,
    })
    .width(largura)
    .height(altura)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pares: &[(&str, &str)]) -> ContextMap {
        pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn as_pontas_nao_se_cruzam() {
        let p = ProgramaFaixa {
            inicio: 20.0,
            fim: 60.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            cor: String::new(),
            readonly: false,
            chave_inicio: "ini".into(),
            chave_fim: "fim".into(),
            on_change: String::new(),
            on_release: String::new(),
        };
        // Empurrar o início para 80 o encosta no fim (60) e para ali.
        let m = p.mensagens(Ponta::Inicio, 80.0).unwrap();
        assert!(matches!(
            m,
            EngineMessage::ContextPatch(ref v) if v[0] == ("ini".to_string(), "60".to_string())
        ));
    }

    #[test]
    fn com_as_pontas_juntas_o_clique_pega_o_fim() {
        // Senão a faixa de largura zero — o estado inicial de um filtro —
        // travaria: só daria para arrastar para trás.
        let p = ProgramaFaixa {
            inicio: 50.0,
            fim: 50.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            cor: String::new(),
            readonly: false,
            chave_inicio: "ini".into(),
            chave_fim: "fim".into(),
            on_change: String::new(),
            on_release: String::new(),
        };
        assert_eq!(p.ponta_mais_perto(50.0), Ponta::Fim);
    }

    #[test]
    fn a_roleta_vira_dentro_de_si() {
        let p = ProgramaRoleta {
            itens: vec!["jan".into(), "fev".into(), "mar".into()],
            indice: 2,
            visiveis: 3,
            altura_item: 34.0,
            chave: "mes".into(),
            on_change: String::new(),
        };
        // Do último para o primeiro, sem parar na ponta — o `wrapping` do
        // QAbstractSpinBox.
        let m = p.anda(1).unwrap();
        assert!(matches!(
            m,
            EngineMessage::ContextPatch(ref v) if v[0].1 == "jan"
        ));
        // E na volta também.
        let p = ProgramaRoleta { indice: 0, ..p };
        let m = p.anda(-1).unwrap();
        assert!(matches!(
            m,
            EngineMessage::ContextPatch(ref v) if v[0].1 == "mar"
        ));
    }

    #[test]
    fn a_lista_aceita_as_tres_formas() {
        let c = ctx(&[("meses", r#"["jan","fev"]"#)]);
        assert_eq!(lista_de(&c, "meses"), vec!["jan", "fev"]);
        assert_eq!(lista_de(&c, "a, b ,c"), vec!["a", "b", "c"]);
        assert_eq!(
            lista_de(&c, r#"[{"label":"Um","id":"1"}]"#),
            vec!["Um".to_string()]
        );
    }

    #[test]
    fn o_retangulo_e_positivo_em_qualquer_direcao() {
        let r = entre(Point::new(100.0, 80.0), Point::new(40.0, 20.0));
        assert_eq!((r.x, r.y, r.width, r.height), (40.0, 20.0, 60.0, 60.0));
    }

    #[test]
    fn o_laco_escreve_o_conjunto_nomeado_do_listview() {
        let p = ProgramaLaco {
            alvos: vec![
                Alvo {
                    id: "a".into(),
                    r: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 30.0,
                        height: 30.0,
                    },
                },
                Alvo {
                    id: "b".into(),
                    r: Rectangle {
                        x: 200.0,
                        y: 0.0,
                        width: 30.0,
                        height: 30.0,
                    },
                },
            ],
            chave_selecao: "sel".into(),
            on_select: String::new(),
            cor: String::new(),
            atual: None,
            selecionados: Vec::new(),
        };
        assert_eq!(
            p.tocados(Rectangle {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0
            }),
            "a"
        );
        assert_eq!(
            p.tocados(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 40.0
            }),
            "a,b"
        );
    }
}
