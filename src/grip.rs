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
    /// Arrasto livre em **duas dimensões** — o habilitador B da Onda 11: uma
    /// janela interna (`MdiArea`) ou o painel flutuante de um `<dock>`. As
    /// bordas são as do container-pai, em pixels de janela.
    ///
    /// É o único `Alvo` que ignora [`Arrasto::eixo`] (arrasta nos dois eixos
    /// de uma vez) e escreve em **duas** chaves — `Arrasto::chave` para `x`,
    /// [`Arrasto::chave_y`] para `y` — porque a posição de uma janela é dois
    /// números nomeados pelo app, a mesma forma que o `<daterangepicker>` usa
    /// para `start`/`end`.
    Ponto {
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    },
    /// Arrasto que **não escreve nada durante o gesto** e comete um valor
    /// discreto **na soltura** — o habilitador D da Onda 12, para o `<dock>`.
    ///
    /// Enquanto o dedo está apertado, `aplica` só ancora a origem nos dois
    /// eixos (é o que dá a [`Arrasto::modo_no_release`] o `dx`/`dy` do gesto
    /// inteiro). No `DragEnd`, o motor chama `modo_no_release` com o ponto de
    /// soltura: um delta abaixo de `limiar` nos dois eixos **não muda nada**
    /// (foi um clique, não um arrasto); acima dele, o eixo dominante escolhe a
    /// borda (`left`/`right`/`top`/`bottom`) e o valor vai para
    /// [`Arrasto::chave_modo`].
    ///
    /// É a resposta ao que a Onda 11 cortou: trocar `<splitter>`↔`<stack>` de
    /// pai **entre quadros**, dirigido por uma chave nomeada, em vez de
    /// reparentar no meio do arrasto.
    Zona { limiar: f32 },
}

/// Um arrasto em curso. Serializa para [`GRIP_CONTEXT`] e volta de lá.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrasto {
    /// A chave do app que este arrasto reescreve. Para [`Alvo::Ponto`], é a
    /// do eixo `x`; a do `y` é [`Arrasto::chave_y`]. Para [`Alvo::Zona`] não é
    /// escrita durante o gesto (ver [`Arrasto::chave_modo`]).
    pub chave: String,
    /// A chave do eixo `y`, só para [`Alvo::Ponto`]. `None` nos outros alvos,
    /// que escrevem uma chave só (ou nenhuma, no caso de [`Alvo::Zona`]).
    pub chave_y: Option<String>,
    /// A chave que recebe a **borda escolhida** na soltura, só para
    /// [`Alvo::Zona`] — `left`/`right`/`top`/`bottom`. `None` nos outros alvos.
    pub chave_modo: Option<String>,
    /// Qual trilha (para [`Alvo::Trilha`]); ignorado pelos outros dois.
    pub indice: usize,
    /// Ignorado por [`Alvo::Ponto`], que sempre arrasta nos dois eixos.
    pub eixo: Eixo,
    /// O zero do arrasto no eixo `x` (ou no único eixo, fora de
    /// [`Alvo::Ponto`]), em coordenadas de janela. `None` até o primeiro
    /// movimento — ver a nota do módulo.
    pub origem: Option<f32>,
    /// O zero do arrasto no eixo `y`, só para [`Alvo::Ponto`].
    pub origem_y: Option<f32>,
    /// O valor que a chave `x` (ou a única) tinha quando o arrasto começou: a
    /// largura da trilha, o índice da página, ou a posição `x` da janela.
    pub valor0: f32,
    /// O valor que a chave `y` tinha quando o arrasto começou, só para
    /// [`Alvo::Ponto`].
    pub valor0_y: f32,
    pub alvo: Alvo,
}

/// `Some(v)` vira `"v"`; `None` vira `"?"` — a origem em aberto de
/// [`Arrasto::escrever`]/[`Arrasto::ler`], fatorada porque a Onda 11 passou a
/// ter duas (uma por eixo).
fn fmt_origem(o: Option<f32>) -> String {
    match o {
        Some(v) => format!("{v}"),
        None => "?".to_string(),
    }
}

fn ler_origem(s: &str) -> Option<Option<f32>> {
    if s == "?" {
        Some(None)
    } else {
        s.parse::<f32>().ok().map(Some)
    }
}

