use index_allocator::ArcUnwrapOrErr;
use index_allocator::ResourceLimits;
use gpu::ResourceFactories;
use gpu::ManagedAccelerationStructureDescriptorSet;
use crate::blas::BLAS;
use gpu::RayTracingContext;
use crate::tlas::TLAS;
use resource_store::ResourceBuffers;
use anyhow::Result;
use ash::vk::DeviceSize;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub struct RayTracing {
    pub context: RayTracingContext,

    pub blas: Arc<BLAS>,
    pub tlas: Vec<Arc<TLAS>>,
}

impl RayTracing {
    pub fn new(
        frames_in_flight: u32,
        resource_limits: ResourceLimits,
        context: RayTracingContext,
        resource_factories: Arc<ResourceFactories>,
        frame_counter: Arc<AtomicU64>,
        resource_buffers: &ResourceBuffers,
        acceleration_structures_descriptor_set: &Option<ManagedAccelerationStructureDescriptorSet>,
    ) -> Result<Self> {
        let blas = Arc::new(BLAS::new(
            frames_in_flight,
            resource_limits,
            resource_factories.clone(),
            frame_counter,
            resource_buffers,
        )?);

        let tlas = (0..frames_in_flight)
            .map(|frame_index| {
                TLAS::new(
                    frame_index,
                    resource_limits,
                    &context,
                    &resource_factories,
                    acceleration_structures_descriptor_set,
                )
                .map(Arc::new)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            context,

            blas,
            tlas,
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.blas.try_unwrap()?.destroy(resource_factories)?;

        for tlas in self.tlas {
            tlas.try_unwrap()?.destroy(resource_factories)?;
        }

        Ok(())
    }
}

pub fn align_up(value: DeviceSize, alignment: DeviceSize) -> DeviceSize {
    (value + alignment - 1) & !(alignment - 1)
}
