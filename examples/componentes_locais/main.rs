//! **Componentes declarados na própria tela** — a terceira forma de ter um
//! componente no glacier-ui.
//!
//! Rode com: `cargo run --example componentes_locais`
//!
//! As outras duas trazem de um arquivo:
//!
//! ```xml
//! <import name="CartaoServico" from="cartao.gv" />
//! <link rel="component" href="cartao.gv" as="CartaoServico" />
//! ```
//!
//! Esta não traz de lugar nenhum — o componente **é** o que está escrito entre
//! as tags, dentro do `<resources>`:
//!
//! ```xml
//! <resources>
//!     <component name="CartaoServico">
//!         <props><prop name="nome" /></props>
//!         <card title="{nome}"> … </card>
//!     </component>
//! </resources>
//! ```
//!
//! # Por que ela existe
//!
//! Porque a maior parte dos componentes de uma tela é pequena e **só serve
//! àquela tela**: a linha de um item, o cabeçalho de um cartão, um rótulo com
//! um `<badge>` do lado. Obrigar cada um desses a virar arquivo troca três
//! linhas de markup por um arquivo, um caminho relativo e um `<import>` — e
//! espalha por seis arquivos o que se lê melhor num.
//!
//! É a mesma razão de existir um `<style>` inline ao lado do
//! `<link rel="stylesheet">`, e a mesma escolha: a forma curta para o que é
//! local, o arquivo para o que é compartilhado.
//!
//! # A casca é a mesma, de propósito
//!
//! O que se escreve entre `<component name="X">` e `</component>` é **byte a
//! byte** o que se escreveria num `.gv` próprio: `<component>`, o `<props>`
//! dentro dele, o layout depois. A única diferença é o `name` — no arquivo o
//! nome vem do registro, aqui ele precisa ser dito.
//!
//! Promover uma declaração a arquivo é, literalmente, recortar e colar. Este
//! exemplo prova isso: a tag `<LinhaLog/>` está declarada aqui na tela, e a
//! `<CartaoServico/>` veio de `cartao_servico.gv` — os dois arquivos têm a
//! mesma forma, e trocar um pelo outro não pede uma linha de reescrita.
//!
//! # O que o app precisa saber
//!
//! Nada. Repare que este `main.rs` não menciona `LinhaLog` nem `Metrica`: o
//! motor lê as declarações do `<resources>` na hora de registrar a tela, do
//! mesmo jeito que já lia os `<import>`. O `update` daqui trata só as ações que
//! **a tela** escreveu — inclusive as que estão dentro dos componentes locais,
//! porque uma declaração inline não tem arquivo, logo não tem `<script>`, logo
//! não tem dono próprio.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct ComponentesLocais;

impl Component for ComponentesLocais {
    fn name(&self) -> &str {
        "componentes_locais"
    }

    fn template(&self) -> Template {
        Template::File("examples/componentes_locais/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        // Os dados que os componentes locais consomem. Nenhuma chave aqui é do
        // widget: são do app, e é o `for-each` que as reparte item a item.
        ctx.set(
            "servicos",
            r##"[{"id":"api","nome":"API pública","estado":"online","cor":"#A6E3A1","uptime":"31 dias"},
                {"id":"db","nome":"Banco primário","estado":"lento","cor":"#F9E2AF","uptime":"12 dias"},
                {"id":"fila","nome":"Fila de eventos","estado":"parado","cor":"#F38BA8","uptime":"—"}]"##
                .to_string(),
        );
        ctx.set(
            "log",
            r#"[{"hora":"09:14","nivel":"info","texto":"deploy concluído"},
                {"hora":"09:31","nivel":"aviso","texto":"latência acima de 400 ms"},
                {"hora":"09:47","nivel":"erro","texto":"fila sem consumidor"},
                {"hora":"10:02","nivel":"info","texto":"réplica sincronizada"}]"#
                .to_string(),
        );
        ctx.set(
            "metricas",
            r#"[{"rotulo":"Requisições/min","valor":"1.284","delta":"+12%"},
                {"rotulo":"Erros 5xx","valor":"3","delta":"-40%"},
                {"rotulo":"p95","valor":"310 ms","delta":"+8%"}]"#
                .to_string(),
        );
        ctx.set("selecionado", "api".to_string());
        ctx.set("status", "Pronto".to_string());
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // As ações escritas DENTRO dos componentes locais chegam aqui, sem
        // prefixo de dono — é o que faz uma declaração inline ser útil para as
        // peças de uma tela só.
        //
        // O id viaja DENTRO da ação (`detalhar:api`), o padrão do `SpinBox`: um
        // clique não carrega valor, e é assim que o componente diz de qual item
        // ele é. Aqui a prop `{s.id}` do `for-each` monta a string.
        let (nome, alvo) = match action.split_once(':') {
            Some((n, a)) => (n, a),
            None => (action, ""),
        };
        match nome {
            "reiniciar" => ctx.set("status", format!("Reiniciando {alvo}…")),
            "detalhar" => {
                ctx.set("selecionado", alvo.to_string());
                ctx.set("status", format!("Detalhe de {alvo}"));
            }
            _ => {}
        }
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Componentes locais")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(ComponentesLocais)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("componentes_locais");
        })
        .run()
}
