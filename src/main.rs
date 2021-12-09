use std::io::{Write, Result};

const DICT_SIZE: usize = 0x80;

fn ascii_char(b: u8) -> String {
    if (b >= 32 && b <= 126) {
        format!("'{}'", b as char)
    } else {
        format!("0x{:02x}", b)
    }
}

fn ascii_buf<'a, I>(bytes: I) -> String
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
    config: CompressorConfig,

    inner: W,
    write_buf: Vec<u8>,

    dict: Vec<u8>,
    head: usize,

    map: [u32; 0x100],
    chain: Vec<u32>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Token {
    Literal {
        byte: u8,
    },
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal { byte } => write!(f, "<LIT> {}", ascii_char(*byte)),
        }
    }
}

impl<W: Write> Write for Compressor<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let (consumed, tok) = self.next_token(buf);
        self.write_token(&tok)?;
        Ok(consumed)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.write_all(&self.write_buf)?;
        Ok(())
    }
}

impl<W: Write> Compressor<W> {
    pub fn new(inner: W, config: CompressorConfig) -> Self {
        if config.dict_size > std::u32::MAX.try_into().unwrap() {
            panic!("Dictionary must be less than or equal to {} bytes!", std::u32::MAX);
        }

        Self {
            inner,
            dict: Vec::with_capacity(config.dict_size),
            head: 0,
            map: [std::u32::MAX; 0x100],
            chain: Vec::with_capacity(config.dict_size),
            write_buf: vec![],
            config,
        }
    }

    pub fn finish(mut self) -> Result<W> {
        self.flush()?;
        Ok(self.inner)
    }

    fn next_token(&self, lookahead: &[u8]) -> (usize, Token) {
        let tok = Token::Literal {
            byte: lookahead[0],
        };
        (1, tok)
    }

    fn write_token(&mut self, tok: &Token) -> Result<()> {
        println!("{:?}", tok);
        Ok(()) 
    }
}

fn main() -> Result<()> {
    let to: Vec<u8> = Vec::new();

    let mut comp = Compressor::new(to, CompressorConfig {
        dict_size: DICT_SIZE,
    });

    write!(comp, "Hey, banana-ass! To banana or not to banana?")?;

    let out = comp.finish()?;
    println!("{}", ascii_buf(out.iter()));

    Ok(())
}
