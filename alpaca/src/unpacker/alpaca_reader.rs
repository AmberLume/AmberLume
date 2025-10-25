use crate::alpaca::alpaca_header::AlpacaHeader;
use crate::alpaca::alpaca_index_entry::AlpacaIndexEntry;
use anyhow::Result;
use bincode::{config, decode_from_slice};
use memmap2::Mmap;
use std::fs::File;
use std::path::PathBuf;

pub struct AlpacaReader {
    pub header: AlpacaHeader,
    pub entries: Vec<AlpacaIndexEntry>,

    mmap: Mmap,
}

impl AlpacaReader {
    pub fn parse(path: PathBuf) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let header_slice = &mmap[0..AlpacaHeader::SIZE as usize];
        let header = AlpacaHeader::read(&header_slice)?;

        let entries_slice = &mmap
            [(header.index_offset as usize)..((header.index_offset + header.index_size) as usize)];
        let entries = Self::read_index(entries_slice)?;

        Ok(Self {
            header,
            entries,

            mmap,
        })
    }

    pub fn read_slice(&self, entry: &AlpacaIndexEntry) -> Result<&[u8]> {
        let slice_data = &self.mmap[entry.offset as usize..(entry.offset + entry.size) as usize];

        Ok(slice_data)
    }

    fn read_index(slice: &[u8]) -> Result<Vec<AlpacaIndexEntry>> {
        let config = config::standard();
        let (entries, _): (Vec<AlpacaIndexEntry>, _) = decode_from_slice(&slice, config)?;

        Ok(entries)
    }
}
