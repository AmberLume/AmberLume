use crate::resource_scope::acceleration_structure_resource_scope::AccelerationStructureResourceScope;
use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::resource_scope::image_resource_scope::ImageResourceScope;
use crate::resource_scope::readback_scope::ReadbackScope;

pub struct RecordScopes<'scope> {
    pub image: &'scope ImageResourceScope,
    pub buffer: &'scope BufferResourceScope,
    pub acceleration_structure: &'scope AccelerationStructureResourceScope,
    pub readback: &'scope ReadbackScope,
}
