use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub struct AlpacaIndexEntry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
}
