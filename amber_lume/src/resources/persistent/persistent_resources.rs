use std::sync::Arc;
use crate::resources::persistent::persistent_descriptor_set_layouts::PersistentDescriptorSetLayouts;
use crate::resources::persistent::persistent_pipeline_layouts::PersistentPipelineLayouts;
use crate::resources::persistent::persistent_samplers::PersistentSamplers;
use crate::resources::resource_factories::ResourceFactories;
use anyhow::Result;
use ash::vk::{Format, SampleCountFlags};
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::DescriptorIndexManagers;
use crate::resources::persistent::persistent_descriptor_sets::PersistentDescriptorSets;
use crate::resources::persistent::persistent_images::PersistentImages;
use crate::resources::persistent::persistent_materials::PersistentMaterials;
use crate::resources::persistent::persistent_models::PersistentModels;

pub struct PersistentResources {
    pub samplers: PersistentSamplers,
    pub images: PersistentImages,
    pub materials: PersistentMaterials,
    pub models: PersistentModels,
    pub descriptor_set_layouts: PersistentDescriptorSetLayouts,
    pub descriptor_sets: PersistentDescriptorSets,
    pub pipeline_layouts: PersistentPipelineLayouts,
}

impl PersistentResources {
    pub fn create(
        resource_loader: Arc<ResourceLoader>,
        resource_factories: &ResourceFactories,
        buffer_manager: &BufferManager,
        index_managers: &DescriptorIndexManagers,
        format: Format,
        samples: SampleCountFlags,
    ) -> Result<Self> {
        let samplers = PersistentSamplers::create(
            &resource_factories.sampler_factory,
        )?;
        
        let images = PersistentImages::create(
            resource_loader.clone(),
            &resource_factories.managed_image_factory,
            &index_managers.image_index_manager,
            format,
            samples,
        )?;

        let materials = PersistentMaterials::create(
            resource_loader.clone(),
            &index_managers.material_index_manager,
            &buffer_manager,
        )?;

        let models = PersistentModels::create(
            resource_loader.clone(),
            &materials,
            &index_managers.index_index_manager,
            &index_managers.vertex_index_manager,
            &index_managers.submesh_index_manager,
            &index_managers.model_index_manager,
            &buffer_manager,
        )?;

        let descriptor_set_layouts = PersistentDescriptorSetLayouts::create(
            &resource_factories.descriptor_set_layout_factory,
        )?;

        let descriptor_sets = PersistentDescriptorSets::create(
            &resource_factories.descriptor_set_factory,
            &descriptor_set_layouts,
        )?;
        
        let pipeline_layouts = PersistentPipelineLayouts::create(
            &resource_factories.pipeline_layout_factory,
            &descriptor_set_layouts,
        )?;

        Ok(Self { 
            samplers,
            images,
            materials,
            models,
            descriptor_set_layouts,
            descriptor_sets,
            pipeline_layouts,
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.descriptor_set_layouts.destroy(&resource_factories.descriptor_set_layout_factory);
        self.pipeline_layouts.destroy(&resource_factories.pipeline_layout_factory);
        self.images.destroy(&resource_factories.managed_image_factory)?;
        self.samplers.destroy(&resource_factories.sampler_factory)?;

        Ok(())
    }
}
