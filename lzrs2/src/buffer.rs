//! This module provides buffer structures to be used for a variety of purposes, such as the back
//! end of a dictionary.

use std::{io, cmp};

pub struct RingBuf {
    buf: Box<[u8]>,
    head: usize,
    len: usize,
}

/// Reads 8 bytes in little endian order at `index`. Panics if there `index..index+8` is out
/// of bounds.
#[inline(always)]
fn read_u64(buf: &[u8], index: usize) -> u64 {
    u64::from_le_bytes(
        buf[index..index+8].try_into().unwrap()
    )
}

/// Writes 8 bytes in little endian order to `index`. Panics if there `index..index+8` is out
/// of bounds.
#[inline(always)]
fn write_u64(src: u64, buf: &mut [u8], index: usize) {
    buf[index..index+8].copy_from_slice(
        &u64::to_le_bytes(src)
    );
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
        }
    }

    /// See [`read_u64()`].
    #[inline(always)]
    fn read_u64(&self, index: usize) -> u64 {
        read_u64(&self.buf, index)
    }

    /// See [`write_u64()`].
    #[inline(always)]
    fn write_u64(&mut self, src: u64, index: usize) {
        write_u64(src, &mut self.buf, index);
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
    #[inline(always)]
    fn wrap_offset(&self, offset: usize) -> usize {
        self.wrap(self.head + offset)
    }

    /// Returns slices such that the first slice is the oldest written data and the second slice is
    /// the newest data (at the head).
    pub fn as_slices(&self) -> (&[u8], &[u8]) {
        let (head, tail) = self.buf.split_at(self.head);

        // Only `self.len` bytes behind the head are valid.
        //
        // If the head slice length is smaller than the total length, then only the tail must be
        // trimmed. It will keep the last `(self.len - head.len())` bytes.
        //
        // Otherwise, the head must be trimmed. It will keep the last `self.len` bytes.
        if self.len > head.len() {
            (
                &tail[tail.len()-(self.len-head.len())..],
                head,
            )
        } else {
            (
                &[],
                &head[head.len()-self.len..],
            )
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
                self.write_u64(
                    read_u64(buf, i),
                    self.head+i
                );
            }

            // copy the remaining bytes
            self.buf[self.head+chunk_bytes..self.head+ahead]
                .copy_from_slice(&buf[chunk_bytes..ahead]);

            // move head and buf
            self.head = self.wrap_offset(ahead);
            buf = &buf[ahead..];
        }

        self.len = cmp::min(self.len+len, self.buf.len());
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
	use anyhow::Result;

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
            assert_eq!($tail, tail, "Expected tail: \"{}\", actual: \"{}\".", p![$tail], p![tail]);
            assert_eq!($head, head, "Expected head: \"{}\", actual: \"{}\".", p![$head], p![head]);
        };
    }

    #[test]
    fn test_ring_buf_write() -> Result<()> {
		rb!{ rb[8] };
        test!((b"", b""), rb);

        rb.write_all(b"abcdef")?;
        test!((b"", b"abcdef"), rb);

		rb.write_all(b"abcdef")?;
		test!((b"efab", b"cdef"), rb);

		rb!{ rb[8] @ 6 };
		rb.write_all(b"abcdef")?;
		test!((b"ab", b"cdef"), rb);

		Ok(())
    }
}