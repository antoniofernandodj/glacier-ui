//! **Onda 8b, em Luau**: `component` e `dialog`, nos dois sentidos da
//! composição.
//!
//! Rode com: `cargo run --example onda8b_luau`
//!
//! A Onda 8 mostrou que o corpo de um `<dialog>` é um template comum. Isso
//! abre duas composições que este exemplo separa:
//!
//! # A — um `component` que ENCAPSULA o diálogo inteiro
//!
//! `<EditorRotulo/>` (`views/editor_rotulo.gv`) traz, dentro de si:
//!
//!   * a declaração `<dialog name="editor_rotulo_dlg">` — `load_defines` é uma
//!     varredura recursiva da árvore, então um `<dialog>` declarado dentro de
//!     um `<component>` é recolhido no registro exatamente como um do
//!     `<resources>` da tela;
//!   * o botão que o abre (`on_click="dialog:editor_rotulo_dlg"`);
//!   * o `<script>` com o handler — e é por isso que o botão *Salvar* despacha
//!     `EditorRotulo::salvar`, com o `::` de dono: sem o prefixo a ação iria
//!     para a tela, não para o componente (`GlacierUI::route_to_owner`).
//!
//! A tela não declara nada disso. Só escreve `<EditorRotulo/>`.
//!
//! # B — um `dialog` cujo CORPO usa um `component`
//!
//! O `<dialog name="editar_perfil">` do `<resources>` da tela monta dois
//! `<CampoForm>` (rótulo + `<slot/>`) no corpo. Um `<component>` avaliado no
//! contexto do app, como qualquer nó — o `<textinput>` de cada campo continua
//! ligado a uma chave `__dialog.*` literal, e o handler `salvar_perfil` mora
//! na tela.
//!
//! Os dois diálogos são **singleton e globais** (moram num mapa só, keyed pelo
//! `name`): o nome de cada um tem de ser único no app todo.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Onda 8b (Luau)")
        .main(|motor: &mut glacier_ui::GlacierUI| {
            if let Err(e) = motor.register_component("onda8b_luau", "examples/onda8b_luau/app.gv") {
                eprintln!("Erro ao registrar a tela: {e}");
            }
            motor.set_initial_screen("onda8b_luau");
        })
        .run()
}
