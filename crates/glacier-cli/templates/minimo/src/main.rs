use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    let daemon = GlacierDaemon::new();
    daemon.run()
}
