//! O **arrasto** como capacidade — o habilitador A da Onda 9.
//!
//! # Por que este arquivo existe
//!
//! Até a 0.94 o motor arrastava em **três** lugares que não se conheciam:
//!
//! | Onde | O que guardava o arrasto |
//! |---|---|
//! | reordenar uma lista | `__drag_key` + `{var}.__dragging` por item ([`crate::DRAG_KEY_CONTEXT`]) |
//! | a alça de coluna do `<tableheader>` | `__colgrip` = `chave\|indice\|x0\|w0`, aplicado num `arrasta_coluna` de quarenta linhas |
//! | o giro do `<dial>` | `EstadoDial { arrastando }` no `canvas::Program::State` ([`crate::gauges`]) |
//!
//! Os três já tinham resolvido, cada um por si, os dois problemas difíceis do
//! arrasto — e nenhum dos três estava catalogado como *capacidade* no §3 do
//! `PLANO_WIDGETS.md`, que é a terceira vez que o gargalo real não estava na
//! lista (a medição de colunas da Onda 6 e o corpo do diálogo da Onda 8 foram
//! as outras duas).
//!
//! Este módulo é o segundo deles — o `__colgrip` — com a **conta fatorada para
//! fora**. O terceiro (o `<dial>`) continua onde estava, e de propósito: quem
//! desenha o próprio widget num `canvas` já ganha um `State` por instância do
//! `iced` e não precisa de chave global nenhuma. O que precisa de chave global
//! é o arrasto sobre widgets que têm **filhos** — um `<splitter>` não é um
//! canvas, e o `<swipeview>` menos ainda.
//!
//! # Os dois problemas difíceis, e onde a resposta já estava
//!
//! **1. O zero do arrasto não é o clique.** Enquanto não há alça presa o motor
//! *não escuta o mouse*, então `last_cursor_pos` guarda a posição de um menu
//! aberto meia hora atrás. Por isso um [`Arrasto`] nasce com [`Arrasto::origem`]
//! em `None` e é o **primeiro** movimento que a preenche: custa um quadro que
//! ninguém vê e é exato daí em diante.
//!
//! **2. O listener só existe entre o pressionar e o soltar.**
//! `GlacierUI::precisa_do_cursor` liga o `listen_with` do movimento só enquanto
//! [`GRIP_CONTEXT`] existir. A nota lá tem o número medido: 70 movimentos por
//! segundo davam 65 quadros, 110 davam 10. Um arrasto genérico que ligasse o
//! listener para sempre estrangularia a rolagem do app inteiro.
//!
//! # O que sobrou para generalizar
//!
//! Só o que `arrasta_coluna` tinha embutido: o **eixo** ([`Eixo`]) e o
//! **mapeamento** de delta de pixel para valor ([`Alvo`]). Dois mapeamentos
//! cobrem os três consumidores:
//!
//! - [`Alvo::Trilha`] — pixels viram a medida de **uma trilha** numa lista no
//!   formato do `columns` do `<grid>` (`"160 fill 90"`). É o `<tableheader>`
//!   (que era o dono do código) e o `<splitter>`, que é a mesma alça aplicada a
//!   um container em vez de a um cabeçalho;
//! - [`Alvo::Indice`] — cada `passo` pixels andam **um inteiro** numa chave. É
//!   o `<swipeview>`: arrastar meia tela vira a página.
//!
//! O soltar continua sendo o [`crate::EngineMessage::DragEnd`], que já era a
//! mensagem de soltar de dois dos três — só existe um botão do mouse, e só um
//! arrasto por vez.

use iced::Point;

use crate::ContextMap;

/// A chave do motor onde vive o arrasto em curso. **Um por app** — não por
/// instância, e nunca precisou ser: só se arrasta uma coisa por vez numa tela.
///
/// Sucede o `__colgrip` da Onda 6, que guardava a mesma coisa para um consumidor
/// só. O formato é o mesmo de sempre, campos separados por `|`, porque o
/// contexto é um mapa de strings — ver [`Arrasto::escrever`].
pub(crate) const GRIP_CONTEXT: &str = "__grip";

/// Em que eixo o movimento do ponteiro conta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eixo {
    /// Horizontal: colunas de tabela, `<splitter>` de painéis lado a lado,
    /// `<swipeview>`.
    X,
    /// Vertical: `<splitter direction="v">`.
    Y,
}

impl Eixo {
    fn coord(self, p: Point) -> f32 {
        match self {
            Self::X => p.x,
            Self::Y => p.y,
        }
    }

