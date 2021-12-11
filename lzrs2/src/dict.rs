use std::{cmp, io};

/// Size of the rolling dictionary of already encoded data.
///
/// This is not always the size of the dictionary that *is* matched against, because when we load
/// data into the lookahead buffer we actually lose some dictionary space which becomes occupied by
/// positions we cannot start a match at.
///
/// This must be a power of two.
const DICT_SIZE: usize = 1 << 5;

/// The mask used on dictionary indexes for fast modulus operation.
const DICT_MASK: usize = (DICT_SIZE as u32 - 1) as usize;

/// The maximum size of a match in bytes.
const MAX_MATCH_LEN: usize = 8;

/// The size of the lookahead buffer which coexists with the dictionary buffer.
///
/// Since a match cannot include the first byte of itself, a maximum of `MAX_MATCH_LEN - 1` bytes
/// in a match can come from the lookahead.
const LA_SIZE: usize = MAX_MATCH_LEN - 1;

/// The size of the buffer used to store the dictionary and the lookahead.
///
/// The dictionary buffer stores the lookahead so as to allow for lengths which are greater than
/// the distance. We also include an additional `LA_SIZE` bytes at the end which mirror the first
/// `LA_SIZE` bytes to allow for matching in the forward direction only, which speeds up
/// matching.
const DICT_BUF_SIZE: usize = DICT_SIZE + LA_SIZE;

/// The main dictionary structure which is used to perform searches.
///
/// The searches are performed on the lookahead bytes which have already been loaded into the
/// dictionary.
pub struct Dictionary {
    /// The buffer containing both the bytes already seen and the bytes in the lookahead.
    ///
    /// The extra `MAX_MATCH_LEN-1` bytes at the end ensure that when doing matching, we can
    /// *always* just match forward without needing to do any wrapping. In order to do this, the
    /// `MAX_MATCH_LEN-1` bytes at the front of the buffer are duplicated at the end of the buffer.
    ///
    /// Conceptually, this buffer is actually two buffers: the dictionary buffer and the lookahead
    /// buffer. When bytes are loaded into the lookahead buffer, it could shrink the size of the
    /// dictionary.
    buf: Box<[u8; DICT_BUF_SIZE]>,

    /// The number of loaded dictionary bytes which are valid for searching against.
    ///
    /// This should grow to a maximum of `DICT_SIZE` bytes, but it will shrink when we load data
    /// into the lookahead buffer. This is because when we load data into the lookahead, the output
    /// is not yet known. However, if we start *within* the dictionary range, we can continue
    /// matching past it.
    size: usize,

    /// The current head position, behind which `size` bytes have already been loaded and are
    /// thus valid for searching, and in front of which `la_size` bytes are matched against.
    ///
    /// This should always be smaller than `DICT_SIZE`. In order to efficiently wrap the head
    /// around, we can use `head &= DICT_MASK` since the `DICT_SIZE` is a power of two.
    head: usize,

    /// The number of bytes of lookahead that have been loaded and are valid to be matched against.
    la_size: usize,
}

/// The dictionary structure which is used for matching against.
///
/// Implements `Write` for writing into the lookahead buffer.
impl Dictionary {
    pub fn new() -> Self {
        Self {
            buf: Box::new([0; DICT_BUF_SIZE]),
            size: 0,
            head: 0,
            la_size: 0,
        }
    }

    /// "Moves" `n` bytes from the lookahead buffer into the dictionary.
    pub fn add_to_dict(&mut self, n: usize) {
        assert!(n <= self.la_size);
        self.head = (self.head + n) & DICT_MASK;
        self.la_size -= n;
    }
}

impl io::Write for Dictionary {
    /// Attempts to load all of the bytes in `buf` into the lookahead buffer. Returns the number of
    /// bytes loaded, which will be a maximum of `MAX_MATCH_LEN-1` bytes.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // We can load at maximum the number of bytes remaining in the lookahead buffer.
        let len = cmp::min(buf.len(), LA_SIZE - self.la_size);

        // Update buffer sizes.
        self.la_size += len;
        self.size = cmp::min(self.size, DICT_SIZE - self.la_size);
        
        // TODO: Store in hash chain
     
        self.buf[self.head..self.head+len].copy_from_slice(&buf[..len]);
        
        // Make sure that the last `LA_SIZE` bytes mirror the first `LA_SIZE` bytes.
        if self.head < LA_SIZE {
            // Bytes which overlaps the first `LA_SIZE` bytes.
            let overlap = cmp::min(LA_SIZE-self.head, len);
            self.buf[self.head+DICT_SIZE..][..overlap].copy_from_slice(&buf[..overlap]);
        }

        Ok(len)
    }

    /// Writes all of the lookahead buffer into the dictionary.
    fn flush(&mut self) -> io::Result<()> {
        self.add_to_dict(self.la_size);
        Ok(())
    }
}
