use rkyv::rend::u32_le;
use std::slice::from_raw_parts;
use rkyv::primitive::ArchivedF32;

pub fn as_u32_slice(slice: &[u32_le]) -> &[u32] {
    unsafe { from_raw_parts(slice.as_ptr() as *const u32, slice.len()) }
}

pub fn as_f32_slice<const N: usize>(arr: &[ArchivedF32; N]) -> [f32; N] {
    arr.map(|v| v.into())
}
