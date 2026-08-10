use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use anyhow::Result;
use ash::vk::{DeviceSize, Extent2D, Format, ImageCreateFlags, ImageUsageFlags};
use crate::limits::RenderLimits;
use gpu::create_cpu_to_gpu_heap_buffer;
use gpu::BufferInfo;
use gpu::ImageViewDescription;
use gpu::ResourceFactories;
use render_graph::PassGraphState;
use render_graph::HeapAllocator;
use render_graph::ImageBlueprint;
use render_graph::ImageSize;
use render_graph::VirtualImage;
use render_graph::ImageResourceScope;
use gpu::BindingLayout;
use gpu::Bindless;

pub struct RenderState {
    pub cpu_to_gpu_allocator: HeapAllocator,
    pub image_scope: ImageResourceScope,
    pub bindless: Bindless,
    pub pass_graph_state: Option<PassGraphState>,

    pub brdf_lut_image: VirtualImage,
    pub sh_image: VirtualImage,
}

impl RenderState {
    pub fn new(
        resource_factories: &ResourceFactories,
        limits: &RenderLimits,
        binding_layout: &BindingLayout,
        current_frame: Arc<AtomicU64>,
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

        let bindless = Bindless::new(
            &binding_layout.descriptor_set_manager,
            &limits.resource_limits,
            limits.frames_in_flight,
            current_frame,
        );

        let mut image_scope = ImageResourceScope::new();

        let brdf_lut_image = image_scope.create_image(
            "brdf_lut",
            ImageBlueprint {
                size: ImageSize::absolute(256, 256),
                format: Format::R16G16_SFLOAT,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::TRANSFER_DST,
                array_layers: 1,
                flags: ImageCreateFlags::empty(),
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: true,
            },
        );

        let sh_image = image_scope.create_image(
            "sh",
            ImageBlueprint {
                size: ImageSize::absolute(9, 1),
                format: Format::R16G16B16A16_SFLOAT,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::TRANSFER_DST,
                array_layers: 1,
                flags: ImageCreateFlags::empty(),
                image_view_description: ImageViewDescription::default_2d_color(),
                sampled: false,
            },
        );

        image_scope.build(
            Extent2D::default(),
            Extent2D::default(),
            &resource_factories.managed_image_factory,
            &bindless.graph_textures,
            &bindless.storage_images,
        )?;

        Ok(Self {
            cpu_to_gpu_allocator,
            image_scope,
            brdf_lut_image,
            sh_image,
            bindless,
            pass_graph_state: Some(PassGraphState::new()),
        })
    }

    pub fn destroy(
        self,
        resource_factories: &ResourceFactories,
    ) -> Result<()> {
        if let Some(pass_graph_state) = self.pass_graph_state {
            pass_graph_state.destroy(
                &resource_factories.managed_image_factory,
                &resource_factories.buffer_factory,
            )?;
        }
        self.image_scope.destroy(&resource_factories.managed_image_factory)?;
        self.cpu_to_gpu_allocator.destroy(&resource_factories.buffer_factory)?;

        Ok(())
    }
}
