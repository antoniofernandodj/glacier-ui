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
//!   quadros, dirigida por uma chave nomeada.
//! ```
//!
//! # Fechar, restaurar, lembrar
//!
//! - **`✕`** esconde o painel (guarda o modo atual em `<mode>__prev`).
//! - **A aba `▸`** na borda o traz de volta — para onde ele estava, não só
//!   para a borda default.
//! - **`❒` / `▣`** flutua / reacopla.
//! - **`on_change="salvar_layout"`** dispara depois de cada mudança (botão ou
//!   arrasto). O handler grava o layout num arquivo; `init` o lê de volta —
//!   então o dock **lembra** onde estava entre execuções.
use std::path::PathBuf;

use glacier_ui::{Component, Context, GlacierDaemon, Template};

/// Onde o layout é persistido. `temp_dir` para o exemplo não sujar o repo.
fn caminho_layout() -> PathBuf {
    std::env::temp_dir().join("glacier-dock-exemplo.json")
}

/// Grava as quatro chaves de layout no arquivo. Livre (não method) para o
/// atalho da barra e o `on_change` do `<dock>` chamarem sem esbarrar num
/// borrow de `ctx`.
fn persistir(ctx: &Context) {
    let ler = |k: &str| ctx.get(k).cloned().unwrap_or_default();
    let layout = serde_json::json!({
        "lado": ler("lado"),
        "tam_painel": ler("tam_painel"),
        "painel_x": ler("painel_x"),
        "painel_y": ler("painel_y"),
    });
    let _ = std::fs::write(caminho_layout(), layout.to_string());
}

struct Dock;

impl Component for Dock {
    fn name(&self) -> &str {
        "dock"
    }

    fn template(&self) -> Template {
        Template::File("examples/dock/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // Defaults.
        ctx.set("lado", "left".to_string());
        ctx.set("tam_painel", "240 fill".to_string());
        ctx.set("painel_x", "80".to_string());
        ctx.set("painel_y", "80".to_string());

        // Lembrar: se há um layout salvo, ele vence os defaults.
        if let Ok(bruto) = std::fs::read_to_string(caminho_layout())
            && let Ok(salvo) = serde_json::from_str::<serde_json::Value>(&bruto)
        {
            for chave in ["lado", "tam_painel", "painel_x", "painel_y"] {
                if let Some(v) = salvo.get(chave).and_then(|v| v.as_str()) {
                    ctx.set(chave, v.to_string());
                }
            }
        }

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
            "// Arraste a faixa de título do painel para uma borda.\n// Solte: ele reancora ali.\n//\n// `✕` esconde; a aba `▸` traz de volta.\n// O layout é lembrado entre execuções (temp_dir/glacier-dock-exemplo.json).".to_string(),
        );
        let lado = ctx.get("lado").cloned().unwrap_or_default();
        ctx.set("status", format!("painel: {lado} (lembrado, se havia layout salvo)"));
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // `on_change` do `<dock>`: disparado depois de cada mudança de estado
        // (botão do cabeçalho ou arrasto de reancoragem).
        if action == "salvar_layout" {
            persistir(ctx);
            let lado = ctx.get("lado").cloned().unwrap_or_default();
            ctx.set("status", format!("layout salvo — painel: {lado}"));
            return;
        }
        // Atalhos da barra: `<button on_click="lado:left">`.
        if let Some((chave, valor)) = action.split_once(':') {
            ctx.set(chave, valor.to_string());
            if chave == "lado" {
                persistir(ctx);
                ctx.set("status", format!("layout salvo — painel: {valor}"));
            }
            return;
        }
        // O braço genérico: a ação É o nome da chave (o `<listview>`, o
        // `<texteditor>`).
        if let Some(v) = value {
            ctx.set(action, v.to_string());
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
