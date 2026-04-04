use crate::ids::FrameIndex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::render_graph::pass::Pass;
use crate::render::statistics::interval::gpu_interval_measurement::GpuIntervalMeasurement;
use crate::statistics::time_measurement::TimeMeasurement;
use anyhow::Result;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub struct PassStatistics<P: Pass> {
    pub prepare: u64,
    pub collect_render_commands: u64,

    pub dispatch: u64,

    pub meta: <P as Pass>::Statistics,
}

pub struct PassStatisticsMeasurement {
    pub prepare: TimeMeasurement,
    pub collect_render_commands: TimeMeasurement,

    pub dispatch_measurement: GpuIntervalMeasurement,
}

impl PassStatisticsMeasurement {
    pub fn new(
        label: &str,
        device_context: &DeviceContext,
        resource_factories: &ResourceFactories,
        renderer_limits: &RendererLimits,
    ) -> Result<Self> {
        let dispatch_measurement = GpuIntervalMeasurement::new(
            &device_context,
            label,
            &resource_factories.query_pool_factory,
            &resource_factories.buffer_factory,
            renderer_limits.frames_in_flight,
        )?;

        Ok(Self {
            prepare: TimeMeasurement::new(),
            collect_render_commands: TimeMeasurement::new(),

            dispatch_measurement,
        })
    }
    
    pub fn collect<P: Pass>(&self, pass: &P, frame_index: FrameIndex) -> PassStatistics<P> {
        PassStatistics {
            prepare: self.prepare.collect(),
            collect_render_commands: self.collect_render_commands.collect(),

            dispatch: self.dispatch_measurement.collect(frame_index),

            meta: pass.statistics(frame_index),
        }
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.dispatch_measurement.destroy(&buffer_factory)?;

        Ok(())
    }
}
