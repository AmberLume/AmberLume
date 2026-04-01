use crate::render::pass::culling_indirect::render_view_culling_indirect_statistics::CullingIndirectStatistics;
use crate::render::pass::pass_statistics::PassStatistics;

pub struct PassesStatistics {
    pub culling: PassStatistics,
    pub culling_meta: CullingIndirectStatistics, 
    pub depth: PassStatistics,
    pub shadows: PassStatistics,
    pub shadow_mask: PassStatistics,
    pub main: PassStatistics,
    pub physics_debug: PassStatistics,
    pub ui: PassStatistics,
}
