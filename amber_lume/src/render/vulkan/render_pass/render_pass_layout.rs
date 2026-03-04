use glam::Mat4;

pub struct RenderView {
    pub projection_view: Mat4,
}

pub struct RenderViewsLayout {
    pub main: RenderView,
    pub global_shadow_cascades: Vec<RenderView>,
}

impl RenderViewsLayout {
    pub fn count(&self) -> u32 {
        let mut count: u32 = 0;

        count += 1;
        count += self.global_shadow_cascades.len() as u32;

        count
    }

    pub fn get_main_index(&self) -> u32 {
        0
    }

    pub fn get_shadow_cascade_index(&self, index: u32) -> u32 {
        // Single main render view
        1 + index
    }
}