    fn letra(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
        }
    }

    fn de_letra(s: &str) -> Self {
        if s == "y" { Self::Y } else { Self::X }
    }
}

/// O que o delta de pixel vira.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alvo {
    /// A medida de **uma trilha** dentro de uma lista de trilhas.
    ///
    /// `min`/`max` são pixels, e o `min` não é estética: uma trilha de largura
    /// zero some da tela **junto com a alça dela**, e não haveria como trazê-la
    /// de volta.
    Trilha { min: f32, max: f32 },
    /// Um índice inteiro, um degrau a cada `passo` pixels, saturando em `[0, max]`.
    Indice { passo: f32, max: usize },
}

/// Um arrasto em curso. Serializa para [`GRIP_CONTEXT`] e volta de lá.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrasto {
    /// A chave do app que este arrasto reescreve.
    pub chave: String,
    /// Qual trilha (para [`Alvo::Trilha`]); ignorado pelo [`Alvo::Indice`].
    pub indice: usize,
    pub eixo: Eixo,
    /// O zero do arrasto, em coordenadas de janela. `None` até o primeiro
    /// movimento — ver a nota do módulo.
    pub origem: Option<f32>,
    /// O valor que a chave tinha quando o arrasto começou: a largura da trilha,
    /// ou o índice da página.
    pub valor0: f32,
    pub alvo: Alvo,
}

