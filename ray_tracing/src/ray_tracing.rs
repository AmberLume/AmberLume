use index_allocator::ResourceLimits;
use gpu::ResourceFactories;
use crate::acceleration_structure_factory::AccelerationStructureFactory;
use crate::blas::BLAS;
use crate::blas_request_queue::BLASRequestQueue;
use crate::rt_limits::RTLimits;
use crate::tlas::TLAS;
use gpu::DebugUtils;
use resource_store::ResourceBuffers;
use anyhow::Result;
use ash::khr::acceleration_structure::Device as AccelerationStructureDevice;
use ash::vk::DeviceSize;
use ash::{Device, Instance};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub struct RayTracing {
    pub resource_factories: Arc<ResourceFactories>,

    pub as_loader: AccelerationStructureDevice,
    pub rt_limits: RTLimits,
    pub factory: Arc<AccelerationStructureFactory>,

    pub blas: BLAS,
    pub tlas: Vec<TLAS>,
}

impl RayTracing {
    pub fn new(
        frames_in_flight: u32,
        resource_limits: ResourceLimits,
        rt_limits: RTLimits,
        instance: &Instance,
        device: &Device,
        debug_utils: Arc<DebugUtils>,
        resource_factories: Arc<ResourceFactories>,
        request_queue: Arc<BLASRequestQueue>,
        frame_counter: Arc<AtomicU64>,
        resource_buffers: &ResourceBuffers,
    ) -> Result<Self> {
        let as_loader = AccelerationStructureDevice::new(instance, device);
        let factory = Arc::new(AccelerationStructureFactory::new(as_loader.clone(), debug_utils));

        let blas = BLAS::new(
            frames_in_flight,
            resource_limits,
            rt_limits,
            &as_loader,
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
                    &rt_limits,
                    &as_loader,
                    &factory,
                    &resource_factories.buffer_factory,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            resource_factories,

            as_loader,
            rt_limits,
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
