use std::io::{Write, Result};
use lzrs_lib::prelude::{Writer, Config};
use tracing::Level;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let to: Vec<u8> = Vec::new();

    let mut comp = Writer::new(to, Config {
        dict_size: 0x80,
    });

    write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();

    Ok(())
}
