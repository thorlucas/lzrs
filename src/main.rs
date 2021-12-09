use std::io::{Write, Result};

const DICT_SIZE: usize = 0x80;

fn ascii<'a, I>(bytes: I) -> String
    where 
        I: IntoIterator<Item = &'a u8>
{
    let ascii_bytes: Vec<u8> = bytes.into_iter().map(|b| match b {
        b if *b >= 32 && *b <= 126 => *b,
        _ => '.' as u8,
    }).collect();
    String::from_utf8(ascii_bytes).unwrap()
}

struct CompressorConfig {
    dict_size: usize,
}

struct Compressor<W: Write> {
    inner: W,
    config: CompressorConfig,

    buffer: Box<[u8]>,
    head: usize,
}

impl<W: Write> Write for Compressor<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> Compressor<W> {
    pub fn new(inner: W, config: CompressorConfig) -> Self {
        let buffer = {
            let mut buffer = Vec::with_capacity(config.dict_size);
            unsafe {
                buffer.set_len(config.dict_size);
            }
            buffer.into_boxed_slice()
        };

        Self {
            inner,
            config,
            buffer,
            head: 0,
        }
    }

    fn finish(mut self) -> Result<W> {
        self.flush()?;
        Ok(self.inner)
    }
}

fn main() -> Result<()> {
    let to: Vec<u8> = Vec::new();

    let mut comp = Compressor::new(to, CompressorConfig {
        dict_size: DICT_SIZE,
    });

    write!(comp, "Hey, banana-ass! To banana or not to banana?")?;

    let out = comp.finish()?;
    println!("{}", ascii(out.iter()));

    Ok(())
}
