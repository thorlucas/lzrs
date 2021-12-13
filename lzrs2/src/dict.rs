use crate::buffer::prelude::*;

pub struct Dictionary<B> {
    buffer: B,
}

impl<B> Dictionary<B>
where
    B: Buffer,
{
    pub fn with_capacity(capacity: usize) {
        Self {}
    }
}
