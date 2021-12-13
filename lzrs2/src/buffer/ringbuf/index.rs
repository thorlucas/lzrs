use super::*;

impl Buffer for RingBuf {
    #[inline]
    fn get(&self, index: usize) -> Option<&u8> {
        // TODO: We're assuming here the buffer length never shrinks!
        if index >= self.n - self.len && index < self.len {
            Some(unsafe { &*self.get_unchecked(index) })
        } else {
            None
        }
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, index: usize) -> *const u8 {
        self.buf.get_unchecked(self.wrap(index))
    }
}

impl ops::Index<usize> for RingBuf {
    type Output = u8;

    /// Indexing a [`RingBuf`] with a `usize` is defined as indexing **from the first byte ever
    /// written**, like a "virtual buffer". In other words, the index for each new added byte will
    /// increment forever. This implementation ensures that we safely differentiate between data
    /// that has been overwritten.
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .expect(&format!("Index {} out of bounds.", index))
    }
}

// We have a bit of a problem here... so let's leave this out for now.
/*
impl<'a> ops::Index<ops::RangeFull> for &'a RingBuf {
    type Output = Slice<'a>;

    fn index(&self, index: ops::RangeFull) -> &Self::Output {
        &Slice {
            data: self.buf.as_ptr(),
            mask: self.mask,
            // tail position
            offset: self.wrap_offset_signed(-(self.len as isize + 1)),
            len: self.len,
            _b: PhantomData::default(),
        }
    }
}
*/

// Because of Rust's awesome borrow rules, it's actually impossible for this data to change while
// we have a reference to this! So we can literally just store the offset and the length and
// calculate indexes by using the mask. Additionally, **it is impossible to have an illegal
// slice**, so we don't need to store any more data related to verifying integrity.
pub struct Slice<'a> {
    /// The buffer slice of the actual data
    data: *const u8,
    /// The mask that is applied to the index
    mask: usize,
    /// The offset from the start of the buffer that this slice begins at
    offset: usize,
    /// The length of the slice
    len: usize,

    _b: PhantomData<&'a RingBuf>,
}

impl Slice<'_> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }
}

unsafe impl<'a> SliceIndex<Slice<'a>> for usize {
    type Output = u8;

    #[inline]
    fn get(self, slice: &Slice) -> Option<&'a Self::Output> {
        if self < slice.len() {
            unsafe { Some(&*self.get_unchecked(slice)) }
        } else {
            None
        }
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const Slice) -> *const Self::Output {
        (*slice).data.add(((*slice).offset + self) & (*slice).mask)
    }

    #[inline]
    fn index(self, slice: &Slice) -> &'a Self::Output {
        if self < slice.len() {
            unsafe { &*self.get_unchecked(slice) }
        } else {
            panic!("Index {} out of bounds.", self);
        }
    }
}
