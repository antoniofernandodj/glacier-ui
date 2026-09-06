//! Os quatro gráficos da Onda 7 — `<sparkline>`, `<linechart>`, `<barchart>` e
//! `<piechart>`, a §2.13 inteira do `PLANO_WIDGETS.md`, que estava em 0 de 6.
//!
//! # `canvas` na mão, e não `plotters`
//!
//! A decisão que a §4 do plano deixou aberta desde a primeira revisão, fechada
//! aqui por três razões, em ordem de peso:
//!
//! 1. **A cor.** Todo widget deste motor tira a cor da paleta do tema — é o que
//!    faz um `.gss` e um `theme.json` valerem para a tela inteira. O `plotters`
//!    traz o sistema de estilo dele, e um gráfico que ignora o tema é um
//!    retângulo estrangeiro no meio do app.
//! 2. **A manutenção.** O `plotters-iced` oficial parou no `iced 0.13`; para o
//!    0.14 só existe um fork de comunidade. Seria pôr seis linhas do catálogo
//!    atrás de uma dependência que pode não acompanhar o próximo bump.
//! 3. **O tamanho.** O `Cargo.toml` deste crate já trata o binário como caro
//!    (ele compila `wgpu`, `naga`, Luau e os codecs estaticamente). Uma árvore
//!    de desenho inteira por causa de eixos não se paga.
//!
//! O que o `plotters` traria de graça — eixos, escala e rótulos — é o que
//! [`crate::canvas::escala`] e a [`Moldura`] daqui resolvem em duzentas linhas,
//! e essas duzentas linhas servem os quatro gráficos.
//!
//! # O que fica de fora, de propósito
//!
//! **Séries múltiplas.** Um `items` é uma série; comparar duas no mesmo eixo
//! pediria uma segunda convenção de dados (`series="[{nome, pontos}]"`), uma
//! legenda e uma paleta por série. É trabalho de verdade e não é o gargalo de
//! nada: o caso comum de um painel é um número por gráfico. Fica anotado como
//! o próximo passo natural desta família, não como buraco.

use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};

use crate::ContextMap;
use crate::canvas::{Serie, compacto, cor_ou, escala, fracao, rotulo};
use crate::parser::UiNode;
use crate::widget::{EngineMessage, parse_hex_color, parse_length};

/// A área onde o dado é desenhado, já descontadas as margens dos rótulos.
///
/// Existe porque os três gráficos cartesianos fazem exatamente a mesma conta —
/// e porque errá-la em um só produziria um gráfico que *quase* encosta no eixo,
/// que é o tipo de defeito que ninguém reporta e todo mundo vê.
#[derive(Debug, Clone, Copy)]
pub struct Moldura {
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub altura: f32,
}

impl Moldura {
    /// As margens: à esquerda cabem os números do eixo Y, embaixo os rótulos do
    /// X. Sem eixos, tudo vira 2px de respiro — que é o `<sparkline>`.
    pub fn nova(bounds: Rectangle, eixos: bool) -> Self {
        let (esq, base, topo, dir) = if eixos {
            (36.0, 18.0, 8.0, 10.0)
        } else {
            (2.0, 2.0, 2.0, 2.0)
        };
        Self {
            x: esq,
            y: topo,
            largura: (bounds.width - esq - dir).max(1.0),
            altura: (bounds.height - topo - base).max(1.0),
        }
    }

    /// O x do i-ésimo ponto de `n`. Um ponto só cai no meio, senão dividiria
    /// por zero e o gráfico sumiria.
    pub fn px(&self, i: usize, n: usize) -> f32 {
        if n <= 1 {
            return self.x + self.largura / 2.0;
        }
        self.x + self.largura * (i as f32 / (n - 1) as f32)
    }

    /// O y de um valor. Invertido, porque o Y da tela cresce para baixo e o de
    /// um gráfico cresce para cima.
    pub fn py(&self, v: f64, min: f64, max: f64) -> f32 {
        self.y + self.altura * (1.0 - fracao(v, min, max))
    }

