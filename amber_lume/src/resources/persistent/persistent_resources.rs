use std::sync::Arc;
use crate::resources::persistent::persistent_pipeline_layouts::PersistentPipelineLayouts;
use crate::resources::persistent::persistent_samplers::PersistentSamplers;
use crate::resources::resource_factories::ResourceFactories;
use anyhow::Result;
use ash::vk::{Format, SampleCountFlags};
use crate::limits::renderer_limits::RendererLimits;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::device::device_context::DeviceContext;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::descriptor_set_manager::DescriptorSetManager;
use crate::resources::persistent::persistent_images::PersistentImages;
use crate::resources::persistent::persistent_materials::PersistentMaterials;
use crate::resources::persistent::persistent_meshes::PersistentMeshes;
use crate::resources::persistent::persistent_shadows::PersistentShadows;
use crate::resources::persistent::persistent_skeletons::PersistentSkeletons;

pub struct PersistentResources {
    pub descriptor_set_manager: DescriptorSetManager,
    
    pub samplers: PersistentSamplers,
    pub images: PersistentImages,
    pub skeletons: PersistentSkeletons,
    pub materials: PersistentMaterials,
    pub meshes: PersistentMeshes,
    pub pipeline_layouts: PersistentPipelineLayouts,
    pub shadows: PersistentShadows,
}

impl PersistentResources {
    pub fn create(
        device_context: &DeviceContext,
        resource_loader: Arc<ResourceLoader>,
        resource_factories: &ResourceFactories,
        renderer_limits: &RendererLimits,
        buffer_manager: &BufferManager,
        index_managers: &IndexManagers,
        format: Format,
        samples: SampleCountFlags,
    ) -> Result<Self> {
        let descriptor_set_manager = DescriptorSetManager::new(
            device_context.device.clone(),
            &resource_factories.descriptor_set_layout_factory,
            &resource_factories.descriptor_set_factory,
            renderer_limits,
        )?;
        
        let samplers = PersistentSamplers::create(
            &resource_factories.sampler_factory,
        )?;
        
        let images = PersistentImages::create(
            resource_loader.clone(),
            &resource_factories.managed_image_factory,
            &index_managers.texture_descriptors_index_manager,
            &descriptor_set_manager,
            &samplers,
            format,
            samples,
        )?;

        let skeletons = PersistentSkeletons::create(
            &resource_loader,
            &renderer_limits,
            &index_managers.skeletons_index_manager,
            &index_managers.skeleton_bones_index_manager,
            &buffer_manager,
        )?;

        let materials = PersistentMaterials::create(
            resource_loader.clone(),
            &index_managers.material_index_manager,
            &buffer_manager,
            &images,
        )?;

        let meshes = PersistentMeshes::create(
            resource_loader.clone(),
            &materials,
            &index_managers.index_index_manager,
            &index_managers.vertex_index_manager,
            &index_managers.mesh_index_manager,
            &index_managers.submesh_index_manager,
            &buffer_manager,
        )?;
        
        let pipeline_layouts = PersistentPipelineLayouts::create(
            &resource_factories.pipeline_layout_factory,
            &descriptor_set_manager,
        )?;

        let shadows = PersistentShadows::create(
            &index_managers,
            &resource_factories.managed_image_factory,
            &renderer_limits,
            &descriptor_set_manager,
            &samplers,
        )?;

        Ok(Self {
            descriptor_set_manager,
            
            samplers,
            images,
            skeletons,
            materials,
            meshes,
            pipeline_layouts,
            shadows,
        })
    }

    pub fn destroy(
        self,
        index_managers: &IndexManagers,
        resource_factories: &ResourceFactories,
    ) -> Result<()> {
        self.shadows.destroy(&index_managers, &resource_factories.managed_image_factory)?;
        self.pipeline_layouts.destroy(&resource_factories.pipeline_layout_factory);
        self.images.destroy(&resource_factories.managed_image_factory)?;
        self.samplers.destroy(&resource_factories.sampler_factory)?;

        self.descriptor_set_manager.destroy(&resource_factories.descriptor_set_layout_factory);
        
        Ok(())
    }
}
