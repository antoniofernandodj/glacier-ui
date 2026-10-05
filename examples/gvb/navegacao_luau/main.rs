//! Navegação decidida pelo `<script>` Lua (`navigate`/`navigate_back`), ao
//! invés dos atributos declarativos `navigateTo`/`navigateBack` (ver o
//! exemplo `navegacao`, que só troca de tela).
//!
//! Aqui o clique em "Entrar" só navega se a validação em Lua passar — o
//! próprio botão não sabe (nem pode saber, sendo declarativo) para onde vai.
//!
//! Rode com: `cargo run --example gvb_navegacao_luau`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gvb/navegacao_luau/login.gvb").run()
}
