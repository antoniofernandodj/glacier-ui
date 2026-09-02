//! Medida do custo da abstração: quanto a árvore de [`UiNode`] custa em memória
//! e em tempo, comparada com o mesmo layout escrito à mão em `iced`.
//!
//! Não roda no `cargo test` comum (é `#[ignore]`, porque mede tempo e mediria
//! ruído numa máquina carregada). Para rodar:
//!
//! ```sh
//! cargo test --release --test perf_arvore -- --ignored --nocapture
//! ```
//!
//! Os números da 0.73 (antes) e da 0.74 (depois), com o que cada coluna
//! significa, estão no `CHANGELOG.md` da 0.74.0.

use glacier_ui::{Component, Context, GlacierUI, NodeType, Template, UiNode};
use std::time::Instant;

/// Uma tela realista: lista de pedidos, 7 nós por linha.
struct Tela;
impl Component for Tela {
    fn name(&self) -> &str {
        "Tela"
    }
    fn template(&self) -> Template {
        Template::Inline(
            r##"<component>
              <Column spacing="6" padding="12">
                <Text content="Pedidos ({total})" size="22" bold="true" />
                <Scrollable height="fill">
                  <Column spacing="4">
                    <ForEach items="linhas" var="l">
                      <Row spacing="8" padding="6" background="#1b1f27" border_radius="6">
                        <Text content="{l.id}" size="13" />
                        <Text content="{l.cliente}" size="13" width="fill" />
                        <Badge content="{l.status}" />
                        <Text content="{l.valor}" size="13" />
                        <Button text="abrir" on_click="abrir" />
                      </Row>
                    </ForEach>
                  </Column>
                </Scrollable>
              </Column>
            </component>"##
                .to_string(),
        )
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {}
}

fn linhas(n: usize, marca: &str) -> String {
    let mut s = String::from("[");
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"id\":\"#{i}\",\"cliente\":\"Cliente {i} {marca}\",\"status\":\"pago\",\"valor\":\"R$ {i},00\"}}"
        ));
    }
    s.push(']');
    s
}

/// Bytes que a árvore ocupa: o `size_of` de cada nó, a capacidade das strings
/// que ele guarda e **só a folga** do vetor de filhos (os filhos em si já são
/// contados na recursão — contá-los de novo pela capacidade inflaria tudo).
fn pesa(n: &UiNode, nos: &mut usize, bytes: &mut usize) {
    *nos += 1;
    *bytes += std::mem::size_of::<UiNode>();
    for s in [
        n.width.as_deref(),
        n.height.as_deref(),
        n.padding.as_deref(),
        n.background.as_deref(),
        n.class.as_deref(),
        n.id.as_deref(),
        n.border_color(),
        n.gradient(),
        n.font(),
    ]
    .into_iter()
    .flatten()
    {
        *bytes += s.len();
    }
    if let NodeType::Text { content, .. } = &n.kind {
        *bytes += content.capacity();
    }
    let folga = n.children.capacity().saturating_sub(n.children.len());
    *bytes += folga * std::mem::size_of::<UiNode>();
    for c in n.children.iter() {
        pesa(c, nos, bytes);
    }
}

fn rss_kb() -> usize {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        .unwrap_or(0)
}

#[test]
#[ignore = "mede tempo; rodar sob demanda com --release --ignored"]
fn custo_da_arvore() {
    println!("\nsize_of::<UiNode>()   = {} B", size_of::<UiNode>());
    println!("size_of::<NodeType>() = {} B\n", size_of::<NodeType>());
    println!(
        "{:>5} {:>7} {:>9} {:>9} {:>11} {:>11} {:>11} {:>11}",
        "N", "nós", "árvore", "RSS", "render", "reaval.", "c/ cache", "iced"
    );
    for n in [25usize, 100, 500, 2000] {
        rodada(n);
    }
    println!();
}

fn rodada(n: usize) {
    let rss0 = rss_kb();
    let mut m = GlacierUI::new();
    m.register(Box::new(Tela)).unwrap();
    m.define_data("total", &n.to_string());
    m.define_data("linhas", &linhas(n, "a"));
    m.navigate_to("Tela");
    m.reevaluate_all().unwrap();
    let rss1 = rss_kb();

    let (mut nos, mut bytes) = (0, 0);
    pesa(m.evaluated("Tela").unwrap(), &mut nos, &mut bytes);

    // Custo por frame: caminhar a árvore e montar os `Element` do iced.
    const FRAMES: u32 = 200;
    let t = Instant::now();
    for _ in 0..FRAMES {
        std::hint::black_box(m.render("Tela").unwrap());
    }
    let render = t.elapsed() / FRAMES;

    // Custo por mudança de estado que **invalida** a lista.
    const MUD: u32 = 20;
    let t = Instant::now();
    for i in 0..MUD {
        m.define_data("linhas", &linhas(n, &format!("v{i}")));
        m.reevaluate_all().unwrap();
    }
    let reaval = t.elapsed() / MUD;

    // Mudança que NÃO toca a lista: é o cache de avaliação que se mede aqui.
    let t = Instant::now();
    for i in 0..MUD {
        m.define_data("total", &format!("{i}"));
        m.reevaluate_all().unwrap();
    }
    let com_cache = t.elapsed() / MUD;

    // Referência: o mesmo layout montado à mão em iced.
    let t = Instant::now();
    for _ in 0..FRAMES {
        let mut col = iced::widget::Column::new().spacing(4);
        for i in 0..n {
            col = col.push(
                iced::widget::row![
                    iced::widget::text(format!("#{i}")).size(13),
                    iced::widget::text(format!("Cliente {i}")).size(13),
                    iced::widget::text("pago").size(13),
                    iced::widget::text(format!("R$ {i},00")).size(13),
                    iced::widget::button("abrir"),
                ]
                .spacing(8),
            );
        }
        let el: iced::Element<'_, ()> = col.into();
        std::hint::black_box(el);
    }
    let manual = t.elapsed() / FRAMES;

    println!(
        "{n:>5} {nos:>7} {:>7} KB {:>6} KB {:>11} {:>11} {:>11} {:>11}",
        bytes / 1024,
        rss1.saturating_sub(rss0),
        format!("{render:?}"),
        format!("{reaval:?}"),
        format!("{com_cache:?}"),
        format!("{manual:?}"),
    );
}
