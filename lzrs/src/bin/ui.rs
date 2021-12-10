use std::thread;
use std::io::Result;

use lzrs::app::{run, App};
use tracing::info;


fn main() -> Result<()> {
    let app = App::new();

    run(app, || {
        thread::spawn(|| {
            use lzrs_lib::{Writer, Config};
            use std::io::Write;

            let to: Vec<u8> = Vec::new();
            let mut comp = Writer::new(to, Config { dict_size: 0x80 });

            info!("Hello?");

            write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();
        });
    })?;

    Ok(())
}
