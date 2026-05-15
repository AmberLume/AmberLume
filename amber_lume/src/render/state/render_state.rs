use anyhow::Result;
use ash::vk::DeviceSize;
use crate::limits::AmberLumeLimits;
use crate::render::buffer::typed::cpu_to_gpu_heap_buffer::create_cpu_to_gpu_heap_buffer;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::render_graph::state::pass_graph_state::PassGraphState;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::index_managers::IndexManagers;
use crate::resources::persistent_shadows::PersistentShadows;

pub struct RenderState {
    pub cpu_to_gpu_allocator: HeapAllocator,
    pub persistent_shadows: PersistentShadows,
    pub pass_graph_state: Option<PassGraphState>,
}

impl RenderState {
    pub fn new(
        resource_factories: &ResourceFactories,
        limits: &AmberLumeLimits,
        index_managers: &IndexManagers,
        binding_layout: &BindingLayout,
    ) -> Result<Self> {
        let cpu_to_gpu_buffer = create_cpu_to_gpu_heap_buffer(
            &resource_factories.buffer_factory,
            limits.frames_in_flight,
            limits.resource_limits.max_frame_heap_size as DeviceSize,
        )?;
        let cpu_to_gpu_allocator = HeapAllocator::create(
            cpu_to_gpu_buffer.into_managed_buffer(),
            limits.resource_limits.max_frame_heap_size as DeviceSize,
            limits.frames_in_flight,
        )?;

        let persistent_shadows = PersistentShadows::create(
            index_managers,
            &resource_factories.managed_image_factory,
            &limits.shadow_map_limits,
            &binding_layout.descriptor_set_manager,
        )?;

        Ok(Self {
            cpu_to_gpu_allocator,
            persistent_shadows,
            pass_graph_state: Some(PassGraphState::new()),
        })
    }

    pub fn destroy(
        self,
        resource_factories: &ResourceFactories,
        index_managers: &IndexManagers,
    ) -> Result<()> {
        if let Some(pass_graph_state) = self.pass_graph_state {
            pass_graph_state.destroy(
                &resource_factories.managed_image_factory,
                &resource_factories.buffer_factory,
            )?;
        }
        self.persistent_shadows.destroy(index_managers, &resource_factories.managed_image_factory)?;
        self.cpu_to_gpu_allocator.destroy(&resource_factories.buffer_factory)?;

        Ok(())
    }
}
