//! O **vocabulário de formas** do `<canvas>` — o habilitador da Onda 13 do
//! `PLANO_WIDGETS.md`.
//!
//! # A decisão da Onda 7, executada
//!
//! A Onda 7 pôs o `canvas` do `iced` no motor como **capacidade**
//! ([`crate::canvas`]) e escreveu a condicional: *"se o `Canvas` sair, sai como
//! vocabulário declarativo (`<path>`, `<arc>`, `<circle>`)"*. É o que este
//! módulo faz — `<canvas>` como pai, e sob ele nós de forma, cada um lido no
//! render e traduzido para um [`canvas::Path`]. Nenhum callback imperativo: o
//! `.gv` descreve a geometria, o `.gss` o traço.
//!
//! # Geometria é dado, traço é estilo
//!
//! Pela regra do `CLAUDE.md`: `cx="{x}"`, `d="{traçado}"`, `points="{serie}"`
//! são **dado** e ficam inline no `.gv` (interpolados no eval, como
//! `background="{cor}"`); `fill`/`stroke`/`stroke_width` são **estilo** e saem
//! de uma classe `.gss` com nome de papel — reusando os campos que já existem
//! (`background` = `fill`, `border_color` = `stroke`, `border_width` =
//! `stroke_width`), com apelidos no parser do `.gss` e no `.gv`.
//!
//! # O que fica de fora, por escrito
//!
//! O comando `A`/`a` de `<path>` (arco elíptico) — pede a decomposição elíptica
//! que nenhuma das formas nomeadas precisa; use `<arc>` para arco circular.
//! Interação (`on_click`/hover numa forma) e animação: a Onda 7 já decidiu que
//! um alvo de clique dentro de um `canvas` é geometria que o `.gv` não
//! descreve.

use std::collections::HashMap;

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::{Color, Point, Rectangle, Size, mouse};

use crate::widget::EngineMessage;

/// As oito formas nomeadas. `<canvas>` é o pai; cada uma é um nó filho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormaKind {
    /// `<path d="M0 0 L10 10 …">` — subconjunto de comandos SVG (`M L H V C Q Z`,
    /// maiúsculo absoluto, minúsculo relativo). Sem `A` (ver o doc do módulo).
    Path,
    /// `<arc cx cy r start sweep>` — ângulos em graus, `start=0` à direita,
    /// `sweep` positivo horário (a convenção de [`crate::canvas::graus`]).
    Arc,
    /// `<circle cx cy r>`.
    Circle,
    /// `<rect x y w h>` (mais `rx` para cantos, opcional).
    Rect,
    /// `<line x1 y1 x2 y2>`.
    Line,
    /// `<polyline points="x,y x,y …">` — aberta.
    Polyline,
    /// `<polygon points="x,y x,y …">` — fechada.
    Polygon,
    /// `<text x y>conteúdo</text>` — o `size` vem do `.gss` (`size:`).
    Text,
}

/// Uma forma já avaliada, pronta para virar `Path` no render.
#[derive(Debug, Clone)]
pub struct Forma {
    pub kind: FormaKind,
    /// Atributos de geometria, já interpolados (`"cx" -> "40"`, `"d" -> "M0 0…"`).
    /// Um mapa em vez de doze `Option<String>`: as formas não compartilham os
    /// mesmos atributos e a leitura é por nome.
    pub geo: HashMap<String, String>,
    /// O texto de um `<text>`.
    pub texto: String,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: f32,
    /// Corpo do `<text>`, do `.gss` (`size:`). Default 13.
    pub text_size: f32,
}

impl Forma {
    /// Um atributo de geometria como `f32`, ou `padrao` se ausente/inválido.
    fn n(&self, chave: &str, padrao: f32) -> f32 {
        self.geo
            .get(chave)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(padrao)
    }

