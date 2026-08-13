use crate::alpaca::alpaca_header::AlpacaHeader;
use anyhow::{Result, bail};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

impl AlpacaHeader {
    pub fn read(slice: &[u8]) -> Result<Self> {
        if slice.len() != Self::SIZE as usize {
            bail!("Invalid slice size of alpaca header");
        }

        let mut cursor = Cursor::new(slice);

        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        let version = cursor.read_u32::<LittleEndian>()?;
        let index_offset = cursor.read_u64::<LittleEndian>()?;
        let index_size = cursor.read_u64::<LittleEndian>()?;
        let flags = cursor.read_u32::<LittleEndian>()?;

        let mut reserved = [0u8; 36];
        cursor.read_exact(&mut reserved)?;

        let header = Self {
            magic,
            version,
            index_offset,
            index_size,
            flags,
            reserved,
        };

        header.validate()?;

        Ok(header)
    }

    fn validate(&self) -> Result<()> {
        if self.magic != Self::MAGIC {
            bail!("Invalid alpaca magic");
        }

        if self.version != Self::VERSION {
            bail!(
                "Unsupported alpaca version {}, expected {}",
                self.version,
                Self::VERSION,
            );
        }

        Ok(())
    }
}
