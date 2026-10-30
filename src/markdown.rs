//! `<markdown>`: CommonMark (com tabelas desligadas e tachado ligado) convertido em
//! widgets do `iced` a cada `view`.
//!
//! Não usa o `iced::widget::markdown`: ele pede que os `Item`s já parseados vivam
//! fora do `view` (`view(&items)` empresta deles), e aqui o texto muda a cada
//! avaliação — viria um cache por nó só para contornar o empréstimo. Converter
//! direto dos eventos do parser dá `Element`s que **possuem** suas strings.
//!
//! Suporta: títulos, parágrafos, ênfase/negrito/tachado/código inline, links
//! (abrem pela ação built-in `open:`), listas (com ou sem número, aninhadas),
//! citação, bloco de código e linha horizontal. HTML cru é ignorado.

use iced::widget::text::Span;
use iced::widget::{Space, column, container, rich_text, row, span, text};
use iced::{Background, Color, Element, Font, Length, font};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::widget::EngineMessage;

const LINK: Color = Color::from_rgb(0.537, 0.706, 0.980);

enum Quadro {
    Raiz,
    Lista(Option<u64>),
    Item,
    Citacao,
}

struct Montador<'a> {
    pilha: Vec<(Quadro, Vec<Element<'a, EngineMessage>>)>,
    inline: Vec<Span<'a, String, Font>>,
    negrito: u32,
    italico: u32,
    tachado: u32,
    link: Option<String>,
    codigo: Option<String>,
    tamanho: f32,
    titulo: Option<HeadingLevel>,
}

impl<'a> Montador<'a> {
    fn novo(tamanho: f32) -> Self {
        Self {
            pilha: vec![(Quadro::Raiz, Vec::new())],
            inline: Vec::new(),
            negrito: 0,
            italico: 0,
            tachado: 0,
            link: None,
            codigo: None,
            tamanho,
            titulo: None,
        }
    }

    fn empurrar(&mut self, e: Element<'a, EngineMessage>) {
        if let Some((_, filhos)) = self.pilha.last_mut() {
            filhos.push(e);
        }
    }

    fn texto(&mut self, conteudo: String, mono: bool) {
        let mut f = if mono { Font::MONOSPACE } else { Font::DEFAULT };
        let mut tam = self.tamanho;
        if let Some(n) = self.titulo {
            f.weight = font::Weight::Bold;
            tam = self.tamanho * escala(n);
        }
        if self.negrito > 0 {
            f.weight = font::Weight::Bold;
        }
        if self.italico > 0 {
            f.style = font::Style::Italic;
        }
        let mut s: Span<'a, String, Font> = span(conteudo).font(f).size(tam);
        if self.tachado > 0 {
            s = s.strikethrough(true);
        }
        if let Some(url) = &self.link {
            s = s.color(LINK).underline(true).link(url.clone());
        }
        self.inline.push(s);
    }

    /// Fecha o parágrafo/título/item apertado em curso, se houver trecho pendente.
    fn descarregar(&mut self) {
        if self.inline.is_empty() {
            return;
        }
        let spans = std::mem::take(&mut self.inline);
        let t = rich_text(spans)
            .on_link_click(|url: String| EngineMessage::UiClick(format!("open:{url}")))
            .width(Length::Fill);
        self.empurrar(t.into());
    }

