mod contador;

use glacier_ui::GlacierDaemon;

use crate::contador::Contador;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(erro) = motor.register(Box::new(Contador::new())) {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("contador");
        })
        .run()
}
