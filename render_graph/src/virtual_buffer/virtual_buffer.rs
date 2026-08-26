use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use anyhow::Result;
use ash::vk::DeviceSize;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VirtualBuffer {
    pub handle: u32,
}

impl VirtualBuffer {
    pub fn new(handle: u32) -> Self {
        Self { handle }
    }
}

impl VirtualBuffer {
    pub fn stage_slice<T>(self, buffer_scope: &mut BufferResourceScope, data: &[T]) -> Result<()> {
        buffer_scope.bind_dynamic_slice(self, data)
    }

    pub fn reserve_region(self, buffer_scope: &mut BufferResourceScope, size: DeviceSize) -> Result<()> {
        buffer_scope.bind_dynamic_region(self, size)
    }

    pub fn alignment(self, buffer_scope: &BufferResourceScope) -> Result<DeviceSize> {
        buffer_scope.dynamic_alignment(self)
    }
}
