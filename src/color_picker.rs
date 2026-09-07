//! O `<colorwheel>` da Onda 8: a roda de matiz mais o quadrado de
//! saturação/valor — o miolo do `QColorDialog`.
//!
//! # O que a Onda 7 pagou por este widget
//!
//! O `ColorDialog` estava na proposta da Onda 7 e ficou de fora por tamanho.
//! O que mudou desde então é que a onda deixou pronto exatamente o que faltava:
//! [`crate::canvas`] (os arcos e o anel desenhados por polilinha, um segmento
//! por grau) e o `<maskedinput>` da Onda 4, que é o campo hexadecimal sem uma
//! linha nova. O que sobra aqui é a conversão HSV↔RGB e o gesto.
//!
//! # O valor mora numa chave, como em todo o resto
//!
//! `<colorwheel value="cor" />` escreve `#rrggbb` na chave `cor`, e lê dela o
//! ponto onde desenhar os dois cursores. Não há estado de cor no widget: o
//! `Program::State` guarda só **qual dos dois controles** está sendo arrastado
//! agora, que é o que morre com o gesto.
//!
//! É a mesma regra do `<dial>` (0.93), do `<slider>` e do `<rating>` — e é o
//! que faz o `ColorDialog` não precisar de estado por instância, que é como o
//! catálogo o classificava.
//!
//! # Por que a roda é de segmentos, e não um gradiente
//!
//! O `canvas` do `iced` preenche caminhos com uma cor sólida ou um gradiente
//! linear/radial; um gradiente **angular** (que é o que uma roda de matiz é)
//! ele não tem. Então a roda são 180 setores de 2°, cada um com a sua cor — a
//! mesma técnica do `anel` da Onda 7, e a resolução em que a emenda entre
//! setores deixa de ser visível a olho nu num anel dessa espessura.

use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};

use crate::ContextMap;
use crate::widget::{EngineMessage, parse_hex_color};

/// Espessura do anel de matiz, como fração do raio.
const ESPESSURA_ANEL: f32 = 0.18;
/// Folga entre o anel e o quadrado de saturação/valor, como fração do raio.
const FOLGA: f32 = 0.06;

/// Converte HSV (h em graus 0..360, s e v em 0..1) para RGB.
///
/// A forma canônica do algoritmo — sexteto, `f`, e os três casos por setor.
/// Está aqui e não no [`crate::canvas`] porque é a única coisa deste módulo que
/// mais alguém poderia querer, e mover algo para lá antes do segundo
/// consumidor é adivinhar.
pub fn hsv_para_rgb(h: f32, s: f32, v: f32) -> Color {
    let h = h.rem_euclid(360.0) / 60.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::from_rgb(r + m, g + m, b + m)
}

