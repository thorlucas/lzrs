use std::cmp;

use super::{FastCmp, ReadU64, WriteU64};

impl<T: AsRef<[u8]> + ?Sized> ReadU64 for &T {
    #[inline(always)]
    fn read_u64_unchecked(&self, index: usize) -> u64 {
        u64::from_le_bytes(self.as_ref()[index..index + 8].try_into().unwrap())
    }
}

impl<T: AsMut<[u8]> + ?Sized> WriteU64 for &mut T {
    #[inline(always)]
    fn write_u64_unchecked(&mut self, src: u64, index: usize) {
        self.as_mut()[index..index + 8].copy_from_slice(&u64::to_le_bytes(src));
    }
}

impl<S, T> FastCmp<T> for S
where
    S: AsRef<[u8]> + ?Sized,
    T: AsRef<[u8]>,
{
    fn match_length(&self, other: T) -> usize {
        let other = other.as_ref();
        let this = self.as_ref();

        let max_len = cmp::min(this.len(), other.len());
        let mut len = 0;

        // floor(ahead/8)*8
        let chunk_bytes = max_len & (!7);

        // compare 8 bytes at a time
        while len < chunk_bytes {
            if self.read_u64_unchecked(len) == other.read_u64_unchecked(len) {
                len += 8;
            } else {
                break;
            }
        }

        // compare 1 byte at a time
        while len < max_len {
            if this[len] == other[len] {
                len += 1;
            } else {
                break;
            }
        }

        len
    }
}
