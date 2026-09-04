//! **Onda 6** do `PLANO_WIDGETS.md`: a grade — uma medição, seis widgets.
//!
//! Rode com: `cargo run --example onda6`
//!
//! Esta é a onda cara, e ela é cara **uma vez só**. O documento catalogava dois
//! itens separados como caros — o `Grid` ("o `iced` não tem grade") e o
//! `TableView` ("**grande**: cabeçalho, seleção, sort, edição") — sem notar que
//! a parte difícil dos dois é **a mesma**: descobrir a largura de uma coluna a
//! partir de todas as células que passam por ela, antes de desenhar qualquer
//! uma.
//!
//! ```text
//! Habilitador — medição de colunas   (motor, `src/grid.rs`)
//!
//! 1. Grid        — a grade, e o teste do mecanismo       (primitiva)
//! 2. Flow/Wrap   — a fileira que quebra linha            (primitiva)
//! 3. TableHeader — ordenar e redimensionar               (primitiva)
//! 4. TableView   — cabeçalho + corpo, ordem e seleção    (primitiva)
//! 5. TreeView    — recursão + conjunto nomeado de nós    (primitiva)
//! 6. ColumnView  — navegação Miller (o Finder)           (primitiva)
//! ```
//!
//! # As duas correções de rota desta onda
//!
//! **O `Flow` já existia**, e não no motor: `Row::wrap()` do próprio `iced`
//! quebra linha exatamente como o plano descrevia, com alinhamento por linha e
//! vão vertical próprio. `<flow>` são três linhas em `widget.rs` e não passa
//! pela medição — a sexta vez que este projeto descobre que um item catalogado
//! como caro já estava pronto, e a primeira em que quem já tinha era a
//! biblioteca de baixo.
//!
//! **A ligação a coleção também já existia.** O §3 lista "binding a coleção
//! (model/view)" como o habilitador P2 "caro, e o maior investimento restante".
//! `items="chave"` — um array JSON numa chave de contexto — é o que o
//! `<menu items>` e o `<tabbar items>` usam desde sempre. O que faltava era a
//! medição, e convencionar seleção e ordenação; as duas últimas são o padrão do
//! `SpinBox`, que já estava escrito.
//!
//! # E a décima linha marcada como bloqueada sem estar
//!
//! O `TreeView`. O §3 o punha atrás do "estado por instância" — o item que este
//! documento chamou por três revisões de "o desbloqueio de maior alavancagem".
//! O conjunto de nós abertos é um **conjunto nomeado** (`"raiz,raiz/src"`),
//! exatamente como as seções de um `<accordion>`, e o `contains` que o
//! destrancou saiu na 0.84.
//!
//! # O que o app escreve, e o que ele não escreve
//!
//! O `update` abaixo trata **duas** ações, e nenhuma delas ordena, seleciona,
//! abre um nó ou redimensiona uma coluna: tudo isso o widget faz escrevendo nas
//! chaves que este arquivo nomeou. O que ele faz é o que nenhum widget podia
//! saber — o que "limpar" e "fechar tudo" significam nesta tela.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Onda6;

/// As linhas da tabela. Em app de verdade viriam de um backend; aqui são
/// literais para o exemplo não depender de rede.
///
/// A coluna `replicas` é a que prova a ordenação numérica: ordenada como texto,
/// `"10"` viria antes de `"9"`.
const SERVICOS: &[(&str, &str, &str, &str, &str)] = &[
    ("api", "API pública", "produção", "12", "41"),
    ("db", "Banco primário", "produção", "3", "78"),
    ("cache", "Redis", "produção", "9", "23"),
    ("fila", "Fila de eventos", "produção", "4", "55"),
    ("cdn", "CDN", "borda", "24", "12"),
    ("auth", "Autenticação", "produção", "6", "62"),
    ("mail", "Disparo de e-mail", "homologação", "2", "8"),
    ("logs", "Coletor de logs", "produção", "10", "91"),
    ("cron", "Agendador", "homologação", "1", "5"),
    ("busca", "Índice de busca", "produção", "8", "70"),
];

impl Component for Onda6 {
    fn name(&self) -> &str {
        "onda6"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda6/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set(
            "abas",
            r#"[{"id":"layout","label":"Grid + Flow"},
                {"id":"tabela","label":"TableView + TableHeader"},
                {"id":"arvore","label":"TreeView + ColumnView"}]"#
                .to_string(),
        );
        ctx.set("aba", "layout".to_string());

