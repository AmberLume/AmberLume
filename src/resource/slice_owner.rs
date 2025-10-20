use memmap2::Mmap;

pub enum SliceOwner {
    Mmap { data: Mmap },
    Vec { data: Vec<u8> },
}

impl SliceOwner {
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SliceOwner::Mmap { data } => &data[..],
            SliceOwner::Vec { data } => data,
        }
    }
}
