//! A navegação do `<wizard>` (Onda 8): o cabeçalho de passo e a fileira de
//! botões que anda entre eles.
//!
//! # Por que isto é primitiva, e o `<wizard>` não
//!
//! O `<wizard>` inteiro é um **builtin** — ele precisa hospedar as páginas, e
//! página é `<slot>`, que é mecanismo de componente. Mas um builtin é um
//! template, e template não calcula: dizer "este é o primeiro passo, então
//! `Voltar` fica inerte" exige achar a posição de `active` dentro de `steps`,
//! e não há como escrever isso em markup.
//!
//! Então a onda faz o que a Onda 5 fez com o `<tabs>`: **parte em dois**. O
//! composto (páginas + navegação) é o builtin; a aritmética é esta primitiva,
//! que recebe a lista de passos como texto e resolve tudo no render.
//!
//! É o mesmo desenho da `<pagination>` da Onda 4 — inclusive na peça que o faz
//! caber em tão pouco código: a [`EngineMessage::ContextPatch`], que deixa uma
//! primitiva **escrever a chave sozinha**, sem `update` de componente nenhum.
//! O botão inerte no limite também é dela: um builtin não consegue desabilitar
//! um botão por condição, e é o limite conhecido que fez a `<pagination>` subir
//! de nível na 0.85.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::ContextMap;
use crate::widget::EngineMessage;

/// O que a fileira de botões precisa saber, calculado a partir da lista de
/// passos e do passo atual.
///
/// Existe separado do render por duas razões. A primeira é testável: as quatro
/// regras de um wizard (voltar inerte no começo, finalizar no fim, travar sem
/// validação, saturar em vez de dar a volta) são aritmética, e aritmética se
/// testa sem montar `Element` nenhum. A segunda é que o cabeçalho e os botões
/// fazem a **mesma** conta — achar `atual` dentro de `passos` — e fazê-la duas
/// vezes seria dois lugares para ela divergir.
#[derive(Debug, Clone, PartialEq)]
pub struct Plano {
    /// O índice do passo atual.
    pub i: usize,
    /// Quantos passos há no total.
    pub total: usize,
    /// `Voltar` fica inerte.
    pub primeiro: bool,
    /// `Avançar` vira `Finalizar`.
    pub ultimo: bool,
    /// A página validou (o `QWizardPage::isComplete()`).
    pub pode_avancar: bool,
}

impl Plano {
    /// Calcula o plano. `atual` é o **id** do passo (o valor da chave), não o
    /// índice.
    pub fn novo(passos: &[&str], atual: &str, valid: &str) -> Self {
        let total = passos.len();
        // Um passo que não está na lista — a primeira abertura, com a chave
        // vazia, ou um id que alguém renomeou — é o passo zero. É o que faz
        // `<wizard>` funcionar sem o app semear a chave antes, e o que faz um
        // id trocado degradar para "começa do início" em vez de para uma tela
        // vazia.
        let i = passos.iter().position(|p| *p == atual).unwrap_or(0);
        Self {
            i,
            total,
            primeiro: i == 0,
            ultimo: i + 1 >= total,
            // Ausente = todo passo é válido. Um wizard sem validação declarada
            // não pode nascer travado.
            pode_avancar: valid.trim().is_empty() || verdadeiro(valid),
        }
    }

    /// O índice para onde `Avançar` leva — saturando no fim.
    ///
    /// Saturar, nunca dar a volta: um wizard que salta do último passo para o
    /// primeiro parece ter perdido o que o usuário preencheu.
    pub fn proximo(&self) -> usize {
        (self.i + 1).min(self.total.saturating_sub(1))
    }

    /// O índice para onde `Voltar` leva — saturando no começo.
    pub fn anterior(&self) -> usize {
        self.i.saturating_sub(1)
    }
}

/// Um item da fileira: rótulo, ação e se está inerte.
///
/// `msg: None` é o botão **inerte** — presente, cinza, sem resposta ao clique.
/// Sumir com ele seria pior: a fileira dançaria a cada passo, e o `Avançar`
/// mudaria de lugar debaixo do cursor.
struct Botao {
    rotulo: String,
    msg: Option<EngineMessage>,
    destaque: bool,
}