    /// O `Path` do traço/preenchimento comum. `None` para `Text` (desenhado à
    /// parte) e para uma forma sem geometria utilizável.
    fn caminho(&self) -> Option<Path> {
        match self.kind {
            FormaKind::Circle => Some(Path::circle(
                Point::new(self.n("cx", 0.0), self.n("cy", 0.0)),
                self.n("r", 0.0).max(0.0),
            )),
            FormaKind::Rect => {
                let (x, y) = (self.n("x", 0.0), self.n("y", 0.0));
                let (w, h) = (self.n("w", 0.0).max(0.0), self.n("h", 0.0).max(0.0));
                let rx = self.n("rx", 0.0).max(0.0).min(w.min(h) / 2.0);
                if rx <= 0.5 {
                    Some(Path::new(|b| b.rectangle(Point::new(x, y), Size::new(w, h))))
                } else {
                    Some(retangulo_arredondado(x, y, w, h, rx))
                }
            }
            FormaKind::Line => Some(Path::new(|b| {
                b.move_to(Point::new(self.n("x1", 0.0), self.n("y1", 0.0)));
                b.line_to(Point::new(self.n("x2", 0.0), self.n("y2", 0.0)));
            })),
            FormaKind::Arc => {
                let centro = Point::new(self.n("cx", 0.0), self.n("cy", 0.0));
                let r = self.n("r", 0.0).max(0.0);
                let inicio = self.n("start", 0.0);
                let sweep = self.n("sweep", 360.0);
                // Com `fill`, um arco é um SETOR (fecha pelo centro); só com
                // traço, é a curva aberta.
                if self.fill.is_some() {
                    Some(crate::canvas::anel(centro, r, r, inicio, sweep))
                } else {
                    Some(crate::canvas::arco(centro, r, inicio, sweep))
                }
            }
            FormaKind::Polyline | FormaKind::Polygon => {
                let pts = pontos(self.geo.get("points").map(String::as_str).unwrap_or(""));
                if pts.len() < 2 {
                    return None;
                }
                let fechar = self.kind == FormaKind::Polygon;
                Some(Path::new(|b| {
                    b.move_to(pts[0]);
                    for p in &pts[1..] {
                        b.line_to(*p);
                    }
                    if fechar {
                        b.close();
                    }
                }))
            }
            FormaKind::Path => {
                caminho_svg(self.geo.get("d").map(String::as_str).unwrap_or(""))
            }
            FormaKind::Text => None,
        }
    }
}

/// O `canvas::Program` de um `<canvas>`: desenha a lista de formas, na ordem do
/// markup (a de cima primeiro, como o `<stack>` da Onda 11).
pub struct ProgramaFormas {
    pub formas: Vec<Forma>,
}

impl canvas::Program<EngineMessage> for ProgramaFormas {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let cor_texto = theme.extended_palette().background.base.text;

        for forma in &self.formas {
            if forma.kind == FormaKind::Text {
                let cor = forma.fill.or(forma.stroke).unwrap_or(cor_texto);
                frame.fill_text(canvas::Text {
                    content: forma.texto.clone(),
                    position: Point::new(forma.n("x", 0.0), forma.n("y", 0.0)),
                    color: cor,
                    size: forma.text_size.into(),
                    ..canvas::Text::default()
                });
                continue;
            }

            let Some(path) = forma.caminho() else {
                continue;
            };

            if let Some(c) = forma.fill {
                frame.fill(&path, c);
            }
            if let Some(c) = forma.stroke {
                frame.stroke(
                    &path,
                    Stroke::default().with_color(c).with_width(forma.stroke_width),
                );
            }
            // Nem traço nem preenchimento declarados: um traço na cor do texto
            // do tema, para a forma não sair invisível — a mesma degradação do
            // `font_for` que cai na fonte padrão.
            if forma.fill.is_none() && forma.stroke.is_none() {
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(cor_texto)
                        .with_width(forma.stroke_width),
                );
            }
        }

        vec![frame.into_geometry()]
    }
}

/// `"x,y x,y x,y"` → pontos. Aceita vírgula ou espaço entre `x` e `y`, e
/// qualquer um dos dois entre pares — a mesma folga que o `points` de um SVG
/// tem. Tokens que não parseiam são pulados.
fn pontos(bruto: &str) -> Vec<Point> {
    let nums: Vec<f32> = bruto
        .split([',', ' ', '\t', '\n', '\r'])
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse::<f32>().ok())
        .collect();
    nums.chunks_exact(2)
        .map(|c| Point::new(c[0], c[1]))
        .collect()
}

