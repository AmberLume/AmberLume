use index_allocator::ArcUnwrapOrErr;
use index_allocator::DeferredDestroy;
use index_allocator::ResourceLimits;
use gpu::ResourceFactories;
use gpu::ManagedAccelerationStructureDescriptorSet;
use gpu::RayTracingContext;
use crate::blas_cache::BlasCache;
use crate::tlas::TLAS;
use anyhow::Result;
use ash::vk::DeviceSize;
use std::sync::Arc;

pub struct RayTracing {
    pub context: RayTracingContext,

    pub tlas: Vec<Arc<TLAS>>,
    pub blas_cache: Arc<BlasCache>,
}

impl RayTracing {
    pub fn new(
        context: &RayTracingContext,
        resource_factories: Arc<ResourceFactories>,
        deferred_destroy: Arc<DeferredDestroy>,
        frames_in_flight: u32,
        resource_limits: ResourceLimits,
        acceleration_structures_descriptor_set: &Option<ManagedAccelerationStructureDescriptorSet>,
    ) -> Result<Self> {
        let tlas = (0..frames_in_flight)
            .map(|frame_index| {
                TLAS::new(
                    frame_index,
                    resource_limits,
                    context,
                    &resource_factories,
                    acceleration_structures_descriptor_set,
                )
                .map(Arc::new)
            })
            .collect::<Result<Vec<_>>>()?;

        let blas_cache = Arc::new(BlasCache::new(
            resource_factories,
            deferred_destroy,
            resource_limits,
        ));

        Ok(Self {
            context: context.clone(),

            tlas,
            blas_cache,
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for tlas in self.tlas {
            tlas.try_unwrap()?.destroy(resource_factories)?;
        }

        self.blas_cache.try_unwrap()?.retire_all();

        Ok(())
    }
}

pub fn align_up(value: DeviceSize, alignment: DeviceSize) -> DeviceSize {
    (value + alignment - 1) & !(alignment - 1)
}
