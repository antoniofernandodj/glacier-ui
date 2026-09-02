//! Instrumentação de quadro, ligada por variável de ambiente.
//!
//! Existe para responder **uma** pergunta, que nenhuma medida fora do app
//! consegue responder: de um quadro lento, quanto é o motor e quanto é o resto?
//!
//! O motor cronometra **duas** coisas suas: o `render` (percorrer a árvore
//! avaliada e montar os `Element`) e o `dispatch` (tratar uma mensagem — o
//! `update` do componente, os handlers Luau, a reavaliação da árvore). O que
//! sobra do intervalo entre duas chamadas de `view` é o **resto**: layout,
//! moldagem de texto e desenho, tudo dentro do `iced` e do `wgpu`.
//!
//! A separação existe porque a primeira versão desta instrumentação media só o
//! render e chamava todo o resto de "fora do motor" — o que atribuía ao `iced`
//! trabalho que era do `dispatch` e dos handlers do app. Um quadro que oscila
//! entre 40 e 5 quadros por segundo com a mesma árvore não é custo de pixel, é
//! travada de `update`; sem separar as duas, não dava para ver a diferença.
//!
//! ```sh
//! GLACIER_PERF=1 ./meu-app
//! ```
//!
//! ```text
//! [glacier perf] 58 quadros 1.00s 58.0fps | nós 1682 | quadro 17.2ms méd 21.0 p95 34.1 máx
//!                render 0.43 méd 0.71 p95 | dispatch 1.20/quadro (14 msgs, 4.9 máx)
//!                resto 15.6ms/quadro (90.7%)
//! ```
//!
//! Como decidir, olhando as três parcelas:
//!
//! - **`render` grande** → é o motor montando `Element` demais. Menos nós
//!   (`virtualize`, ver `PRIMITIVAS.md`).
//! - **`dispatch` grande** → é tratamento de mensagem: `update`, handlers Luau,
//!   reavaliação. Olhe quantas mensagens por quadro e qual a mais cara.
//! - **`resto` grande com árvore pequena** → é o `iced`/`wgpu`: layout,
//!   moldagem de texto, rasterização. Aí o motor não tem o que fazer, e a
//!   investigação vira qual adaptador o `wgpu` escolheu.
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
    /// Tempo somado em `dispatch` desde o início da janela, e quantas mensagens
    /// foram tratadas — o custo do `update` do app, que antes se escondia
    /// dentro do "resto".
    dispatch: Duration,
    /// A mensagem mais cara da janela: é ela que trava um quadro sozinha.
    dispatch_max: Duration,
    mensagens: u32,
    /// Quantas de cada tipo. No `iced` **toda mensagem provoca um quadro**, e
    /// uma tela parada que recebe cento e cinquenta mensagens por segundo está
    /// redesenhando cento e cinquenta vezes sem nada ter mudado. Saber qual
    /// mensagem é essa é a diferença entre consertar e adivinhar.
    por_tipo: Vec<(&'static str, u32)>,
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
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let mut renders = j.renders.clone();
    let mut quadros = j.quadros.clone();
    let (render_med, render_p95) = med_p95(&mut renders);
    let (quadro_med, quadro_p95) = med_p95(&mut quadros);
    let quadro_max = quadros.iter().copied().max().map(ms).unwrap_or(0.0);
    let n = j.renders.len();
    let fps = n as f64 / decorrido.as_secs_f64();
    // O `dispatch` acontece entre dois quadros, então o custo dele se dilui
    // por quadro — é assim que ele se compara com o render e com o resto.
    let disp_por_quadro = ms(j.dispatch) / n as f64;
    // O que sobra do intervalo depois de tirar as duas parcelas do motor:
    // layout, texto, GPU e, num app folgado, a espera pelo vsync.
    let resto = (quadro_med - render_med - disp_por_quadro).max(0.0);
    let parte_resto = if quadro_med > 0.0 {
        resto / quadro_med * 100.0
    } else {
        0.0
    };
    let mut tipos = j.por_tipo.clone();
    tipos.sort_unstable_by_key(|(_, n)| std::cmp::Reverse(*n));
    let quais: Vec<String> = tipos
        .iter()
        .take(4)
        .map(|(t, n)| format!("{t}×{n}"))
        .collect();
    eprintln!(
        "[glacier perf] {n} quadros {:.2}s {fps:.1}fps | nós {} | quadro {quadro_med:.1}ms méd \
         {quadro_p95:.1} p95 {quadro_max:.1} máx\n               render {render_med:.3} méd \
         {render_p95:.3} p95 | dispatch {disp_por_quadro:.3}/quadro ({} msgs, {:.3} máx) | \
         resto {resto:.1}ms/quadro ({parte_resto:.1}%)\n               msgs: {}",
        decorrido.as_secs_f64(),
        j.nos,
        j.mensagens,
        ms(j.dispatch_max),
        if quais.is_empty() {
            "—".to_string()
        } else {
            quais.join("  ")
        },
    );
}

/// Anota uma mensagem tratada. Chamada só quando [`ligado`] é verdadeiro.
pub(crate) fn anota_dispatch(duracao: Duration, tipo: &'static str) {
    JANELA.with(|j| {
        let mut j = j.borrow_mut();
        j.dispatch += duracao;
        j.dispatch_max = j.dispatch_max.max(duracao);
        j.mensagens += 1;
        // Busca linear numa lista de no máximo algumas dezenas de tipos, e só
        // com a instrumentação ligada: mais barato que um mapa aqui.
        match j.por_tipo.iter_mut().find(|(t, _)| *t == tipo) {
            Some((_, n)) => *n += 1,
            None => j.por_tipo.push((tipo, 1)),
        }
    });
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
