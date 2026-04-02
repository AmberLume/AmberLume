use crate::render::pass::culling_indirect::culling_indirect_pass::CullingIndirectPass;
use crate::render::pass::depth::depth_render_pass::DepthPass;
use crate::render::pass::main::main_render_pass::MainPass;
use crate::render::pass::physics_debug::physics_debug_render_pass::PhysicsDebugPass;
use crate::render::pass::shadow_mask::shadow_mask_render_pass::ShadowMaskPass;
use crate::render::pass::shadows::shadows_render_pass::ShadowsPass;
use crate::render::pass::ui::ui_render_pass::UiPass;
use anyhow::Result;
use crate::ids::FrameIndex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_statistics::PassStatisticsMeasurement;
use crate::render::pass::passes_statistics::PassesStatistics;
use crate::render::pass::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::statistics::interval::gpu_interval_measurement::GpuIntervalMeasurement;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;

pub struct PassRegistry {
    total_dispatch_measurement: GpuIntervalMeasurement,

    culling_indirect_statistics_measurement: PassStatisticsMeasurement,
    culling_indirect_pass: CullingIndirectPass,
    
    depth_statistics_measurement: PassStatisticsMeasurement,
    depth_pass: DepthPass,
    
    shadows_statistics_measurement: PassStatisticsMeasurement,
    shadows_pass: ShadowsPass,
    
    shadow_mask_statistics_measurement: PassStatisticsMeasurement,
    shadow_mask_pass: ShadowMaskPass,
    
    main_statistics_measurement: PassStatisticsMeasurement,
    main_pass: MainPass,
    
    physics_debug_statistics_measurement: PassStatisticsMeasurement,
    physics_debug_pass: PhysicsDebugPass,
    
    ui_statistics_measurement: PassStatisticsMeasurement,
    ui_pass: UiPass,
}

impl PassRegistry {
    pub fn create(
        device_context: &DeviceContext,
        resource_factories: &ResourceFactories,
        renderer_limits: &RendererLimits,
        culling_indirect_pass: CullingIndirectPass,
        depth_pass: DepthPass,
        shadows_pass: ShadowsPass,
        shadow_mask_pass: ShadowMaskPass,
        main_pass: MainPass,
        physics_debug_pass: PhysicsDebugPass,
        ui_pass: UiPass,
    ) -> Result<Self> {
        let total_dispatch_measurement = GpuIntervalMeasurement::new(
            &device_context,
            "total_dispatch",
            &resource_factories.query_pool_factory,
            &resource_factories.buffer_factory,
            renderer_limits.frames_in_flight,
        )?;

        Ok(Self {
            total_dispatch_measurement,

            culling_indirect_statistics_measurement: PassStatisticsMeasurement::new("culling_indirect", &device_context, &resource_factories, &renderer_limits)?,
            culling_indirect_pass,
            
            depth_statistics_measurement: PassStatisticsMeasurement::new("depth", &device_context, &resource_factories, &renderer_limits)?,
            depth_pass,
            
            shadows_statistics_measurement: PassStatisticsMeasurement::new("shadow", &device_context, &resource_factories, &renderer_limits)?,
            shadows_pass,
            
            shadow_mask_statistics_measurement: PassStatisticsMeasurement::new("shadow_mask", &device_context, &resource_factories, &renderer_limits)?,
            shadow_mask_pass,
            
            main_statistics_measurement: PassStatisticsMeasurement::new("main", &device_context, &resource_factories, &renderer_limits)?,
            main_pass,
            
            physics_debug_statistics_measurement: PassStatisticsMeasurement::new("physics_debug", &device_context, &resource_factories, &renderer_limits)?,
            physics_debug_pass,
            
            ui_statistics_measurement: PassStatisticsMeasurement::new("ui", &device_context, &resource_factories, &renderer_limits)?,
            ui_pass,
        })
    }
    
