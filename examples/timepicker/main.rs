//! Exemplo de uso do widget builtin TimePicker.
//!
//! Rode com: `cargo run --example timepicker`

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - TimePicker Demo")
        .main(|motor| {
            let template = "examples/timepicker/app.gv".to_string();
            if let Err(e) = motor.register_component("timepicker", &template) {
                eprintln!("Erro ao registrar: {}", e);
            }
            motor.set_initial_screen("timepicker");
        })
        .run()
}
