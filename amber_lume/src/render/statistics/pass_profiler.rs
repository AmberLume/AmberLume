use std::collections::HashMap;
use crate::render::device::device_context::DeviceContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::statistics::interval::gpu_interval_measurement::GpuIntervalMeasurement;
use crate::statistics::time_measurement::TimeMeasurement;
use anyhow::Result;
use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::render_graph::pass::Pass;

pub struct PassProfiler {
    profiles: HashMap<String, PassProfileMeasurement>,
}

struct PassProfileMeasurement {
    pub prepare_data: TimeMeasurement,
    pub record_commands: TimeMeasurement,

    pub dispatch_time: GpuIntervalMeasurement,
}

impl PassProfileMeasurement {
    pub fn new(
        name: &str,
        device_context: &DeviceContext,
        resource_factories: &ResourceFactories,
        frame_count: u32,
    ) -> Result<Self> {
        let prepare_data = TimeMeasurement::new();
        let record_commands = TimeMeasurement::new();

        let dispatch_time = GpuIntervalMeasurement::new(
            &device_context,
            &format!("{}_dispatch", name),
            &resource_factories.query_pool_factory,
            &resource_factories.buffer_factory,
            frame_count,
        )?;

        Ok(Self {
            prepare_data,
            record_commands,

            dispatch_time,
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        drop(self.prepare_data);
        drop(self.record_commands);

        self.dispatch_time.destroy(&resource_factories.buffer_factory)
    }
}

pub struct PassProfile {
    pub name: String,

    pub prepare_data: u64,
    pub record_commands: u64,

    pub dispatch_time: u64,
}

impl PassProfiler {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        name: String,
        device_context: &DeviceContext,
        resource_factories: &ResourceFactories,
        frame_count: u32,
    ) -> Result<()> {
        let profile = PassProfileMeasurement::new(
            &name,
            &device_context,
            &resource_factories,
            frame_count,
        )?;

        self.profiles.insert(name, profile);

        Ok(())
    }

    pub fn prepare_start<P: Pass>(&mut self, pass: &P) {
        self.profiles
            .get(&pass.name()).unwrap()
            .prepare_data
            .start();
    }

    pub fn prepare_finish<P: Pass>(&mut self, pass: &P) {
        self.profiles
            .get(&pass.name()).unwrap()
            .prepare_data
            .finish();
    }

    pub fn record_commands_start<P: Pass>(&mut self, pass: &P) {
        self.profiles
            .get(&pass.name()).unwrap()
            .record_commands
            .start();
    }

    pub fn record_commands_finish<P: Pass>(&mut self, pass: &P) {
        self.profiles
            .get(&pass.name()).unwrap()
            .record_commands
            .finish();
    }

    pub fn dispatch_start<P: Pass>(&mut self, pass: &P, context: &PassContext) {
        self.profiles
            .get(&pass.name()).unwrap()
            .dispatch_time
            .record_start(context.command_recording.command_buffer, context.frame_index, 0);
    }

    pub fn dispatch_finish<P: Pass>(&mut self, pass: &P, context: &PassContext) {
        self.profiles
            .get(&pass.name()).unwrap()
            .dispatch_time
            .record_end(context.command_recording.command_buffer, context.frame_index, 0);
    }

    pub fn collect(&self, frame_index: FrameIndex) -> Vec<PassProfile> {
        self.profiles.iter().map(|(name, profile)| {
            PassProfile {
                name: name.clone(),

                prepare_data: profile.prepare_data.collect(),
                record_commands: profile.record_commands.collect(),

                dispatch_time: profile.dispatch_time.collect(frame_index),
            }
        }).collect()
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for (_, profile) in self.profiles {
            profile.destroy(&resource_factories)?;
        }

        Ok(())
    }
}
