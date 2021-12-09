use std::io::{Write, Result};

use crate::{ascii_char, Config};

pub struct Compressor<W> {
    dict_size: usize,

    inner: W,
    write_buf: Vec<u8>,

    dict: Vec<u8>,
    head: usize,

    map: [u32; 0x100],
    chain: Vec<u32>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Token {
    Literal {
        byte: u8,
    },
    Rep {
        distance: usize,
        length: usize,
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal { byte } => write!(f, "<LIT {}>", ascii_char(*byte)),
            Self::Rep { distance, length } => write!(f, "<REP {}, {}>", distance, length),
        }
    }
}

impl<W: Write> Write for Compressor<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let (consumed, tok) = self.next_token(buf);

        self.write_to_dictionary(&buf[..consumed]);
        self.write_token(&tok)?;
        Ok(consumed)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.write_all(&self.write_buf)?;
        Ok(())
    }
}

impl<W: Write> Compressor<W> {
    pub fn new(inner: W, config: Config) -> Self {
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
            dict_size: config.dict_size,
        }
    }

    pub fn finish(mut self) -> Result<W> {
        self.flush()?;
        Ok(self.inner)
    }

    fn next_token(&self, lookahead: &[u8]) -> (usize, Token) {
        let mut best_match = (0, None);

        let mut last = None;
        while let Some(match_index) = self.next_match_index(last, lookahead) {
            let len = self.match_len(match_index, lookahead);

            if len > best_match.0 {
                best_match = (len, Some(match_index))
            }

            last = Some(match_index);
        }

        if let (len, Some(index)) = best_match {
            (len,
            Token::Rep {
                length: len,
                distance: self.distance(index)
            })
        } else {
            (1,
            Token::Literal {
                byte: lookahead[0],
            })
        }
    }

    fn write_token(&mut self, _tok: &Token) -> Result<()> {
        Ok(()) 
    }

    /// Returns the maximum length match from the dictionary, starting at the dictionary index
    /// `at`.
    fn match_len(&self, at: usize, lookahead: &[u8]) -> usize {
        // The length of the match
        let mut len = 0;

        // This is the maximum that len can get before it wraps over onto the output
        // When we read from the dict, we have to mod len with over_eln before getting the offset.
        // That way, our read will repeat as appropriate.
        let over_len = self.distance(at) + 1;

        while self.dict[(at + (len % over_len)) % self.dict_size] == lookahead[len] {
            len += 1;
        }

        len
    }

    fn next_match_index(&self, last: Option<usize>, lookahead: &[u8]) -> Option<usize> {
        let index = if let Some(last) = last {
            self.chain[last] as usize
        } else {
            self.map[lookahead[0] as usize] as usize
        };

        if let Some(item) = self.dict.get(index) {
            if *item == lookahead[0] {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the index as the distance from the head, where 0 distance means the last item added
    /// to the dictionary.
    fn distance(&self, index: usize) -> usize {
        if self.head > index {
            self.head - index - 1
        } else {
            self.dict_size - index + self.head - 1
        }
    }

    fn write_to_dictionary(&mut self, bytes: &[u8]) {
        for b in bytes {
            let first_match = self.map[*b as usize];
            self.map[*b as usize] = self.head as u32;

            if self.dict.len() < self.dict_size {
                self.dict.push(*b);
                self.chain.push(first_match);
            } else {
                self.dict[self.head] = *b;
                self.chain[self.head] = first_match;
            }

            self.head = (self.head + 1) % self.dict_size;
        }
    }
}
