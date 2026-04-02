use anyhow::Result;
use ash::vk::{AccessFlags, Extent3D, Format, ImageLayout, ImageTiling, ImageType, ImageUsageFlags, PipelineStageFlags, SampleCountFlags, SharingMode};
use crate::limits::renderer_limits::{RendererLimits, ShadowMapFormat};
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::resources::index_managers::IndexManagers;
use crate::resources::descriptor_set_manager::{DescriptorSetManager, GlobalDescriptorSetBindings};
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::sampler_registry::SamplerType;

pub struct PersistentShadows {
    pub global_shadow_array_descriptor_id: ResourceId,
    pub global_shadow_array: ManagedImage,
}

impl PersistentShadows {
    pub fn create(
        index_managers: &IndexManagers,
        managed_image_factory: &ManagedImageFactory,
        renderer_limits: &RendererLimits,
        descriptor_set_manager: &DescriptorSetManager,
        image_state_tracker: &mut ImageStateTracker,
    ) -> Result<Self> {
        let global_shadow_cascade_count = renderer_limits.shadow_map_limits.global_cascades.len() as u32;
        let global_shadow_array_descriptor_id = index_managers.shadow_array_descriptors_index_manager.acquire().unwrap();
        let global_shadow_array = managed_image_factory.allocate(
            "global_shadow_array",
            ImageDescription {
                image_type: ImageType::TYPE_2D,
                format: match renderer_limits.shadow_map_limits.format {
                    ShadowMapFormat::D16 => Format::D16_UNORM,
                    ShadowMapFormat::D32 => Format::D32_SFLOAT,
                },
                extent: Extent3D {
                    width: renderer_limits.shadow_map_limits.resolution,
                    height: renderer_limits.shadow_map_limits.resolution,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: global_shadow_cascade_count,
                samples: SampleCountFlags::TYPE_1,
                tiling: ImageTiling::OPTIMAL,
                usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                sharing_mode: SharingMode::EXCLUSIVE,
            },
            ImageViewDescription::default_2d_array_depth(global_shadow_cascade_count),
        )?;
        descriptor_set_manager.write(
            GlobalDescriptorSetBindings::ShadowArray,
            global_shadow_array_descriptor_id,
            &global_shadow_array,
            SamplerType::Shadow,
        );

        image_state_tracker.register_persistent(
            global_shadow_array.image,
            ImageLayout::UNDEFINED,
            AccessFlags::empty(),
            PipelineStageFlags::TOP_OF_PIPE,
        );

        Ok(Self {
            global_shadow_array_descriptor_id,
            global_shadow_array,
        })
    }

    pub fn destroy(
        self,
        index_managers: &IndexManagers,
        managed_image_factory: &ManagedImageFactory,
    ) -> Result<()> {
        managed_image_factory.destroy_image(self.global_shadow_array)?;

        index_managers.shadow_array_descriptors_index_manager.release(self.global_shadow_array_descriptor_id);

        Ok(())
    }
}