    pub fn run_each(
        &self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        image_state_tracker: &mut ImageStateTracker,
    ) -> Result<()> {
        self.total_dispatch_measurement.record_start(
            pass_context.command_recording.command_buffer,
            pass_context.frame_index,
            0,
        );

        self.run_pass(&self.culling_indirect_pass, &self.culling_indirect_statistics_measurement, frame_data_context, pass_context, image_state_tracker)?;
        self.run_pass(&self.depth_pass, &self.depth_statistics_measurement, frame_data_context, pass_context, image_state_tracker)?;
        self.run_pass(&self.shadows_pass, &self.shadows_statistics_measurement, frame_data_context, pass_context, image_state_tracker)?;
        self.run_pass(&self.shadow_mask_pass, &self.shadow_mask_statistics_measurement, frame_data_context, pass_context, image_state_tracker)?;
        self.run_pass(&self.main_pass, &self.main_statistics_measurement, frame_data_context, pass_context, image_state_tracker)?;
        self.run_pass(&self.physics_debug_pass, &self.physics_debug_statistics_measurement, frame_data_context, pass_context, image_state_tracker)?;
        self.run_pass(&self.ui_pass, &self.ui_statistics_measurement, frame_data_context, pass_context, image_state_tracker)?;

        self.total_dispatch_measurement.record_end(
            pass_context.command_recording.command_buffer,
            pass_context.frame_index,
            0,
        );

        Ok(())
    }

    fn run_pass<P: Pass>(
        &self,
        pass: &P,
        statistics_measurement: &PassStatisticsMeasurement,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        image_state_tracker: &mut ImageStateTracker,
    ) -> Result<()> {
        let is_enabled = pass.is_enabled();

        if is_enabled {
            statistics_measurement.dispatch_measurement.record_start(
                pass_context.command_recording.command_buffer,
                pass_context.frame_index,
                0,
            );

            statistics_measurement.prepare.start();
            let data = pass.prepare_data(&frame_data_context)?;
            statistics_measurement.prepare.finish();

            statistics_measurement.collect_render_commands.start();
            pass.record_commands(&pass_context, image_state_tracker, data)?;
            statistics_measurement.collect_render_commands.finish();

            statistics_measurement.dispatch_measurement.record_end(
                pass_context.command_recording.command_buffer,
                pass_context.frame_index,
                0,
            );
        }

        Ok(())
    }
    
    pub fn statistics(&self, frame_index: FrameIndex) -> PassesStatistics {
        PassesStatistics {
            total_dispatch: self.total_dispatch_measurement.collect(frame_index),

            culling: self.culling_indirect_statistics_measurement.collect(&self.culling_indirect_pass, frame_index),
            depth: self.depth_statistics_measurement.collect(&self.depth_pass, frame_index),
            shadows: self.shadows_statistics_measurement.collect(&self.shadows_pass, frame_index),
            shadow_mask: self.shadow_mask_statistics_measurement.collect(&self.shadow_mask_pass, frame_index),
            main: self.main_statistics_measurement.collect(&self.main_pass, frame_index),
            physics_debug: self.physics_debug_statistics_measurement.collect(&self.physics_debug_pass, frame_index),
            ui: self.ui_statistics_measurement.collect(&self.ui_pass, frame_index),
        }
    }
    
    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.total_dispatch_measurement.destroy(&resource_factories.buffer_factory)?;

        self.culling_indirect_statistics_measurement.destroy(&resource_factories.buffer_factory)?;
        self.culling_indirect_pass.destroy(&resource_factories)?;

        self.depth_statistics_measurement.destroy(&resource_factories.buffer_factory)?;
        self.depth_pass.destroy(&resource_factories)?;

        self.shadows_statistics_measurement.destroy(&resource_factories.buffer_factory)?;
        self.shadows_pass.destroy(&resource_factories)?;

        self.shadow_mask_statistics_measurement.destroy(&resource_factories.buffer_factory)?;
        self.shadow_mask_pass.destroy(&resource_factories)?;

        self.main_statistics_measurement.destroy(&resource_factories.buffer_factory)?;
        self.main_pass.destroy(&resource_factories)?;

        self.physics_debug_statistics_measurement.destroy(&resource_factories.buffer_factory)?;
        self.physics_debug_pass.destroy(&resource_factories)?;

        self.ui_statistics_measurement.destroy(&resource_factories.buffer_factory)?;
        self.ui_pass.destroy(&resource_factories)?;

        Ok(())
    }
}
