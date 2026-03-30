use rkyv::primitive::ArchivedF32;

pub fn as_f32_slice<const N: usize>(arr: &[ArchivedF32; N]) -> [f32; N] {
    arr.map(|v| v.into())
}
