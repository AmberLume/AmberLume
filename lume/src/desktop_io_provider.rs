use amber_lume::resources::io_provider::IOProvider;
use std::fs::read;
use std::path::Path;

// pub enum SliceOwner {
//     Mmap { data: Mmap },
//     Vec { data: Vec<u8> },
// }
//
// impl SliceOwner {
//     #[inline]
//     pub fn as_slice(&self) -> &[u8] {
//         match self {
//             SliceOwner::Mmap { data } => &data[..],
//             SliceOwner::Vec { data } => data,
//         }
//     }
// }

pub struct DesktopIOProvider;

impl DesktopIOProvider {
    // const SMALL: u64 = 64 * 1024;

    pub fn new() -> Self {
        Self {}
    }
}

impl IOProvider for DesktopIOProvider {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        // let meta = metadata(path).unwrap();
        //
        // if meta.len() <= Self::SMALL {
        Some(read(path).unwrap())
        // } else {
        //     let file = File::open(path).unwrap();
        //
        //     let data = unsafe { Mmap::map(&file).unwrap() };
        //
        //     Some(data)
        // }
    }
}
