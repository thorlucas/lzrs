//! Preliminary benchmarks show that actually direct comparison *is faster* than copy comparison,
//! meaning it doesn't even make sense to copy lookahead data into the dictionary buffer
//! beforehand. This means we are free to have the dictionary structure simply be a regular ring
//! buffer. In this case, it makes more sense to have a general `Dictionary` trait, which can be
//! implemented any way. Then, we provide a `RingBuffer` struct.

use std::{cmp, io};

/// A dictionary which is used for conducting searches. A lookahead can be loaded into the
/// dictionary, and then the lengths of matches at specific offsets can be queried. It can also
/// retrieve matches given an offset.
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
    // thus valid for searching, and in front of which (inclusive) `self.la_size` bytes are matched
    // against.
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
    /// buffer capacity of `la_cap`. The dictionary capacity *must* be a power of two. The
    /// dictionary will be capable of making matches of length *up to* `la_cap+1`.
    pub fn new(dict_cap: usize, la_cap: usize) -> Self {
        assert!((dict_cap-1) & dict_cap == 0 && dict_cap != 0, "Dictionary sizes that are not powers of two are not supported!");

        Self {
            dict_cap,
            la_cap,
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

    /// Returns the index mod the dictionary size.
    #[inline(always)]
    fn wrap(&self, index: usize) -> usize {
        index & (self.dict_cap - 1)
    }
 
    /// Returns the index mod the dictionary size, where negative numbers wrap around the other
    /// way.
    #[inline(always)]
    fn wrap_signed(&self, index: isize) -> usize {
        self.wrap(index as usize)
    }

    /// Substracts the `value` from the `index`, and returns the wrapped index.
    #[inline(always)]
    fn wrap_sub(&self, index: usize, value: usize) -> usize {
        self.wrap(index.wrapping_sub(value))
    }

    /// Copies `len` bytes into the buffer at position `pos`, ensuring that the mirrored portion is
    /// preserved. Does not update the head position.
    fn wrapped_copy(&mut self, buf: &[u8], mut pos: usize, len: usize) {
        for &c in &buf[..len] {
            self.buf[pos] = c;
            if pos < self.la_cap {
                self.buf[pos + self.dict_cap] = c;
            }
            pos = self.wrap(pos + 1);
        }
    }

    /// Copies `len` bytes into the buffer at position `pos` from itself, starting at position
    /// `from`.
    fn wrapped_copy_self(&mut self, mut from: usize, mut pos: usize, len: usize) {
        for _ in 0..len {
            let c = self.buf[from];
            self.buf[pos] = c;
            if pos < self.la_cap {
                self.buf[pos + self.dict_cap] = c;
            }
            pos = self.wrap(pos + 1);
            from = self.wrap(from + 1);
        }
    }


    /// Clears the lookahead buffer.
    #[inline]
    pub fn clear_lookahead(&mut self) {
        self.la_size = 0;
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
        self.clear_lookahead();

        self.wrapped_copy(buf, self.head, buf.len());

        self.dict_size = cmp::min(self.dict_size + buf.len(), self.dict_cap);
        self.head = self.wrap(self.head + buf.len());
    }

    /// Moves `n` bytes from the lookahead buffer into the dictionary, returning a slice to the
    /// committed bytes.
    pub fn commit_lookahead_bytes(&mut self, n: usize) -> &[u8] {
        // TODO: Update hash chain
        assert!(n <= self.la_size);

        // Since there will always be at least `la_cap` bytes mirrored at the end of the buffer,
        // and since `n` will always be smaller than `la_cap`, this will always be fine.
        let bytes = &self.buf[self.head..self.head+n];

        self.head = self.wrap(self.head + n);
        self.dict_size = self.dict_size + n;
        self.la_size -= n;

        bytes
    }

    /// Returns two slices corresponding to the current valid dictionary buffer.
    ///
    /// The concatenation of both slices would represent the whole dictionary from tail to head.
    pub fn dictionary(&self) -> (&[u8], &[u8]) {
        let tail = self.wrap_sub(self.head, self.dict_size);
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

    /// Reads 8 bytes from the dictionary at index `pos` and casts it into a `u64` in little
    /// endian.
    #[inline]
    fn read_unaligned_u64(&self, pos: usize) -> u64 {
        let bytes: [u8; 8] = self.buf[pos..pos+8].try_into().unwrap();
        u64::from_le_bytes(bytes)
    }

    /// Returns the length of the longest match against the stored lookahead at `distance+1` bytes
    /// behind the head, up to a maximum match length of the number of bytes currently loaded into
    /// the lookahead buffer.
    ///
    /// **Note** that a distance of zero means the last byte added to the dictionary.
    pub fn match_length(&self, distance: usize) -> usize {
        assert!(distance < self.dict_size, "distance {} out of bounds (dictionary size {})", distance, self.dict_size);

        if self.la_size == 0 {
            return 0;
        }

        let pos = self.wrap_sub(self.head, distance + 1);

        let mut len = 0;
        let max_len = self.la_size;
        
        // Match 8 bytes at a time until there is less than 8 bytes remaining to the maximum.
        while max_len - len >= 8 {
            let dict_val = self.read_unaligned_u64(pos + len);
            let la_val = self.read_unaligned_u64(self.head + len);

            if dict_val == la_val {
                len += 8;
            } else {
                break;
            }
        }

        // Match one byte at a time
        while len < max_len {
            if self.buf[pos + len] == self.buf[self.head + len] {
                len +=1;
            } else {
                break;
            }
        }

        len
    }

    /// Returns the `length` match at `distance` from the head.
    ///
    /// This will load bytes into the lookahead as it does so.
    pub fn load_match_into_lookahead(&mut self, distance: usize, length: usize) -> &[u8] {
        assert!(distance < self.dict_size);
        assert!(length+self.la_size <= self.la_cap);

        let pos = self.wrap_sub(self.head, distance + 1);
        self.wrapped_copy_self(pos, self.head+self.la_size, length);            

        self.la_size += length;
        &self.buf[pos..pos+length]
    }

    /// Loads a match like [`load_match_into_lookahead()`] but commits the match immediately to the dictionary.
    /// 
    /// This invalidates any lookahead currently present.
    pub fn load_match_into_dictionary(&mut self, distance: usize, length: usize) -> &[u8] {
        assert!(distance < self.dict_size);

        let pos = self.wrap_sub(self.head, distance + 1);
        self.wrapped_copy_self(pos, self.head, length);            

        self.clear_lookahead();
        self.dict_size = cmp::min(self.dict_size + length, self.dict_cap);
        self.head = self.wrap(self.head + length); 

        &self.buf[pos..pos+length]
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
        assert_eq!(b"ab", dict.commit_lookahead_bytes(2));
        assert_eq!(2, dict.head);
        assert_eq!(2, dict.dict_size);
        assert_eq!(2, dict.la_size);
        assert_eq!(b"cd", dict.lookahead());

        dict.add_to_lookahead(b"foo");
        assert_eq!(b"abcdfo--abcd", &dict.buf[..]);
        assert_eq!(4, dict.la_size);
        assert_eq!(b"cdfo", dict.lookahead());

        assert_eq!(b"cdf", dict.commit_lookahead_bytes(3));
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

    #[test]
    fn test_match_length() {
        let mut dict = dict!(8, 4);
        dict.add_to_dictionary(b"abcd");

        assert_eq!(0, dict.match_length(0));
        assert_eq!(0, dict.match_length(2));

        dict.add_to_lookahead(b"abcd");
        assert_eq!(4, dict.match_length(3));

        dict.clear_lookahead();
        dict.add_to_lookahead(b"bc");
        assert_eq!(2, dict.match_length(2));

        // Test that unaligned 8 byte reads work
        let mut dict = dict!(32, 16);
        dict.add_to_dictionary(b"--1234567+1234--");
        dict.add_to_lookahead(b"1234567+1234567+");
        assert_eq!(12, dict.match_length(13));

        dict.clear_lookahead();
        dict.add_to_lookahead(b"1234567+12");
        assert_eq!(10, dict.match_length(13));
    }

    #[test]
    fn test_match_length_over() { 
        let mut dict = dict!(8, 4);
        dict.add_to_dictionary(b"ban");
        dict.add_to_lookahead(b"ana");
        assert_eq!(3, dict.match_length(1));

        let mut dict = dict!(16, 12);
        dict.add_to_dictionary(b"bad+");
        dict.add_to_lookahead(b"ad+ad+ad+ad");
        assert_eq!(11, dict.match_length(2));

        let mut dict = dict!(8, 4);
        dict.add_to_dictionary(b"abcd");
        dict.add_to_lookahead(b"dddd");
        assert_eq!(4, dict.match_length(0));
    }

    #[test]
    fn test_load_match() {
        // Into lookahead
        let mut dict = dict!(16, 4);
        dict.add_to_dictionary(b"ban");
        assert_eq!(b"ana", dict.load_match_into_lookahead(1, 3)); 

        assert_eq!(b"ana", dict.commit_lookahead_bytes(3));
        let (buf, _) = dict.dictionary();
        assert_eq!(b"banana", buf);

        let mut dict = dict!(16, 12);
        dict.add_to_dictionary(b"bad+");
        assert_eq!(b"ad+ad+ad+ad", dict.load_match_into_lookahead(2, 11));

        let mut dict = dict!(8, 4);
        dict.add_to_dictionary(b"abcd");
        assert_eq!(b"dddd", dict.load_match_into_lookahead(0, 4));

        // Into dictionary
        let mut dict = dict!(16, 4);
        dict.add_to_dictionary(b"bad+");
        assert_eq!(b"ad+ad+ad+ad", dict.load_match_into_dictionary(2, 11));
        assert_eq!(b"bad+ad+ad+ad+ad-bad+", &dict.buf[..]);
        assert_eq!(0, dict.la_size);
        assert_eq!(15, dict.dict_size);
    }
}
