pub trait AssetBytes: Send + Sync {
    fn bytes(&self) -> &[u8];
}
