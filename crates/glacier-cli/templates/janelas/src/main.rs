use glacier_ui::GlacierDaemon;

// A janela, o ícone, a bandeja, a instância única e a geometria lembrada estão
// declarados no cabeçalho de `views/painel.gv` (`<screen>`, `<app>` e `<tray>`).
// O `main.rs` só diz qual template abre.
fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main_template("views/painel.gv")
        .run()
}
