use crate::alpaca::alpaca_header::AlpacaHeader;
use crate::alpaca::index_entry::{IndexEntry, ArchivedIndexEntry};
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::util::AlignedVec;
use rkyv::vec::ArchivedVec;
use rkyv::{access, deserialize};
use crate::unpacker::asset_data::AssetData;

pub struct AlpacaReader {
    pub entries: Vec<IndexEntry>,

    asset: Box<dyn AssetData>,
}

impl AlpacaReader {
    pub fn parse(asset: Box<dyn AssetData>) -> Result<Self> {
        let bytes = asset.bytes();

        let header_slice = &bytes[0..AlpacaHeader::SIZE as usize];
        let header = AlpacaHeader::read(&header_slice)?;

        let slice_start = header.index_offset as usize;
        let slice_end = (header.index_offset + header.index_size) as usize;

        let entries_slice = &bytes[slice_start..slice_end];
        let entries = Self::read_index(entries_slice)?;

        Ok(Self { entries, asset })
    }

    pub fn read_slice(&self, entry: &IndexEntry) -> Result<&[u8]> {
        let bytes = self.asset.bytes();
        let slice_data = &bytes[entry.offset as usize..(entry.offset + entry.size) as usize];

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
