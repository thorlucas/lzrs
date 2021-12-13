//! This module provides buffer structures to be used for a variety of purposes, such as the back
//! end of a dictionary.

mod raw;
pub mod ringbuf;

pub mod prelude {
    pub use super::{raw::*, ringbuf::RingBuf, Buffer};
}

pub trait Buffer {}

/// See [`std::slice::SliceIndex`].
pub unsafe trait BufferIndex<T> {
    type Output: ?Sized;

    fn get(self, buffer: &T) -> Option<&Self::Output>;

    unsafe fn get_unchecked(self, buffer: &T) -> *const Self::Output;

    #[track_caller]
    fn index(self, buffer: &T) -> &Self::Output;
}
