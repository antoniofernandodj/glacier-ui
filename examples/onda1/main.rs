//! **Onda 1** do `PLANO_WIDGETS.md` §6: os quatro widgets que o `iced` já
//! sustentava e que só faltava expor — `Slider`, `Space`, `Radio`/`RadioGroup`
//! e `Avatar`.
//!
//! Rode com: `cargo run --example onda1`
//!
//! # As duas metades desta tela
//!
//! Ela existe para deixar visível a diferença entre **primitiva** e **builtin**,
//! que aqui aparece lado a lado com o mesmo dado:
//!
//! - `<slider>` e `<radio>` são **primitivas**. Como o `<textinput>` e o
//!   `<checkbox>`, elas **não escrevem** a chave de contexto sozinhas: disparam
//!   a ação com o valor novo, e quem grava é o `update` abaixo. É por isso que
//!   esta tela tem handlers — sem eles, arrastar o cursor não moveria nada.
//! - `<radiogroup>` é um **builtin** sobre `<radio>`. Ele tem `update` próprio,
//!   grava a chave sozinho, e por isso não aparece handler nenhum aqui para ele.
//!
//! O grupo "Plano" mostra os dois caminhos escrevendo na **mesma** chave, e o
//! grupo "Zoom" mostra um `<slider>` e um `<spinbox>` (da onda anterior) também
//! na mesma chave — o par que o Qt usa o tempo todo.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Onda1;

impl Component for Onda1 {
    fn name(&self) -> &str {
        "onda1"
    }

    fn template(&self) -> Template {
        Template::File("examples/onda1/app.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        ctx.set("volume", "42".to_string());
        ctx.set("brilho", "0.60".to_string());
        ctx.set("graves", "3".to_string());
        // A mesma chave que o `<spinbox>` da onda 2 edita, para os dois
        // andarem juntos.
        ctx.set("zoom", "100".to_string());

        // As opções do `<radiogroup>` vêm de uma coleção do contexto, como as
        // abas do `<tabbar>`: o `for-each` do motor lê chave, não texto solto.
        ctx.set(
            "planos",
            r#"[{"id":"free","label":"Grátis"},
                {"id":"pro","label":"Pro"},
                {"id":"team","label":"Time"}]"#
                .to_string(),
        );
        ctx.set("plano", "pro".to_string());
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // O segundo parâmetro é o valor novo que a primitiva mandou junto — o
        // número do `<slider>`, o `value` da opção do `<radio>`. Guardar é
        // literalmente isto; o widget já formatou (o `<slider>` arredonda pelas
        // casas do `step`, então `step="0.05"` chega como "0.60", não como
        // "0.6000000238418579").
        let Some(novo) = value else { return };
        let chave = match action {
            "ajustar_volume" => "volume",
            "ajustar_brilho" => "brilho",
            "ajustar_graves" => "graves",
            "ajustar_zoom" => "zoom",
            // O `<radio>` escrito à mão: o valor que chega é o `value` da opção
            // clicada. Compare com o `<radiogroup>` ao lado, que não precisa
            // deste braço.
            "escolher_plano" => "plano",
            _ => return,
        };
        ctx.set(chave, novo.to_string());
    }
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 1")
        .main(|motor| {
            if let Err(e) = motor.register(Box::new(Onda1)) {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda1");
        })
        .run()
}
