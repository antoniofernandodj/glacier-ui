//! **Onda 12** do `PLANO_WIDGETS.md`: o `<dock>` — o painel acoplável que a
//! Onda 11 tinha cortado.
//!
//! Rode com: `cargo run --example dock` (nesta máquina, com `WGPU_BACKEND=gl`).
//!
//! ```text
//! O habilitador D — grip::Alvo::Zona
//!
//!   O cabeçalho do painel dispara um arrasto que NÃO escreve nada durante o
//!   gesto e comete a borda na SOLTURA (Arrasto::modo_no_release, via DragEnd).
//!   O render_dock seguinte lê a chave `mode` e monta o pai certo —
//!   <splitter> acoplado, <stack> flutuante. A troca de pai acontece ENTRE
//!   quadros, dirigida por uma chave nomeada, e é isso que a Onda 11 não
//!   tinha feito.
//! ```
//!
//! # As duas formas de mover o painel
//!
//! - **Botões do cabeçalho:** `❒` flutua / `▣` reacopla, `—` esconde.
//! - **Arrastar o cabeçalho** para uma borda reancora ali na soltura; um
//!   movimento pequeno é tratado como clique e não muda nada.
//!
//! O `update` abaixo trata **uma** ação genérica: a chave `mode` é escrita
//! pelo binding legado (`ctx[ação] = valor`), como todo widget de chave nomeada
//! deste motor.
use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Dock;

impl Component for Dock {
    fn name(&self) -> &str {
        "dock"
    }

    fn template(&self) -> Template {
        Template::File("examples/dock/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // Onde o painel está. `left`/`right`/`top`/`bottom`/`float`/`hidden`.
        ctx.set("lado", "left".to_string());
        // A trilha do <splitter> quando acoplado — `<medida do painel> fill`, no
        // eixo do dock (largura à esquerda/direita, altura acima/abaixo).
        ctx.set("tam_painel", "240 fill".to_string());
        // Posição do painel quando flutuante.
        ctx.set("painel_x", "80".to_string());
        ctx.set("painel_y", "80".to_string());

        ctx.set(
            "arquivos",
            r#"[{"id":"main.rs","label":"main.rs"},
                {"id":"app.gv","label":"app.gv"},
                {"id":"app.gss","label":"app.gss"}]"#
                .to_string(),
        );
        ctx.set("arquivo", "main.rs".to_string());
        ctx.set(
            "doc",
            "// Arraste a faixa de título do painel para uma borda.\n// Solte: ele reancora ali.\n//\n// Um movimento pequeno conta como clique — nada muda.".to_string(),
        );
        ctx.set("status", "painel à esquerda".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // Os atalhos da barra são `<button on_click="lado:left">` — payload no
        // formato `chave:valor`, a convenção `nome:sufixo` do dispatcher.
        if let Some((chave, valor)) = action.split_once(':') {
            ctx.set(chave, valor.to_string());
            if chave == "lado" {
                ctx.set("status", format!("painel: {valor}"));
            }
            return;
        }
        // O braço genérico: a ação É o nome da chave. Cobre o `mode` que os
        // botões e o arrasto do `<dock>` escrevem (binding legado), mais o
        // `<listview>` e o `<texteditor>`.
        if let Some(v) = value {
            ctx.set(action, v.to_string());
            if action == "lado" {
                ctx.set("status", format!("painel: {v}"));
            }
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier — Onda 12 (dock)")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Dock)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("dock");
        })
        .run()
}
