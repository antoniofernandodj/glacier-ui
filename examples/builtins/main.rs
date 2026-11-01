//! Demonstra um componente **embutido** da própria `glacier-ui`: `<badge/>`.
//!
//! O app registra só a sua tela (`Home`). Ele nunca registra `Badge` — a lib já
//! o registrou sozinha em `GlacierUI::new()`, então a tag fica disponível em
//! qualquer template, como uma primitiva. Veja `src/builtins.rs`.

use glacier_ui::{Component, Context, GlacierDaemon, Template};

struct Home;

impl Component for Home {
    fn name(&self) -> &str {
        "home"
    }

    fn template(&self) -> Template {
        // `<badge/>` não é registrado por este app — vem embutido na lib.
        Template::Inline(
            r##"<container padding="40" width="fill" height="fill" background="#1E1E2E" align_x="Center" align_y="Center">
                <column spacing="20" align="Center">
                    <text size="24" bold="true" color="#CDD6F4">
                      Widgets embutidos da glacier-ui
                    </text>
                    <row spacing="12" align="Center">
                        <badge />
                        <badge badge_text="Novo" badge_bg="#A6E3A1" />
                        <badge badge_text="Beta" badge_bg="#F9E2AF" />
                        <badge badge_text="3" badge_bg="#F38BA8" />
                        <badge badge_text="Grande" badge_bg="#CBA6F7" badge_size="20" />
                    </row>
                </column>
            </container>"##
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {}
}

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Widgets embutidos")
        .main(|motor| {
            // Só a tela. `Badge` já está disponível — não é registrado aqui.
            motor.register(Box::new(Home)).expect("registrar 'home'");
            motor.set_initial_screen("home");
        })
        .run()
}
