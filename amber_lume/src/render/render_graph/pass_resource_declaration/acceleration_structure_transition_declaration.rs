use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use ash::vk::{AccessFlags, PipelineStageFlags};

pub struct AccelerationStructureTransitionDeclaration {
    pub acceleration_structure: VirtualAccelerationStructure,
    pub access: AccessFlags,
    pub stage: PipelineStageFlags,
}

impl AccelerationStructureTransitionDeclaration {
    pub fn new(
        acceleration_structure: VirtualAccelerationStructure,
        access: AccessFlags,
        stage: PipelineStageFlags,
    ) -> Self {
        Self {
            acceleration_structure,
            access,
            stage,
        }
    }
}