    pub fn base(&self) -> f32 {
        self.y + self.altura
    }
}

/// A cor da i-ésima fatia/barra quando o markup não escolheu nenhuma.
///
/// Um ciclo pela paleta estendida do tema, e não um arco-íris fixo: é o que faz
/// um `<piechart>` continuar parecendo parte do app quando o `theme.json` muda.
/// Dez cores antes de repetir — mais que isso, um gráfico de setores já não é
/// legível de qualquer jeito.
fn cor_ciclica(pal: &iced::theme::palette::Extended, i: usize) -> Color {
    let ciclo = [
        pal.primary.base.color,
        pal.success.base.color,
        pal.warning.base.color,
        pal.danger.base.color,
        pal.secondary.base.color,
        pal.primary.strong.color,
        pal.success.strong.color,
        pal.warning.strong.color,
        pal.danger.strong.color,
        pal.secondary.strong.color,
    ];
    ciclo[i % ciclo.len()]
}

/// Os limites do eixo Y: o que o markup fixou, ou o que a série pede.
///
/// Fixar importa mais do que parece — dois gráficos lado a lado com escalas
/// automáticas diferentes são a forma clássica de mentir com um painel sem
/// querer.
fn limites(serie: &Serie, min: &str, max: &str, alvo_marcas: usize) -> (f64, f64, f64) {
    let (auto_min, auto_max) = serie.faixa();
    let fixo_min = min.trim().parse::<f64>().ok();
    let fixo_max = max.trim().parse::<f64>().ok();

    let e = escala(
        fixo_min.unwrap_or(auto_min),
        fixo_max.unwrap_or(auto_max),
        alvo_marcas,
    );
    (
        fixo_min.unwrap_or(e.min),
        fixo_max.unwrap_or(e.max),
        e.passo,
    )
}

