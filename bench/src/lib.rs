//! Here we're simply interested in finding what the difference is between using a sort of "virtual
//! buffer" for comparison or other methods.
//!
//! I suspect that copying into the buffer first before comparison will become slightly faster once
//! we reach a lot of iterations of comparison using that buffer. Otherwise, comparing directly
//! will be nearly the same speed and any difference will be negligable.

pub mod batched;

use rand::{Fill, self};

pub fn rand_buffer(size: usize) -> Box<[u8]> {
    let mut buf: Box<[u8]> = {
        let mut v = Vec::with_capacity(size);
        unsafe { v.set_len(size); }
        v.into_boxed_slice()
    };
    buf.try_fill(&mut rand::thread_rng()).unwrap();
    buf
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum MatchType {
    /// No match at `(distance)`
    None(usize),

    /// Non-overlapping match with `(distance, length)`
    Simple(usize, usize),

    /// Overlapping match with `(distance, length)`
    Overlapping(usize, usize),
}

#[derive(Default)]
#[allow(dead_code)]
pub struct DictTest {
    query_size: Option<usize>,
    match_type: Option<MatchType>,
}

impl DictTest {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn query_size(self, query_size: usize) -> Self {
        Self {
            query_size: Some(query_size),
            match_type: self.match_type,
        }
    }

    pub fn no_match(self, distance: usize) -> Self {
        Self {
            query_size: self.query_size,
            match_type: Some(MatchType::None(distance)),
        }
    }


    pub fn simple_match(self, distance: usize, length: usize) -> Self {
        Self {
            query_size: self.query_size,
            match_type: Some(MatchType::Simple(distance, length)),
        }
    }

    pub fn overlapping_match(self, distance: usize, length: usize) -> Self {
        Self {
            query_size: self.query_size,
            match_type: Some(MatchType::Overlapping(distance, length)),
        }
    }

    pub fn setup(&self, buffer: &DictBuf) -> (usize, usize, Box<[u8]>) {
        let mut query = {
            let size = self.query_size.unwrap();
            let mut vec: Vec<u8> = Vec::with_capacity(size);
            unsafe { vec.set_len(size); }
            vec.into_boxed_slice()
        };

        // Set up the query string
        let (dist, len) = match self.match_type.unwrap() {
            // Ensure there is no match by xoring
            MatchType::None(d) => {
                query[0] = buffer.buf[buffer.head-d-1] ^ 0xFF;
                (0, 0)
            },
            MatchType::Simple(d, l) => {
                assert!(l <= query.len());
                assert!(d+1 >= l, "Match with distance {} and length {} would overlap!", d, l); 
                // Make first l bytes of query match l bytes at head-d-1
                query[..l].copy_from_slice(&buffer.buf[buffer.head-d-1..buffer.head-d-1+l]);
                // Make the next byte of query not match
                if query.len() > l { 
                    query[l] = buffer.buf[buffer.head-d-1+l] ^ 0xFF;
                }

                (d, l)
            },
            MatchType::Overlapping(d, l) => {
                assert!(l <= query.len());
                assert!(d+1 < l, "Match with distance {} and length {} would not overlap!", d, l);
                // Split query into the pattern and tail portions
                let pattern = &buffer.buf[buffer.head-d-1..buffer.head];
                let mut tail = &mut query[..l];
                // Repeatedly copy pattern into tail for the whole tail
                loop {
                    if tail.len() > pattern.len() {
                        tail[..pattern.len()].copy_from_slice(pattern);
                        tail = &mut tail[pattern.len()..];
                    } else {
                        tail.copy_from_slice(&pattern[..tail.len()]);
                        break;
                    }
                }
                // Make the next byte not match
                if query.len() > l { 
                    query[l] = buffer.buf[buffer.head-d-1+l] ^ 0xFF;
                }

                (d, l)
            },
        };

        (dist, len, query)
    }
}

pub struct DictBuf {
    pub buf: Box<[u8]>,
    pub head: usize,
    pub la_len: Option<usize>,
}

impl DictBuf {
    /// Create an uninitialized buffer of `size` bytes.
    pub fn new(size: usize) -> Self {
        let mut vec = Vec::with_capacity(size);
        unsafe { 
            vec.set_len(size);
        }
        Self {
            buf: vec.into_boxed_slice(),
            head: 0,
            la_len: None,
        }
    }

    pub fn copy_la(&mut self, query: &[u8]) {
        for (q, c) in query.iter().zip(&mut self.buf[self.head..]) {
            *c = *q;
        }
        self.la_len = Some(query.len());
    }
}

pub fn fill_rand(buf: &mut[u8]) {
    buf.try_fill(&mut rand::thread_rng()).unwrap();
}

#[inline(always)]
pub fn read_unaligned_u64(buf: &[u8], index: usize) -> u64 {
    let bytes: [u8; 8] = buf[index..index+8].try_into().unwrap();
    u64::from_le_bytes(bytes)
}

#[inline(always)]
pub fn read_byte(buf: &[u8], index: usize) -> u8 {
    buf[index]
}

/// Read from `head[index]` until `index >= split`, then read from `tail[index-split]`.
#[inline(always)]
pub fn read_byte_segmented(head: &[u8], tail: &[u8], split: usize, index: usize) -> u8 {
    if index >= split {
        tail[index-split]
    } else {
        head[index]
    }
}

pub fn external_compare(buf: &DictBuf, distance: usize, query: &[u8]) -> usize {
    let pos = buf.head - distance - 1;
    let match_buf = &buf.buf[pos..];
    let split = buf.head - pos;

    let mut len = 0;
    let max_len = query.len();

    while len < max_len {
        if read_byte_segmented(match_buf, query, split, len) == read_byte(query, len) {
            len +=1;
        } else {
            break;
        }
    }

    len
}

pub fn internal_compare(buf: &DictBuf, distance: usize) -> usize {
    let pos = buf.head - distance - 1;
    let match_buf = &buf.buf[pos..];
    let split = buf.head - pos;

    let mut len = 0;
    let max_len = buf.la_len.unwrap();

    while len < max_len {
        if read_byte(match_buf, len) == read_byte(match_buf, len + split) {
            len +=1;
        } else {
            break;
        }
    }

    len
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_external() {
        let query_size = 256;
        let repeat_len = 32;
        let match_lens = [64, 128, 192, 256];

        let mut buf = DictBuf::new(512);
        buf.head = buf.buf.len() - query_size;
        fill_rand(&mut buf.buf);

        let tests = match_lens.map(|match_len| {
            DictTest::default()
                .query_size(query_size)
                .overlapping_match(repeat_len-1, match_len)
                .setup(&buf)
        });
        
        for (dist, len, query) in tests.iter() {
            assert_eq!(*len, external_compare(&buf, *dist, query));
        }
    }

    #[test]
    fn test_internal() {
        let query_size = 256;
        let repeat_len = 32;
        let match_lens = [64, 128, 192, 256];

        let mut buf = DictBuf::new(512);
        buf.head = buf.buf.len() - query_size;
        fill_rand(&mut buf.buf);

        let tests = match_lens.map(|match_len| {
            DictTest::default()
                .query_size(query_size)
                .overlapping_match(repeat_len-1, match_len)
                .setup(&buf)
        });
        
        for (dist, len, query) in tests.iter() {
            buf.copy_la(query);
            assert_eq!(*len, internal_compare(&buf, *dist));
        }
    }
}
