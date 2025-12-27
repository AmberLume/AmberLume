use anyhow::Result;

pub trait BufferWrapper {
    fn allocate_space(&self, count: usize) -> Result<u64>;

    fn destroy(&mut self) -> Result<()>;
}
