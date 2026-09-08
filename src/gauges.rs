//! Os três medidores da Onda 7: `<dial>`, `<gauge>` e `<lcdnumber>` — o
//! `QDial`, o *Gauge* do QML e o `QLCDNumber`.
//!
//! Os três desenham no `canvas` (ver [`crate::canvas`], o habilitador da onda),
//! e os três seguem a regra que o `<slider>` e o `<rating>` já seguiam: **o
//! valor mora na chave que o markup nomeia**. O `canvas::Program::State` do
//! `iced` guarda só o que morre com o gesto — se a alça do `<dial>` está presa
//! agora —, então N medidores na mesma tela não se veem.
//!
//! # A reclassificação
//!
//! O `PLANO_WIDGETS.md` catalogava os três como **componente** e marcava
//! `<dial>` com `●` ("exige estado por instância"). Não exige, e pela terceira
//! vez na história deste catálogo: o `Spinner` (0.66) e o `Rating` (0.85) já
//! tinham descido de `●` para primitiva pelo mesmo motivo. O que parecia estado
//! por instância era o **valor**, e o valor sempre coube numa chave que o app
//! nomeia — `value="volume"`, como `value="nota"`. O que sobra de verdadeiro
//! estado interno (o arrasto em curso) é do widget, e o `iced` já o guarda.

use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};

use crate::ContextMap;
use crate::canvas::{anel, com_casas, cor_ou, fracao, na_borda, rotulo, segmentos};
use crate::widget::{EngineMessage, parse_hex_color};

/// O ângulo em que todo medidor circular deste módulo começa, e quanto ele
/// varre — 135° (embaixo à esquerda) girando 270° até 405° (embaixo à direita).
///
/// São os números do `QDial` e de praticamente todo medidor de painel, e o
/// motivo é físico: a abertura de baixo é onde o eixo de um ponteiro real
/// entraria, e é onde o olho aceita que a escala termine.
pub const INICIO_PADRAO: f32 = 135.0;
pub const VARREDURA_PADRAO: f32 = 270.0;

/// Lê o valor de um medidor: **primeiro como chave** do contexto, e só depois
/// como número escrito à mão.
///
/// A ordem importa e é a mesma do `<progressbar>`: `value="cpu"` é o nome de
/// uma chave, não a string "cpu". O fallback para literal existe porque
/// `<gauge>` e `<lcdnumber>` são apresentacionais e aparecem muito numa tela de
/// exemplo com número fixo — mas um `<dial>`, que **escreve** de volta, sem
/// chave não tem onde gravar, e é por isso que ele exige uma.
pub(crate) fn valor_de(context: &ContextMap, chave: &str, padrao: f64) -> f64 {
    context
        .get(chave)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .or_else(|| chave.trim().parse::<f64>().ok())
        .unwrap_or(padrao)
}

/// Prende o valor na faixa e no degrau. `step <= 0` significa contínuo.
pub(crate) fn no_degrau(v: f64, min: f64, max: f64, step: f64) -> f64 {
    let v = v.clamp(min.min(max), max.max(min));
    if step <= 0.0 {
        return v;
    }
    min + ((v - min) / step).round() * step
}

