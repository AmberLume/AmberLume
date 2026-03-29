use crate::alpaca::alpaca_header::AlpacaHeader;
use crate::alpaca::index_entry::{IndexEntry, ArchivedIndexEntry};
use anyhow::Result;
use memmap2::Mmap;
use rkyv::rancor::Error;
use rkyv::util::AlignedVec;
use rkyv::vec::ArchivedVec;
use rkyv::{access, deserialize};
use std::fs::File;
use std::path::PathBuf;

pub struct AlpacaReader {
    pub entries: Vec<IndexEntry>,

    mmap: Mmap,
}

impl AlpacaReader {
    pub fn parse(path: &PathBuf) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let header_slice = &mmap[0..AlpacaHeader::SIZE as usize];
        let header = AlpacaHeader::read(&header_slice)?;

        let entries_slice = &mmap
            [(header.index_offset as usize)..((header.index_offset + header.index_size) as usize)];
        let entries = Self::read_index(entries_slice)?;

        Ok(Self { entries, mmap })
    }

    pub fn read_slice(&self, entry: &IndexEntry) -> Result<&[u8]> {
        let slice_data = &self.mmap[entry.offset as usize..(entry.offset + entry.size) as usize];

        Ok(slice_data)
    }

    fn read_index(slice: &[u8]) -> Result<Vec<IndexEntry>> {
        let mut aligned = AlignedVec::<8>::new();
        aligned.extend_from_slice(slice);

        let archived = access::<ArchivedVec<ArchivedIndexEntry>, Error>(&aligned)?;

        let deserialized = deserialize::<Vec<IndexEntry>, Error>(archived)?;

        Ok(deserialized)
    }
}