/// Converte RGB para HSV. A volta do [`hsv_para_rgb`].
///
/// Um cinza (saturação zero) não tem matiz definido, e a função devolve `0`
/// para ele. Isso é matematicamente certo e visualmente irritante: arrastar o
/// valor até o preto e voltar perderia o matiz que o usuário tinha escolhido.
/// Quem chama resolve isso guardando o matiz anterior — ver
/// [`ProgramaRoda::hsv_atual`].
pub fn rgb_para_hsv(c: Color) -> (f32, f32, f32) {
    let (r, g, b) = (c.r, c.g, c.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d.abs() < f32::EPSILON {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let s = if max.abs() < f32::EPSILON {
        0.0
    } else {
        d / max
    };
    (h.rem_euclid(360.0), s, max)
}

/// Lê uma cor de `#rrggbb` — o mesmo `parse_hex_color` do resto do motor,
/// reexportado aqui para quem trabalha com a roda não precisar saber onde ele
/// mora.
pub fn hex_para_cor(s: &str) -> Option<Color> {
    parse_hex_color(s.trim())
}

/// Escreve uma cor como `#rrggbb`. Sempre em minúsculas e sempre com seis
/// dígitos: o valor vai para uma chave de contexto, e uma chave que ora tem
/// `#FFF` ora `#ffffff` faz duas comparações de igualdade discordarem.
pub fn para_hex(c: Color) -> String {
    let byte = |f: f32| (f.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", byte(c.r), byte(c.g), byte(c.b))
}

/// Lê um hexadecimal **em digitação** e diz se ele já é uma cor.
///
/// O `#` é opcional e o texto é normalizado para minúsculas com `#` na frente,
/// porque é isso que vai para a chave — e uma chave que ora tem `#FFF` ora
/// `#ffffff` faz duas comparações de igualdade discordarem.
///
/// Aceita **seis** dígitos hexadecimais, com o `#` opcional, e nada mais.
///
/// # Por que a forma curta (`#f0a`) fica de fora
///
/// Ela é uma cor válida em CSS, e a tentação de aceitá-la é grande. Mas ela é
/// também um **estado intermediário** de quem está digitando seis: no caminho
/// de `#ff8800`, o texto passa por `#ff8` — que, lido como forma curta, é
/// `#ffff88`, um amarelo que ninguém pediu. Aceitá-la trocaria o defeito que
/// esta função existe para consertar (a roda piscando **branco**) por um
/// parente mais discreto e mais confuso: a roda piscando **outra cor**.
///
/// Seis dígitos deixam a regra sem exceção — o campo comete exatamente quando
/// o texto é uma cor inteira, e em nenhum outro instante. Quem quer `#f0a`
/// escreve `#ff00aa`, ou aponta na roda, que é para o que ela serve.
pub fn hex_completo(digitado: &str) -> Option<String> {
    let bruto = digitado.trim().trim_start_matches('#');
    if bruto.len() != 6 || !bruto.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    // Passa pelo parser do motor mesmo tendo validado o formato: é ele quem
    // decide o que seis dígitos significam, e duplicar essa regra aqui seria
    // criar um segundo lugar para ela divergir.
    parse_hex_color(&format!("#{bruto}")).map(para_hex)
}

/// Qual dos dois controles o gesto pegou.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Arrasto {
    #[default]
    Nenhum,
    /// O anel de matiz.
    Matiz,
    /// O quadrado de saturação × valor.
    SatVal,
}

/// O estado que morre com o gesto — e é só isto. A cor está na chave.
#[derive(Debug, Default)]
pub struct EstadoRoda {
    arrasto: Arrasto,
}

struct ProgramaRoda {
    chave: String,
    /// A cor atual, já lida da chave.
    cor: Color,
    /// O matiz a usar quando a cor atual é um cinza — ver [`rgb_para_hsv`].
    /// Vem do próprio contexto, numa chave irmã, para sobreviver ao quadro.
    matiz_reserva: f32,
    on_change: String,
    readonly: bool,
}

impl ProgramaRoda {
    /// O HSV da cor atual, com o matiz preservado quando ela é um cinza.
    fn hsv_atual(&self) -> (f32, f32, f32) {
        let (h, s, v) = rgb_para_hsv(self.cor);
        if s.abs() < f32::EPSILON {
            (self.matiz_reserva, s, v)
        } else {
            (h, s, v)
        }
    }

    fn geometria(bounds: Rectangle) -> (Point, f32, f32, f32) {
        let centro = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let raio = (bounds.width.min(bounds.height) / 2.0 - 2.0).max(1.0);
        let raio_interno = raio * (1.0 - ESPESSURA_ANEL);
        // O maior quadrado inscrito no círculo de raio `r` tem lado `r·√2`.
        let lado = (raio_interno * (1.0 - FOLGA)) * std::f32::consts::SQRT_2;
        (centro, raio, raio_interno, lado)
    }

    fn mensagem(&self, cor: Color, h: f32) -> EngineMessage {
        let hex = para_hex(cor);
        // Duas chaves irmãs viajam junto com a cor, e cada uma resolve um
        // problema diferente:
        //
        // `__h` — o matiz. Sem ele, arrastar o valor até o preto zeraria a
        // saturação, o matiz viraria 0 (vermelho) e voltar devolveria a cor
        // errada.
        //
        // `__hex` — o **texto** da cor, para um campo editável acompanhar a
        // roda. Ele é sempre igual à cor cometida neste instante; o que o
        // separa da chave principal é o sentido contrário, quando alguém
        // digita: aí o campo escreve só em `__hex`, e a cor só é cometida
        // quando o texto vira uma cor de verdade. É o que impede a roda de
        // piscar branco enquanto se digita `#f`, `#ff`, `#ff8`.
        let patch = vec![
            (self.chave.clone(), hex.clone()),
            (format!("{}__h", self.chave), format!("{h:.1}")),
            (format!("{}__hex", self.chave), hex.clone()),
        ];
        if self.on_change.is_empty() {
            EngineMessage::ContextPatch(patch)
        } else {
            EngineMessage::UiInputChanged {
                action: self.on_change.clone(),
                value: hex,
            }
        }
    }
}

impl canvas::Program<EngineMessage> for ProgramaRoda {
    type State = EstadoRoda;

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
        let (centro, raio, raio_interno, lado) = Self::geometria(bounds);
        let (h, s, v) = self.hsv_atual();

        // Onde o cursor está, e o que ele pega ali.
        let alvo = |p: Point| -> Arrasto {
            let d = (p.x - centro.x).hypot(p.y - centro.y);
            if d > raio_interno && d <= raio {
                Arrasto::Matiz
            } else if (p.x - centro.x).abs() <= lado / 2.0 && (p.y - centro.y).abs() <= lado / 2.0 {
                Arrasto::SatVal
            } else {
                Arrasto::Nenhum
            }
        };

        // O que o ponto significa para o controle que está sendo arrastado.
        // Repare que aqui NÃO se pergunta de novo onde o cursor está: uma vez
        // pego o anel, arrastar para fora dele continua girando o matiz. É o
        // comportamento de todo seletor de cor, e o oposto do que sair
        // consultando `alvo` a cada movimento daria.
        let aplica = |arrasto: Arrasto, p: Point| -> Option<(Color, f32)> {
            match arrasto {
                Arrasto::Matiz => {
                    let (dx, dy) = (p.x - centro.x, p.y - centro.y);
                    if dx.hypot(dy) < 2.0 {
                        return None; // no eixo, o ângulo é ruído
                    }
                    let novo_h = dy.atan2(dx).to_degrees().rem_euclid(360.0);
                    Some((hsv_para_rgb(novo_h, s, v), novo_h))
                }
                Arrasto::SatVal => {
                    let sx = ((p.x - centro.x) / lado + 0.5).clamp(0.0, 1.0);
                    // O eixo Y da tela cresce para baixo, e o valor cresce para
                    // cima: 1 menos a fração.
                    let sy = 1.0 - ((p.y - centro.y) / lado + 0.5).clamp(0.0, 1.0);
                    Some((hsv_para_rgb(h, sx, sy), h))
                }
                Arrasto::Nenhum => None,
            }
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                let a = alvo(p);
                if a == Arrasto::Nenhum {
                    return None;
                }
                state.arrasto = a;
                let (cor, novo_h) = aplica(a, p)?;
                Some(canvas::Action::publish(self.mensagem(cor, novo_h)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.arrasto != Arrasto::Nenhum => {
                let p = cursor.position_in(bounds)?;
                let (cor, novo_h) = aplica(state.arrasto, p)?;
                Some(canvas::Action::publish(self.mensagem(cor, novo_h)).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.arrasto != Arrasto::Nenhum =>
            {
                state.arrasto = Arrasto::Nenhum;
                Some(canvas::Action::capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let (centro, raio, raio_interno, lado) = Self::geometria(bounds);
        let (h, s, v) = self.hsv_atual();

        // 1. O anel de matiz, 180 setores de 2°. Ver o cabeçalho do módulo
        //    para por que não é um gradiente.
        let passo = 2.0_f32;
        let mut a = 0.0_f32;
        while a < 360.0 {
            // Meio grau de sobreposição fecha a emenda: sem ela, o
            // antialiasing deixa uma linha de fundo entre setores vizinhos.
            let setor = crate::canvas::anel(centro, raio, raio - raio_interno, a, passo + 0.5);
            frame.fill(&setor, hsv_para_rgb(a + passo / 2.0, 1.0, 1.0));
            a += passo;
        }

        // 2. O quadrado de saturação × valor. Uma grade de células sólidas —
        //    o `iced` tem gradiente linear em um eixo, e este quadrado varia em
        //    dois. 24×24 é onde a banda deixa de ser perceptível num quadrado
        //    desse tamanho, e são 576 quads: caro para uma tela, barato para um
        //    modal que existe por alguns segundos.
        let celulas = 24;
        let passo_c = lado / celulas as f32;
        let canto = Point::new(centro.x - lado / 2.0, centro.y - lado / 2.0);
        for ix in 0..celulas {
            for iy in 0..celulas {
                let sc = (ix as f32 + 0.5) / celulas as f32;
                let vc = 1.0 - (iy as f32 + 0.5) / celulas as f32;
                frame.fill_rectangle(
                    Point::new(canto.x + ix as f32 * passo_c, canto.y + iy as f32 * passo_c),
                    // Meio pixel a mais fecha a emenda entre células, pelo
                    // mesmo motivo dos setores.
                    Size::new(passo_c + 0.5, passo_c + 0.5),
                    hsv_para_rgb(h, sc, vc),
                );
            }
        }

        // 3. Os dois cursores. Ambos são anéis de contorno, não bolinhas
        //    cheias: um marcador opaco esconde justamente a cor que ele aponta.
        let no_anel = crate::canvas::na_borda(centro, (raio + raio_interno) / 2.0, h);
        frame.stroke(
            &Path::circle(no_anel, (raio - raio_interno) / 2.0 - 1.0),
            Stroke::default()
                .with_color(contraste(hsv_para_rgb(h, 1.0, 1.0)))
                .with_width(2.0),
        );

        let alvo = Point::new(
            canto.x + s.clamp(0.0, 1.0) * lado,
            canto.y + (1.0 - v.clamp(0.0, 1.0)) * lado,
        );
        frame.stroke(
            &Path::circle(alvo, 6.0),
            Stroke::default()
                .with_color(contraste(self.cor))
                .with_width(2.0),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !self.readonly && cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Preto ou branco, o que se enxergar sobre `fundo`.
///
/// A luminância percebida, não a média dos canais: o verde pesa seis vezes o
/// azul, e um marcador branco sobre amarelo puro some se a conta for a média.
fn contraste(fundo: Color) -> Color {
    let l = 0.299 * fundo.r + 0.587 * fundo.g + 0.114 * fundo.b;
    if l > 0.6 { Color::BLACK } else { Color::WHITE }
}

/// Monta o `<colorwheel>`.
pub fn render_color_wheel<'a>(
    context: &'a ContextMap,
    value_var: &'a str,
    size: f32,
    on_change: &'a str,
    readonly: bool,
) -> Element<'a, EngineMessage> {
    // Uma chave vazia ou com lixo vira branco — a mesma degradação silenciosa
    // de todo widget do motor diante de uma chave ainda não semeada.
    let cor = context
        .get(value_var)
        .and_then(|s| parse_hex_color(s.trim()))
        .unwrap_or(Color::WHITE);
    let matiz_reserva = context
        .get(&format!("{value_var}__h"))
        .and_then(|s| s.trim().parse::<f32>().ok())
        .unwrap_or(0.0);

    let lado = if size > 0.0 { size } else { 220.0 };
    Canvas::new(ProgramaRoda {
        chave: value_var.to_string(),
        cor,
        matiz_reserva,
        on_change: on_change.to_string(),
        readonly,
    })
    .width(Length::Fixed(lado))
    .height(Length::Fixed(lado))
    .into()
}
