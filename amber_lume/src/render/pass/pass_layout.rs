use crate::ids::ChunkIndex;
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

pub struct RenderView {
    pub view_projection: ViewProjectionMatrix,
}

pub struct RenderViewsLayout {
    pub main: RenderView,
    pub cascade_count: u32,
}

impl RenderViewsLayout {
    pub fn count(&self) -> u32 {
        1 + self.cascade_count
    }

    pub fn get_main_index(&self) -> ChunkIndex {
        ChunkIndex { value: 0 }
    }

    pub fn get_shadow_cascade_index(&self, index: u32) -> ChunkIndex {
        ChunkIndex { value: 1 + index }
    }
}
