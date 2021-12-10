use std::thread;
use std::io::Result;

use lzrs::{layer::{self, make_subscriber}, ui::{start, self}};
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> Result<()> {
    start(ui::Config {

    }, |step_rx| {
        use lzrs_lib::prelude::{Writer, Config};
        use std::io::Write;

        println!("making subscriber");
        let subscriber = make_subscriber(layer::Config { step_rx });
        subscriber.init();

        thread::spawn(move || {
            println!("making work thread");
            let to: Vec<u8> = Vec::new();
            let mut comp = Writer::new(to, Config { dict_size: 0x80 });

            write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();
        });
    })?;

    Ok(())
}