/// Um retângulo de cantos arredondados, montado à mão (o `path::Builder` do
/// `iced` não tem um pronto). Quatro linhas e quatro quartos de círculo.
fn retangulo_arredondado(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    Path::new(|b| {
        b.move_to(Point::new(x + r, y));
        b.line_to(Point::new(x + w - r, y));
        b.quadratic_curve_to(Point::new(x + w, y), Point::new(x + w, y + r));
        b.line_to(Point::new(x + w, y + h - r));
        b.quadratic_curve_to(Point::new(x + w, y + h), Point::new(x + w - r, y + h));
        b.line_to(Point::new(x + r, y + h));
        b.quadratic_curve_to(Point::new(x, y + h), Point::new(x, y + h - r));
        b.line_to(Point::new(x, y + r));
        b.quadratic_curve_to(Point::new(x, y), Point::new(x + r, y));
        b.close();
    })
}

/// Um `<path d="…">` num `canvas::Path`. Subconjunto de SVG: `M L H V C Q Z`,
/// maiúsculo = absoluto, minúsculo = relativo ao ponto atual. `A` (arco
/// elíptico) não — ver o doc do módulo. `None` se não há comando utilizável.
fn caminho_svg(d: &str) -> Option<Path> {
    let toks = tokenizar_path(d);
    if toks.is_empty() {
        return None;
    }
    let mut algo = false;
    let path = Path::new(|b| {
        let mut i = 0;
        let mut atual = Point::ORIGIN;
        let mut inicio_sub = Point::ORIGIN;
        // Lê `n` números a partir de `i`, avançando o cursor. Falta de número
        // encerra o comando.
        while i < toks.len() {
            let Tok::Cmd(cmd) = toks[i] else {
                i += 1; // número solto sem comando — pula
                continue;
            };
            i += 1;
            let rel = cmd.is_ascii_lowercase();
            let num = |toks: &[Tok], i: &mut usize| -> Option<f32> {
                while *i < toks.len() {
                    match toks[*i] {
                        Tok::Num(n) => {
                            *i += 1;
                            return Some(n);
                        }
                        Tok::Cmd(_) => return None,
                    }
                }
                None
            };
            match cmd.to_ascii_uppercase() {
                'M' => {
                    if let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) {
                        let p = desloca(atual, x, y, rel);
                        b.move_to(p);
                        atual = p;
                        inicio_sub = p;
                        algo = true;
                        // Pares seguintes de um `M` são `L` implícito.
                        while let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) {
                            let p = desloca(atual, x, y, rel);
                            b.line_to(p);
                            atual = p;
                        }
                    }
                }
                'L' => {
                    while let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) {
                        let p = desloca(atual, x, y, rel);
                        b.line_to(p);
                        atual = p;
                        algo = true;
                    }
                }
                'H' => {
                    while let Some(x) = num(&toks, &mut i) {
                        let p = Point::new(if rel { atual.x + x } else { x }, atual.y);
                        b.line_to(p);
                        atual = p;
                        algo = true;
                    }
                }
                'V' => {
                    while let Some(y) = num(&toks, &mut i) {
                        let p = Point::new(atual.x, if rel { atual.y + y } else { y });
                        b.line_to(p);
                        atual = p;
                        algo = true;
                    }
                }
                'C' => {
                    while let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) = (
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                    ) {
                        let c1 = desloca(atual, x1, y1, rel);
                        let c2 = desloca(atual, x2, y2, rel);
                        let p = desloca(atual, x, y, rel);
                        b.bezier_curve_to(c1, c2, p);
                        atual = p;
                        algo = true;
                    }
                }
                'Q' => {
                    while let (Some(x1), Some(y1), Some(x), Some(y)) = (
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                        num(&toks, &mut i),
                    ) {
                        let c = desloca(atual, x1, y1, rel);
                        let p = desloca(atual, x, y, rel);
                        b.quadratic_curve_to(c, p);
                        atual = p;
                        algo = true;
                    }
                }
                'Z' => {
                    b.close();
                    atual = inicio_sub;
                }
                _ => {} // comando não suportado (ex.: A) — ignora, não quebra
            }
        }
    });
    algo.then_some(path)
}

