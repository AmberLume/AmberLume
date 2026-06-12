use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;

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
