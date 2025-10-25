#[repr(C)]
#[derive(Clone, Copy)]
pub struct AlpacaHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub index_offset: u64,
    pub index_size: u64,
    pub flags: u32,
    pub reserved: [u8; 36],
}
