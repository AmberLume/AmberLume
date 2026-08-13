use anyhow::Result;

pub trait ResourceReader: Send + Sync {
    fn get_resource(&self, name: &str) -> Result<&[u8]>;
}
