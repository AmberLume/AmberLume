use crate::alpaca::alpaca_header::AlpacaHeader;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Seek, SeekFrom, Write};

impl AlpacaHeader {
    pub const MAGIC: [u8; 4] = *b"ALP\0";
    pub const SIZE: u64 = 64;

    pub fn new(version: u32, index_offset: u64, index_size: u64) -> Self {
        assert_eq!(size_of::<AlpacaHeader>() as u64, Self::SIZE);

        Self {
            magic: Self::MAGIC,
            version,
            index_offset,
            index_size,
            flags: 0,
            reserved: [0; 36],
        }
    }

    pub fn write<W: Write + Seek>(&self, mut writer: W) -> anyhow::Result<()> {
        writer.seek(SeekFrom::Start(0))?;

        writer.write_all(&self.magic)?;
        writer.write_u32::<LittleEndian>(self.version)?;
        writer.write_u64::<LittleEndian>(self.index_offset)?;
        writer.write_u64::<LittleEndian>(self.index_size)?;
        writer.write_u32::<LittleEndian>(self.flags)?;
        writer.write_all(&self.reserved)?;

        Ok(())
    }
}
