use crate::resources::model::model_backend::PrimitiveAllocation;
use glam::Mat4;
use std::sync::Arc;

pub struct RenderSnapshot {
    pub view_projection: Mat4,

    pub entities: Arc<Vec<PrimitiveAllocation>>,
}