/// O texto de um valor de medidor: `decimals` casas, e sem `-0`.
pub(crate) fn escreve(v: f64, decimals: usize) -> String {
    let s = com_casas(v, decimals);
    if s.trim_start_matches('-').chars().all(|c| c == '0' || c == '.') {
        s.trim_start_matches('-').to_string()
    } else {
        s
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// <dial> — o knob rotativo
// ─────────────────────────────────────────────────────────────────────────────

/// O que o `<dial>` guarda entre eventos: só o arrasto.
///
/// **Não** é o valor — o valor é do app, e está na chave. Se este `State`
/// guardasse o valor, dois `<dial>` na mesma tela ainda funcionariam (o `iced`
/// dá um `State` por widget), mas o app não conseguiria ler nem escrever o que
/// a pessoa girou, que é o ponto todo.
#[derive(Debug, Default)]
pub struct EstadoDial {
    arrastando: bool,
}

/// `<dial>` — ver [`crate::parser::NodeType::Dial`].
pub struct ProgramaDial {
    pub valor: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub marcas: usize,
    pub espessura: f32,
    pub cor: String,
    pub mostra_valor: bool,
    pub decimals: usize,
    pub readonly: bool,
    /// A chave que recebe o valor, e a ação que o substitui quando o app
    /// prefere gravar sozinho — o par de sempre.
    pub chave: String,
    pub on_change: String,
    pub on_release: String,
}

impl ProgramaDial {
    /// O ângulo do cursor dentro da faixa útil do medidor, em `[0, 1]`.
    ///
    /// `None` quando o cursor está na abertura de baixo (fora dos 270°): ali
    /// não há valor a que corresponder, e chutar o mais próximo faria o knob
    /// saltar do mínimo ao máximo quando o dedo passa raspando pela base.
    fn fracao_no_cursor(bounds: Rectangle, cursor: mouse::Cursor) -> Option<f32> {
        let p = cursor.position_in(bounds)?;
        let centro = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let (dx, dy) = (p.x - centro.x, p.y - centro.y);
        // Muito perto do eixo o ângulo é ruído — meio pixel de tremor viraria
        // meia volta.
        if dx.hypot(dy) < 4.0 {
            return None;
        }
        let mut a = dy.atan2(dx).to_degrees();
        if a < INICIO_PADRAO {
            a += 360.0;
        }
        let f = (a - INICIO_PADRAO) / VARREDURA_PADRAO;
        (0.0..=1.0).contains(&f).then_some(f)
    }

    fn mensagem(&self, novo: f64, acao: &str) -> EngineMessage {
        let texto = escreve(novo, self.decimals.max(casas_do_passo(self.step)));
        if acao.is_empty() {
            EngineMessage::ContextPatch(vec![(self.chave.clone(), texto)])
        } else {
            EngineMessage::UiInputChanged {
                action: acao.to_string(),
                value: texto,
            }
        }
    }
}

/// Quantas casas o passo pede. `step="0.5"` escrevendo `"3"` perderia a metade
/// no caminho de volta — é a mesma conta que o `<slider>` faz a partir do
/// `step` como escrito.
pub(crate) fn casas_do_passo(step: f64) -> usize {
    if step <= 0.0 || step >= 1.0 {
        return 0;
    }
    let s = format!("{step}");
    s.split_once('.').map(|(_, d)| d.len()).unwrap_or(0).min(4)
}

impl canvas::Program<EngineMessage> for ProgramaDial {
    type State = EstadoDial;

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
                state.arrastando = true;
                let novo = no_degrau(
                    self.min + (self.max - self.min) * f as f64,
                    self.min,
                    self.max,
                    self.step,
                );
                Some(canvas::Action::publish(self.mensagem(novo, &self.on_change)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.arrastando => {
                let f = Self::fracao_no_cursor(bounds, cursor)?;
                let novo = no_degrau(
                    self.min + (self.max - self.min) * f as f64,
                    self.min,
                    self.max,
                    self.step,
                );
                // Sem mensagem quando o valor não mudou: arrastar dentro do
                // mesmo degrau publicaria uma reavaliação por pixel.
                if (novo - self.valor).abs() < f64::EPSILON {
                    return Some(canvas::Action::capture());
                }
                Some(canvas::Action::publish(self.mensagem(novo, &self.on_change)).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.arrastando =>
            {
                state.arrastando = false;
                // `on_release` é a porta de quem não quer um efeito colateral
                // por pixel arrastado — rede, disco —, e é a mesma do
                // `<slider>`. Sem ele, soltar não diz nada: o valor já foi.
                if self.on_release.is_empty() {
                    Some(canvas::Action::capture())
                } else {
                    Some(
                        canvas::Action::publish(self.mensagem(self.valor, &self.on_release))
                            .and_capture(),
                    )
                }
            }
            // A roda anda um degrau, como num `QDial` focado. `step="0"`
            // (contínuo) anda um centésimo da faixa, senão a roda não faria
            // nada.
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let passos = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 40.0,
                };
                if passos.abs() < f32::EPSILON {
                    return None;
                }
                let degrau = if self.step > 0.0 {
                    self.step
                } else {
                    (self.max - self.min) / 100.0
                };
                let novo = no_degrau(
                    self.valor + degrau * passos as f64,
                    self.min,
                    self.max,
                    self.step,
                );
                if (novo - self.valor).abs() < f64::EPSILON {
                    return None;
                }
                Some(canvas::Action::publish(self.mensagem(novo, &self.on_change)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let pal = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let centro = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let raio = (bounds.width.min(bounds.height) / 2.0 - 2.0).max(1.0);
        let cor = cor_ou(&self.cor, pal.primary.base.color);
        let f = fracao(self.valor, self.min, self.max);

        // A trilha inteira primeiro, o preenchido por cima: um anel só, em duas
        // passadas, é o que garante que as duas bordas coincidam exatamente.
        frame.fill(
            &anel(
                centro,
                raio,
                self.espessura,
                INICIO_PADRAO,
                VARREDURA_PADRAO,
            ),
            pal.background.strong.color,
        );
        if f > 0.0 {
            frame.fill(
                &anel(
                    centro,
                    raio,
                    self.espessura,
                    INICIO_PADRAO,
                    VARREDURA_PADRAO * f,
                ),
                cor,
            );
        }

        // As marcas, quando o markup as pede. `notches="10"` são onze traços
        // (as duas pontas contam), que é como uma régua se lê.
        if self.marcas > 0 {
            let interno = raio - self.espessura - 3.0;
            for i in 0..=self.marcas {
                let a = INICIO_PADRAO + VARREDURA_PADRAO * (i as f32 / self.marcas as f32);
                frame.stroke(
                    &Path::line(
                        na_borda(centro, interno, a),
                        na_borda(centro, interno - 5.0, a),
                    ),
                    Stroke::default()
                        .with_color(pal.background.strong.color)
                        .with_width(1.0),
                );
            }
        }

        // O corpo do knob e o ponteiro. O ponteiro sai do miolo e não do
        // centro: um traço que nasce no eixo vira um borrão quando o knob é
        // pequeno.
        let corpo = raio - self.espessura - 8.0;
        if corpo > 4.0 {
            frame.fill(&Path::circle(centro, corpo), pal.background.weak.color);
            frame.stroke(
                &Path::circle(centro, corpo),
                Stroke::default()
                    .with_color(pal.background.strong.color)
                    .with_width(1.0),
            );
            let a = INICIO_PADRAO + VARREDURA_PADRAO * f;
            frame.stroke(
                &Path::line(
                    na_borda(centro, corpo * 0.35, a),
                    na_borda(centro, corpo * 0.82, a),
                ),
                Stroke::default().with_color(cor).with_width(2.5),
            );
        }

        if self.mostra_valor {
            frame.fill_text(rotulo(
                escreve(self.valor, self.decimals),
                centro,
                (corpo * 0.5).clamp(9.0, 20.0),
                pal.background.base.text,
            ));
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

/// Monta o `<dial>` a partir do nó já avaliado.
#[allow(clippy::too_many_arguments)]
pub fn render_dial<'a>(
    context: &ContextMap,
    value_var: &str,
    min: f32,
    max: f32,
    step: f32,
    size: f32,
    notches: usize,
    color: &str,
    show_value: bool,
    decimals: usize,
    readonly: bool,
    on_change: &str,
    on_release: &str,
) -> Element<'a, EngineMessage> {
    let (min, max) = (min as f64, max as f64);
    let valor = valor_de(context, value_var, min).clamp(min.min(max), max.max(min));
    let lado = size.clamp(32.0, 600.0);

    Canvas::new(ProgramaDial {
        valor,
        min,
        max,
        step: step as f64,
        marcas: notches.min(60),
        // A espessura acompanha o tamanho: um anel de 12px num knob de 40 não
        // deixa miolo nenhum.
        espessura: (lado * 0.11).clamp(4.0, 18.0),
        cor: color.to_string(),
        mostra_valor: show_value,
        decimals,
        readonly,
        chave: value_var.to_string(),
        on_change: on_change.to_string(),
        on_release: on_release.to_string(),
    })
    .width(Length::Fixed(lado))
    .height(Length::Fixed(lado))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// <gauge> — o medidor de arco
// ─────────────────────────────────────────────────────────────────────────────

/// Uma faixa colorida do `<gauge>`: até onde ela vai, e de que cor.
#[derive(Debug, Clone, Copy)]
pub struct Faixa {
    pub ate: f64,
    pub cor: Color,
}

/// Lê as faixas de `bands`, que pode ser o **nome de uma chave** ou o JSON
/// escrito direto no atributo.
///
/// ```xml
/// <gauge value="cpu" bands='[{"to":60,"color":"#A6E3A1"},{"to":85,"color":"#F9E2AF"},{"to":100,"color":"#F38BA8"}]' />
/// ```
///
/// As duas formas porque as duas aparecem: os limites de um medidor de CPU são
/// constantes do desenho (JSON inline), mas os de um medidor de meta vêm do
/// backend (chave).
pub fn faixas(context: &ContextMap, bruto: &str) -> Vec<Faixa> {
    let texto = context.get(bruto).map(String::as_str).unwrap_or(bruto);
    let Ok(serde_json::Value::Array(itens)) = serde_json::from_str::<serde_json::Value>(texto)
    else {
        return Vec::new();
    };
    let mut v: Vec<Faixa> = itens
        .iter()
        .filter_map(|item| {
            let o = item.as_object()?;
            let pega = |nomes: &[&str]| nomes.iter().find_map(|n| o.get(*n));
            let ate = pega(&["to", "ate", "até", "max"])?.as_f64()?;
            let cor = pega(&["color", "cor"])
                .and_then(|c| c.as_str())
                .and_then(parse_hex_color)?;
            Some(Faixa { ate, cor })
        })
        .collect();
    // Ordenadas por limite: o desenho pinta uma sobre a outra do fim para o
    // começo, e uma lista fora de ordem apagaria as anteriores.
    v.sort_by(|a, b| a.ate.partial_cmp(&b.ate).unwrap_or(std::cmp::Ordering::Equal));
    v
}

/// `<gauge>` — ver [`crate::parser::NodeType::Gauge`].
pub struct ProgramaGauge {
    pub valor: f64,
    pub min: f64,
    pub max: f64,
    pub inicio: f32,
    pub varredura: f32,
    pub espessura: f32,
    pub cor: String,
    pub faixas: Vec<Faixa>,
    pub agulha: bool,
    pub mostra_valor: bool,
    pub decimals: usize,
    pub unidade: String,
    pub legenda: String,
}

impl canvas::Program<EngineMessage> for ProgramaGauge {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let pal = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let centro = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let raio = (bounds.width.min(bounds.height) / 2.0 - 2.0).max(1.0);
        let cor = cor_ou(&self.cor, pal.primary.base.color);

        frame.fill(
            &anel(centro, raio, self.espessura, self.inicio, self.varredura),
            pal.background.strong.color,
        );

        if self.faixas.is_empty() {
            let f = fracao(self.valor, self.min, self.max);
            if f > 0.0 {
                frame.fill(
                    &anel(centro, raio, self.espessura, self.inicio, self.varredura * f),
                    cor,
                );
            }
        } else {
            // Do fim para o começo: cada faixa cobre da origem até o limite
            // dela, e a de cima é sempre a menor — assim as emendas somem, que
            // é o que um anel de duas cores exige (ver o doc de `anel`).
            for faixa in self.faixas.iter().rev() {
                let f = fracao(faixa.ate, self.min, self.max);
                if f <= 0.0 {
                    continue;
                }
                frame.fill(
                    &anel(centro, raio, self.espessura, self.inicio, self.varredura * f),
                    faixa.cor,
                );
            }
        }

        if self.agulha {
            let a = self.inicio + self.varredura * fracao(self.valor, self.min, self.max);
            let ponta = raio - self.espessura - 4.0;
            if ponta > 6.0 {
                frame.stroke(
                    &Path::line(na_borda(centro, ponta * 0.15, a), na_borda(centro, ponta, a)),
                    Stroke::default()
                        .with_color(pal.background.base.text)
                        .with_width(2.0),
                );
                frame.fill(
                    &Path::circle(centro, 3.5),
                    pal.background.base.text,
                );
            }
        }

        // O número no meio, a unidade colada nele e a legenda embaixo — a
        // pilha que todo medidor de painel tem, e a razão de o `<gauge>` não
        // precisar de um `<text>` ao lado no markup.
        // A pilha do meio: número em cima, legenda embaixo. As duas alturas
        // saem do MIOLO (raio menos o anel), não do raio — senão um medidor de
        // anel grosso escreve por cima do próprio arco. E o número sobe meia
        // linha quando há legenda, senão os dois se encavalam.
        let corpo = raio - self.espessura;
        let tamanho = (corpo * 0.42).clamp(9.0, 30.0);
        if self.mostra_valor && corpo > 10.0 {
            let texto = if self.unidade.is_empty() {
                escreve(self.valor, self.decimals)
            } else {
                format!("{}{}", escreve(self.valor, self.decimals), self.unidade)
            };
            let sobe = if self.legenda.is_empty() {
                0.0
            } else {
                tamanho * 0.42
            };
            frame.fill_text(rotulo(
                texto,
                Point::new(centro.x, centro.y - sobe),
                tamanho,
                pal.background.base.text,
            ));
        }
        if !self.legenda.is_empty() && corpo > 10.0 {
            frame.fill_text(rotulo(
                self.legenda.clone(),
                Point::new(centro.x, centro.y + tamanho * 0.62),
                (corpo * 0.2).clamp(8.0, 13.0),
                pal.background.strong.color,
            ));
        }

        vec![frame.into_geometry()]
    }
}

/// Monta o `<gauge>`.
#[allow(clippy::too_many_arguments)]
pub fn render_gauge<'a>(
    context: &ContextMap,
    value_var: &str,
    min: f32,
    max: f32,
    size: f32,
    thickness: f32,
    start: f32,
    sweep: f32,
    color: &str,
    bands: &str,
    needle: bool,
    show_value: bool,
    decimals: usize,
    unit: &str,
    label: &str,
) -> Element<'a, EngineMessage> {
    let (min, max) = (min as f64, max as f64);
    let lado = size.clamp(32.0, 800.0);
    Canvas::new(ProgramaGauge {
        valor: valor_de(context, value_var, min).clamp(min.min(max), max.max(min)),
        min,
        max,
        inicio: start,
        varredura: sweep.clamp(-360.0, 360.0),
        espessura: thickness.clamp(1.0, lado / 2.0),
        cor: color.to_string(),
        faixas: faixas(context, bands),
        agulha: needle,
        mostra_valor: show_value,
        decimals,
        unidade: unit.to_string(),
        legenda: label.to_string(),
    })
    .width(Length::Fixed(lado))
    .height(Length::Fixed(lado))
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// <lcdnumber> — o display de sete segmentos
// ─────────────────────────────────────────────────────────────────────────────

/// `<lcdnumber>` — ver [`crate::parser::NodeType::LcdNumber`].
pub struct ProgramaLcd {
    pub texto: String,
    pub altura: f32,
    pub cor: String,
    /// Desenhar os segmentos apagados num tom fraco, como num display de
    /// verdade. É o que faz um `1` parecer um `1` num mostrador e não um traço
    /// solto no vazio.
    pub fantasma: bool,
}

impl ProgramaLcd {
    /// A largura que o texto ocupa. Ponto e dois-pontos custam menos que um
    /// dígito, exatamente como num display físico — é isso que faz `12:34`
    /// caber onde caberiam cinco dígitos.
    fn largura(&self) -> f32 {
        let d = self.altura * 0.6;
        let vao = self.altura * 0.12;
        self.texto
            .chars()
            .map(|c| if separador(c) { d * 0.35 } else { d } + vao)
            .sum::<f32>()
            .max(1.0)
    }
}

fn separador(c: char) -> bool {
    matches!(c, '.' | ',' | ':')
}

/// Os sete retângulos de um dígito, na célula dada. Ver a tabela de
/// [`crate::canvas::segmentos`] para a ordem dos bits.
fn desenha_digito(frame: &mut Frame, x: f32, y: f32, w: f32, h: f32, bits: u8, cor: Color) {
    let t = h * 0.13;
    let meio = h / 2.0;
    let barra = |i: u8| bits & (1 << i) != 0;
    let mut pinta = |px: f32, py: f32, pw: f32, ph: f32| {
        frame.fill(
            &Path::rectangle(Point::new(x + px, y + py), Size::new(pw, ph)),
            cor,
        );
    };
    let vertical = (meio - t * 1.5).max(0.5);

    if barra(0) {
        pinta(t, 0.0, w - 2.0 * t, t); // a
    }
    if barra(1) {
        pinta(w - t, t, t, vertical); // b
    }
    if barra(2) {
        pinta(w - t, meio + t * 0.5, t, vertical); // c
    }
    if barra(3) {
        pinta(t, h - t, w - 2.0 * t, t); // d
    }
    if barra(4) {
        pinta(0.0, meio + t * 0.5, t, vertical); // e
    }
    if barra(5) {
        pinta(0.0, t, t, vertical); // f
    }
    if barra(6) {
        pinta(t, meio - t * 0.5, w - 2.0 * t, t); // g
    }
}

impl canvas::Program<EngineMessage> for ProgramaLcd {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let pal = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let aceso = cor_ou(&self.cor, pal.primary.base.color);
        let apagado = Color {
            a: 0.16,
            ..aceso
        };

        let h = self.altura;
        let d = h * 0.6;
        let vao = h * 0.12;
        // Centrado nos dois eixos: o widget declara a altura, e a largura sai
        // do texto — deixar sobra à esquerda seria pedir ao markup que
        // compensasse na mão.
        let mut x = ((bounds.width - self.largura()) / 2.0).max(0.0);
        let y = ((bounds.height - h) / 2.0).max(0.0);

        for c in self.texto.chars() {
            if separador(c) {
                let l = d * 0.35;
                let r = (h * 0.07).max(1.0);
                let cx = x + l / 2.0;
                if c == ':' {
                    frame.fill(&Path::circle(Point::new(cx, y + h * 0.32), r), aceso);
                    frame.fill(&Path::circle(Point::new(cx, y + h * 0.68), r), aceso);
                } else {
                    frame.fill(&Path::circle(Point::new(cx, y + h - r), r), aceso);
                }
                x += l + vao;
                continue;
            }

            let bits = segmentos(c);
            if self.fantasma {
                desenha_digito(&mut frame, x, y, d, h, 0b1111111, apagado);
            }
            if bits != 0 {
                desenha_digito(&mut frame, x, y, d, h, bits, aceso);
            }
            x += d + vao;
        }

        vec![frame.into_geometry()]
    }
}

/// Monta o `<lcdnumber>`.
///
/// `digits` preenche à esquerda até o comprimento pedido — com zeros quando o
/// markup pede (`pad="0"`), com espaço em branco por padrão. É o
/// `setDigitCount` do `QLCDNumber`, e serve para o mostrador não mudar de
/// largura quando o número passa de 9 para 10.
#[allow(clippy::too_many_arguments)]
pub fn render_lcd_number<'a>(
    context: &ContextMap,
    value_var: &str,
    digits: usize,
    size: f32,
    color: &str,
    decimals: usize,
    pad_zeros: bool,
    ghost: bool,
) -> Element<'a, EngineMessage> {
    // O texto vem da chave como está — um `12:34` de relógio é texto, não
    // número. Só quando ele parseia como número é que `decimals` entra.
    let bruto = context
        .get(value_var)
        .cloned()
        .unwrap_or_else(|| value_var.to_string());
    let mut texto = match bruto.trim().parse::<f64>() {
        Ok(n) => escreve(n, decimals),
        Err(_) => bruto.trim().to_string(),
    };
    if digits > 0 {
        let faltam = digits.saturating_sub(texto.chars().filter(|c| !separador(*c)).count());
        if faltam > 0 {
            let enchimento: String = std::iter::repeat_n(if pad_zeros { '0' } else { ' ' }, faltam)
                .collect();
            // O sinal fica na frente do enchimento: `-007`, nunca `00-7`.
            texto = match texto.strip_prefix('-') {
                Some(resto) => format!("-{enchimento}{resto}"),
                None => format!("{enchimento}{texto}"),
            };
        }
    }

    let altura = size.clamp(8.0, 400.0);
    let programa = ProgramaLcd {
        texto,
        altura,
        cor: color.to_string(),
        fantasma: ghost,
    };
    let largura = programa.largura();
    Canvas::new(programa)
        .width(Length::Fixed(largura))
        .height(Length::Fixed(altura))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pares: &[(&str, &str)]) -> ContextMap {
        let mut m = ContextMap::default();
        for (k, v) in pares {
            m.insert(k.to_string(), v.to_string());
        }
        m
    }

    #[test]
    fn valor_e_chave_primeiro_e_literal_depois() {
        let c = ctx(&[("cpu", "42")]);
        assert_eq!(valor_de(&c, "cpu", 0.0), 42.0);
        assert_eq!(valor_de(&c, "70", 0.0), 70.0, "literal é o fallback");
        assert_eq!(valor_de(&c, "inexistente", 5.0), 5.0);
    }

    #[test]
    fn degrau_prende_na_faixa_e_no_passo() {
        assert_eq!(no_degrau(7.3, 0.0, 10.0, 1.0), 7.0);
        assert_eq!(no_degrau(7.6, 0.0, 10.0, 1.0), 8.0);
        assert_eq!(no_degrau(-5.0, 0.0, 10.0, 1.0), 0.0);
        assert_eq!(no_degrau(99.0, 0.0, 10.0, 1.0), 10.0);
        // Passo zero é contínuo, não divisão por zero.
        assert_eq!(no_degrau(7.3, 0.0, 10.0, 0.0), 7.3);
        // Degrau que não divide a faixa continua ancorado no mínimo.
        assert_eq!(no_degrau(3.0, 1.0, 10.0, 2.0), 3.0);
    }

    #[test]
    fn casas_saem_do_passo_como_escrito() {
        assert_eq!(casas_do_passo(1.0), 0);
        assert_eq!(casas_do_passo(0.5), 1);
        assert_eq!(casas_do_passo(0.25), 2);
        assert_eq!(casas_do_passo(0.0), 0, "contínuo não força casas");
    }

    #[test]
    fn faixas_aceitam_chave_e_json_inline() {
        let inline = r##"[{"to":85,"color":"#F9E2AF"},{"to":60,"color":"#A6E3A1"}]"##;
        let c = ctx(&[("limites", inline)]);

        for origem in [inline, "limites"] {
            let f = faixas(&c, origem);
            assert_eq!(f.len(), 2);
            // Sempre ordenadas: o desenho depende disso para as emendas não
            // aparecerem.
            assert_eq!(f[0].ate, 60.0);
            assert_eq!(f[1].ate, 85.0);
        }
        assert!(faixas(&c, "não é json").is_empty());
    }

    #[test]
    fn lcd_mede_separador_mais_estreito_que_digito() {
        let relogio = ProgramaLcd {
            texto: "12:34".into(),
            altura: 40.0,
            cor: String::new(),
            fantasma: false,
        };
        let cinco = ProgramaLcd {
            texto: "12345".into(),
            altura: 40.0,
            cor: String::new(),
            fantasma: false,
        };
        assert!(
            relogio.largura() < cinco.largura(),
            "`12:34` tem de caber onde `12345` cabe"
        );
    }
}