impl Arrasto {
    /// `chave|chave_y|indice|eixo|origem|origem_y|valor0|valor0_y|tipo|a|b|c|d`,
    /// com `?` numa origem em aberto e `chave_y` vazio fora de [`Alvo::Ponto`].
    ///
    /// `origem_y`/`valor0_y`/`chave_y` só carregam algo para [`Alvo::Ponto`];
    /// os outros dois alvos escrevem `0`/vazio ali, e nunca os leem de volta —
    /// `aplica` decide por `self.alvo`, não por esses campos estarem
    /// presentes.
    pub fn escrever(&self) -> String {
        let (tipo, a, b, c, d) = match self.alvo {
            Alvo::Trilha { min, max } => ("trilha", min, max, 0.0, 0.0),
            Alvo::Indice { passo, max } => ("indice", passo, max as f32, 0.0, 0.0),
            Alvo::Ponto {
                min_x,
                max_x,
                min_y,
                max_y,
            } => ("ponto", min_x, max_x, min_y, max_y),
            Alvo::Zona { limiar } => ("zona", limiar, 0.0, 0.0, 0.0),
        };
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.chave,
            self.chave_y.as_deref().unwrap_or(""),
            self.chave_modo.as_deref().unwrap_or(""),
            self.indice,
            self.eixo.letra(),
            fmt_origem(self.origem),
            fmt_origem(self.origem_y),
            self.valor0,
            self.valor0_y,
            tipo,
            a,
            b,
            c,
            d
        )
    }

    /// O caminho de volta. Um campo estragado devolve `None` e o arrasto morre
    /// em silêncio — que é o certo: um arrasto meio lido reescreveria a chave
    /// do app com lixo.
    pub fn ler(bruto: &str) -> Option<Self> {
        let campos: Vec<&str> = bruto.split('|').collect();
        let [
            chave,
            chave_y,
            chave_modo,
            indice,
            eixo,
            origem,
            origem_y,
            valor0,
            valor0_y,
            tipo,
            a,
            b,
            c,
            d,
        ] = campos[..]
        else {
            return None;
        };
        let (a, b, c, d) = (
            a.parse::<f32>().ok()?,
            b.parse::<f32>().ok()?,
            c.parse::<f32>().ok()?,
            d.parse::<f32>().ok()?,
        );
        Some(Self {
            chave: chave.to_string(),
            chave_y: (!chave_y.is_empty()).then(|| chave_y.to_string()),
            chave_modo: (!chave_modo.is_empty()).then(|| chave_modo.to_string()),
            indice: indice.parse().ok()?,
            eixo: Eixo::de_letra(eixo),
            origem: ler_origem(origem)?,
            origem_y: ler_origem(origem_y)?,
            valor0: valor0.parse().ok()?,
            valor0_y: valor0_y.parse().ok()?,
            alvo: match tipo {
                "indice" => Alvo::Indice {
                    passo: a.max(1.0),
                    max: b.max(0.0) as usize,
                },
                "ponto" => Alvo::Ponto {
                    min_x: a,
                    max_x: b,
                    min_y: c,
                    max_y: d,
                },
                "zona" => Alvo::Zona { limiar: a.max(1.0) },
                _ => Alvo::Trilha { min: a, max: b },
            },
        })
    }

    /// Os pares `(chave, valor)` a escrever, dada a posição do ponteiro e o
    /// contexto de agora. Vazio quando **não há nada a escrever** — três casos
    /// legítimos: o quadro que só ancora a origem, o valor que não mudou de
    /// verdade, e a chave que já está no valor calculado. Cada entrada custa
    /// uma reavaliação da árvore, então esta é a diferença entre arrastar liso
    /// e arrastar aos tropeços.
    ///
    /// [`Alvo::Ponto`] é o único que pode devolver **duas** entradas — uma por
    /// eixo — porque é o único cujo `chave_y` existe.
    pub fn aplica(&mut self, p: Point, context: &ContextMap) -> Vec<(String, String)> {
        // [`Alvo::Zona`] não escreve nada durante o gesto — só ancora os dois
        // eixos para que o `dx`/`dy` do arrasto inteiro exista na soltura (ver
        // [`Arrasto::modo_no_release`]). A troca de `<splitter>`↔`<stack>`
        // acontece depois, entre quadros.
        if matches!(self.alvo, Alvo::Zona { .. }) {
            if self.origem.is_none() {
                self.origem = Some(p.x);
                self.origem_y = Some(p.y);
            }
            return Vec::new();
        }

        if let Alvo::Ponto {
            min_x,
            max_x,
            min_y,
            max_y,
        } = self.alvo
        {
            // O primeiro movimento só ancora os DOIS eixos de uma vez — um
            // ponto não tem como ancorar um e não o outro.
            let (Some(ox), Some(oy)) = (self.origem, self.origem_y) else {
                self.origem = Some(p.x);
                self.origem_y = Some(p.y);
                return Vec::new();
            };
            let novo_x = format!("{:.0}", (self.valor0 + (p.x - ox)).clamp(min_x, max_x));
            let novo_y = format!("{:.0}", (self.valor0_y + (p.y - oy)).clamp(min_y, max_y));
            let mut saida = Vec::new();
            if context.get(&self.chave) != Some(&novo_x) {
                saida.push((self.chave.clone(), novo_x));
            }
            if let Some(chave_y) = &self.chave_y {
                if context.get(chave_y) != Some(&novo_y) {
                    saida.push((chave_y.clone(), novo_y));
                }
            }
            return saida;
        }

        let c = self.eixo.coord(p);
        // O primeiro movimento depois do clique só ancora.
        let Some(origem) = self.origem else {
            self.origem = Some(c);
            return Vec::new();
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
            Alvo::Ponto { .. } | Alvo::Zona { .. } => {
                unreachable!("tratados nos braços acima")
            }
        };
        if context.get(&self.chave) != Some(&novo) {
            vec![(self.chave.clone(), novo)]
        } else {
            Vec::new()
        }
    }

    /// O `(chave, borda)` a escrever **na soltura** de um [`Alvo::Zona`], ou
    /// `None` quando não há o que fazer: alvo diferente, sem [`Arrasto::chave_modo`],
    /// gesto sem movimento (`origem` nunca ancorou), ou delta abaixo de `limiar`
    /// nos dois eixos — este último é o clique que não virou arrasto, e a borda
    /// não muda.
    ///
    /// Acima do limiar, o **eixo dominante** escolhe: mais horizontal →
    /// `left`/`right` pelo sinal de `dx`; mais vertical → `top`/`bottom` por
    /// `dy`. Não existe "soltou no centro" aqui — isso pediria o retângulo do
    /// widget, que o `grip.rs` de propósito não vê; flutuar é um botão do
    /// cabeçalho, não um alvo de arrasto.
    pub fn modo_no_release(&self, p: Point) -> Option<(String, String)> {
        let Alvo::Zona { limiar } = self.alvo else {
            return None;
        };
        let chave_modo = self.chave_modo.as_ref()?;
        let (ox, oy) = (self.origem?, self.origem_y?);
        let (dx, dy) = (p.x - ox, p.y - oy);
        if dx.abs() < limiar && dy.abs() < limiar {
            return None;
        }
        let borda = if dx.abs() >= dy.abs() {
            if dx < 0.0 { "left" } else { "right" }
        } else if dy < 0.0 {
            "top"
        } else {
            "bottom"
        };
        Some((chave_modo.clone(), borda.to_string()))
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

    /// Um [`Arrasto`] de um eixo só (`Trilha`/`Indice`), com os campos novos
    /// da Onda 11 (`chave_y`/`origem_y`/`valor0_y`) no valor que eles têm
    /// sempre que o alvo não é [`Alvo::Ponto`].
    fn arrasto_1d(chave: &str, indice: usize, eixo: Eixo, origem: Option<f32>, valor0: f32, alvo: Alvo) -> Arrasto {
        Arrasto {
            chave: chave.into(),
            chave_y: None,
            chave_modo: None,
            indice,
            eixo,
            origem,
            origem_y: None,
            valor0,
            valor0_y: 0.0,
            alvo,
        }
    }

    #[test]
    fn ida_e_volta_preserva_o_arrasto() {
        let a = arrasto_1d(
            "larguras",
            2,
            Eixo::Y,
            Some(140.5),
            90.0,
            Alvo::Trilha {
                min: 48.0,
                max: 1200.0,
            },
        );
        assert_eq!(Arrasto::ler(&a.escrever()), Some(a));
    }

    #[test]
    fn origem_em_aberto_sobrevive_a_ida_e_volta() {
        let a = arrasto_1d(
            "p",
            0,
            Eixo::X,
            None,
            200.0,
            Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        );
        let voltou = Arrasto::ler(&a.escrever()).unwrap();
        assert_eq!(voltou.origem, None);
    }

    #[test]
    fn o_primeiro_movimento_so_ancora() {
        let mut a = arrasto_1d(
            "p",
            0,
            Eixo::X,
            None,
            200.0,
            Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        );
        // Nada escrito, e a origem passa a existir.
        assert!(a.aplica(Point::new(500.0, 0.0), &ctx(&[])).is_empty());
        assert_eq!(a.origem, Some(500.0));
        // Do segundo em diante, a conta é exata a partir dali.
        let saida = a.aplica(Point::new(560.0, 0.0), &ctx(&[]));
        assert_eq!(saida, vec![("p".to_string(), "260".to_string())]);
    }

    #[test]
    fn trilha_arrastada_vira_fixa_e_preserva_as_vizinhas() {
        let mut a = arrasto_1d(
            "cols",
            1,
            Eixo::X,
            Some(100.0),
            120.0,
            Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        );
        let c = ctx(&[("cols", "160 fill 90")]);
        let saida = a.aplica(Point::new(150.0, 0.0), &c);
        assert_eq!(saida, vec![("cols".to_string(), "160 170 90".to_string())]);
    }

    #[test]
    fn o_piso_da_trilha_e_o_que_a_mantem_agarravel() {
        let mut a = arrasto_1d(
            "cols",
            0,
            Eixo::X,
            Some(300.0),
            120.0,
            Alvo::Trilha {
                min: 48.0,
                max: 900.0,
            },
        );
        // Puxar 400px para a esquerda daria -280: some da tela junto com a alça.
        let saida = a.aplica(Point::new(-100.0, 0.0), &ctx(&[]));
        assert_eq!(saida, vec![("cols".to_string(), "48".to_string())]);
    }

    #[test]
    fn o_indice_anda_um_degrau_por_passo_e_satura() {
        let mut a = arrasto_1d(
            "pagina",
            0,
            Eixo::X,
            Some(500.0),
            1.0,
            Alvo::Indice {
                passo: 120.0,
                max: 2,
            },
        );
        // Arrastar para a ESQUERDA anda para trás (o conteúdo segue o dedo).
        let saida = a.aplica(Point::new(380.0, 0.0), &ctx(&[]));
        assert_eq!(saida, vec![("pagina".to_string(), "0".to_string())]);
        // E satura: não existe página -1.
        let saida = a.aplica(Point::new(-900.0, 0.0), &ctx(&[]));
        assert_eq!(saida, vec![("pagina".to_string(), "0".to_string())]);
    }

    #[test]
    fn nao_escreve_quando_a_chave_ja_esta_no_valor() {
        let mut a = arrasto_1d(
            "pagina",
            0,
            Eixo::X,
            Some(500.0),
            1.0,
            Alvo::Indice {
                passo: 120.0,
                max: 4,
            },
        );
        // Meio degrau não muda nada, e uma entrada aqui custaria uma
        // reavaliação por pixel arrastado.
        assert!(
            a.aplica(Point::new(520.0, 0.0), &ctx(&[("pagina", "1")]))
                .is_empty()
        );
    }

    #[test]
    fn um_campo_a_menos_mata_o_arrasto_em_silencio() {
        assert_eq!(Arrasto::ler("cols||1|x|?|0|120|0|trilha|48"), None);
        assert_eq!(Arrasto::ler(""), None);
    }

    // ── Alvo::Ponto — o habilitador B da Onda 11 ────────────────────────

    fn arrasto_ponto(chave_x: &str, chave_y: &str, valor0: (f32, f32), alvo: Alvo) -> Arrasto {
        Arrasto {
            chave: chave_x.into(),
            chave_y: Some(chave_y.into()),
            chave_modo: None,
            indice: 0,
            eixo: Eixo::X, // ignorado por Ponto
            origem: None,
            origem_y: None,
            valor0: valor0.0,
            valor0_y: valor0.1,
            alvo,
        }
    }

    #[test]
    fn ponto_ida_e_volta_preserva_as_duas_chaves() {
        let a = Arrasto {
            origem: Some(12.0),
            origem_y: Some(34.0),
            ..arrasto_ponto(
                "win_x",
                "win_y",
                (100.0, 40.0),
                Alvo::Ponto {
                    min_x: 0.0,
                    max_x: 800.0,
                    min_y: 0.0,
                    max_y: 600.0,
                },
            )
        };
        assert_eq!(Arrasto::ler(&a.escrever()), Some(a));
    }

    #[test]
    fn ponto_primeiro_movimento_ancora_os_dois_eixos_de_uma_vez() {
        let mut a = arrasto_ponto(
            "win_x",
            "win_y",
            (100.0, 40.0),
            Alvo::Ponto {
                min_x: 0.0,
                max_x: 800.0,
                min_y: 0.0,
                max_y: 600.0,
            },
        );
        assert!(a.aplica(Point::new(300.0, 200.0), &ctx(&[])).is_empty());
        assert_eq!(a.origem, Some(300.0));
        assert_eq!(a.origem_y, Some(200.0));
    }

    #[test]
    fn ponto_escreve_as_duas_chaves_apos_ancorado() {
        let mut a = arrasto_ponto(
            "win_x",
            "win_y",
            (100.0, 40.0),
            Alvo::Ponto {
                min_x: 0.0,
                max_x: 800.0,
                min_y: 0.0,
                max_y: 600.0,
            },
        );
        let _ = a.aplica(Point::new(300.0, 200.0), &ctx(&[]));
        // Arrasta 20px em X e 10px em Y a partir da âncora.
        let mut saida = a.aplica(Point::new(320.0, 210.0), &ctx(&[]));
        saida.sort();
        assert_eq!(
            saida,
            vec![
                ("win_x".to_string(), "120".to_string()),
                ("win_y".to_string(), "50".to_string()),
            ]
        );
    }

    #[test]
    fn ponto_satura_em_cada_eixo_independentemente() {
        let mut a = arrasto_ponto(
            "win_x",
            "win_y",
            (10.0, 580.0),
            Alvo::Ponto {
                min_x: 0.0,
                max_x: 800.0,
                min_y: 0.0,
                max_y: 600.0,
            },
        );
        let _ = a.aplica(Point::new(0.0, 0.0), &ctx(&[]));
        // X anda pouco (fica dentro dos limites); Y estoura o máximo.
        let mut saida = a.aplica(Point::new(30.0, 500.0), &ctx(&[]));
        saida.sort();
        assert_eq!(
            saida,
            vec![
                ("win_x".to_string(), "40".to_string()),
                ("win_y".to_string(), "600".to_string()),
            ]
        );
    }

    #[test]
    fn ponto_sem_chave_y_escreve_so_x() {
        let mut a = Arrasto {
            chave_y: None,
            ..arrasto_ponto(
                "win_x",
                "ignorada",
                (0.0, 0.0),
                Alvo::Ponto {
                    min_x: 0.0,
                    max_x: 800.0,
                    min_y: 0.0,
                    max_y: 600.0,
                },
            )
        };
        let _ = a.aplica(Point::new(0.0, 0.0), &ctx(&[]));
        let saida = a.aplica(Point::new(10.0, 10.0), &ctx(&[]));
        assert_eq!(saida, vec![("win_x".to_string(), "10".to_string())]);
    }

    // ── Alvo::Zona — o habilitador D da Onda 12 ─────────────────────────

    fn arrasto_zona(limiar: f32) -> Arrasto {
        Arrasto {
            chave: String::new(),
            chave_y: None,
            chave_modo: Some("lado".into()),
            indice: 0,
            eixo: Eixo::X,
            origem: None,
            origem_y: None,
            valor0: 0.0,
            valor0_y: 0.0,
            alvo: Alvo::Zona { limiar },
        }
    }

    #[test]
    fn zona_ida_e_volta_preserva_chave_modo_e_limiar() {
        let a = Arrasto {
            origem: Some(10.0),
            origem_y: Some(20.0),
            ..arrasto_zona(48.0)
        };
        assert_eq!(Arrasto::ler(&a.escrever()), Some(a));
    }

    #[test]
    fn zona_nao_escreve_nada_durante_o_gesto_so_ancora() {
        let mut a = arrasto_zona(40.0);
        assert!(a.aplica(Point::new(300.0, 200.0), &ctx(&[])).is_empty());
        assert_eq!((a.origem, a.origem_y), (Some(300.0), Some(200.0)));
        // Segundo movimento: continua sem escrever.
        assert!(a.aplica(Point::new(80.0, 210.0), &ctx(&[])).is_empty());
    }

    #[test]
    fn zona_na_soltura_escolhe_a_borda_pelo_eixo_dominante() {
        let mut a = arrasto_zona(40.0);
        a.aplica(Point::new(300.0, 200.0), &ctx(&[])); // ancora

        // Puxou para a esquerda, bem além do limiar.
        assert_eq!(
            a.modo_no_release(Point::new(120.0, 210.0)),
            Some(("lado".to_string(), "left".to_string()))
        );
        // Para baixo, dy dominante.
        assert_eq!(
            a.modo_no_release(Point::new(310.0, 400.0)),
            Some(("lado".to_string(), "bottom".to_string()))
        );
        // Movimento minúsculo: foi um clique, a borda não muda.
        assert_eq!(a.modo_no_release(Point::new(305.0, 205.0)), None);
    }

    #[test]
    fn zona_sem_ancora_ou_sem_chave_modo_nao_faz_nada() {
        // Nunca moveu (origem em aberto).
        let a = arrasto_zona(40.0);
        assert_eq!(a.modo_no_release(Point::new(0.0, 500.0)), None);

        // Ancorou, mas sem chave de modo.
        let mut b = Arrasto {
            chave_modo: None,
            ..arrasto_zona(40.0)
        };
        b.aplica(Point::new(300.0, 200.0), &ctx(&[]));
        assert_eq!(b.modo_no_release(Point::new(0.0, 200.0)), None);
    }
}
