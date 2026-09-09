//! **Onda 11** do `PLANO_WIDGETS.md`: o que fica por cima (`<stack>`/`pin`) —
//! e o `<mdiarea>` que sai dela de graça.
//!
//! Rode com: `cargo run --example onda11` (nesta máquina, com
//! `WGPU_BACKEND=gl` — o Vulkan da GPU integrada está quebrado).
//!
//! ```text
//! Habilitador A — o <stack> como capacidade         (motor, `NodeType::Stack`)
//!
//! 1. NotificationDot — pontinho ancorado num canto   (builtin)
//! 2. SplashScreen     — cobre e some                 (builtin)
//!
//! Habilitador B — grip::Alvo::Ponto, o arrasto em 2D (motor, `src/grip.rs`)
//!
//! 3. MdiArea/MdiSubWindow — janelas internas          (primitiva)
//!
//! O troco da §6.3, esvaziado nesta onda
//!
//! 4. QrCode      — o `qr_code` nativo do iced         (primitiva)
//! 5. Chip        — Badge com um "×"                   (builtin)
//! 6. Skeleton    — placeholder de carregamento        (builtin)
//! 7. CommandLink — título + descrição + seta          (builtin)
//! 8. RoundButton — Button com border_radius total     (builtin)
//! ```
//!
//! # A descoberta da onda: `<stack>`/`pin` já existiam no iced
//!
//! Quatro linhas ⬜ do catálogo já escreviam o nome do que faltava —
//! `MdiArea`/`SplashScreen` diziam "stack" na coluna "Base iced",
//! `NotificationDot` dizia "pede Stack dentro do builtin" na nota, e o
//! `RubberBand` da Onda 9 registrou por escrito que "o motor não tem
//! `<stack>` no markup". O `widget.rs` até usava `stack!` uma vez, hardcoded,
//! para centralizar o percentual sobre o `<progressbar>`. Nunca virou tag.
//!
//! # A 17ª correção de nível
//!
//! `MdiArea` estava marcado `Comp ●` — "exige estado por instância". Não
//! exige: sub-janelas são um valor que o app nomeia, quatro chaves por janela
//! (`x`/`y`/`w`/`h`), a mesma forma que o `<rangeslider>` usa para duas. Mover
//! e redimensionar são o MESMO `grip::Alvo::Ponto`, só com limites diferentes.
use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Onda11;

impl Component for Onda11 {
    fn name(&self) -> &str {
        "onda11"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda11/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set(
            "abas",
            r#"[{"id":"empilhamento","label":"Stack + NotificationDot + Splash"},
                {"id":"mdi","label":"MdiArea"},
                {"id":"troco","label":"QrCode + Chip + Skeleton + CommandLink"}]"#
                .to_string(),
        );
        ctx.set("aba", "empilhamento".to_string());

        // ── 1. NotificationDot ──────────────────────────────────────────
        ctx.set("tem_notificacao", "true".to_string());

        // ── 2. SplashScreen ─────────────────────────────────────────────
        ctx.set("carregando", "true".to_string());

        // ── 3. MdiArea ──────────────────────────────────────────────────
        //
        // Cada janela nomeia QUATRO chaves. Sem seed nenhum elas cairiam no
        // cascade (20 + índice×24) — aqui seedamos duas para mostrar as duas
        // formas, e deixamos a terceira sem seed de propósito.
        ctx.set("editor_x", "40".to_string());
        ctx.set("editor_y", "40".to_string());
        ctx.set("editor_w", "360".to_string());
        ctx.set("editor_h", "220".to_string());
        ctx.set("console_x", "260".to_string());
        ctx.set("console_y", "160".to_string());
        // console_w/console_h sem seed: nasce no default_w/default_h do markup.

        // ── 5. Chip ─────────────────────────────────────────────────────
        ctx.set(
            "tags",
            r#"["produção","staging","us-east-1"]"#.to_string(),
        );

        // ── 8. QrCode ───────────────────────────────────────────────────
        ctx.set("qr_texto", "https://glacier-ui.dev".to_string());

        ctx.set("status", "Pronto".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            // O "×" de um chip: remove o item da lista pelo nome.
            cmd if cmd.starts_with("remover_tag:") => {
                let alvo = cmd.trim_start_matches("remover_tag:");
                let tags: Vec<String> = ctx
                    .get("tags")
                    .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|t| t != alvo)
                    .collect();
                ctx.set("tags", serde_json::to_string(&tags).unwrap_or_default());
                ctx.set("status", format!("Removida: {alvo}"));
            }
            "fechar_splash" => {
                ctx.set("carregando", String::new());
                ctx.set("status", "Splash fechada — o conteúdo estava por baixo o tempo todo".to_string());
            }
            "abrir_splash" => {
                ctx.set("carregando", "true".to_string());
            }
            "alternar_notificacao" => {
                let ligado = ctx
                    .get("tem_notificacao")
                    .is_some_and(|v| !v.trim().is_empty() && v.trim() != "false");
                ctx.set(
                    "tem_notificacao",
                    if ligado { String::new() } else { "true".to_string() },
                );
            }
            "instalar_tipica" => ctx.set("status", "CommandLink: instalação típica escolhida".to_string()),
            "instalar_custom" => ctx.set("status", "CommandLink: instalação customizada escolhida".to_string()),
            "novo_item" => ctx.set("status", "RoundButton: novo item".to_string()),
            // O braço genérico de sempre: a ação É o nome da chave (o
            // `<textinput on_change="qr_texto">` do QrCode, o `<tabbar>`).
            chave if value.is_some() => {
                let v = value.unwrap_or_default();
                ctx.set(chave, v.to_string());
            }
            _ => {}
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 11")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda11)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda11");
        })
        .run()
}
