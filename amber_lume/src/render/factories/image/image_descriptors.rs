#[derive(Clone)]
pub struct ImageDescriptors {
    pub full: Option<u32>,
    pub storage_mips: Option<Vec<u32>>,
}
