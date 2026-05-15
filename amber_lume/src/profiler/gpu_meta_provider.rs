use std::any::Any;
use std::sync::Arc;

use anyhow::Result;
use bytemuck::Pod;

use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::statistics::meta::meta_statistics::MetaStatistics;
use crate::utils::arc_utils::ArcUnwrapOrErr;

pub trait GpuMetaProvider: Send + Sync {
    fn read(&self, frame_index: FrameIndex) -> Box<dyn Any + Send + Sync>;

    fn destroy(self: Arc<Self>, resource_factories: &ResourceFactories) -> Result<()>;
}

impl<T: Pod + Send + Sync + 'static> GpuMetaProvider for MetaStatistics<T> {
    fn read(&self, frame_index: FrameIndex) -> Box<dyn Any + Send + Sync> {
        Box::new(self.collect(frame_index).to_vec())
    }

    fn destroy(self: Arc<Self>, resource_factories: &ResourceFactories) -> Result<()> {
        self.try_unwrap()?
            .destroy(&resource_factories.buffer_factory)
    }
}