    fn evento(&mut self, ev: Event<'_>) {
        match ev {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => self.titulo = Some(level),
                Tag::Strong => self.negrito += 1,
                Tag::Emphasis => self.italico += 1,
                Tag::Strikethrough => self.tachado += 1,
                Tag::Link { dest_url, .. } => self.link = Some(dest_url.to_string()),
                Tag::CodeBlock(_) => self.codigo = Some(String::new()),
                Tag::List(n) => {
                    self.descarregar();
                    self.pilha.push((Quadro::Lista(n), Vec::new()));
                }
                Tag::Item => self.pilha.push((Quadro::Item, Vec::new())),
                Tag::BlockQuote(_) => {
                    self.descarregar();
                    self.pilha.push((Quadro::Citacao, Vec::new()));
                }
                _ => {}
            },
            Event::End(fim) => match fim {
                TagEnd::Paragraph => self.descarregar(),
                TagEnd::Heading(_) => {
                    self.descarregar();
                    self.titulo = None;
                }
                TagEnd::Strong => self.negrito = self.negrito.saturating_sub(1),
                TagEnd::Emphasis => self.italico = self.italico.saturating_sub(1),
                TagEnd::Strikethrough => self.tachado = self.tachado.saturating_sub(1),
                TagEnd::Link => self.link = None,
                TagEnd::CodeBlock => {
                    if let Some(codigo) = self.codigo.take() {
                        let corpo = text(codigo.trim_end_matches('\n').to_string())
                            .font(Font::MONOSPACE)
                            .size(self.tamanho * 0.9);
                        self.empurrar(
                            container(corpo)
                                .padding(10)
                                .width(Length::Fill)
                                .style(|_| container::Style {
                                    background: Some(Background::Color(Color::from_rgba(
                                        0.5, 0.5, 0.5, 0.15,
                                    ))),
                                    border: iced::Border {
                                        radius: 6.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .into(),
                        );
                    }
                }
                TagEnd::Item => {
                    self.descarregar();
                    if let Some((Quadro::Item, filhos)) = self.pilha.pop() {
                        let marcador = self.proximo_marcador();
                        let corpo = column(filhos).spacing(4).width(Length::Fill);
                        let linha = row![text(marcador).size(self.tamanho), corpo].spacing(8);
                        self.empurrar(linha.into());
                    }
                }
                TagEnd::List(_) => {
                    if let Some((Quadro::Lista(_), filhos)) = self.pilha.pop() {
                        let c = column(filhos).spacing(4).padding(iced::Padding {
                            left: 8.0,
                            ..Default::default()
                        });
                        self.empurrar(c.into());
                    }
                }
                TagEnd::BlockQuote(_) => {
                    self.descarregar();
                    if let Some((Quadro::Citacao, filhos)) = self.pilha.pop() {
                        let barra = container(Space::new())
                            .width(3)
                            .height(Length::Fill)
                            .style(|_| container::Style {
                                background: Some(Background::Color(Color::from_rgba(
                                    0.5, 0.5, 0.5, 0.6,
                                ))),
                                ..Default::default()
                            });
                        let l = row![barra, column(filhos).spacing(6).width(Length::Fill)]
                            .spacing(10)
                            .height(Length::Shrink);
                        self.empurrar(l.into());
                    }
                }
                _ => {}
            },
            Event::Text(t) => match &mut self.codigo {
                Some(buf) => buf.push_str(&t),
                None => self.texto(t.to_string(), false),
            },
            Event::Code(t) => self.texto(t.to_string(), true),
            Event::SoftBreak => self.texto(" ".into(), false),
            Event::HardBreak => self.texto("\n".into(), false),
            Event::Rule => {
                self.descarregar();
                self.empurrar(
                    container(Space::new())
                        .width(Length::Fill)
                        .height(1)
                        .style(|_| container::Style {
                            background: Some(Background::Color(Color::from_rgba(
                                0.5, 0.5, 0.5, 0.5,
                            ))),
                            ..Default::default()
                        })
                        .into(),
                );
            }
            _ => {}
        }
    }

    /// O marcador do item que acabou de fechar: `•` ou `N.`; avança o contador
    /// da lista que o contém (que é o topo da pilha, agora que o item saiu).
    fn proximo_marcador(&mut self) -> String {
        match self.pilha.last_mut() {
            Some((Quadro::Lista(Some(n)), _)) => {
                let m = format!("{n}.");
                *n += 1;
                m
            }
            _ => "•".to_string(),
        }
    }
}

fn escala(n: HeadingLevel) -> f32 {
    match n {
        HeadingLevel::H1 => 2.0,
        HeadingLevel::H2 => 1.6,
        HeadingLevel::H3 => 1.35,
        HeadingLevel::H4 => 1.2,
        _ => 1.05,
    }
}

/// Converte `fonte` (Markdown) em widgets. `tamanho` é o corpo do texto.
pub(crate) fn view<'a>(fonte: &str, tamanho: f32) -> Element<'a, EngineMessage> {
    let mut m = Montador::novo(tamanho);
    for ev in Parser::new_ext(fonte, Options::ENABLE_STRIKETHROUGH) {
        m.evento(ev);
    }
    m.descarregar();
    // Itens soltos de uma pilha mal fechada (HTML cru, por exemplo) não somem.
    while m.pilha.len() > 1 {
        let (_, filhos) = m.pilha.pop().unwrap();
        for f in filhos {
            m.empurrar(f);
        }
    }
    let filhos = m.pilha.pop().map(|(_, f)| f).unwrap_or_default();
    column(filhos).spacing(8).width(Length::Fill).into()
}
