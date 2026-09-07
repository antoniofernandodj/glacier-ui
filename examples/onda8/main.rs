//! **Onda 8** do `PLANO_WIDGETS.md`: o diálogo que carrega markup — e os seis
//! widgets que saem dele.
//!
//! Rode com: `cargo run --example onda8`
//!
//! ```text
//! Habilitador — o modal ganha CORPO e RETORNO   (motor: dialogs.rs, lib.rs, luau/)
//!
//! 1. Dialog          — o `<dialog>` próprio, que é o QDialog     (motor + tag)
//! 2. InputDialog     — pede texto, número ou item de lista       (diálogo)
//! 3. ProgressDialog  — progresso cancelável                      (diálogo)
//! 4. ColorDialog     — roda/HSV/hex, sobre o canvas da Onda 7    (diálogo)
//! 5. StackView       — o QStackedWidget, formalizado             (builtin)
//! 6. Wizard          — passos com voltar/avançar/finalizar       (builtin+prim)
//! ```
//!
//! Este exemplo mostra o lado **Rust/declarativo** da onda: o `<dialog>` do
//! item 1 escrito no `.gv`, o wizard, o `<stackview>` e a roda de cor avulsa.
//! Os itens 2, 3 e 4 como *diálogos* são API da camada Luau — eles estão em
//! `cargo run --example onda8_luau`.
//!
//! # A frase que esta onda derruba
//!
//! O `DIALOGS.md` dizia, e estava certo quando foi escrito: *"um diálogo é
//! transiente e construído inteiramente em Rust, **sem markup**"*. Isso vale
//! enquanto todo diálogo é uma caixa de mensagem. Deixa de valer no instante em
//! que um diálogo precisa de um **campo** — o `QInputDialog::getText` é um
//! `QLineEdit` dentro de um cartão, e o motor já sabe desenhar `<textinput>`,
//! estilizá-lo pelo `.gss` e ligá-lo a uma chave.
//!
//! Escrever um segundo caminho de render, em Rust, para cada diálogo com
//! conteúdo seria escrever o motor duas vezes — o mesmo erro que a Onda 7
//! evitou ao recusar o `<canvas>` com callback imperativo.
//!
//! # A reclassificação, e por que esta é diferente das dez anteriores
//!
//! O catálogo marcava `InputDialog`, `ProgressDialog` e `ColorDialog` com `●`
//! ("exige estado por instância"). Nenhum exige — e aqui a marca é
//! *estruturalmente* impossível: o diálogo é **singleton** no motor, então
//! nunca existe uma segunda instância com que colidir. O que o usuário digita
//! mora numa chave (`__dialog.nome`), como em qualquer `<textinput>`.
//!
//! As dez reclassificações anteriores descobriram que o estado era o *valor*.
//! Esta descobre algo mais simples e mais fácil de checar antes de escrever
//! qualquer linha: **o widget não pode ter estado por instância porque não pode
//! existir duas vezes**.

use glacier_ui::{
    ButtonRole, Component, Context, DialogButton, DialogSpec, GlacierDaemon, Template,
};

struct Onda8;

impl Component for Onda8 {
    fn name(&self) -> &str {
        "onda8"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda8/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set(
            "abas",
            r#"[{"id":"modal","label":"O modal com corpo"},
                {"id":"wizard","label":"Wizard"},
                {"id":"cor","label":"Cor + StackView"}]"#
                .to_string(),
        );
        ctx.set("aba", "modal".to_string());

