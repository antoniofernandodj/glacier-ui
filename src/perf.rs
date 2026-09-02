//! Instrumentação de quadro, ligada por variável de ambiente.
//!
//! Existe para responder **uma** pergunta, que nenhuma medida fora do app
//! consegue responder: de um quadro lento, quanto é o motor e quanto é o resto?
//!
//! O motor sabe cronometrar a parte dele — percorrer a árvore avaliada e montar
//! os `Element` do `iced`. O que vem depois (medir o layout, moldar o texto,
//! desenhar na GPU) acontece dentro do `iced` e do `wgpu`, fora do alcance
//! daqui. Mas dá para medi-lo **por diferença**: o intervalo entre duas
//! chamadas de `view` é o quadro inteiro, e o que sobra depois de descontar o
//! render é tudo que não é o motor.
//!
//! ```sh
//! GLACIER_PERF=1 ./meu-app
//! ```
//!
//! ```text
//! [glacier perf] 58 quadros em 1.00s = 58.0 fps  |  render 0.43ms méd, 0.71ms p95
//!                nós 1682  |  motor 2.5% do quadro  |  fora do motor 16.8ms/quadro
//! ```
//!
//! A linha que decide é a última: **fora do motor**. Se ela for a quase
//! totalidade do quadro, otimizar o glacier não vai adiantar — o tempo está no
//! layout/desenho do `iced`, e a saída é entregar menos nós a ele (ver
//! `virtualize` em `PRIMITIVAS.md`).
//!
//! **Como ler.** Num app que não dá conta do quadro não há tempo ocioso, então
//! o "fora do motor" é trabalho de verdade — layout, moldagem de texto, GPU. Num
//! app folgado, boa parte dele é espera pelo vsync, e o número não quer dizer
//! nada: compare os dois casos rolando e parado.
//!
//! A conta supõe que o `iced` chama `view` **uma vez por quadro**, que é o que
//! ele faz. Com mais de uma janela aberta, as medidas se somam num relatório só
//! (o coletor é por thread, e a UI toda roda numa).
//!
//! Desligado, o custo é uma leitura de `bool` já resolvida por quadro.

use std::cell::RefCell;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// `GLACIER_PERF` definida e diferente de `0`/`false`/vazio.
///
/// Lida **uma vez** por processo: a variável não muda em tempo de execução, e
/// consultar o ambiente por quadro seria a instrumentação virando o problema.
pub(crate) fn ligado() -> bool {
    static LIGADO: OnceLock<bool> = OnceLock::new();
    *LIGADO.get_or_init(|| {
        std::env::var("GLACIER_PERF").is_ok_and(|v| {
            let v = v.trim();
            !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
        })
    })
}

/// De quanto em quanto tempo uma linha é impressa. Curto demais polui a saída;
/// longo demais esconde uma travada que dura pouco.
const INTERVALO: Duration = Duration::from_secs(1);

#[derive(Default)]
struct Janela {
    /// Quando esta janela de medida começou.
    inicio: Option<Instant>,
    /// Fim do render anterior — a origem do intervalo entre quadros.
    ultimo_quadro: Option<Instant>,
    /// Duração de cada render desta janela, para tirar média e p95.
    renders: Vec<Duration>,
    /// Intervalo entre quadros consecutivos.
    quadros: Vec<Duration>,
    /// Nós da última árvore renderizada.
    nos: usize,
}

thread_local! {
    /// Por thread porque a UI do `iced` roda numa só, e isso dispensa
    /// sincronização no caminho de quadro.
    static JANELA: RefCell<Janela> = RefCell::new(Janela::default());
}

/// Anota um render concluído: quanto ele levou e de quantos nós.
///
/// Imprime uma linha quando a janela de medida fecha. Chamada só quando
/// [`ligado`] é verdadeiro.
pub(crate) fn anota(duracao: Duration, nos: usize) {
    let agora = Instant::now();
    JANELA.with(|j| {
        let mut j = j.borrow_mut();
        let inicio = *j.inicio.get_or_insert(agora);
        if let Some(anterior) = j.ultimo_quadro {
            j.quadros.push(agora.saturating_duration_since(anterior));
        }
        j.ultimo_quadro = Some(agora);
        j.renders.push(duracao);
        j.nos = nos;

        let decorrido = agora.saturating_duration_since(inicio);
        if decorrido >= INTERVALO && j.renders.len() > 1 {
            relata(&j, decorrido);
            *j = Janela {
                // A janela nova continua de onde esta parou: sem isto, o
                // primeiro intervalo dela sairia como zero.
                ultimo_quadro: Some(agora),
                ..Default::default()
            };
        }
    });
}

/// Média e percentil 95 de uma amostra, em milissegundos.
fn med_p95(amostra: &mut [Duration]) -> (f64, f64) {
    if amostra.is_empty() {
        return (0.0, 0.0);
    }
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let media = amostra.iter().copied().map(ms).sum::<f64>() / amostra.len() as f64;
    amostra.sort_unstable();
    // `len-1` no pior caso, então o índice nunca passa do fim.
    let i = ((amostra.len() as f64 * 0.95).ceil() as usize).min(amostra.len()) - 1;
    (media, ms(amostra[i]))
}

fn relata(j: &Janela, decorrido: Duration) {
    let mut renders = j.renders.clone();
    let mut quadros = j.quadros.clone();
    let (render_med, render_p95) = med_p95(&mut renders);
    let (quadro_med, quadro_p95) = med_p95(&mut quadros);
    let fps = j.renders.len() as f64 / decorrido.as_secs_f64();
    // O quadro é o intervalo entre dois renders; o que não é render é tudo o
    // que acontece depois dele — layout, texto, GPU, e o tempo ocioso à espera
    // do vsync. Num app que não dá conta do quadro, não há ocioso: o que
    // aparecer aqui é trabalho.
    let fora = (quadro_med - render_med).max(0.0);
    let parte = if quadro_med > 0.0 {
        render_med / quadro_med * 100.0
    } else {
        0.0
    };
    eprintln!(
        "[glacier perf] {} quadros em {:.2}s = {fps:.1} fps  |  render {render_med:.3}ms méd, \
         {render_p95:.3}ms p95  |  nós {}  |  motor {parte:.1}% do quadro  |  \
         fora do motor {fora:.3}ms/quadro (p95 do quadro {quadro_p95:.3}ms)",
        j.renders.len(),
        decorrido.as_secs_f64(),
        j.nos,
    );
}

/// Conta os nós de uma árvore avaliada. Só roda com a instrumentação ligada.
pub(crate) fn conta_nos(no: &crate::parser::UiNode) -> usize {
    1 + no.children.iter().map(conta_nos).sum::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O p95 tem de cair numa amostra existente, e o índice nunca passar do
    /// fim — inclusive com um elemento só (onde média e p95 coincidem).
    #[test]
    fn p95_nao_passa_do_fim() {
        let mut um = [Duration::from_millis(7)];
        assert_eq!(med_p95(&mut um), (7.0, 7.0));

        let mut cem: Vec<Duration> = (1..=100).map(Duration::from_millis).collect();
        let (media, p95) = med_p95(&mut cem);
        assert_eq!(media, 50.5);
        assert_eq!(p95, 95.0, "o 95º de 1..=100");
    }

    /// Amostra vazia não pode dividir por zero nem indexar nada.
    #[test]
    fn p95_de_amostra_vazia() {
        assert_eq!(med_p95(&mut []), (0.0, 0.0));
    }
}