impl Arrasto {
    /// `chave|indice|eixo|origem|valor0|alvo|a|b`, com `?` na origem em aberto.
    pub fn escrever(&self) -> String {
        let origem = match self.origem {
            Some(o) => format!("{o}"),
            None => "?".to_string(),
        };
        let (tipo, a, b) = match self.alvo {
            Alvo::Trilha { min, max } => ("trilha", min, max),
            Alvo::Indice { passo, max } => ("indice", passo, max as f32),
        };
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            self.chave,
            self.indice,
            self.eixo.letra(),
            origem,
            self.valor0,
            tipo,
            a,
            b
        )
    }

    /// O caminho de volta. Um campo estragado devolve `None` e o arrasto morre
    /// em silêncio — que é o certo: um arrasto meio lido reescreveria a chave
    /// do app com lixo.
    pub fn ler(bruto: &str) -> Option<Self> {
        let campos: Vec<&str> = bruto.split('|').collect();
        let [chave, indice, eixo, origem, valor0, tipo, a, b] = campos[..] else {
            return None;
        };
        let (a, b) = (a.parse::<f32>().ok()?, b.parse::<f32>().ok()?);
        Some(Self {
            chave: chave.to_string(),
            indice: indice.parse().ok()?,
            eixo: Eixo::de_letra(eixo),
            origem: (origem != "?").then(|| origem.parse().ok()).flatten(),
            valor0: valor0.parse().ok()?,
            alvo: match tipo {
                "indice" => Alvo::Indice {
                    passo: a.max(1.0),
                    max: b.max(0.0) as usize,
                },
                _ => Alvo::Trilha { min: a, max: b },
            },
        })
    }

    /// O valor novo da chave, dada a posição do ponteiro e o contexto de agora.
    ///
    /// Devolve `None` quando **não há nada a escrever** — três casos, todos
    /// legítimos: o quadro que só ancora a origem, o valor que não mudou de
    /// verdade, e a chave que já está no valor calculado. Cada `Some` custa uma
    /// reavaliação da árvore, então esta é a diferença entre arrastar liso e
    /// arrastar aos tropeços.
    pub fn aplica(&mut self, p: Point, context: &ContextMap) -> Option<(String, String)> {
        let c = self.eixo.coord(p);
        // O primeiro movimento depois do clique só ancora.
        let Some(origem) = self.origem else {
            self.origem = Some(c);
            return None;
        };
        let delta = c - origem;
        let novo = match self.alvo {
            Alvo::Trilha { min, max } => {
                let medida = (self.valor0 + delta).clamp(min, max);
                let atual = context.get(&self.chave).map(String::as_str).unwrap_or("");
                let mut trilhas: Vec<String> = if atual.trim().is_empty() {
                    Vec::new()
                } else {
                    atual.split_whitespace().map(str::to_string).collect()
                };
                // Uma trilha arrastada à mão deixa de ser flexível — é o que a
                // pessoa acabou de pedir ao arrastá-la. As que ainda não
                // existem entram como `auto`, para não mudar o layout à
                // esquerda do que se está mexendo.
                while trilhas.len() <= self.indice {
                    trilhas.push("auto".to_string());
                }
                trilhas[self.indice] = format!("{medida:.0}");
                trilhas.join(" ")
            }
            Alvo::Indice { passo, max } => {
                let degraus = (delta / passo).round();
                let i = (self.valor0 + degraus).clamp(0.0, max as f32);
                format!("{i:.0}")
            }
        };
        (context.get(&self.chave) != Some(&novo)).then_some((self.chave.clone(), novo))
    }
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
    fn ida_e_volta_preserva_o_arrasto() {
        let a = Arrasto {
            chave: "larguras".into(),
            indice: 2,
            eixo: Eixo::Y,
            origem: Some(140.5),
            valor0: 90.0,
            alvo: Alvo::Trilha {
                min: 48.0,
                max: 1200.0,
            },
        };
        assert_eq!(Arrasto::ler(&a.escrever()), Some(a));
    }

    #[test]
    fn origem_em_aberto_sobrevive_a_ida_e_volta() {
        let a = Arrasto {
            chave: "p".into(),
            indice: 0,
            eixo: Eixo::X,
            origem: None,
            valor0: 200.0,
            alvo: Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        };
        let voltou = Arrasto::ler(&a.escrever()).unwrap();
        assert_eq!(voltou.origem, None);
    }

    #[test]
    fn o_primeiro_movimento_so_ancora() {
        let mut a = Arrasto {
            chave: "p".into(),
            indice: 0,
            eixo: Eixo::X,
            origem: None,
            valor0: 200.0,
            alvo: Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        };
        // Nada escrito, e a origem passa a existir.
        assert_eq!(a.aplica(Point::new(500.0, 0.0), &ctx(&[])), None);
        assert_eq!(a.origem, Some(500.0));
        // Do segundo em diante, a conta é exata a partir dali.
        let (k, v) = a.aplica(Point::new(560.0, 0.0), &ctx(&[])).unwrap();
        assert_eq!((k.as_str(), v.as_str()), ("p", "260"));
    }

    #[test]
    fn trilha_arrastada_vira_fixa_e_preserva_as_vizinhas() {
        let mut a = Arrasto {
            chave: "cols".into(),
            indice: 1,
            eixo: Eixo::X,
            origem: Some(100.0),
            valor0: 120.0,
            alvo: Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        };
        let c = ctx(&[("cols", "160 fill 90")]);
        let (_, v) = a.aplica(Point::new(150.0, 0.0), &c).unwrap();
        assert_eq!(v, "160 170 90");
    }

    #[test]
    fn o_piso_da_trilha_e_o_que_a_mantem_agarravel() {
        let mut a = Arrasto {
            chave: "cols".into(),
            indice: 0,
            eixo: Eixo::X,
            origem: Some(300.0),
            valor0: 120.0,
            alvo: Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        };
        // Puxar 400px para a esquerda daria -280: some da tela junto com a alça.
        let (_, v) = a.aplica(Point::new(-100.0, 0.0), &ctx(&[])).unwrap();
        assert_eq!(v, "48");
    }

    #[test]
    fn o_indice_anda_um_degrau_por_passo_e_satura() {
        let mut a = Arrasto {
            chave: "pagina".into(),
            indice: 0,
            eixo: Eixo::X,
            origem: Some(500.0),
            valor0: 1.0,
            alvo: Alvo::Indice {
                passo: 120.0,
                max: 2,
            },
        };
        // Arrastar para a ESQUERDA anda para trás (o conteúdo segue o dedo).
        let (_, v) = a.aplica(Point::new(380.0, 0.0), &ctx(&[])).unwrap();
        assert_eq!(v, "0");
        // E satura: não existe página -1.
        let (_, v) = a.aplica(Point::new(-900.0, 0.0), &ctx(&[])).unwrap();
        assert_eq!(v, "0");
    }

    #[test]
    fn nao_escreve_quando_a_chave_ja_esta_no_valor() {
        let mut a = Arrasto {
            chave: "pagina".into(),
            indice: 0,
            eixo: Eixo::X,
            origem: Some(500.0),
            valor0: 1.0,
            alvo: Alvo::Indice {
                passo: 120.0,
                max: 4,
            },
        };
        // Meio degrau não muda nada, e um `Some` aqui custaria uma reavaliação
        // por pixel arrastado.
        assert_eq!(
            a.aplica(Point::new(520.0, 0.0), &ctx(&[("pagina", "1")])),
            None
        );
    }

    #[test]
    fn um_campo_a_menos_mata_o_arrasto_em_silencio() {
        assert_eq!(Arrasto::ler("cols|1|x|?|120|trilha|48"), None);
        assert_eq!(Arrasto::ler(""), None);
    }
}
