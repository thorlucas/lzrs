#[inline(always)]
pub fn read_u64_inline(buf: &[u8], index: usize) -> u64 {
    let bytes: [u8; 8] = buf[index..index+8].try_into().unwrap();
    u64::from_le_bytes(bytes)
}

/*
#[inline(never)]
pub fn read_u64_non_inline(buf: &[u8], index: usize) -> u64 {
    read_u64_inline(buf, index)
}
*/

/*
#[inline(always)]
pub fn read_u64_unsafe_inline(buf: &[u8], index: usize) -> u64 {
    let ptr: *const [u8] = &buf[index..index+8];
    let ptr: *const u64 = ptr as *const u64;
    unsafe { *ptr }
}
*/

/*
#[cfg(test)]
#[test]
pub fn test_read_u64_unsafe() {
    let bytes: &[u8; 8] = b"foo&bar!";
    let le = u64::from_le_bytes(*bytes);
    let be = u64::from_be_bytes(*bytes);

    let read = read_u64_unsafe_inline(bytes, 0);
    assert!(be == read || le == read);
}
*/

#[inline(always)]
pub fn read_u8_inline(buf: &[u8], index: usize) -> u8 {
    buf[index]
}

/*
#[inline(never)]
pub fn read_u8_non_inline(buf: &[u8], index: usize) -> u8 {
    read_u8_inline(buf, index)
}
*/

#[inline(always)]
pub fn write_u64_inline(src: u64, dst: &mut[u8], index: usize) {
    dst[index..index+8].copy_from_slice(
        &u64::to_le_bytes(src)
    );
}
