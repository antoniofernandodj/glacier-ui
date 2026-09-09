//! O `canvas` como capacidade do motor — o **habilitador da Onda 7** do
//! `PLANO_WIDGETS.md`, e o que destrava de uma vez a família de medidores
//! (`<dial>`, `<gauge>`, `<lcdnumber>`) e a §2.13 inteira (`<sparkline>`,
//! `<linechart>`, `<barchart>`, `<piechart>`).
//!
//! # Por que o `canvas` não vira tag
//!
//! O `iced` tem `canvas`, e expor um `<canvas>` ao markup seria a tradução
//! literal do `QGraphicsView`. O projeto não faz isso, e a razão é a promessa
//! da primeira linha do `README`: estrutura, estilo e comportamento
//! **declarados**. Uma superfície de desenho livre devolveria ao app um bloco
//! de código imperativo que o `.gv` não sabe ler, o `.gss` não sabe estilizar e
//! o lado Luau não alcança — três regressões para ganhar uma.
//!
//! Então o `canvas` fica aqui, como **capacidade**: este módulo é a caixa de
//! ferramentas de desenho, e o que o app vê são sete tags que se comportam como
//! qualquer outro widget do motor — valor numa chave que ele nomeia, cor da
//! paleta do tema, `on_change` para delegar.
//!
//! (O `<canvas>` avulso continua catalogado, em P3 e marcado `●` na §2.6. Se um
//! dia sair, sai como *outra* coisa: um vocabulário declarativo de desenho —
//! `<path>`, `<arc>`, `<circle>` interpolando o contexto —, não como um
//! callback em Rust.)
//!
//! # O que mora aqui
//!
//! Só o que **mais de um** dos sete widgets usa:
//!
//! - [`arco`] e [`anel`] — a geometria de todo medidor circular;
//! - [`Serie`] — a leitura de uma chave de contexto como série de dados, na
//!   mesma convenção `items="chave"` que o `<select>` e o `<autocomplete>` já
//!   usam;
//! - [`escala`] — os degraus "bonitos" (1·2·5 × 10ⁿ) de um eixo;
//! - [`segmentos`] — a tabela de sete segmentos do `<lcdnumber>`;
//! - [`compacto`] — o número como uma pessoa o lê num eixo.
//!
//! O desenho de cada widget mora com ele: os medidores em `src/gauges.rs`, os
//! gráficos em `src/charts.rs`.
//!
//! # A disciplina que todos seguem
//!
//! O `canvas::Program` do `iced` tem um `type State` próprio, guardado na
//! árvore de widgets. **Ele não é estado do app.** Vale para o que morre com o
//! quadro — se a alça de um `<dial>` está presa agora —, nunca para o valor: o
//! valor mora na chave que o markup nomeia, como no `<slider>` e no `<rating>`.
//! É o que faz N instâncias do mesmo widget conviverem numa tela sem se ver, e
//! é a mesma conclusão a que as ondas 4 e 6 chegaram ao reclassificar o
//! `Spinner` e o `Rating` de `●` para primitiva.

use iced::widget::canvas::{Path, Text};
use iced::{Color, Point, Radians};

use crate::ContextMap;
use crate::widget::parse_hex_color;

/// Um ponto de uma série: o rótulo que o eixo mostra e o número que o desenho
/// usa.
#[derive(Debug, Clone, PartialEq)]
pub struct Ponto {
    pub rotulo: String,
    pub valor: f64,
}

/// Uma série de dados lida de uma chave do contexto.
///
/// A convenção do `items` é a **mesma** do `<select>` e do `<autocomplete>` —
/// uma chave que guarda um array JSON —, com as duas formas que um gráfico
/// aceita:
///
/// ```json
/// [12, 19, 7, 24]
/// [{"label": "Jan", "value": 12}, {"label": "Fev", "value": 19}]
/// ```
///
/// Na primeira, o rótulo é o índice (`"1"`, `"2"`, …) — é o que um
/// `<sparkline>` quer, e ele nem desenha rótulo. Na segunda, `label` e `value`
/// com os sinônimos que o resto do markup aceita (`rotulo`/`nome`,
/// `valor`/`y`), porque um JSON que veio de um backend raramente escolhe o
/// nome que a gente teria escolhido.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Serie {
    pub pontos: Vec<Ponto>,
}

