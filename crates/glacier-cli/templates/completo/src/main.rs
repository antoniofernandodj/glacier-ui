use glacier_ui::GlacierDaemon;

// Sem `.main`, o runner abre `views/app.gvb`, cuja raiz é o `app(...)`. A janela
// sem moldura, a geometria lembrada e o diretório de dados estão declarados nele
// (`decorations = false` e `remember_geometry` no `app`).
fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new().run()
}
