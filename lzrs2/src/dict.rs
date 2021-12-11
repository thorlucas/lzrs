use std::{cmp, io};

/// The main dictionary structure which is used to perform searches.
///
/// The searches are performed on the lookahead bytes which have already been loaded into the
/// dictionary.
pub struct Dictionary {
    // Size of the rolling dictionary of already encoded data.
    //
    // This is not always the size of the dictionary that *is* matched against, because when we load
    // data into the lookahead buffer we actually lose some dictionary space which becomes occupied by
    // positions we cannot start a match at.
    //
    // This must be a power of two.
    dict_cap: usize,

    // The size of the lookahead buffer which coexists with the dictionary buffer.
    //
    // Since a match cannot include the first byte of itself, a maximum of `MAX_MATCH_LEN - 1` bytes
    // in a match can come from the lookahead.
    la_cap: usize,

    // The mask used on dictionary indexes for fast modulus operation.
    mask: usize,

    // The buffer containing both the bytes already seen and the bytes in the lookahead.
    //
    // The extra `MAX_MATCH_LEN-1` bytes at the end ensure that when doing matching, we can
    // *always* just match forward without needing to do any wrapping. In order to do this, the
    // `MAX_MATCH_LEN-1` bytes at the front of the buffer are duplicated at the end of the buffer.
    //
    // Conceptually, this buffer is actually two buffers: the dictionary buffer and the lookahead
    // buffer. When bytes are loaded into the lookahead buffer, it could shrink the size of the
    // dictionary.
    buf: Box<[u8]>,

    // The current head position, behind which `dict_size` bytes have already been loaded and are
    // thus valid for searching, and in front of which `self.la_size` bytes are matched against.
    //
    // This should always be smaller than `self.capacity`. In order to efficiently wrap the head
    // around, we can use `head &= self.mask` since the `self.capacity` is a power of two.
    head: usize,

    // The number of loaded dictionary bytes which are valid for searching against.
    //
    // This should grow to a maximum of `self.capacity` bytes, but it will shrink when we load data
    // into the lookahead buffer. This is because when we load data into the lookahead, the output
    // is not yet known. However, if we start *within* the dictionary range, we can continue
    // matching past it.
    dict_size: usize,

    // The number of bytes of lookahead that have been loaded and are valid to be matched against.
    la_size: usize,
}

/// The dictionary structure which is used for matching against.
///
/// Implements [`std::io::Write`] for writing into the lookahead buffer.
impl Dictionary {
    /// Creates a new empty dictionary with a dictionary capacity of `dict_cap` and a lookahead
    /// buffer capacity of `la_cap`. The dictionary capacity *must* be a power of two.
    pub fn new(dict_cap: usize, la_cap: usize) -> Self {
        assert!((dict_cap-1) & dict_cap == 0);

        Self {
            dict_cap,
            la_cap,
            mask: dict_cap - 1,
            buf: {
                let mut v = Vec::with_capacity(dict_cap + la_cap);
                unsafe {
                    v.set_len(dict_cap + la_cap);
                }
                v.into_boxed_slice()
            },
            dict_size: 0,
            head: 0,
            la_size: 0,
        }
    }

    // Copies `len` bytes into the buffer at position `pos`, ensuring that the mirrored portion is
    // preserved. Does not update the head position.
    fn wrapped_copy(&mut self, buf: &[u8], mut pos: usize, len: usize) {
        pos = pos & self.mask;
        for &c in &buf[..len] {
            self.buf[pos] = c;
            if pos < self.la_cap {
                self.buf[pos + self.dict_cap] = c;
            }
            pos = (pos + 1) & self.mask;
        }
    }

    /// Attempts to load all of the bytes in `buf` into the lookahead buffer. Returns the number of
    /// bytes loaded.
    pub fn add_to_lookahead(&mut self, buf: &[u8]) -> usize {
         // We can load at maximum the number of bytes remaining in the lookahead buffer.
        let len = cmp::min(buf.len(), self.la_cap - self.la_size);

        self.wrapped_copy(buf, self.head + self.la_size, len);

        // Update buffer sizes.
        self.la_size += len;
        self.dict_size = cmp::min(self.dict_size, self.dict_cap - self.la_size);  
     
        len
    }

    /// Add bytes from `buf` directly into the dictionary buffer, **overwriting the existing
    /// lookahead.** This invalidates the lookahead buffer, setting its size to zero.
    ///
    /// If the buffer size is greater than the dictionary's capacity, it will only keep the last
    /// bytes.
    pub fn add_to_dictionary(&mut self, buf: &[u8]) {
        self.la_size = 0;

        self.wrapped_copy(buf, self.head, buf.len());

        self.dict_size = cmp::min(self.dict_size + buf.len(), self.dict_cap);
        self.head = (self.head + buf.len()) & self.mask;
    }

