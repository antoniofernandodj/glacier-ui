//! **Onda 2** do `PLANO_WIDGETS.md`: os seis widgets que o `<slot/>` destrancou
//! — `GroupBox`, `Frame`, `Card`, `ToolButton`, `ToolBar`/`StatusBar` e `TabBar`.
//!
//! Rode com: `cargo run --example onda2`
//!
//! Nenhum deles é registrado aqui: a lib registra todos sozinha em
//! `GlacierUI::new()` (ver `src/builtins/mod.rs`), então as tags funcionam como
//! primitivas. O que o app faz é o de sempre — registrar a tela, semear as
//! chaves iniciais e tratar as ações que ele mesmo escreveu no template.
//!
//! # O que este exemplo demonstra de novo
//!
//! O `<slot/>`: até a 0.64 os filhos escritos dentro de uma tag de componente
//! eram **descartados** na expansão, e por isso nenhum widget-recipiente podia
//! ser builtin. Repare no `on_click="salvar"` de dentro do `<GroupBox>` no
//! template: ele chega no `update` **desta tela**, não no do `GroupBox`. O
//! conteúdo do slot é avaliado no contexto e com o dono de quem o escreveu.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Onda2;

impl Component for Onda2 {
    fn name(&self) -> &str {
        "onda2"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda2/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // As abas do `<TabBar>` vêm de uma coleção do contexto, como as de um
        // `<Menu items="…">` — o `for-each` do motor lê chave, não texto solto
        // no atributo.
        ctx.set(
            "abas",
            r#"[{"id":"grupos","label":"GroupBox"},
                {"id":"molduras","label":"Frame"},
                {"id":"cartoes","label":"Card"}]"#
                .to_string(),
        );
        // A chave da aba ativa. Quem escreve nela dali em diante é o próprio
        // `<TabBar>` (padrão SpinBox: a chave vem por prop, a ação a carrega) —
        // o app só dá o valor inicial.
        ctx.set("aba", "grupos".to_string());
        ctx.set("status", "Pronto".to_string());

        // Chaves dos controles que moram dentro dos recipientes. Um `<SpinBox>`
        // cuja chave nunca foi escrita nasce em branco.
        ctx.set("usar_proxy", "true".to_string());
        ctx.set("host", "127.0.0.1".to_string());
        ctx.set("porta", "8080".to_string());
        ctx.set("timeout", "30".to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // Só as ações que ESTA tela escreveu. As dos widgets não passam por
        // aqui: o `<TabBar>` trata o clique de aba no `update` dele, e o
        // `<GroupBox>`/`<ToolBar>` não tratam nada (são recipientes puros).
        let recado = match action {
            "novo" => "Novo documento",
            "salvar" => "Configuração salva",
            "excluir" => "Item excluído",
            "salvar_rede" => "Rede salva — o clique veio de dentro do GroupBox",
            _ => return,
        };
        ctx.set("status", recado.to_string());
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 2")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda2)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda2");
        })
        .run()
}
