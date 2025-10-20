use crate::resource::slice_owner::SliceOwner;
use memmap2::Mmap;
use std::fs::{File, metadata, read};
use std::path::PathBuf;

pub struct ResourceIO;

impl ResourceIO {
    const SMALL: u64 = 64 * 1024;

    pub fn new() -> Self {
        Self
    }

    pub fn read(&self, path: &PathBuf) -> SliceOwner {
        let meta = metadata(path.clone()).unwrap();

        if meta.len() <= Self::SMALL {
            let data = read(path).unwrap();

            SliceOwner::Vec { data }
        } else {
            let file = File::open(path).unwrap();

            let data = unsafe { Mmap::map(&file).unwrap() };

            SliceOwner::Mmap { data }
        }
    }
}
