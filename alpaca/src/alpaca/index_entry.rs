use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndexEntry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
}
