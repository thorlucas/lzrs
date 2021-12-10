use std::io::{Write, Result};
use lzrs_lib::prelude::{Writer, Config};

fn main() -> Result<()> {
    let to: Vec<u8> = Vec::new();

    let mut comp = Writer::new(to, Config {
        dict_size: 0x80,
    });

    write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();

    Ok(())
}
