//! **Onda 4** do `PLANO_WIDGETS.md`: os widgets que têm **função**.
//!
//! Rode com: `cargo run --example onda4`
//!
//! A onda anterior entregou uma primitiva; esta entrega sete linhas do
//! catálogo, e o que as une não é a aparência — é o `update`. Todas fazem
//! **conta**: a janela de números de uma paginação, a alternância de um item
//! num conjunto, a máscara de um CPF, o clamp de um preço com duas casas.
//!
//! ```text
//! 1. Pagination   — a aritmética de página          (primitiva)
//! 2. ListView     — seleção simples e múltipla      (builtin)
//! 3. Accordion    — várias seções abertas           (builtin, usa `contains`)
//!    ToolBox      — uma seção por vez               (builtin)
//! 4. ButtonBox    — papéis + ordem da plataforma    (builtin)
//! 5. MaskedInput  — cru na chave, mascarado na tela (primitiva)
//! 6. Rating       — estrelas com prévia no hover    (primitiva)
//! 7. decimals     — o QDoubleSpinBox fechado        (prop do SpinBox)
//! ```
//!
//! # As duas reclassificações desta onda
//!
//! O plano marcava `Pagination` e `Rating` como **builtins**, e os dois
//! viraram **primitivas** — pela mesma causa, que é o achado desta leva:
//! ambos são *repetição dirigida por um número*, não por uma coleção. O
//! `for-each` do motor lê uma chave com um array; a janela `4 5 6` e as cinco
//! estrelas não existem em array nenhum — são **derivadas** da página atual e
//! do `max`, e derivar é justamente o que um template não faz.
//!
//! É a quinta e a sexta aplicação da mesma lição (`TimePicker`, `DateEdit`,
//! `Calendar`, `Accordion`, e agora estas duas), e a primeira vez que a causa
//! aparece duas vezes seguidas — o que sugere que ela merece um nome.
//!
//! # O que o app escreve, e o que ele não escreve
//!
//! Repare no `update` abaixo: ele trata **cinco** ações, e nenhuma delas é de
//! um widget. Paginação, seleção, seções, estrelas e máscara gravam as chaves
//! sozinhos — o app só nomeia as chaves e reage ao que quer.

use glacier_ui::{Component, Context, GlacierDaemon, Template};
use serde_json::Value::Array as SerdeArray;
struct Onda4;

/// A lista inteira, da qual a paginação recorta uma página. Em app de verdade
/// isso viria de um backend; aqui é literal para o exemplo não depender de
/// rede.
const SERVICOS: &[(&str, &str)] = &[
    ("api", "API pública"),
    ("db", "Banco primário"),
    ("cache", "Redis"),
    ("fila", "Fila de eventos"),
    ("cdn", "CDN"),
    ("auth", "Autenticação"),
    ("mail", "Disparo de e-mail"),
    ("logs", "Coletor de logs"),
    ("cron", "Agendador"),
    ("busca", "Índice de busca"),
    ("relat", "Relatórios"),
    ("backup", "Backup noturno"),
];

/// Quantos itens cabem numa página. O `<pagination>` não sabe disto — ele
/// conta **páginas**, não itens; recortar a lista é do app, porque só o app
/// sabe o que é um item.
const POR_PAGINA: usize = 4;

impl Onda4 {
    /// Recorta a página visível e a grava na chave que o `<listview>` lê.
    ///
    /// Esta é a divisão de trabalho da paginação: o widget escreve o número da
    /// página, o app decide o que aquele número significa.
    fn recortar(&self, ctx: &mut Context) {
        let pagina = ctx
            .get("pagina")
            .and_then(|p| p.trim().parse::<usize>().ok())
            .unwrap_or(1)
            .max(1);
        let de = (pagina - 1) * POR_PAGINA;
        let itens: Vec<serde_json::Value> = SERVICOS
            .iter()
            .skip(de)
            .take(POR_PAGINA)
            .map(|(id, label)| {
                serde_json::json!({ "id": id, "label": label, "sub": format!("id: {id}") })
            })
            .collect();
        ctx.set("servicos", SerdeArray(itens).to_string());
        ctx.set(
            "faixa",
            format!(
                "{}–{} de {}",
                de + 1,
                (de + POR_PAGINA).min(SERVICOS.len()),
                SERVICOS.len()
            ),
        );
    }
}

