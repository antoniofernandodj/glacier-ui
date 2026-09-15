use glacier_ui::GlacierDaemon;

// Sem `.main`, o runner abre `views/app.gv`. A janela sem moldura, a geometria
// lembrada e o diretório de dados estão declarados no cabeçalho dele
// (`<screen decorations="false">` e `<app>`).
fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new().run()
}
