use ash::vk::{CommandBuffer, DescriptorBindingFlags, DescriptorSet, DescriptorSetLayout, DescriptorType, PipelineBindPoint, PipelineLayout, ShaderStageFlags};
use crate::factories::descriptor_set_layout::descriptor_set_layout_factory::{DescriptorSetLayoutBindingDescription, DescriptorSetLayoutFactory};
use anyhow::Result;
use ash::Device;
use crate::resource_limits::ResourceLimits;
use crate::factories::descriptor_set::descriptor_set_factory::DescriptorSetFactory;
use crate::factories::sampler::sampler_factory::SamplerFactory;
use crate::binding_layout::managed_acceleration_structure_descriptor_set::ManagedAccelerationStructureDescriptorSet;
use crate::binding_layout::managed_descriptor_set::ManagedDescriptorSet;
use crate::sampler_registry::SamplerRegistry;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GlobalDescriptorSetBindings {
    Sampler = 0,
    Texture = 1,
    ShadowArray = 2,
    StorageImage = 3,
    GraphTexture = 4,
    AccelerationStructure = 5,
}

pub struct DescriptorSetManager {
    device: Device,

    handle: DescriptorSet,
    layout: DescriptorSetLayout,

    pub textures_descriptor_set: ManagedDescriptorSet,
    pub shadow_arrays_descriptor_set: ManagedDescriptorSet,
    pub storage_images_descriptor_set: ManagedDescriptorSet,
    pub graph_textures_descriptor_set: ManagedDescriptorSet,
    pub acceleration_structures_descriptor_set: Option<ManagedAccelerationStructureDescriptorSet>,

    sampler_registry: SamplerRegistry,
}

impl DescriptorSetManager {
    pub fn new(
        device: Device,
        layout_factory: &DescriptorSetLayoutFactory,
        set_factory: &DescriptorSetFactory,
        sampler_factory: &SamplerFactory,
        limits: &ResourceLimits,
        ray_tracing: bool,
        frames_in_flight: u32,
    ) -> Result<Self> {
        let max_textures = limits.max_texture_descriptors;
        let max_shadow_arrays = limits.max_shadow_array_descriptors;
        let max_storage_images = limits.max_storage_image_descriptors;
        let max_graph_textures = limits.max_graph_texture_descriptors;

        let sampler_registry = SamplerRegistry::create(&sampler_factory)?;
        let samplers = sampler_registry.all();

        let mut bindings = vec![
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::Texture,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::SAMPLED_IMAGE,
                descriptor_count: max_textures,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                immutable_samplers: Vec::new(),
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::ShadowArray,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::SAMPLED_IMAGE,
                descriptor_count: max_shadow_arrays,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                immutable_samplers: Vec::new(),
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::StorageImage,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::STORAGE_IMAGE,
                descriptor_count: max_storage_images,
                stage_flags: ShaderStageFlags::COMPUTE,
                immutable_samplers: Vec::new(),
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::Sampler,
                binding_flags: DescriptorBindingFlags::empty(),
                descriptor_type: DescriptorType::SAMPLER,
                descriptor_count: samplers.len() as u32,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                immutable_samplers: samplers.to_vec(),
            },
            DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::GraphTexture,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND,
                descriptor_type: DescriptorType::SAMPLED_IMAGE,
                descriptor_count: max_graph_textures,
                stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX | ShaderStageFlags::COMPUTE,
                immutable_samplers: Vec::new(),
            },
        ];

        if ray_tracing {
            bindings.push(DescriptorSetLayoutBindingDescription {
                binding: GlobalDescriptorSetBindings::AccelerationStructure,
                binding_flags: DescriptorBindingFlags::PARTIALLY_BOUND,
                descriptor_type: DescriptorType::ACCELERATION_STRUCTURE_KHR,
                descriptor_count: frames_in_flight,
                stage_flags: ShaderStageFlags::COMPUTE,
                immutable_samplers: Vec::new(),
            });
        }

        let layout = layout_factory.create_descriptor_set_layout(
            "global",
            &bindings,
        )?;

        let descriptors_count = bindings.iter().map(|b| b.descriptor_count).sum::<u32>();
        let handle = set_factory.create_descriptor_set(
            "global",
            &[layout],
            &[descriptors_count]
        )?;

        let textures_descriptor_set = ManagedDescriptorSet::new(
            device.clone(),
            handle,
            GlobalDescriptorSetBindings::Texture,
            DescriptorType::SAMPLED_IMAGE,
        );
        let shadow_arrays_descriptor_set = ManagedDescriptorSet::new(
            device.clone(),
            handle,
            GlobalDescriptorSetBindings::ShadowArray,
            DescriptorType::SAMPLED_IMAGE,
        );
        let storage_images_descriptor_set = ManagedDescriptorSet::new(
            device.clone(),
            handle,
            GlobalDescriptorSetBindings::StorageImage,
            DescriptorType::STORAGE_IMAGE,
        );
        let graph_textures_descriptor_set = ManagedDescriptorSet::new(
            device.clone(),
            handle,
            GlobalDescriptorSetBindings::GraphTexture,
            DescriptorType::SAMPLED_IMAGE,
        );
        let acceleration_structures_descriptor_set = ray_tracing.then(|| {
            ManagedAccelerationStructureDescriptorSet::new(
                device.clone(),
                handle,
                GlobalDescriptorSetBindings::AccelerationStructure,
            )
        });

        Ok(Self {
            device,

            handle,
            layout,

            textures_descriptor_set,
            shadow_arrays_descriptor_set,
            storage_images_descriptor_set,
            graph_textures_descriptor_set,
            acceleration_structures_descriptor_set,

            sampler_registry,
        })
    }

    pub fn bind(&self, command_buffer: CommandBuffer, pipeline_layout: PipelineLayout) {
        for bind_point in [PipelineBindPoint::GRAPHICS, PipelineBindPoint::COMPUTE] {
            unsafe {
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    bind_point,
                    pipeline_layout,
                    0,
                    &[self.handle],
                    &[],
                );
            }
        }
    }

    pub fn layout(&self) -> &DescriptorSetLayout {
        &self.layout
    }

    pub fn destroy(
        self,
        sampler_factory: &SamplerFactory,
        descriptor_set_layout_factory: &DescriptorSetLayoutFactory,
    ) {
        self.sampler_registry.destroy(&sampler_factory);

        descriptor_set_layout_factory.destroy_descriptor_set_layout(self.layout);
    }
}
