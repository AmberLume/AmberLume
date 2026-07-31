use alpaca::unpacker::asset_data::AssetData;
use crate::asset_bytes::AssetBytes;

pub struct AssetBytesSource {
    asset: Box<dyn AssetBytes>,
}

impl AssetBytesSource {
    pub fn new(asset: Box<dyn AssetBytes>) -> Self {
        Self { asset }
    }
}

impl AssetData for AssetBytesSource {
    fn bytes(&self) -> &[u8] {
        self.asset.bytes()
    }
}
