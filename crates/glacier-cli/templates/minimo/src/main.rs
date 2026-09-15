use glacier_ui::{GlacierDaemon, iced::Result};

fn main() -> Result {
    GlacierDaemon::new()
        .run()
}
