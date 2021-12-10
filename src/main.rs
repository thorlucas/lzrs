use std::io::{Write, Result};
use lzrs::{Compressor, Config, ascii_buf};
use tracing::debug_span;

fn main() -> Result<()> {
    lzrs::debug::init()?;

    let to: Vec<u8> = Vec::new();

    let mut comp = Compressor::new(to, Config {
        dict_size: 0x80,
    });

    std::thread::spawn(|| {
        let span = debug_span!("test");
        let _enter = span.enter();

        println!("Testing!...");
        std::thread::sleep_ms(1000);
    });

    std::thread::spawn(move || {
        write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();
        let out = comp.finish().unwrap();

        println!("{}", ascii_buf(out.iter()));
    });

    Ok(())
}
