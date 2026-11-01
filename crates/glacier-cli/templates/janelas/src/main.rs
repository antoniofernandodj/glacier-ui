use glacier_ui::GlacierDaemon;

// A janela, o ícone, a bandeja, a instância única, a geometria lembrada e as
// telas estão declarados em `views/painel.gv`, cuja raiz é o `app(...)`. O
// `main.rs` só diz qual manifesto abre.
fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main_template("views/painel.gv")
        .run()
}