/// Desenha a grade e os números do eixo Y, mais os rótulos do X.
///
/// Os rótulos do X são **ralos quando não cabem**: escrever doze meses num
/// gráfico de 200px produz uma tarja preta, não uma legenda. A conta é a mesma
/// que o `<pagination>` faz com a janela de páginas — mostrar o que cabe, e o
/// que cabe é uma divisão.
#[allow(clippy::too_many_arguments)]
fn desenha_eixos(
    frame: &mut Frame,
    pal: &iced::theme::palette::Extended,
    m: &Moldura,
    serie: &Serie,
    min: f64,
    max: f64,
    passo: f64,
    grade: bool,
    centrado_na_barra: bool,
) {
    let fraco = pal.background.strong.color;
    let texto = Color {
        a: 0.75,
        ..pal.background.base.text
    };

    let e = crate::canvas::Escala { min, max, passo };
    for marca in e.marcas() {
        let y = m.py(marca, min, max);
        if grade {
            frame.stroke(
                &Path::line(Point::new(m.x, y), Point::new(m.x + m.largura, y)),
                Stroke::default()
                    .with_color(Color { a: 0.35, ..fraco })
                    .with_width(1.0),
            );
        }
        frame.fill_text(canvas::Text {
            content: compacto(marca),
            position: Point::new(m.x - 6.0, y),
            color: texto,
            size: 10.0.into(),
            align_x: iced::alignment::Horizontal::Right.into(),
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }

    // O eixo Y e o eixo X, por cima da grade.
    frame.stroke(
        &Path::line(Point::new(m.x, m.y), Point::new(m.x, m.base())),
        Stroke::default().with_color(fraco).with_width(1.0),
    );
    frame.stroke(
        &Path::line(
            Point::new(m.x, m.base()),
            Point::new(m.x + m.largura, m.base()),
        ),
        Stroke::default().with_color(fraco).with_width(1.0),
    );

    let n = serie.len();
    if n == 0 {
        return;
    }
    let cabem = ((m.largura / 34.0).floor() as usize).max(1);
    let salto = n.div_ceil(cabem).max(1);
    for (i, ponto) in serie.pontos.iter().enumerate() {
        if i % salto != 0 {
            continue;
        }
        let x = if centrado_na_barra {
            m.x + m.largura * ((i as f32 + 0.5) / n as f32)
        } else {
            m.px(i, n)
        };
        frame.fill_text(rotulo(
            ponto.rotulo.clone(),
            Point::new(x, m.base() + 9.0),
            10.0,
            texto,
        ));
    }
}

/// O aviso de série vazia, escrito no meio da moldura.
///
/// Um gráfico sem dados que desenha **nada** é indistinguível de um gráfico
/// quebrado; um que diz "sem dados" é uma tela carregando. Os quatro fazem
/// igual.
fn vazio(frame: &mut Frame, pal: &iced::theme::palette::Extended, bounds: Rectangle) {
    frame.fill_text(rotulo(
        "sem dados".into(),
        Point::new(bounds.width / 2.0, bounds.height / 2.0),
        11.0,
        pal.background.strong.color,
    ));
}

// ─────────────────────────────────────────────────────────────────────────────
// <sparkline> e <linechart> — a mesma linha, com e sem moldura
// ─────────────────────────────────────────────────────────────────────────────

/// `<sparkline>` e `<linechart>` — ver [`crate::parser::NodeType::LineChart`].
///
/// Um programa só para os dois porque a diferença entre eles é literalmente um
/// booleano: um `<sparkline>` é um `<linechart>` sem eixos, sem grade e sem
/// pontos. Duplicar o desenho para isso seria duplicar o lugar onde um bug de
/// escala pode morar.
pub struct ProgramaLinha {
    pub serie: Serie,
    pub min: String,
    pub max: String,
    pub cor: String,
    pub area: bool,
    pub pontos: bool,
    pub eixos: bool,
    pub grade: bool,
    pub espessura: f32,
}

impl canvas::Program<EngineMessage> for ProgramaLinha {
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
        if self.serie.is_empty() {
            vazio(&mut frame, pal, bounds);
            return vec![frame.into_geometry()];
        }

        let m = Moldura::nova(bounds, self.eixos);
        let (min, max, passo) = limites(&self.serie, &self.min, &self.max, 4);
        let cor = cor_ou(&self.cor, pal.primary.base.color);
        let n = self.serie.len();

        if self.eixos {
            desenha_eixos(
                &mut frame, pal, &m, &self.serie, min, max, passo, self.grade, false,
            );
        }

        // A área primeiro, a linha por cima: o preenchimento é o mesmo caminho
        // fechado pela base, e desenhá-lo depois cobriria a linha.
        if self.area {
            let caminho = Path::new(|b| {
                b.move_to(Point::new(m.px(0, n), m.base()));
                for (i, p) in self.serie.pontos.iter().enumerate() {
                    b.line_to(Point::new(m.px(i, n), m.py(p.valor, min, max)));
                }
                b.line_to(Point::new(m.px(n - 1, n), m.base()));
                b.close();
            });
            frame.fill(&caminho, Color { a: 0.18, ..cor });
        }

        let linha = Path::new(|b| {
            for (i, p) in self.serie.pontos.iter().enumerate() {
                let ponto = Point::new(m.px(i, n), m.py(p.valor, min, max));
                if i == 0 {
                    b.move_to(ponto);
                } else {
                    b.line_to(ponto);
                }
            }
        });
        frame.stroke(
            &linha,
            Stroke::default()
                .with_color(cor)
                .with_width(self.espessura)
                .with_line_join(canvas::LineJoin::Round)
                .with_line_cap(canvas::LineCap::Round),
        );

        // Os pontos só quando cabem: vinte bolinhas em 200px viram uma corda.
        if self.pontos && n <= 40 {
            for (i, p) in self.serie.pontos.iter().enumerate() {
                frame.fill(
                    &Path::circle(
                        Point::new(m.px(i, n), m.py(p.valor, min, max)),
                        self.espessura + 0.8,
                    ),
                    cor,
                );
            }
        }

        vec![frame.into_geometry()]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// <barchart>
// ─────────────────────────────────────────────────────────────────────────────

/// `<barchart>` — ver [`crate::parser::NodeType::BarChart`].
pub struct ProgramaBarras {
    pub serie: Serie,
    pub min: String,
    pub max: String,
    pub cor: String,
    /// Uma cor por barra (ciclando a paleta do tema) em vez de uma cor só. É o
    /// que separa "quanto por mês" (uma cor) de "quanto por categoria" (uma
    /// cor cada).
    pub colorido: bool,
    pub eixos: bool,
    pub grade: bool,
}

impl canvas::Program<EngineMessage> for ProgramaBarras {
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
        if self.serie.is_empty() {
            vazio(&mut frame, pal, bounds);
            return vec![frame.into_geometry()];
        }

        let m = Moldura::nova(bounds, self.eixos);
        // Uma barra nasce do zero, não do menor valor da série: um gráfico de
        // barras que começa em 40 exagera a diferença entre 41 e 42, e é o
        // erro de leitura mais comum que um painel produz.
        let (min, max, passo) = {
            let (mut lo, hi, passo) = limites(&self.serie, &self.min, &self.max, 4);
            if self.min.trim().is_empty() && lo > 0.0 {
                lo = 0.0;
            }
            (lo, hi, passo)
        };
        let n = self.serie.len();

        if self.eixos {
            desenha_eixos(
                &mut frame, pal, &m, &self.serie, min, max, passo, self.grade, true,
            );
        }

        let passo_x = m.largura / n as f32;
        let largura = (passo_x * 0.68).max(1.0);
        let zero = m.py(0.0f64.clamp(min, max), min, max);

        for (i, p) in self.serie.pontos.iter().enumerate() {
            let cor = if self.colorido {
                cor_ciclica(pal, i)
            } else {
                cor_ou(&self.cor, pal.primary.base.color)
            };
            let y = m.py(p.valor, min, max);
            let (topo, altura) = if y <= zero {
                (y, zero - y)
            } else {
                (zero, y - zero)
            };
            let x = m.x + passo_x * i as f32 + (passo_x - largura) / 2.0;
            // Altura mínima de 1px: uma barra de valor quase-zero que
            // desaparece some do gráfico e some da leitura.
            frame.fill(
                &Path::rectangle(Point::new(x, topo), Size::new(largura, altura.max(1.0))),
                cor,
            );
        }

        vec![frame.into_geometry()]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// <piechart>
// ─────────────────────────────────────────────────────────────────────────────

/// `<piechart>` — ver [`crate::parser::NodeType::PieChart`].
pub struct ProgramaPizza {
    pub serie: Serie,
    /// Buraco do meio, em fração do raio. `0` é pizza, `0.6` é rosquinha.
    pub buraco: f32,
    /// Escrever a porcentagem sobre cada fatia.
    pub percentuais: bool,
    /// Cores por fatia declaradas no markup (JSON de hex). Vazio = a paleta do
    /// tema.
    pub cores: Vec<Color>,
}

impl canvas::Program<EngineMessage> for ProgramaPizza {
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
        let total = self.serie.soma_positiva();
        if self.serie.is_empty() || total <= 0.0 {
            vazio(&mut frame, pal, bounds);
            return vec![frame.into_geometry()];
        }

        let centro = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let raio = (bounds.width.min(bounds.height) / 2.0 - 4.0).max(1.0);
        let espessura = if self.buraco > 0.0 {
            raio * (1.0 - self.buraco.clamp(0.0, 0.95))
        } else {
            raio
        };

        // Começa às 12 horas (−90°) porque é de onde todo olho começa a ler uma
        // pizza — o 0° do canvas é às 3 horas.
        let mut angulo = -90.0f32;
        for (i, p) in self.serie.pontos.iter().enumerate() {
            if p.valor <= 0.0 {
                continue;
            }
            let varredura = (p.valor / total) as f32 * 360.0;
            let cor = self
                .cores
                .get(i)
                .copied()
                .unwrap_or_else(|| cor_ciclica(pal, i));
            frame.fill(
                &crate::canvas::anel(centro, raio, espessura, angulo, varredura),
                cor,
            );

            // A porcentagem só cabe em fatia que tem onde escrever — abaixo de
            // uns 8% o número sai por cima da vizinha e piora a leitura em vez
            // de melhorar.
            if self.percentuais && varredura >= 28.0 {
                let meio = angulo + varredura / 2.0;
                let r = raio - espessura / 2.0;
                frame.fill_text(rotulo(
                    format!("{:.0}%", p.valor / total * 100.0),
                    crate::canvas::na_borda(centro, r, meio),
                    10.0,
                    contraste(cor),
                ));
            }
            angulo += varredura;
        }

        vec![frame.into_geometry()]
    }
}

/// Preto ou branco, o que se lê melhor sobre a cor dada.
///
/// A luminância percebida, não a média dos canais: sobre o amarelo de um tema
/// claro, branco some — e é justamente o amarelo que costuma ser a terceira
/// fatia.
fn contraste(c: Color) -> Color {
    let l = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;
    if l > 0.6 {
        Color::from_rgb(0.1, 0.1, 0.1)
    } else {
        Color::WHITE
    }
}

/// Cores declaradas no markup: chave do contexto ou JSON inline, como o
/// `bands` do `<gauge>`.
fn cores_de(context: &ContextMap, bruto: &str) -> Vec<Color> {
    if bruto.trim().is_empty() {
        return Vec::new();
    }
    let texto = context.get(bruto).map(String::as_str).unwrap_or(bruto);
    match serde_json::from_str::<serde_json::Value>(texto) {
        Ok(serde_json::Value::Array(v)) => v
            .iter()
            .filter_map(|c| c.as_str().and_then(parse_hex_color))
            .collect(),
        // Sem JSON, uma lista separada por vírgula — `colors="#f00,#0f0"` é o
        // que alguém escreve antes de ler a doc, e funcionar é mais barato do
        // que explicar.
        _ => texto
            .split(',')
            .filter_map(|c| parse_hex_color(c.trim()))
            .collect(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Montagem
// ─────────────────────────────────────────────────────────────────────────────

/// Largura e altura do gráfico: o que o nó declarou, ou o default da tag.
///
/// Passa pelo `parse_length` do motor, então `width="fill"` e `width="fill 2"`
/// valem num gráfico como valem numa `<row>` — é o que permite um painel de
/// quatro gráficos dividir a linha sem contas no markup.
fn tamanho(node: &UiNode, padrao_w: f32, padrao_h: f32) -> (Length, Length) {
    let w = if node.width.is_some() {
        parse_length(&node.width)
    } else {
        Length::Fixed(padrao_w)
    };
    let h = if node.height.is_some() {
        parse_length(&node.height)
    } else {
        Length::Fixed(padrao_h)
    };
    (w, h)
}

/// Monta `<linechart>` e `<sparkline>`.
#[allow(clippy::too_many_arguments)]
pub fn render_line_chart<'a>(
    node: &UiNode,
    context: &ContextMap,
    items_var: &str,
    min: &str,
    max: &str,
    color: &str,
    area: bool,
    points: bool,
    axes: bool,
    grid: bool,
    thickness: f32,
) -> Element<'a, EngineMessage> {
    let (w, h) = tamanho(node, if axes { 320.0 } else { 120.0 }, if axes { 180.0 } else { 34.0 });
    Canvas::new(ProgramaLinha {
        serie: Serie::ler(context, items_var),
        min: min.to_string(),
        max: max.to_string(),
        cor: color.to_string(),
        area,
        pontos: points,
        eixos: axes,
        grade: grid,
        espessura: thickness.clamp(0.5, 12.0),
    })
    .width(w)
    .height(h)
    .into()
}

/// Monta `<barchart>`.
#[allow(clippy::too_many_arguments)]
pub fn render_bar_chart<'a>(
    node: &UiNode,
    context: &ContextMap,
    items_var: &str,
    min: &str,
    max: &str,
    color: &str,
    colorful: bool,
    axes: bool,
    grid: bool,
) -> Element<'a, EngineMessage> {
    let (w, h) = tamanho(node, 320.0, 180.0);
    Canvas::new(ProgramaBarras {
        serie: Serie::ler(context, items_var),
        min: min.to_string(),
        max: max.to_string(),
        cor: color.to_string(),
        colorido: colorful,
        eixos: axes,
        grade: grid,
    })
    .width(w)
    .height(h)
    .into()
}

/// Monta `<piechart>`.
pub fn render_pie_chart<'a>(
    node: &UiNode,
    context: &ContextMap,
    items_var: &str,
    size: f32,
    donut: f32,
    percentages: bool,
    colors: &str,
) -> Element<'a, EngineMessage> {
    let lado = size.clamp(32.0, 800.0);
    let (w, h) = tamanho(node, lado, lado);
    Canvas::new(ProgramaPizza {
        serie: Serie::ler(context, items_var),
        buraco: donut,
        percentuais: percentages,
        cores: cores_de(context, colors),
    })
    .width(w)
    .height(h)
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

    fn serie(json: &str) -> Serie {
        Serie::ler(&ctx(&[("s", json)]), "s")
    }

    #[test]
    fn moldura_sem_eixos_usa_a_area_toda() {
        let b = Rectangle::new(Point::ORIGIN, Size::new(100.0, 40.0));
        let com = Moldura::nova(b, true);
        let sem = Moldura::nova(b, false);
        assert!(sem.largura > com.largura, "o sparkline não paga margem");
        assert_eq!(sem.x, 2.0);
    }

    #[test]
    fn py_inverte_o_eixo() {
        let m = Moldura::nova(
            Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0)),
            false,
        );
        // O maior valor fica em CIMA, ou seja, no menor y.
        assert!(m.py(10.0, 0.0, 10.0) < m.py(0.0, 0.0, 10.0));
        assert_eq!(m.py(10.0, 0.0, 10.0), m.y);
        assert_eq!(m.py(0.0, 0.0, 10.0), m.base());
    }

    #[test]
    fn px_de_um_ponto_so_cai_no_meio() {
        let m = Moldura::nova(
            Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0)),
            false,
        );
        assert_eq!(m.px(0, 1), m.x + m.largura / 2.0);
        assert_eq!(m.px(0, 2), m.x);
        assert_eq!(m.px(1, 2), m.x + m.largura);
    }

    #[test]
    fn limites_fixos_ganham_do_automatico() {
        let s = serie("[10, 20, 30]");
        let (min, max, _) = limites(&s, "0", "100", 4);
        assert_eq!((min, max), (0.0, 100.0));

        // Sem fixar, a escala arredonda para fora — o topo da série não pode
        // encostar na borda.
        let (min, max, _) = limites(&s, "", "", 4);
        assert!(min <= 10.0 && max >= 30.0);
    }

    #[test]
    fn cores_aceitam_json_chave_e_lista_por_virgula() {
        let c = ctx(&[("paleta", r##"["#ff0000","#00ff00"]"##)]);
        assert_eq!(cores_de(&c, "paleta").len(), 2);
        assert_eq!(cores_de(&c, r##"["#ff0000"]"##).len(), 1);
        assert_eq!(cores_de(&c, "#ff0000, #00ff00, #0000ff").len(), 3);
        assert!(cores_de(&c, "").is_empty());
    }

    #[test]
    fn contraste_escolhe_o_que_se_le() {
        assert_eq!(contraste(Color::WHITE).r, 0.1, "sobre claro, texto escuro");
        assert_eq!(contraste(Color::BLACK), Color::WHITE);
        // O caso que motivou a luminância percebida em vez da média.
        assert_eq!(contraste(Color::from_rgb(0.98, 0.89, 0.69)).r, 0.1);
    }

    #[test]
    fn pizza_ignora_fatia_negativa_sem_comer_as_vizinhas() {
        let s = serie("[10, -5, 10]");
        assert_eq!(s.soma_positiva(), 20.0);
    }
}
