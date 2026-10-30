//! Verificador, SEM janela, dos seletores `:first-child`, `:last-child` e `:nth-child()`
//! dentro de um `foreach`: avalia a tela e imprime o fundo de cada item. Ver
//! `docs/GSS_MODERNO.md`.
//!
//!   cargo run --example css_estrutural_check

use glacier_ui::{Component, Context, GlacierUI, NodeType, Template, UiNode};

struct Lista(usize);

impl Component for Lista {
    fn name(&self) -> &str {
        "lista"
    }
    fn template(&self) -> Template {
        Template::Inline(
            r#"<screen><column><foreach items="itens" var="s"><container class="item"><text>{s}</text></container></foreach></column></screen>"#
                .to_string(),
        )
    }
    fn init(&mut self, ctx: &mut Context) {
        let itens: Vec<String> = (1..=self.0).map(|n| format!("\"i{n}\"")).collect();
        ctx.set("itens", format!("[{}]", itens.join(",")));
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {}
}

fn fundos(n: &UiNode, out: &mut Vec<Option<String>>) {
    if matches!(n.kind, NodeType::Container) {
        out.push(n.background.clone());
    }
    for c in n.children.iter() {
        fundos(c, out);
    }
}

fn rodar(total: usize) -> Vec<Option<String>> {
    let mut motor = GlacierUI::new();
    std::fs::write(
        "/tmp/css_estrutural_check.gss",
        ".item { background: #111111; }\n.item:nth-child(odd) { background: #222222; }\n.item:first-child { background: #AA0000; }\n.item:last-child { background: #00AA00; }\n.item:nth-child(3n) { background: #0000AA; }\n",
    )
    .unwrap();
    motor.load_stylesheet("/tmp/css_estrutural_check.gss").expect("gss");
    motor.register(Box::new(Lista(total))).expect("registrar");
    motor.set_initial_screen("lista");
    let raiz = motor.evaluated("lista").expect("avaliar");
    let mut v = Vec::new();
    fundos(raiz, &mut v);
    v
}

fn main() {
    let mut falhas = 0;
    let mut confere = |nome: &str, ok: bool| {
        println!("{} {nome}", if ok { "ok  " } else { "FALHA" });
        if !ok {
            falhas += 1;
        }
    };
    let b = |s: &str| Some(s.to_string());

    let v = rodar(6);
    println!("6 itens: {v:?}");
    confere("6 itens: 1º é first-child (vermelho)", v[0] == b("#AA0000"));
    confere("6 itens: 2º é par (base)", v[1] == b("#111111"));
    confere("6 itens: 3º é 3n (azul), que vence o odd por vir depois", v[2] == b("#0000AA"));
    confere("6 itens: 5º é odd (cinza)", v[4] == b("#222222"));
    confere("6 itens: 6º é 3n, declarado DEPOIS de last-child (azul)", v[5] == b("#0000AA"));

    // O total entra na chave do cache: com 4 itens o último é outro.
    let v = rodar(4);
    println!("4 itens: {v:?}");
    confere("4 itens: o 4º agora é o last-child (verde)", v[3] == b("#00AA00"));
    confere("4 itens: o 3º segue 3n (azul)", v[2] == b("#0000AA"));

    let v = rodar(1);
    println!("1 item: {v:?}");
    confere("1 item: é first-child E last-child; last-child vem depois (verde)", v[0] == b("#00AA00"));

    if falhas > 0 {
        std::process::exit(1);
    }
    println!("tudo certo");
}
