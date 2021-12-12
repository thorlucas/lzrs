#[inline(always)]
pub fn read_u64_inline(buf: &[u8], index: usize) -> u64 {
    let bytes: [u8; 8] = buf[index..index+8].try_into().unwrap();
    u64::from_le_bytes(bytes)
}

#[inline(never)]
pub fn read_u64_non_inline(buf: &[u8], index: usize) -> u64 {
    read_u64_inline(buf, index)
}

#[inline(always)]
pub fn read_u8_inline(buf: &[u8], index: usize) -> u8 {
    buf[index]
}

#[inline(never)]
pub fn read_u8_non_inline(buf: &[u8], index: usize) -> u8 {
    read_u8_inline(buf, index)
}
