use crate::render::pass::culling_indirect::culling_indirect_render_pass::CullingIndirectPass;
use crate::render::pass::depth::depth_render_pass::DepthPass;
use crate::render::pass::main::main_render_pass::MainPass;
use crate::render::pass::physics_debug::physics_debug_render_pass::PhysicsDebugPass;
use crate::render::pass::shadow_mask::shadow_mask_render_pass::ShadowMaskPass;
use crate::render::pass::shadows::shadows_render_pass::ShadowsPass;
use crate::render::pass::ui::ui_render_pass::UiPass;
use anyhow::Result;
use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_statistics::PassStatisticsMeasurement;
use crate::render::pass::passes_statistics::PassesStatistics;
use crate::render::pass::pass::Pass;
use crate::render::pass::pass_context::PassContext;

pub struct PassRegistry {
    culling_indirect_statistics_measurement: PassStatisticsMeasurement,
    pub culling_indirect_pass: CullingIndirectPass,
    
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
        culling_indirect_pass: CullingIndirectPass,
        depth_pass: DepthPass,
        shadows_pass: ShadowsPass,
        shadow_mask_pass: ShadowMaskPass,
        main_pass: MainPass,
        physics_debug_pass: PhysicsDebugPass,
        ui_pass: UiPass,
    ) -> Self {
        Self {
            culling_indirect_statistics_measurement: PassStatisticsMeasurement::new(),
            culling_indirect_pass,
            
            depth_statistics_measurement: PassStatisticsMeasurement::new(),
            depth_pass,
            
            shadows_statistics_measurement: PassStatisticsMeasurement::new(),
            shadows_pass,
            
            shadow_mask_statistics_measurement: PassStatisticsMeasurement::new(),
            shadow_mask_pass,
            
            main_statistics_measurement: PassStatisticsMeasurement::new(),
            main_pass,
            
            physics_debug_statistics_measurement: PassStatisticsMeasurement::new(),
            physics_debug_pass,
            
            ui_statistics_measurement: PassStatisticsMeasurement::new(),
            ui_pass,
        }
    }
    
    pub fn run_each(
        &self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
    ) -> Result<()> {
        self.run_pass(&self.culling_indirect_pass, &self.culling_indirect_statistics_measurement, frame_data_context, pass_context)?;
        self.run_pass(&self.depth_pass, &self.depth_statistics_measurement, frame_data_context, pass_context)?;
        self.run_pass(&self.shadows_pass, &self.shadows_statistics_measurement, frame_data_context, pass_context)?;
        self.run_pass(&self.shadow_mask_pass, &self.shadow_mask_statistics_measurement, frame_data_context, pass_context)?;
        self.run_pass(&self.main_pass, &self.main_statistics_measurement, frame_data_context, pass_context)?;
        self.run_pass(&self.physics_debug_pass, &self.physics_debug_statistics_measurement, frame_data_context, pass_context)?;
        self.run_pass(&self.ui_pass, &self.ui_statistics_measurement, frame_data_context, pass_context)?;
        
        Ok(())
    }

    fn run_pass<P: Pass>(
        &self,
        pass: &P,
        statistics_measurement: &PassStatisticsMeasurement,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
    ) -> Result<()> {
        let is_enabled = pass.is_enabled();

        if is_enabled {
            statistics_measurement.prepare.start();
            let data = pass.prepare_data(&frame_data_context)?;
            statistics_measurement.prepare.finish();

            statistics_measurement.collect_render_commands.start();
            pass.record_commands(&pass_context, data)?;
            statistics_measurement.collect_render_commands.finish();
        }

        Ok(())
    }
    
    pub fn statistics(&self, frame_index: FrameIndex) -> PassesStatistics {
        PassesStatistics {
            culling: self.culling_indirect_statistics_measurement.collect(),
            culling_meta: self.culling_indirect_pass.statistics(frame_index),
            depth: self.depth_statistics_measurement.collect(),
            shadows: self.shadows_statistics_measurement.collect(),
            shadow_mask: self.shadow_mask_statistics_measurement.collect(),
            main: self.main_statistics_measurement.collect(),
            physics_debug: self.physics_debug_statistics_measurement.collect(),
            ui: self.ui_statistics_measurement.collect(),
        }
    }
    
    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.culling_indirect_pass.destroy(&resource_factories)?;
        self.depth_pass.destroy(&resource_factories)?;
        self.shadows_pass.destroy(&resource_factories)?;
        self.shadow_mask_pass.destroy(&resource_factories)?;
        self.main_pass.destroy(&resource_factories)?;
        self.physics_debug_pass.destroy(&resource_factories)?;
        self.ui_pass.destroy(&resource_factories)?;
        
        Ok(())
    }
}
