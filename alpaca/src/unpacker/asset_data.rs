pub trait AssetData: Send + Sync {
    fn bytes(&self) -> &[u8];
}