        // As colunas da tabela: a mesma convenção `items="chave"` que o
        // `<menu>` e o `<tabbar>` já usavam — um array JSON numa chave. O que a
        // Onda 6 acrescentou não foi a ligação, foi a MEDIÇÃO.
        //
        // `align: "right"` numa coluna de número não é enfeite: é o que faz os
        // algarismos alinharem pela unidade, que é como se compara uma coluna
        // de contagem a olho.
        ctx.set(
            "colunas",
            r#"[{"key":"nome",     "label":"Serviço",  "width":"fill"},
                {"key":"ambiente", "label":"Ambiente", "width":"140"},
                {"key":"replicas", "label":"Réplicas", "width":"90",  "align":"right"},
                {"key":"uso",      "label":"Uso %",    "width":"90",  "align":"right"}]"#
                .to_string(),
        );
        ctx.set("servicos", linhas_json());

        // As chaves que a tabela escreve sozinha. Todas nomeadas pelo app —
        // é o padrão do `SpinBox`, e é o que permite duas tabelas na mesma tela.
        ctx.set("servico", "api".to_string());
        ctx.set("marcados", "api,cache".to_string());
        ctx.set("ordem", "nome asc".to_string());
        // As larguras arrastadas. Semeada com o mesmo formato de `columns`, o
        // que permite ao app salvar e restaurar o layout do usuário sem
        // conhecer nada da estrutura interna do widget.
        ctx.set("larguras", "fill 140 90 90".to_string());

        // A árvore: uma coleção aninhada, e o conjunto de nós abertos numa
        // string. `no` e `caminho` guardam um CAMINHO (`raiz/src/main.rs`), que
        // é a identidade de um nó — um `id` repetido em ramos diferentes não
        // colide.
        ctx.set("arvore", arvore_json());
        ctx.set("abertos", "projeto,projeto/src".to_string());
        ctx.set("no", "projeto/src".to_string());
        ctx.set("caminho", "projeto/src".to_string());

        ctx.set("status", "Pronto".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            "limpar_marcados" => {
                ctx.set("marcados", String::new());
                ctx.set("status", "Seleção múltipla limpa".to_string());
            }
            // Fechar tudo é zerar o conjunto — e é isso que torna o conjunto
            // nomeado tão prático: uma string vazia é "nenhum nó aberto", sem
            // um bit por nó para percorrer.
            "fechar_tudo" => {
                ctx.set("abertos", String::new());
                ctx.set("status", "Árvore recolhida".to_string());
            }
            chave if value.is_some() => {
                ctx.set(chave, value.unwrap_or_default().to_string());
            }
            _ => {}
        }
    }
}

/// As linhas em JSON, no formato que o `<tableview>` lê: um objeto por linha,
/// com uma chave por coluna (mais o `id`, que é a identidade da seleção).
fn linhas_json() -> String {
    let linhas: Vec<serde_json::Value> = SERVICOS
        .iter()
        .map(|(id, nome, ambiente, replicas, uso)| {
            serde_json::json!({
                "id": id,
                "nome": nome,
                "ambiente": ambiente,
                "replicas": replicas,
                "uso": uso,
            })
        })
        .collect();
    serde_json::Value::Array(linhas).to_string()
}

/// A árvore em JSON: `{id, label, items}` aninhado, a mesma forma que o
/// `<menu items>` já usava para submenus.
fn arvore_json() -> String {
    serde_json::json!([
        {
            "id": "projeto", "label": "projeto",
            "items": [
                {
                    "id": "src", "label": "src",
                    "items": [
                        { "id": "main.rs",   "label": "main.rs" },
                        { "id": "widget.rs", "label": "widget.rs" },
                        { "id": "grid.rs",   "label": "grid.rs" },
                        {
                            "id": "builtins", "label": "builtins",
                            "items": [
                                { "id": "tabs.rs",   "label": "tabs.rs" },
                                { "id": "drawer.rs", "label": "drawer.rs" }
                            ]
                        }
                    ]
                },
                {
                    "id": "examples", "label": "examples",
                    "items": [
                        { "id": "onda5", "label": "onda5" },
                        { "id": "onda6", "label": "onda6" }
                    ]
                },
                { "id": "Cargo.toml", "label": "Cargo.toml" },
                { "id": "README.md",  "label": "README.md" }
            ]
        }
    ])
    .to_string()
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 6")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda6)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda6");
        })
        .run()
}