        // As opções do `<select>` que vive DENTRO do diálogo. Repare que ela é
        // uma chave comum do app: o corpo do modal é avaliado no contexto do
        // app, então ele enxerga tudo o que a tela enxerga.
        ctx.set(
            "ambientes",
            r#"[{"label":"Produção","value":"prod"},
                {"label":"Homologação","value":"homolog"},
                {"label":"Desenvolvimento","value":"dev"}]"#
                .to_string(),
        );

        ctx.set(
            "planos",
            r#"[{"id":"basico","label":"Básico — 1 vCPU"},
                {"id":"pro","label":"Pro — 4 vCPU"},
                {"id":"time","label":"Time — 16 vCPU"}]"#
                .to_string(),
        );
        ctx.set("plano", "pro".to_string());
        ctx.set("dominio", String::new());
        ctx.set("passo", "plano".to_string());

        ctx.set("painel", "resumo".to_string());
        // As duas chaves da cor nascem iguais: a cometida e o texto que o campo
        // mostra. A `<colorwheel>` reescreve as duas a cada gesto.
        ctx.set("cor", "#89B4FA".to_string());
        ctx.set("cor__hex", "#89B4FA".to_string());

        ctx.set("ultimo", "— nada ainda —".to_string());
        ctx.set("status", "Pronto".to_string());
        self.revalida(ctx);
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            // ── O botão Salvar do `<dialog name="editar_servico">` ───────
            //
            // Ele chega aqui como uma ação COMUM — a mesma rota de um
            // `<Button on_click>`. O motor já fechou o diálogo antes de
            // despachar, e as chaves `__dialog.*` ainda estão no contexto
            // neste instante: são apagadas depois, no fechamento.
            "salvar_servico" => {
                let nome = ctx.get("__dialog.nome").cloned().unwrap_or_default();
                let replicas = ctx.get("__dialog.replicas").cloned().unwrap_or_default();
                let ambiente = ctx.get("__dialog.ambiente").cloned().unwrap_or_default();
                let restart = ctx.get("__dialog.restart").cloned().unwrap_or_default();
                let nome = if nome.trim().is_empty() {
                    "(sem nome)".to_string()
                } else {
                    nome
                };
                ctx.set(
                    "ultimo",
                    format!(
                        "{nome} · {} réplica(s) · {} · restart={}",
                        if replicas.is_empty() { "1" } else { &replicas },
                        if ambiente.is_empty() {
                            "—"
                        } else {
                            &ambiente
                        },
                        if restart == "true" { "sim" } else { "não" },
                    ),
                );
                ctx.set("status", "Serviço salvo".to_string());
            }

            "remover_confirmado" => {
                ctx.set("ultimo", "— removido —".to_string());
                ctx.set("status", "Serviço removido".to_string());
            }

            // ── O mesmo diálogo, aberto do Rust ──────────────────────────
            //
            // `with_body` recebe o NOME de um template já registrado, não o
            // markup: o motor já sabe montar um template por nome
            // (`GlacierUI::render`), e passar o nome mantém o `DialogSpec`
            // barato de clonar e guardar.
            //
            // O nome aqui é o do `<dialog>` do `.gv` — a declaração registra o
            // corpo dela como um template comum, e é isso que faz os dois
            // caminhos (markup e Rust) chegarem ao mesmo lugar.
            "abrir_do_rust" => {
                ctx.set("__dialog.nome", "api-gateway".to_string());
                ctx.set("__dialog.replicas", "3".to_string());
                ctx.set("__dialog.ambiente", "prod".to_string());
                ctx.show_dialog(
                    DialogSpec::new(
                        glacier_ui::DialogIcon::None,
                        "Editar serviço (do Rust)",
                        "As chaves foram semeadas antes de abrir.",
                    )
                    .with_body("editar_servico")
                    .with_button(DialogButton::new(
                        "Cancelar",
                        glacier_ui::DIALOG_CLOSE,
                        ButtonRole::Neutral,
                    ))
                    .with_button(DialogButton::new(
                        "Salvar",
                        "salvar_servico",
                        ButtonRole::Accept,
                    )),
                );
            }

            // ── O wizard ─────────────────────────────────────────────────
            //
            // Repare no que NÃO está aqui: nenhum braço para "avançar" ou
            // "voltar". Quem anda entre os passos é a `<WizardNav>`, e ela
            // escreve a chave sozinha pelo `ContextPatch` — o mesmo caminho da
            // `<pagination>` da Onda 4.
            "finalizar" => {
                let plano = ctx.get("plano").cloned().unwrap_or_default();
                let dominio = ctx.get("dominio").cloned().unwrap_or_default();
                ctx.set("status", format!("Criado: {plano} em {dominio}"));
                ctx.set("passo", "plano".to_string());
            }
            "cancelar_wizard" => {
                ctx.set("passo", "plano".to_string());
                ctx.set("status", "Wizard cancelado".to_string());
            }

            // `painel:resumo` / `painel:detalhe` — a troca de página do
            // `<stackview>`. Duas linhas, porque trocar de página é escrever
            // uma chave: o widget não tem ação própria.
            a if a.starts_with("painel:") => {
                ctx.set("painel", a.trim_start_matches("painel:").to_string());
            }

            // O campo hexadecimal do seletor de cor, e ele mostra o padrão que
            // um app precisa conhecer: **um `<TextInput>` nunca grava a chave
            // sozinho**. Ele despacha `onChange` com o texto novo, e quem
            // escreve é quem trata a ação — sem isso o campo é decorativo.
            //
            // Aqui o texto vai para uma chave de rascunho (`cor__hex`, que é a
            // chave irmã que a própria `<colorwheel>` reescreve a cada gesto) e
            // a cor só é **cometida** quando o texto é uma cor inteira. É o que
            // impede a roda de piscar enquanto alguém digita `#ff8800`.
            "cor_digitada" => {
                let texto = value.unwrap_or_default();
                ctx.set("cor__hex", texto.to_string());
                if let Some(hex) = glacier_ui::color_picker::hex_completo(texto) {
                    ctx.set("cor", hex);
                }
            }

            // O braço genérico: a ação É o nome da chave. É por aqui que os
            // campos do diálogo e do wizard passam — cada um declara
            // `onChange="<nome da chave>"`, que é a convenção mais curta que
            // este motor tem para "grave o que eu digitei".
            chave if value.is_some() => {
                ctx.set(chave, value.unwrap_or_default().to_string());
            }
            _ => {}
        }
        self.revalida(ctx);
    }
}

impl Onda8 {
    /// O `QWizardPage::isComplete()` do passo do domínio.
    ///
    /// Mora no app, e é o ponto: o widget pergunta "esta página validou?" e o
    /// app responde. Um wizard que soubesse validar sozinho estaria adivinhando
    /// as regras de negócio de quem o usa.
    fn revalida(&self, ctx: &mut Context) {
        let passo = ctx.get("passo").cloned().unwrap_or_default();
        let valido = match passo.as_str() {
            "dominio" => !ctx
                .get("dominio")
                .map(|d| d.trim().is_empty())
                .unwrap_or(true),
            _ => true,
        };
        ctx.set("passo_valido", valido.to_string());
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 8")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda8)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda8");
        })
        .run()
}
