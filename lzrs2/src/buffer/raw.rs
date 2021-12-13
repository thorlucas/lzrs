//! Provides utilities for reading from, writing to, and comparing raw byte buffers.

use std::cmp;

/// Reads 8 bytes from a buffer into a a [`u64`] in **little endian order**. Panics on out of
/// bounds.
#[track_caller]
#[inline(always)]
pub fn read_u64(buf: &[u8], index: usize) -> u64 {
    u64::from_le_bytes(buf[index..index + 8].try_into().unwrap())
}

/// Writes 8 bytes into a buffer into a a [`u64`] in **little endian order**. Panics on out of
/// bounds.
#[track_caller]
#[inline(always)]
pub fn write_u64(buf: &mut [u8], index: usize, src: u64) {
    buf[index..index + 8].copy_from_slice(&u64::to_le_bytes(src));
}

/// Calculates the length of the prefix match between two buffers.
pub fn match_length(lhs: &[u8], rhs: &[u8]) -> usize {
    let max_len = cmp::min(lhs.len(), rhs.len());
    let mut len = 0;

    // floor(ahead/8)*8
    let chunk_bytes = max_len & (!7);

    // compare 8 bytes at a time
    while (len < chunk_bytes) && (read_u64(lhs, len) == read_u64(rhs, len)) {
        len += 8;
    }

    // compare 1 byte at a time
    while (len < max_len) && (lhs[len] == rhs[len]) {
        len += 1;
    }

    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_match_length() {
        assert_eq!(
            30,
            match_length(
                b"abcdefg_0123456_abcdefg_0123456_",
                b"abcdefg_0123456_abcdefg_012345"
            )
        );
        assert_eq!(11, match_length(b"abcdefg_0123456_", b"abcdefg_012"));
        assert_eq!(11, match_length(b"abcdefg_012", b"abcdefg_0123456_"));
        assert_eq!(8, match_length(b"abcdefg_", b"abcdefg_012"));
        assert_eq!(3, match_length(b"abc", b"abcdefg_012"));
        assert_eq!(3, match_length(b"abc", b"abc"));
        assert_eq!(0, match_length(b"abc", b""));
        assert_eq!(0, match_length(b"abc", b""));

        let slice: &[u8] = b"abcd";
        let array: [u8; 4] = *b"asdf";

        match_length(slice, &array);
    }
}
