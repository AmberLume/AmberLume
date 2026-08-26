use index_allocator::ResourceLimits;
use gpu::ResourceFactories;
use gpu::ManagedAccelerationStructureDescriptorSet;
use crate::blas::BLAS;
use crate::blas_request_queue::BLASRequestQueue;
use gpu::RayTracingContext;
use crate::tlas::TLAS;
use resource_store::ResourceBuffers;
use anyhow::Result;
use ash::vk::DeviceSize;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub struct RayTracing {
    pub resource_factories: Arc<ResourceFactories>,

    pub context: RayTracingContext,

    pub blas: BLAS,
    pub tlas: Vec<TLAS>,
}

impl RayTracing {
    pub fn new(
        frames_in_flight: u32,
        resource_limits: ResourceLimits,
        context: RayTracingContext,
        resource_factories: Arc<ResourceFactories>,
        request_queue: Arc<BLASRequestQueue>,
        frame_counter: Arc<AtomicU64>,
        resource_buffers: &ResourceBuffers,
        acceleration_structures_descriptor_set: &Option<ManagedAccelerationStructureDescriptorSet>,
    ) -> Result<Self> {
        let blas = BLAS::new(
            frames_in_flight,
            resource_limits,
            resource_factories.clone(),
            request_queue,
            frame_counter,
            resource_buffers,
        )?;

        let tlas = (0..frames_in_flight)
            .map(|frame_index| {
                TLAS::new(
                    frame_index,
                    resource_limits,
                    &context,
                    &resource_factories,
                    acceleration_structures_descriptor_set,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            resource_factories,

            context,

            blas,
            tlas,
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.blas.destroy(resource_factories)?;

        for tlas in self.tlas {
            tlas.destroy(resource_factories)?;
        }

        Ok(())
    }
}

pub fn align_up(value: DeviceSize, alignment: DeviceSize) -> DeviceSize {
    (value + alignment - 1) & !(alignment - 1)
}
