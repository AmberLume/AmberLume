use crate::alpaca::alpaca::Alpaca;
use crate::alpaca::alpaca_header::AlpacaHeader;
use crate::alpaca::alpaca_index_entry::AlpacaIndexEntry;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct AlpacaWriter {
    pub name: String,
    pub path: PathBuf,
    pub align: u64,

    file: File,
    entries: Vec<AlpacaIndexEntry>,
}

impl AlpacaWriter {
    pub const VERSION: u32 = 1;

    pub fn create(name: &str, path: &Path, align: u64) -> Result<Self> {
        let file_name = format!("{}.{}", name, Alpaca::EXTENSION);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path.join(file_name))?;

        file.seek(SeekFrom::Start(AlpacaHeader::SIZE))?;

        Ok(Self {
            name: name.to_string(),
            path: path.to_path_buf(),
            align,

            file,
            entries: Vec::new(),
        })
    }

    pub fn push(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let offset = Self::align_offset(self.file.stream_position()?, self.align);

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;

        let normalized_name = name.replace('\\', "/");

        let entry = AlpacaIndexEntry {
            name: normalized_name,
            offset,
            size: data.len() as u64,
        };

        self.entries.push(entry);

        Ok(())
    }

    pub fn pack(&mut self) -> Result<()> {
        let (index_offset, index_size) = self.write_index()?;

        self.write_header(index_offset, index_size)?;

        Ok(())
    }

    fn write_index(&mut self) -> Result<(u64, u64)> {
        let index_offset = Self::align_offset(self.file.stream_position()?, self.align);
        self.file.seek(SeekFrom::Start(index_offset))?;

        let encoded_index = to_bytes::<Error>(&self.entries)?;
        self.file.write_all(&encoded_index)?;

        let index_size = encoded_index.len() as u64;

        Ok((index_offset, index_size))
    }

    fn write_header(&mut self, index_offset: u64, index_size: u64) -> Result<()> {
        let header = AlpacaHeader::new(Self::VERSION, index_offset, index_size);

        header.write(&mut self.file)?;

        Ok(())
    }

    #[inline]
    pub fn align_offset(offset: u64, align: u64) -> u64 {
        (offset + align - 1) & !(align - 1)
    }
}
