use crate::render::render_pass::culling_indirect::culling_indirect_render_pass::CullingIndirectRenderPass;
use crate::render::render_pass::depth::depth_render_pass::DepthRenderPass;
use crate::render::render_pass::main::main_render_pass::MainRenderPass;
use crate::render::render_pass::physics_debug::physics_debug_render_pass::PhysicsDebugRenderPass;
use crate::render::render_pass::shadow_mask::shadow_mask_render_pass::ShadowMaskRenderPass;
use crate::render::render_pass::shadows::shadows_render_pass::ShadowsRenderPass;
use crate::render::render_pass::ui::ui_render_pass::UiRenderPass;
use anyhow::Result;
use crate::render::render_pass::frame_data_context::FrameDataContext;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;

pub struct PassRegistry {
    culling_indirect_render_pass: CullingIndirectRenderPass,
    depth_render_pass: DepthRenderPass,
    shadows_render_pass: ShadowsRenderPass,
    shadow_mask_render_pass: ShadowMaskRenderPass,
    main_render_pass: MainRenderPass,
    physics_debug_render_pass: PhysicsDebugRenderPass,
    ui_render_pass: UiRenderPass,
}

impl PassRegistry {
    pub fn create(
        culling_indirect_render_pass: CullingIndirectRenderPass,
        depth_render_pass: DepthRenderPass,
        shadows_render_pass: ShadowsRenderPass,
        shadow_mask_render_pass: ShadowMaskRenderPass,
        main_render_pass: MainRenderPass,
        physics_debug_render_pass: PhysicsDebugRenderPass,
        ui_render_pass: UiRenderPass,
    ) -> Self {
        Self {
            culling_indirect_render_pass,
            depth_render_pass,
            shadows_render_pass,
            shadow_mask_render_pass,
            main_render_pass,
            physics_debug_render_pass,
            ui_render_pass,
        }
    }
    
    pub fn run_each(
        &self,
        frame_data_context: &FrameDataContext,
        render_pass_context: &RenderPassContext,
    ) -> Result<()> {
        self.run_pass(&self.culling_indirect_render_pass, frame_data_context, render_pass_context)?;
        self.run_pass(&self.depth_render_pass, frame_data_context, render_pass_context)?;
        self.run_pass(&self.shadows_render_pass, frame_data_context, render_pass_context)?;
        self.run_pass(&self.shadow_mask_render_pass, frame_data_context, render_pass_context)?;
        self.run_pass(&self.main_render_pass, frame_data_context, render_pass_context)?;
        self.run_pass(&self.physics_debug_render_pass, frame_data_context, render_pass_context)?;
        self.run_pass(&self.ui_render_pass, frame_data_context, render_pass_context)?;
        
        Ok(())
    }

    fn run_pass<P: RenderPass>(
        &self,
        pass: &P,
        frame_data_context: &FrameDataContext,
        render_pass_context: &RenderPassContext,
    ) -> Result<()> {
        let is_enabled = pass.is_enabled();

        if is_enabled {
            let data = pass.prepare_data(&frame_data_context)?;

            pass.record_commands(&render_pass_context, data)?;
        }

        Ok(())
    }
    
    pub fn destroy(self) -> Result<()> {
        self.culling_indirect_render_pass.destroy()?;
        self.depth_render_pass.destroy()?;
        self.shadows_render_pass.destroy()?;
        self.shadow_mask_render_pass.destroy()?;
        self.main_render_pass.destroy()?;
        self.physics_debug_render_pass.destroy()?;
        self.ui_render_pass.destroy()?;
        
        Ok(())
    }
}
