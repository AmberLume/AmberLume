use crate::render::pass::culling_indirect::culling_indirect_pass::CullingIndirectPass;
use crate::render::pass::depth::depth_pass::DepthPass;
use crate::render::pass::main::main_pass::MainPass;
use crate::render::pass::pass_statistics::PassStatistics;
use crate::render::pass::physics_debug::physics_debug_pass::PhysicsDebugPass;
use crate::render::pass::shadow_mask::shadow_mask_pass::ShadowMaskPass;
use crate::render::pass::shadows::shadows_pass::ShadowsPass;
use crate::render::pass::ui::ui_render_pass::UiPass;

pub struct PassesStatistics {
    pub total_dispatch: u64,
    
    pub culling: PassStatistics<CullingIndirectPass>,
    pub depth: PassStatistics<DepthPass>,
    pub shadows: PassStatistics<ShadowsPass>,
    pub shadow_mask: PassStatistics<ShadowMaskPass>,
    pub main: PassStatistics<MainPass>,
    pub physics_debug: PassStatistics<PhysicsDebugPass>,
    pub ui: PassStatistics<UiPass>,
}