impl Component for Onda4 {
    fn name(&self) -> &str {
        "onda4"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda4/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set(
            "abas",
            r#"[{"id":"listas","label":"Pagination + ListView"},
                {"id":"secoes","label":"Accordion + ToolBox"},
                {"id":"campos","label":"MaskedInput + Rating"}]"#
                .to_string(),
        );
        ctx.set("aba", "listas".to_string());

        // Paginação: o total de PÁGINAS, que é o que o widget conta.
        ctx.set("pagina", "1".to_string());
        ctx.set(
            "total_paginas",
            SERVICOS.len().div_ceil(POR_PAGINA).to_string(),
        );

        // Seleção: uma chave para o modo simples, outra para o conjunto.
        ctx.set("servico", "api".to_string());
        ctx.set("marcados", "api,cache".to_string());

        // As seções abertas — o conjunto nomeado que o `contains` (0.84)
        // destrancou. Uma string, não um bit por seção.
        ctx.set("abertas", "rede".to_string());
        ctx.set("ferramenta", "medidas".to_string());

        // As três chaves que o conteúdo das seções lê. Sem semeá-las, o
        // `<checkbox>` e o `<toggle>` nascem desmarcados e — o que é pior —
        // NÃO alternam: o clique escreve `true`, mas a chave que o widget lê
        // continua ausente no primeiro quadro. E a `<progressbar>` sem chave
        // desenha a barra vazia, que parece um bug de render e não é.
        ctx.set("usar_proxy", "false".to_string());
        ctx.set("avisar", "true".to_string());
        ctx.set("uso_disco", "41".to_string());

        // Campos.
        ctx.set("cpf", String::new());
        ctx.set("telefone", String::new());
        ctx.set("placa", String::new());
        ctx.set("nota", "4".to_string());
        // Chave própria: o rating de 10 estrelas tem outra escala, e uma nota
        // de 7 numa régua de 5 não quer dizer nada.
        ctx.set("nota_ampla", "7".to_string());
        ctx.set("preco", "19.90".to_string());
        ctx.set("desconto", "0".to_string());

        ctx.set("status", "Pronto".to_string());
        self.recortar(ctx);
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            // A paginação de cima tem `on_change`, então ela **delega**: quem
            // grava `pagina` é este braço. É o mesmo contrato do `<textinput>`
            // e do `<calendar>`, e aqui ele paga o aluguel — o app precisa
            // recortar a lista no mesmo passo em que a página muda, e não há
            // gancho melhor. A paginação de baixo, sem `on_change`, grava
            // sozinha e não passa por aqui.
            "repaginar" => {
                if let Some(p) = value {
                    ctx.set("pagina", p.to_string());
                }
                self.recortar(ctx);
                let p = ctx.get("pagina").cloned().unwrap_or_default();
                ctx.set("status", format!("Página {p}"));
            }
            "salvar" => ctx.set("status", "Cadastro salvo".to_string()),
            "cancelar" => ctx.set("status", "Cancelado".to_string()),
            "excluir" => ctx.set(
                "status",
                "Excluído — e o botão ficou longe dos outros".to_string(),
            ),
            "limpar_marcados" => {
                ctx.set("marcados", String::new());
                ctx.set("status", "Seleção múltipla limpa".to_string());
            }
            chave if value.is_some() => {
                ctx.set(chave, value.unwrap_or_default().to_string());
            }
            _ => {}
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 4")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda4)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda4");
        })
        .run()
}