    /// Moves `n` bytes from the lookahead buffer into the dictionary.
    pub fn commit_lookahead_bytes(&mut self, n: usize) {
        // TODO: Update hash chain
        assert!(n <= self.la_size);
        self.head = (self.head + n) & self.mask;
        self.dict_size = self.dict_size + n;
        self.la_size -= n;
    }

    /// Returns two slices corresponding to the current valid dictionary buffer.
    ///
    /// The concatenation of both slices would represent the whole dictionary from tail to head.
    pub fn dictionary(&self) -> (&[u8], &[u8]) {
        let tail = self.head.wrapping_sub(self.dict_size) & self.mask;
        if tail > self.head {
            (&self.buf[tail..self.dict_cap], &self.buf[0..self.head])
        } else {
            (&self.buf[tail..self.head], &[])
        }
    }

    /// Returns the slice corresponding to the valid lookahead buffer.
    pub fn lookahead(&self) -> &[u8] {
        &self.buf[self.head..self.head+self.la_size]
    }
}

impl io::Write for Dictionary {
    /// Writes to the lookahead buffer. See: [`Dictionary::add_to_lookahead()`].
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(self.add_to_lookahead(buf))
    }

    /// Writes all of the lookahead buffer into the dictionary.
    fn flush(&mut self) -> io::Result<()> {
        self.commit_lookahead_bytes(self.la_size);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! dict {
        ($dc:literal, $la:literal) => {
            {
                let mut d = Dictionary::new($dc, $la);
                let mut i = 0;
                for c in &mut d.buf[..] {
                    *c = if i % 4 == 0 {
                        b'+'
                    } else {
                        b'-'
                    };
                    i += 1;
                }
                d
            }
        };
    }

    #[test]
    fn test_wrapped_copy() {
        let mut dict = dict!(8, 4);

        dict.wrapped_copy(b"abcdefgh", 0, 6);
        assert_eq!(b"abcdef--abcd", &dict.buf[..]);

        dict.wrapped_copy(b"foo", 7, 3);
        assert_eq!(b"oocdef-foocd", &dict.buf[..]);
    }

    #[test]
    fn test_add_to_lookahead() {
        let mut dict = dict!(8, 4);

        assert_eq!(3, dict.add_to_lookahead(b"abc"));
        assert_eq!(b"abc-+---abc-", &dict.buf[..]);
        assert_eq!(3, dict.la_size);

        assert_eq!(1, dict.add_to_lookahead(b"abc"));
        assert_eq!(b"abca+---abca", &dict.buf[..]);
        assert_eq!(4, dict.la_size);

        // Check that the dictionary size is shrunk
        let mut dict = dict!(8, 4);
        dict.wrapped_copy(b"abcd1234", 0, 8);
        dict.head = 3;
        dict.dict_size = 8;
        
        assert_eq!(3, dict.add_to_lookahead(b"---"));
        assert_eq!(b"abc---34abc-", &dict.buf[..]);
        assert_eq!(5, dict.dict_size);
    }

    #[test]
    fn test_add_to_dictionary() { 
        let mut dict = dict!(8, 4);

        dict.add_to_lookahead(b"abcd");
        dict.add_to_dictionary(b"foo");
        assert_eq!(b"food+---food", &dict.buf[..]);
        assert_eq!(3, dict.head);
        assert_eq!(3, dict.dict_size);
        assert_eq!(0, dict.la_size);
    }

    #[test]
    fn test_commit_lookahead() {
        let mut dict = dict!(8, 4);

        dict.add_to_lookahead(b"abcd");
        dict.commit_lookahead_bytes(2);
        assert_eq!(2, dict.head);
        assert_eq!(2, dict.dict_size);
        assert_eq!(2, dict.la_size);

        dict.add_to_lookahead(b"foo");
        assert_eq!(b"abcdfo--abcd", &dict.buf[..]);
        assert_eq!(4, dict.la_size);
    }

    #[test]
    fn test_get_dictionary() {
        let mut dict = dict!(8, 4);
        dict.head = 6;
        dict.add_to_dictionary(b"abcd");
        
        let (tail, head) = dict.dictionary();
        assert_eq!(b"ab", tail);
        assert_eq!(b"cd", head);
    }
    
    #[test]
    fn test_get_lookahead() {
        let mut dict = dict!(8, 4);
        dict.head = 6;
        dict.add_to_lookahead(b"abcd");

        assert_eq!(b"abcd", dict.lookahead());
    }
}