impl Serie {
    /// Lê a chave do contexto. Chave ausente, JSON inválido ou raiz que não é
    /// array devolvem série **vazia** — e uma série vazia é um desenho vazio,
    /// não um erro: o gráfico de uma tela que ainda não carregou os dados é uma
    /// moldura, não um pânico.
    pub fn ler(context: &ContextMap, chave: &str) -> Self {
        let Some(bruto) = context.get(chave) else {
            return Self::default();
        };
        let Ok(serde_json::Value::Array(itens)) =
            serde_json::from_str::<serde_json::Value>(bruto)
        else {
            return Self::default();
        };
        Self::de_array(&itens)
    }

    /// O núcleo de [`Serie::ler`], a partir de um array JSON **já parseado** —
    /// usado por [`crate::charts`] para ler os `points` de dentro de um objeto
    /// de série (`series="[{name, points}]"`, o habilitador C da Onda 11).
    /// Aceita as mesmas duas formas: `[1, 2, 3]` e `[{label, value}]`.
    pub fn de_array(itens: &[serde_json::Value]) -> Self {
        let pontos = itens
            .iter()
            .enumerate()
            .map(|(i, item)| match item {
                serde_json::Value::Object(o) => {
                    let campo = |nomes: &[&str]| -> Option<&serde_json::Value> {
                        nomes.iter().find_map(|n| o.get(*n))
                    };
                    let rotulo = campo(&["label", "rotulo", "rótulo", "nome", "x"])
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            outro => outro.to_string(),
                        })
                        .unwrap_or_else(|| (i + 1).to_string());
                    let valor = campo(&["value", "valor", "y"])
                        .and_then(numero)
                        .unwrap_or(0.0);
                    Ponto { rotulo, valor }
                }
                outro => Ponto {
                    rotulo: (i + 1).to_string(),
                    valor: numero(outro).unwrap_or(0.0),
                },
            })
            .collect();

        Self { pontos }
    }

    pub fn is_empty(&self) -> bool {
        self.pontos.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pontos.len()
    }

    pub fn valores(&self) -> impl Iterator<Item = f64> + '_ {
        self.pontos.iter().map(|p| p.valor)
    }

    /// O menor e o maior valor da série. Série vazia devolve `(0, 1)` — uma
    /// faixa degenerada dividiria por zero na hora de mapear para pixels.
    pub fn faixa(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for v in self.valores() {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        if !min.is_finite() || !max.is_finite() {
            return (0.0, 1.0);
        }
        // Uma série constante (`[5, 5, 5]`) tem faixa zero, e a linha dela
        // sairia colada numa borda ou dividiria por zero. Abrir a faixa em
        // torno do valor é o que a põe no meio, que é onde uma pessoa espera
        // ver uma reta.
        if (max - min).abs() < f64::EPSILON {
            let folga = if min.abs() < f64::EPSILON {
                1.0
            } else {
                min.abs() * 0.5
            };
            return (min - folga, max + folga);
        }
        (min, max)
    }

    /// A soma, para o `<piechart>`. Fatias negativas não existem num setor
    /// circular: elas entram como zero, e a fatia some em vez de comer as
    /// vizinhas.
    pub fn soma_positiva(&self) -> f64 {
        self.valores().filter(|v| *v > 0.0).sum()
    }
}

