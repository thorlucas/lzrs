use std::{cmp, io};

use super::prelude::*;

pub struct RingBuf {
    /// The buffer.
    buf: Box<[u8]>,

    /// The index in the buffer of the next byte to be written, behind which `len` bytes are valid.
    head: usize,

    /// The filled length of the buffer.
    len: usize,

    /// The total number of bytes ever written into the buffer.
    n: usize,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Index {
    /// The actual index in the array if still valid.
    i: usize,

    /// The total position that this byte was written into the buffer. The first byte ever written
    /// would have 0, the next 1, and so on. This allows us to calculate whether or not the index
    /// is still valid; if the ring buffer has length `N` then if `N` or more bytes have been written after
    /// this, it is no longer valid.
    n: usize,
}

impl RingBuf {
    /// Creates a buffer with the capacity of *at least* this many bytes.
    ///
    /// The actual capacity may be greater. It will round up to the nearest power of two for
    /// efficiency purposes.
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        Self {
            buf: {
                let mut v = Vec::with_capacity(capacity);
                unsafe {
                    v.set_len(capacity);
                }
                v.into_boxed_slice()
            },
            head: 0,
            len: 0,
            n: 0,
        }
    }

    /// Reads 8 bytes in little endian order at `index`. Panics if there `index..index+8` is out
    /// of bounds.
    #[inline(always)]
    fn read_u64_unchecked(&self, index: usize) -> u64 {
        u64::from_le_bytes(self.buf[index..index + 8].try_into().unwrap())
    }

    /// See [`write_u64()`].
    #[inline(always)]
    fn write_u64_unchecked(&mut self, src: u64, index: usize) {
        self.buf[index..index + 8].copy_from_slice(&u64::to_le_bytes(src));
    }

    /// Finds bytes of remaining space ahead of `head`.
    #[inline(always)]
    fn remaining_ahead(&self) -> usize {
        self.buf.len() - self.head
    }

    #[inline(always)]
    fn wrap(&self, index: usize) -> usize {
        index & (self.buf.len() - 1)
    }

    /// Wraps the offset from the head onto an index
    #[inline]
    fn wrap_offset(&self, offset: usize) -> usize {
        self.wrap(self.head + offset)
    }

    /// Wraps the offset from the head onto an index
    #[inline]
    fn wrap_offset_signed(&self, offset: isize) -> usize {
        if offset >= 0 {
            self.wrap(self.head + offset as usize)
        } else {
            self.wrap(self.head.wrapping_sub(offset as usize))
        }
    }

    /// Returns slices such that the first slice is the oldest written data and the second slice is
    /// the newest data (at the head).
    fn as_slices(&self) -> (&[u8], &[u8]) {
        let (head, tail) = self.buf.split_at(self.head);

        // Only `self.len` bytes behind the head are valid.
        //
        // If the head slice length is smaller than the total length, then only the tail must be
        // trimmed. It will keep the last `(self.len - head.len())` bytes.
        //
        // Otherwise, the head must be trimmed. It will keep the last `self.len` bytes.
        if self.len > head.len() {
            (&tail[tail.len() - (self.len - head.len())..], head)
        } else {
            (&[], &head[head.len() - self.len..])
        }
    }
}

impl io::Write for RingBuf {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();

        while buf.len() > 0 {
            let ahead = cmp::min(buf.len(), self.remaining_ahead());

            // floor(ahead/8)*8
            let chunk_bytes = ahead & (!7);

            // copy chunks 8 bytes at a time
            for i in (0..chunk_bytes).step_by(8) {
                self.write_u64_unchecked(read_u64(buf, i), self.head + i);
            }

            // copy the remaining bytes
            self.buf[self.head + chunk_bytes..self.head + ahead]
                .copy_from_slice(&buf[chunk_bytes..ahead]);

            // move head and buf
            self.head = self.wrap_offset(ahead);
            buf = &buf[ahead..];
        }

        self.len = cmp::min(self.len + len, self.buf.len());
        self.n += len;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Buffer for RingBuf {}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Write;

    macro_rules! p {
		($($t:tt)*) => {
			String::from_utf8_lossy($($t)*)
		};
	}

    macro_rules! rb {
		($rb:ident) => {};
        ($rb:ident [$cap:literal] $($t:tt)*) => {
            let mut $rb = RingBuf::with_capacity($cap);
			rb!{$rb $($t)*};
        };
		($rb:ident @ $head:literal $($t:tt)*) => {
			$rb.head = $head;
			rb!{$rb $($t)*};
		};
		($rb:ident len $len:literal $($t:tt)*) => {
			$rb.len = $len;
			rb!{$rb $($t)*};
		};
    }

    macro_rules! test {
        (($tail:literal, $head:literal), $rb:ident) => {
            let (tail, head) = $rb.as_slices();
            assert_eq!(
                $tail,
                tail,
                "Expected tail: \"{}\", actual: \"{}\".",
                p![$tail],
                p![tail]
            );
            assert_eq!(
                $head,
                head,
                "Expected head: \"{}\", actual: \"{}\".",
                p![$head],
                p![head]
            );
        };
    }

    #[test]
    fn test_write() -> Result<()> {
        rb! { rb[8] };
        test!((b"", b""), rb);

        rb.write_all(b"abcdef")?;
        test!((b"", b"abcdef"), rb);

        rb.write_all(b"abcdef")?;
        test!((b"efab", b"cdef"), rb);

        rb! { rb[8] @ 6 };
        rb.write_all(b"abcdef")?;
        test!((b"ab", b"cdef"), rb);

        Ok(())
    }
}
