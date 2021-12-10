use std::thread;
use std::io::Result;

use lzrs::{layer, ui};
use tracing::info;


fn main() -> Result<()> {
    let app = ui::App::new();

    layer::start(
        layer::Config {
            writer: app.log_buffer.clone(),
        }
    );

    thread::spawn(move || {
        use lzrs_lib::{Writer, Config};
        use std::io::Write;

        let to: Vec<u8> = Vec::new();
        let mut comp = Writer::new(to, Config { dict_size: 0x80 });

        info!("Hello?");

        write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();
    });

    ui::start(app)?;

    Ok(())
}
