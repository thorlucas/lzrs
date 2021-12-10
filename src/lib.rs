pub mod debug;
mod writer;

pub use writer::Compressor;

pub fn ascii_char(b: u8) -> String {
    if b >= 32 && b <= 126 {
        format!("'{}'", b as char)
    } else {
        format!("0x{:02x}", b)
    }
}

pub fn ascii_buf<'a, I>(bytes: I) -> String
    where 
        I: IntoIterator<Item = &'a u8>
{
    let ascii_bytes: Vec<u8> = bytes.into_iter().map(|b| match b {
        b if *b >= 32 && *b <= 126 => *b,
        _ => '.' as u8,
    }).collect();
    String::from_utf8(ascii_bytes).unwrap()
}

pub struct Config {
    pub dict_size: usize,
}
