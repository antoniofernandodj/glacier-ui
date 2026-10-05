//! Navegação entre telas, toda no markup: cada tela é um template, os botões
//! declaram o destino no próprio arquivo (`navigate_to` / `navigate_back`) e o
//! motor trata a troca em `dispatch`. As telas `perfil` e `config` entram na
//! principal por `<link rel="import" as="…">`, e o estado compartilhado
//! (`user_name`, `user_role`) nasce no `init()` do `<script>` da tela inicial.
//! O campo de nome grava a chave sozinho (`on_change="user_name"`), sem Rust.
use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new().main_template("examples/gvb/navegacao/nav_home.gvb").run()
}
