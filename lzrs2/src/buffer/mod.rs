//! This module provides buffer structures to be used for a variety of purposes, such as the back
//! end of a dictionary.

use std::ops;

pub mod raw;
pub mod ringbuf;

pub mod prelude {
    pub use super::{raw::*, ringbuf::RingBuf, Buffer};
}

pub trait Buffer: ops::Index<usize> {
    fn get(&self, index: usize) -> Option<&u8>;
    unsafe fn get_unchecked(&self, index: usize) -> *const u8;
}

/// See [`std::slice::SliceIndex`].
pub unsafe trait SliceIndex<T: ?Sized> {
    type Output: ?Sized;

    fn get(self, slice: &T) -> Option<&Self::Output>;

    unsafe fn get_unchecked(self, slice: *const T) -> *const Self::Output;

    #[track_caller]
    fn index(self, slice: &T) -> &Self::Output;
}
