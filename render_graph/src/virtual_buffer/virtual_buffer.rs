use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::virtual_buffer::heap_allocator::HeapAllocator;
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
    pub fn stage_slice<T>(
        self,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
        data: &[T],
    ) -> anyhow::Result<()> {
        let physical_buffer = allocator.allocate_for_slice(&data)?;
        physical_buffer.write(&data)?;

        buffer_scope.rebind_buffer(
            self,
            physical_buffer.buffer,
            physical_buffer.offset,
            physical_buffer.size,
            physical_buffer.device_address,
            physical_buffer.mapped_ptr,
        );

        Ok(())
    }
}
