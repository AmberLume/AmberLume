use std::any::Any;
use std::sync::Arc;

use anyhow::Result;

use index_allocator::FrameIndex;
use crate::factories::resource_factories::ResourceFactories;

pub trait GpuMetaProvider: Send + Sync {
    fn read(&self, frame_index: FrameIndex) -> Box<dyn Any + Send + Sync>;

    fn destroy(self: Arc<Self>, resource_factories: &ResourceFactories) -> Result<()>;
}
