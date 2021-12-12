//! This module provides buffer structures to be used for a variety of purposes, such as the back
//! end of a dictionary.

pub struct RingBuf {
    buf: Box<[u8]>,
    head: usize,
}

impl RingBuf {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: {
                let mut v = Vec::with_capacity(capacity);
                unsafe {
                    v.set_len(capacity);
                }
                v.into_boxed_slice()
            },
            head: 0,
        }
    }
}
