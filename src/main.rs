use std::io::{Write, Result};
use lzrs::{Compressor, Config, ascii_buf};

fn main() -> Result<()> {
    let to: Vec<u8> = Vec::new();

    let mut comp = Compressor::new(to, Config {
        dict_size: 0x80,
    });

    #[cfg(feature = "ui")]
    lzrs::ui::spawn_ui()?;

    write!(comp, "Hey, banana-ass! To banana or not to banana?").unwrap();
    let out = comp.finish().unwrap();

    println!("{}", ascii_buf(out.iter()));

    Ok(())
}
