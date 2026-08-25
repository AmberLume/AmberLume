use index_allocator::ResourceLimits;
use gpu::ResourceFactories;
use crate::acceleration_structure_factory::AccelerationStructureFactory;
use crate::blas::BLAS;
use crate::blas_request_queue::BLASRequestQueue;
use gpu::RayTracingContext;
use crate::tlas::TLAS;
use gpu::DebugUtils;
use resource_store::ResourceBuffers;
use anyhow::Result;
use ash::vk::DeviceSize;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub struct RayTracing {
    pub resource_factories: Arc<ResourceFactories>,

    pub context: RayTracingContext,
    pub factory: Arc<AccelerationStructureFactory>,

    pub blas: BLAS,
    pub tlas: Vec<TLAS>,
}

impl RayTracing {
    pub fn new(
        frames_in_flight: u32,
        resource_limits: ResourceLimits,
        context: RayTracingContext,
        debug_utils: Arc<DebugUtils>,
        resource_factories: Arc<ResourceFactories>,
        request_queue: Arc<BLASRequestQueue>,
        frame_counter: Arc<AtomicU64>,
        resource_buffers: &ResourceBuffers,
    ) -> Result<Self> {
        let factory = Arc::new(AccelerationStructureFactory::new(
            context.device.clone(),
            debug_utils,
        ));

        let blas = BLAS::new(
            frames_in_flight,
            resource_limits,
            factory.clone(),
            resource_factories.clone(),
            request_queue,
            frame_counter,
            resource_buffers,
        )?;

        let tlas = (0..frames_in_flight)
            .map(|_| {
                TLAS::new(
                    resource_limits,
                    &context,
                    &factory,
                    &resource_factories.buffer_factory,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            resource_factories,

            context,
            factory,

            blas,
            tlas,
        })
    }

    pub fn destroy(self) -> Result<()> {
        self.blas
            .destroy(&self.factory, &self.resource_factories.buffer_factory)?;

        for tlas in self.tlas {
            tlas.destroy(&self.factory, &self.resource_factories.buffer_factory)?;
        }

        Ok(())
    }
}

pub fn align_up(value: DeviceSize, alignment: DeviceSize) -> DeviceSize {
    (value + alignment - 1) & !(alignment - 1)
}