/// Um número JSON, ou uma string que parseia como número — porque um backend
/// que serializa decimais como texto (`"12.5"`) é comum demais para o motor
/// tratar como erro.
fn numero(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse::<f64>().ok(),
        serde_json::Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Geometria
// ─────────────────────────────────────────────────────────────────────────────

/// Graus em radianos, na convenção do `canvas` do `iced`: **0° é o lado
/// direito** (o eixo X positivo) e o ângulo cresce no **sentido horário**,
/// porque o Y da tela cresce para baixo.
///
/// Existe como função porque a alternativa — espalhar `to_radians()` por três
/// arquivos — foi o que fez a primeira versão do `<gauge>` abrir para o lado
/// errado.
#[inline]
pub fn graus(g: f32) -> Radians {
    Radians(g.to_radians())
}

/// Quantos segmentos aproximam um arco desta abertura e deste raio.
///
/// Um por grau, com piso e teto. Num raio de 100px isso dá cordas de ~1,7px —
/// invisíveis — e num medidor de 40px sobra suavidade de sobra.
fn passos(varredura: f32, raio: f32) -> usize {
    let por_grau = varredura.abs().ceil() as usize;
    let minimo = if raio > 120.0 { 2 * por_grau } else { por_grau };
    minimo.clamp(6, 1440)
}

/// O arco aberto, para traçar. `varredura` é quanto ele anda a partir de
/// `inicio`, em graus e no sentido horário.
///
/// # Por que polilinha, e não `Builder::arc`
///
/// O `arc` do `iced` chama `Builder::ellipse`, e essa função começa com um
/// `move_to` **sempre**. Ou seja: dois arcos no mesmo caminho viram dois
/// sub-caminhos soltos, e o preenchimento de um anel sai deformado — as fatias
/// de uma pizza apareciam com pedaços faltando perto do centro, e as faixas de
/// um `<gauge>` preenchiam até o miolo em vez de ficarem no anel. Com a
/// polilinha o caminho é UM só, e o `close()` fecha o que se espera.
pub fn arco(centro: Point, raio: f32, inicio: f32, varredura: f32) -> Path {
    let n = passos(varredura, raio);
    Path::new(|b| {
        for i in 0..=n {
            let a = inicio + varredura * (i as f32 / n as f32);
            let p = na_borda(centro, raio, a);
            if i == 0 {
                b.move_to(p);
            } else {
                b.line_to(p);
            }
        }
    })
}

/// O anel (arco **fechado**, com espessura), para preencher — e a fatia de
/// pizza quando a espessura chega ao centro.
///
/// É o corpo de todo medidor: a borda externa de ida, a interna de volta. Um
/// caminho só, pelo motivo explicado em [`arco`].
pub fn anel(centro: Point, raio: f32, espessura: f32, inicio: f32, varredura: f32) -> Path {
    let externo = raio.max(0.0);
    let interno = (raio - espessura).max(0.0);
    let n = passos(varredura, raio);
    Path::new(|b| {
        for i in 0..=n {
            let a = inicio + varredura * (i as f32 / n as f32);
            let p = na_borda(centro, externo, a);
            if i == 0 {
                b.move_to(p);
            } else {
                b.line_to(p);
            }
        }
        // Raio interno nulo é uma FATIA, não um anel: fecha pelo centro, que é
        // o que um `<piechart>` sem buraco desenha.
        if interno <= 0.5 {
            b.line_to(centro);
        } else {
            for i in (0..=n).rev() {
                let a = inicio + varredura * (i as f32 / n as f32);
                b.line_to(na_borda(centro, interno, a));
            }
        }
        b.close();
    })
}

/// O ponto sobre a circunferência, no ângulo dado.
#[inline]
pub fn na_borda(centro: Point, raio: f32, angulo: f32) -> Point {
    let r = angulo.to_radians();
    Point::new(centro.x + raio * r.cos(), centro.y + raio * r.sin())
}

/// Onde um valor cai dentro de uma faixa, em `[0, 1]`. Faixa degenerada
/// devolve `0.0` em vez de `NaN` — um `NaN` viraria um desenho invisível, que é
/// o pior modo de falha de um widget que só desenha.
#[inline]
pub fn fracao(valor: f64, min: f64, max: f64) -> f32 {
    if (max - min).abs() < f64::EPSILON {
        return 0.0;
    }
    (((valor - min) / (max - min)) as f32).clamp(0.0, 1.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Eixos
// ─────────────────────────────────────────────────────────────────────────────

/// Um eixo pronto para desenhar: a faixa **arredondada para fora** e o passo
/// entre marcas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Escala {
    pub min: f64,
    pub max: f64,
    pub passo: f64,
}

impl Escala {
    /// As marcas, de `min` a `max` inclusive.
    pub fn marcas(&self) -> Vec<f64> {
        let mut v = Vec::new();
        if self.passo <= 0.0 {
            return v;
        }
        // O contador existe porque somar `passo` acumula erro de ponto
        // flutuante: sem ele, um eixo de 0 a 1 com passo 0,1 às vezes para em
        // 0,9999999 e perde a última marca.
        let quantas = ((self.max - self.min) / self.passo).round() as i64;
        for i in 0..=quantas.clamp(0, 200) {
            v.push(self.min + self.passo * i as f64);
        }
        if v.last().is_none_or(|u| (*u - self.max).abs() > self.passo * 0.5) {
            v.push(self.max);
        }
        v
    }
}

/// Os degraus "bonitos" de um eixo: 1, 2 ou 5 vezes uma potência de dez, o
/// bastante para caber `alvo` marcas.
///
/// É a conta que separa um eixo legível de um que marca 0, 3,714, 7,428. E a
/// faixa sai **arredondada para fora** do dado, senão o maior ponto da série
/// encosta na borda de cima e some no traço.
pub fn escala(min: f64, max: f64, alvo: usize) -> Escala {
    let alvo = alvo.max(2) as f64;
    let bruto = (max - min) / alvo;
    if !bruto.is_finite() || bruto <= 0.0 {
        return Escala {
            min,
            max: max.max(min + 1.0),
            passo: (max - min).abs().max(1.0),
        };
    }

    let magnitude = 10f64.powf(bruto.log10().floor());
    let normalizado = bruto / magnitude;
    let passo = if normalizado <= 1.0 {
        1.0
    } else if normalizado <= 2.0 {
        2.0
    } else if normalizado <= 5.0 {
        5.0
    } else {
        10.0
    } * magnitude;

    Escala {
        min: (min / passo).floor() * passo,
        max: (max / passo).ceil() * passo,
        passo,
    }
}

/// O número como uma pessoa o lê num eixo: sem casas decimais quando não
/// precisa, com sufixo `k`/`M`/`G` quando é grande.
///
/// Um eixo que escreve `1200000` empurra o gráfico para fora da tela; um que
/// escreve `1.2M` cabe. É a mesma escolha que todo painel de métrica faz.
pub fn compacto(v: f64) -> String {
    let a = v.abs();
    let (div, sufixo) = if a >= 1e9 {
        (1e9, "G")
    } else if a >= 1e6 {
        (1e6, "M")
    } else if a >= 1e3 {
        (1e3, "k")
    } else {
        (1.0, "")
    };
    let n = v / div;
    // Uma casa só quando ela diz alguma coisa: `1.2k` sim, `12.0` não.
    if sufixo.is_empty() && (n - n.round()).abs() < 1e-9 {
        format!("{}", n.round() as i64)
    } else if (n - n.round()).abs() < 1e-9 {
        format!("{}{sufixo}", n.round() as i64)
    } else {
        format!("{n:.1}{sufixo}")
    }
}

/// O valor formatado com um número fixo de casas — o que o `<gauge>` e o
/// `<dial>` escrevem no meio, onde o dado é um valor só e o arredondamento do
/// eixo seria mentira.
pub fn com_casas(v: f64, casas: usize) -> String {
    format!("{v:.*}", casas.min(6))
}

// ─────────────────────────────────────────────────────────────────────────────
// Sete segmentos
// ─────────────────────────────────────────────────────────────────────────────

/// Os sete segmentos de um dígito, na ordem clássica `a b c d e f g`:
///
/// ```text
///   ─a─
///  f│ │b
///   ─g─
///  e│ │c
///   ─d─
/// ```
///
/// Um bit por segmento, `a` no menos significativo. Quem não tem forma no
/// display — uma letra qualquer — acende **nada**, que é exatamente o que um
/// `QLCDNumber` faz.
pub fn segmentos(c: char) -> u8 {
    match c {
        '0' => 0b0111111,
        '1' => 0b0000110,
        '2' => 0b1011011,
        '3' => 0b1001111,
        '4' => 0b1100110,
        '5' => 0b1101101,
        '6' => 0b1111101,
        '7' => 0b0000111,
        '8' => 0b1111111,
        '9' => 0b1101111,
        // O punhado de letras que um display de verdade sabe fazer — o
        // bastante para um `Err`, um `HI`/`LO` ou uma unidade curta.
        'a' | 'A' => 0b1110111,
        'b' | 'B' => 0b1111100,
        'c' | 'C' => 0b0111001,
        'd' | 'D' => 0b1011110,
        'e' | 'E' => 0b1111001,
        'f' | 'F' => 0b1110001,
        'h' | 'H' => 0b1110110,
        'l' | 'L' => 0b0111000,
        'o' | 'O' => 0b0111111,
        'p' | 'P' => 0b1110011,
        'r' | 'R' => 0b1010000,
        'u' | 'U' => 0b0111110,
        '-' => 0b1000000,
        _ => 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Cor e texto
// ─────────────────────────────────────────────────────────────────────────────

/// A cor que o markup pediu, ou a do tema quando ele não pediu nada — o mesmo
/// fallback do `color` do `<rating>` e do `<progressbar>`.
#[inline]
pub fn cor_ou(prop: &str, padrao: Color) -> Color {
    parse_hex_color(prop).unwrap_or(padrao)
}

/// Um rótulo de eixo/medidor já com os defaults que todo desenho daqui usa.
///
/// Existe para que os sete widgets não repitam sete vezes a mesma construção
/// de `Text` — e para que mudar o alinhamento de um rótulo seja uma linha, não
/// sete.
pub fn rotulo(conteudo: String, posicao: Point, tamanho: f32, cor: Color) -> Text {
    Text {
        content: conteudo,
        position: posicao,
        color: cor,
        size: tamanho.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        align_y: iced::alignment::Vertical::Center,
        ..Text::default()
    }
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
    fn serie_aceita_as_duas_formas() {
        let c = ctx(&[
            ("nums", "[1, 2.5, 3]"),
            ("objs", r#"[{"label":"Jan","value":10},{"nome":"Fev","valor":"20"}]"#),
        ]);

        let n = Serie::ler(&c, "nums");
        assert_eq!(n.len(), 3);
        assert_eq!(n.pontos[1].valor, 2.5);
        // Sem `label`, o rótulo é a posição — 1-based, como uma pessoa conta.
        assert_eq!(n.pontos[0].rotulo, "1");

        let o = Serie::ler(&c, "objs");
        assert_eq!(o.pontos[0].rotulo, "Jan");
        // O decimal que veio como string continua sendo número.
        assert_eq!(o.pontos[1].valor, 20.0);
        assert_eq!(o.pontos[1].rotulo, "Fev");
    }

    #[test]
    fn serie_ausente_ou_invalida_e_vazia_e_nao_erro() {
        let c = ctx(&[("ruim", "{não é json"), ("obj", r#"{"a":1}"#)]);
        assert!(Serie::ler(&c, "inexistente").is_empty());
        assert!(Serie::ler(&c, "ruim").is_empty());
        assert!(Serie::ler(&c, "obj").is_empty());
    }

    #[test]
    fn faixa_de_serie_constante_abre_em_torno_do_valor() {
        let c = ctx(&[("s", "[5, 5, 5]"), ("z", "[0, 0]")]);
        let (min, max) = Serie::ler(&c, "s").faixa();
        assert!(min < 5.0 && max > 5.0, "a reta precisa cair no meio");
        // Zero não tem "metade de si" — a folga vira 1, senão a faixa
        // continuaria degenerada.
        let (min, max) = Serie::ler(&c, "z").faixa();
        assert_eq!((min, max), (-1.0, 1.0));
    }

    #[test]
    fn escala_arredonda_para_fora_em_degraus_bonitos() {
        let e = escala(0.0, 37.0, 5);
        assert_eq!(e.passo, 10.0);
        assert_eq!((e.min, e.max), (0.0, 40.0));

        let e = escala(3.0, 9.0, 3);
        assert_eq!(e.passo, 2.0);
        assert_eq!((e.min, e.max), (2.0, 10.0));

        // Nenhuma marca pode faltar por erro de ponto flutuante: 0..1 de 0,1 em
        // 0,1 são onze marcas, não dez.
        let e = escala(0.0, 1.0, 10);
        assert_eq!(e.marcas().len(), 11);
    }

    #[test]
    fn fracao_nunca_devolve_nan() {
        assert_eq!(fracao(5.0, 0.0, 10.0), 0.5);
        assert_eq!(fracao(-3.0, 0.0, 10.0), 0.0, "presa no piso");
        assert_eq!(fracao(99.0, 0.0, 10.0), 1.0, "presa no teto");
        assert_eq!(fracao(7.0, 7.0, 7.0), 0.0, "faixa degenerada não é NaN");
    }

    #[test]
    fn compacto_escreve_o_que_uma_pessoa_le() {
        assert_eq!(compacto(12.0), "12");
        assert_eq!(compacto(1000.0), "1k");
        assert_eq!(compacto(1250.0), "1.2k");
        assert_eq!(compacto(3_400_000.0), "3.4M");
        assert_eq!(compacto(-1500.0), "-1.5k");
    }

    #[test]
    fn sete_segmentos_cobre_os_digitos_e_apaga_o_resto() {
        assert_eq!(segmentos('8').count_ones(), 7);
        assert_eq!(segmentos('1').count_ones(), 2);
        assert_eq!(segmentos('-'), 0b1000000);
        assert_eq!(segmentos('z'), 0, "sem forma no display = apagado");
    }
}