/// Desenha o cabeçalho ("Passo 2 de 3") e a fileira de botões.
///
/// Os parâmetros são os atributos do `<wizardnav>`, já interpolados — `steps` e
/// `titles` chegam como as listas separadas por vírgula que o markup escreveu,
/// e `active` sai do contexto pela chave `value_var`.
#[allow(clippy::too_many_arguments)]
pub fn render_wizard_nav<'a>(
    context: &'a ContextMap,
    value_var: &'a str,
    steps: &'a str,
    titles: &'a str,
    valid: &'a str,
    on_finish: &'a str,
    on_cancel: &'a str,
    back_label: &'a str,
    next_label: &'a str,
    finish_label: &'a str,
    cancel_label: &'a str,
    show_header: bool,
) -> Element<'a, EngineMessage> {
    let passos: Vec<&str> = lista(steps);
    if passos.is_empty() {
        // Sem passos não há wizard. Some inteiro, como a `<pagination>` de uma
        // página — a mesma degradação de uma chave ainda não semeada.
        return Space::new()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();
    }
    let rotulos = lista(titles);

    let atual = context.get(value_var).map(|s| s.trim()).unwrap_or("");
    let plano = Plano::novo(&passos, atual, valid);
    let (i, primeiro, ultimo, pode_avancar) =
        (plano.i, plano.primeiro, plano.ultimo, plano.pode_avancar);

    let ir = |destino: usize| -> EngineMessage {
        EngineMessage::ContextPatch(vec![(value_var.to_string(), passos[destino].to_string())])
    };

    let mut botoes: Vec<Botao> = Vec::new();
    if !on_cancel.trim().is_empty() {
        botoes.push(Botao {
            rotulo: nao_vazio(cancel_label, "Cancelar"),
            msg: Some(EngineMessage::UiClick(on_cancel.to_string())),
            destaque: false,
        });
    }
    botoes.push(Botao {
        rotulo: nao_vazio(back_label, "Voltar"),
        msg: if primeiro {
            None
        } else {
            Some(ir(plano.anterior()))
        },
        destaque: false,
    });
    botoes.push(if ultimo {
        Botao {
            rotulo: nao_vazio(finish_label, "Finalizar"),
            // Um `on_finish` não escrito deixa o botão inerte em vez de o
            // fazer não fazer nada em silêncio: o último passo de um wizard
            // sem ação final é quase sempre um esquecimento.
            msg: match (pode_avancar, on_finish.trim().is_empty()) {
                (true, false) => Some(EngineMessage::UiClick(on_finish.to_string())),
                _ => None,
            },
            destaque: true,
        }
    } else {
        Botao {
            rotulo: nao_vazio(next_label, "Avançar"),
            msg: if pode_avancar {
                Some(ir(plano.proximo()))
            } else {
                None
            },
            destaque: true,
        }
    });

    let mut fileira = row![Space::new().width(Length::Fill)].spacing(8);
    for b in botoes {
        fileira = fileira.push(desenha_botao(b));
    }

    if !show_header {
        return fileira.width(Length::Fill).into();
    }

    // O título do passo: o `titles` na posição, e o id como reserva. Uma lista
    // de títulos mais curta que a de passos não é erro — os passos sem título
    // mostram o próprio id, que é feio e informativo, em vez de vazio.
    let titulo = rotulos.get(i).copied().unwrap_or(passos[i]).to_string();
    let posicao = format!("Passo {} de {}", i + 1, plano.total);

    column![
        column![
            text(titulo).size(15),
            text(posicao)
                .size(12)
                .color(Color::from_rgba(0.5, 0.5, 0.5, 1.0)),
        ]
        .spacing(2)
        .width(Length::Fill),
        fileira.width(Length::Fill),
    ]
    .spacing(14)
    .width(Length::Fill)
    .into()
}

/// Quebra uma lista `a,b,c` em pedaços aparados, sem os vazios.
fn lista(bruto: &str) -> Vec<&str> {
    bruto
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// A mesma leitura de verdade do resto do motor: vazio, `false`, `0`, `nao` e
/// `não` são falsos; o resto é verdadeiro.
fn verdadeiro(v: &str) -> bool {
    let v = v.trim().to_ascii_lowercase();
    !(v.is_empty() || v == "false" || v == "0" || v == "nao" || v == "não")
}

fn nao_vazio(v: &str, padrao: &str) -> String {
    let v = v.trim();
    if v.is_empty() {
        padrao.to_string()
    } else {
        v.to_string()
    }
}

/// Um botão da fileira, nos três estados que ela usa: destaque (a ação
/// principal), neutro, e inerte.
fn desenha_botao<'a>(b: Botao) -> Element<'a, EngineMessage> {
    let inerte = b.msg.is_none();
    let destaque = b.destaque;
    let mut btn = button(text(b.rotulo).size(14)).padding([8, 16]).style(
        move |theme: &iced::Theme, status: button::Status| {
            let palette = theme.extended_palette();
            let base = if destaque {
                palette.primary.base.color
            } else {
                palette.background.strong.color
            };
            let texto = if destaque {
                Color::WHITE
            } else {
                palette.background.base.text
            };
            // Inerte esmaece os dois — o fundo E o texto. Esmaecer só o fundo
            // deixa um botão que parece clicável com texto forte por cima.
            let (bg, fg) = if inerte {
                (Color { a: 0.35, ..base }, Color { a: 0.45, ..texto })
            } else {
                match status {
                    button::Status::Hovered => (Color { a: 0.85, ..base }, texto),
                    button::Status::Pressed => (Color { a: 0.7, ..base }, texto),
                    _ => (base, texto),
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: fg,
                border: Border {
                    radius: iced::border::Radius::new(6.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        },
    );
    if let Some(msg) = b.msg {
        btn = btn.on_press(msg);
    }
    container(btn).align_y(Alignment::Center).into()
}
