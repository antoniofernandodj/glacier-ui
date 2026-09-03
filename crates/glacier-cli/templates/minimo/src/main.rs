use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .main(|motor| {
            if let Err(erro) = motor.register_component("contador", "views/contador.gv") {
                eprintln!("{erro}");
            }
            motor.set_initial_screen("contador");
        })
        .run()
}