/// `p + (x, y)` quando relativo, `(x, y)` quando absoluto.
#[inline]
fn desloca(p: Point, x: f32, y: f32, relativo: bool) -> Point {
    if relativo {
        Point::new(p.x + x, p.y + y)
    } else {
        Point::new(x, y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Cmd(char),
    Num(f32),
}

/// Quebra um `d` de SVG em comandos e números. Um número pode vir colado num
/// comando (`M0 0`), separado por vírgula (`10,20`) ou com sinal grudado
/// (`10-20` = `10`, `-20`).
fn tokenizar_path(d: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut num = String::new();
    let bytes: Vec<char> = d.chars().collect();
    let flush = |num: &mut String, out: &mut Vec<Tok>| {
        if !num.is_empty() {
            if let Ok(n) = num.parse::<f32>() {
                out.push(Tok::Num(n));
            }
            num.clear();
        }
    };
    let mut k = 0;
    while k < bytes.len() {
        let c = bytes[k];
        if c.is_ascii_alphabetic() {
            flush(&mut num, &mut out);
            out.push(Tok::Cmd(c));
        } else if c == ',' || c.is_whitespace() {
            flush(&mut num, &mut out);
        } else if c == '-' || c == '+' {
            // Sinal: começa um número novo, a menos que seja o expoente de um.
            if !num.is_empty() && !num.ends_with(['e', 'E']) {
                flush(&mut num, &mut out);
            }
            num.push(c);
        } else if c == '.' {
            // Um segundo ponto começa outro número (`.5.5` = `.5`, `.5`).
            if num.contains('.') {
                flush(&mut num, &mut out);
            }
            num.push(c);
        } else if c.is_ascii_digit() || c == 'e' || c == 'E' {
            num.push(c);
        }
        k += 1;
    }
    flush(&mut num, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forma(kind: FormaKind, geo: &[(&str, &str)]) -> Forma {
        Forma {
            kind,
            geo: geo.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            texto: String::new(),
            fill: None,
            stroke: None,
            stroke_width: 1.0,
            text_size: 13.0,
        }
    }

    #[test]
    fn pontos_aceita_virgula_e_espaco() {
        assert_eq!(pontos("0,0 10,10 20,0").len(), 3);
        assert_eq!(pontos("0 0 10 10").len(), 2);
        // Ímpar: o último número solto é descartado.
        assert_eq!(pontos("0 0 10").len(), 1);
    }

    #[test]
    fn tokeniza_numeros_colados_e_com_sinal() {
        let t = tokenizar_path("M0 0L10-20z");
        assert_eq!(
            t,
            vec![
                Tok::Cmd('M'),
                Tok::Num(0.0),
                Tok::Num(0.0),
                Tok::Cmd('L'),
                Tok::Num(10.0),
                Tok::Num(-20.0),
                Tok::Cmd('z'),
            ]
        );
    }

    #[test]
    fn caminho_svg_reto_produz_path() {
        // M/L/H/V/Z — o subconjunto que sempre sai.
        assert!(caminho_svg("M0 0 L10 10 H20 V0 Z").is_some());
        // Bézier também.
        assert!(caminho_svg("M0 0 C10 0 10 10 0 10").is_some());
        assert!(caminho_svg("M0 0 Q5 10 10 0").is_some());
        // Sem nenhum comando de desenho: nada.
        assert!(caminho_svg("").is_none());
        assert!(caminho_svg("   ").is_none());
        // `A` é ignorado, mas o `M` antes dele conta.
        assert!(caminho_svg("M0 0 A5 5 0 0 1 10 10").is_some());
    }

    #[test]
    fn circle_e_rect_constroem_path() {
        assert!(forma(FormaKind::Circle, &[("cx", "10"), ("cy", "10"), ("r", "5")])
            .caminho()
            .is_some());
        assert!(forma(FormaKind::Rect, &[("x", "0"), ("y", "0"), ("w", "20"), ("h", "10")])
            .caminho()
            .is_some());
        // Rect com cantos.
        assert!(forma(
            FormaKind::Rect,
            &[("x", "0"), ("y", "0"), ("w", "20"), ("h", "10"), ("rx", "3")]
        )
        .caminho()
        .is_some());
    }

    #[test]
    fn polyline_precisa_de_dois_pontos() {
        assert!(forma(FormaKind::Polyline, &[("points", "0,0 10,10")])
            .caminho()
            .is_some());
        assert!(forma(FormaKind::Polyline, &[("points", "0,0")])
            .caminho()
            .is_none());
    }

    #[test]
    fn text_nao_tem_caminho() {
        assert!(forma(FormaKind::Text, &[("x", "0"), ("y", "0")]).caminho().is_none());
    }
}
