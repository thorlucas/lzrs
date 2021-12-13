//! This module provides buffer structures to be used for a variety of purposes, such as the back
//! end of a dictionary.

pub mod raw;
pub mod ringbuf;

pub mod prelude {
    pub use super::{raw::*, ringbuf::RingBuf, Buffer};
}

pub trait Buffer {
    fn get<I>(&self, index: I) -> Option<&<I as BufferIndex<Self>>::Output>
    where
        I: BufferIndex<Self>,
    {
        index.get(self)
    }

    unsafe fn get_unchecked<I>(&self, index: I) -> *const <I as BufferIndex<Self>>::Output
    where
        I: BufferIndex<Self>,
    {
        index.get_unchecked(self)
    }
}

/// See [`std::slice::SliceIndex`].
pub unsafe trait BufferIndex<T: ?Sized> {
    type Output: ?Sized;

    fn get(self, buffer: &T) -> Option<&Self::Output>;

    unsafe fn get_unchecked(self, buffer: &T) -> *const Self::Output;

    #[track_caller]
    fn index(self, buffer: &T) -> &Self::Output;
}
