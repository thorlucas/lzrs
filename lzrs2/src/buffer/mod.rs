//! This module provides buffer structures to be used for a variety of purposes, such as the back
//! end of a dictionary.

mod u8;
pub mod ringbuf;

pub mod prelude {
    pub use super::{FastCmp, WriteU64, ReadU64, Buffer, Distance, ringbuf::RingBuf, u8::*};
}


pub trait ReadU64 {
    /// Read 8 bytes starting at the index **in little endian order**, optionally
    /// without checking for bounds, and instead panicking on error.
    fn read_u64_unchecked(&self, index: usize) -> u64;
}

pub trait WriteU64 {
    /// Write 8 bytes starting at the index **in little endian order**, optionally
    /// without checking for bounds, and instead panicking on error.
    fn write_u64_unchecked(&mut self, src: u64, index: usize);
}

pub trait FastCmp<T> {
    /// Compares `self` to `other`, returning the number of bytes that match from the
    /// front.
    fn match_length(&self, other: T) -> usize;
}

pub trait Buffer {}

/// Represents a distance backwards from the head of buffers. Zero distance means the last byte
/// that was written.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Distance(usize);

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_u8_match_length() {
        assert_eq!(
            30,
            b"abcdefg_0123456_abcdefg_0123456_".match_length(b"abcdefg_0123456_abcdefg_012345")
        );
        assert_eq!(11, b"abcdefg_0123456_".match_length(b"abcdefg_012"));
        assert_eq!(11, b"abcdefg_012".match_length(b"abcdefg_0123456_"));
        assert_eq!(8, b"abcdefg_".match_length(b"abcdefg_012"));
        assert_eq!(3, b"abc".match_length(b"abcdefg_012"));
        assert_eq!(3, b"abc".match_length(b"abc"));
        assert_eq!(0, b"abc".match_length(b""));
        assert_eq!(0, b"abc".match_length(b""));

        let slice: &[u8] = b"abcd";
        let array: &[u8; 4] = b"asdf";
        array.match_length(slice);
        slice.match_length(array);
        slice.match_length(*array);
    }
}
