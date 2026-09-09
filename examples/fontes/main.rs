//! **Onda 10** do `PLANO_WIDGETS.md`: a fonte que o motor não sabia nomear.
//!
//! Rode com: `cargo run --example fontes` (nesta máquina, com
//! `WGPU_BACKEND=gl` — o Vulkan da GPU integrada está quebrado).
//!
//! ```text
//! O habilitador — o registro de famílias           (motor, `src/fonts.rs`)
//!
//!   GlacierDaemon::font_named("DejaVu Serif", bytes)
//!     · embute os bytes no iced   (como `.font(bytes)`)
//!     · registra o NOME            (o que faltava)
//!     · entra na chave `__fonts`   (o `<fontselect>` lê de lá)
//!
//! Os widgets que saem dele
//!
//!   1. font="{familia}" no `.gv`      → resolve pelo registro (antes: silêncio)
//!   2. font_family no `.gss`          → o MESMO funil (`font_for`)
//!   3. <fontselect>                   → builtin, `<listview>` sobre `__fonts`
//!   4. <texteditor font="…">          → o PlainTextEditor da §2.2, agora ✅
//! ```
//!
//! # Por que o exemplo lê `.ttf` do disco
//!
//! Um app de verdade embute os bytes com `include_bytes!` e passa um
//! `&'static [u8]`. Aqui, para não versionar um `.ttf` no repositório, o
//! exemplo lê algumas fontes conhecidas do sistema e as **leaka** para
//! `&'static` — um vazamento de tamanho fixo, no boot, que é exatamente o que
//! o `Box::leak` do nome faz dentro de `src/fonts.rs`. Se um caminho não
//! existir nesta máquina, aquela família só não é registrada e a lista
//! `__fonts` fica mais curta; o exemplo roda igual.
use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Fontes;

impl Component for Fontes {
    fn name(&self) -> &str {
        "fontes"
    }

    fn template(&self) -> Template {
        Template::File("examples/fontes/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // A família escolhida no `<fontselect>` — a chave que o app nomeia, o
        // padrão do `<spinbox>` pela enésima vez. Sem seed, a lista abre sem
        // seleção (degradação aceitável).
        ctx.set("fonte", "DejaVu Serif".to_string());

        // A fonte do `<texteditor>` (o PlainTextEditor): declarável agora que
        // `font_for` consulta o registro.
        ctx.set("fonte_mono", "DejaVu Sans Mono".to_string());

        ctx.set(
            "nota",
            "fn main() {\n    // texto simples, em mono declarável —\n    // o 🟡 do PlainTextEditor era esta onda.\n    println!(\"Onda 10\");\n}"
                .to_string(),
        );

        ctx.set("status", "Pronto".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // O braço genérico de sempre: a ação É o nome da chave (o
        // `<fontselect>` despacha `pick:` e trata sozinho; o que chega aqui é
        // o `on_change` do `<texteditor>`).
        if let Some(v) = value {
            ctx.set(action, v.to_string());
        }
    }
}

/// Lê o primeiro caminho que existir e o promove a `&'static [u8]` (ver o doc
/// do módulo). `None` = nenhuma das variantes de caminho existe aqui.
fn carrega(caminhos: &[&str]) -> Option<&'static [u8]> {
    for caminho in caminhos {
        if let Ok(bytes) = std::fs::read(caminho) {
            return Some(Box::leak(bytes.into_boxed_slice()));
        }
    }
    None
}

fn main() -> iced::Result {
    let mut daemon = GlacierDaemon::new().title("Glacier — Onda 10 (fontes)");

    // Uma entrada por família. O NOME passado aqui é o que `font="…"` no `.gv`
    // e `font_family` no `.gss` passam a resolver — e o que aparece no
    // `<fontselect>`, desenhado nele mesmo.
    let familias: [(&str, &[&str]); 4] = [
        (
            "DejaVu Serif",
            &[
                "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",
                "/usr/share/fonts/dejavu/DejaVuSerif.ttf",
                "/usr/share/fonts/TTF/DejaVuSerif.ttf",
            ],
        ),
        (
            "DejaVu Sans Mono",
            &[
                "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
                "/usr/share/fonts/dejavu/DejaVuSansMono.ttf",
                "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
            ],
        ),
        (
            "Liberation Serif",
            &[
                "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
                "/usr/share/fonts/liberation/LiberationSerif-Regular.ttf",
                "/usr/share/fonts/liberation-serif/LiberationSerif-Regular.ttf",
            ],
        ),
        (
            "Liberation Mono",
            &[
                "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
                "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
                "/usr/share/fonts/liberation-mono/LiberationMono-Regular.ttf",
            ],
        ),
    ];

    for (nome, caminhos) in familias {
        match carrega(caminhos) {
            Some(bytes) => daemon = daemon.font_named(nome, bytes),
            None => eprintln!("fontes: '{nome}' não encontrada nesta máquina — pulando"),
        }
    }

    daemon
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Fontes)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("fontes");
        })
        .run()
}
