use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use ash::vk::{PipelineCache, SampleCountFlags};
use crate::ids::SliceIndex;
use crate::limits::ResourceLimits;
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::resources::resource_transfer::ResourceTransfer;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::resource_buffers::ResourceBuffers;
use crate::resources::store::providers_statistics::ResourcesStatistics;
use crate::resources::store::providers::animation::animation_backend::AnimationBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::resources::store::providers::material::material_backend::MaterialBackend;
use crate::resources::store::providers::mesh::mesh_backend::MeshBackend;
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::store::persistent::persistent_images::PersistentImages;
use crate::resources::store::persistent::persistent_materials::PersistentMaterials;
use crate::resources::store::persistent::persistent_meshes::PersistentMeshes;
use crate::resources::store::persistent::persistent_resources::PersistentResources;
use crate::resources::store::persistent::persistent_skeletons::PersistentSkeletons;
use crate::utils::arc_utils::ArcUnwrapOrErr;

pub struct ResourceStore {
    pub image_provider: Arc<ResourceProvider<ImageBackend>>,
    pub material_provider: Arc<ResourceProvider<MaterialBackend>>,
    pub skeletons_provider: Arc<ResourceProvider<SkeletonBackend>>,
    pub animation_provider: Arc<ResourceProvider<AnimationBackend>>,
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    pub pipeline_provider: Arc<ResourceProvider<PipelineBackend>>,
    pub compute_pipeline_provider: Arc<ResourceProvider<ComputePipelineBackend>>,

    pub persistent_resources: Arc<PersistentResources>,

    pub resource_buffers: ResourceBuffers,
}

impl ResourceStore {
    pub fn new(
        limits: &ResourceLimits,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
        binding_layout: Arc<BindingLayout>,
        resource_reader: Arc<AlpacaResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
        resource_factories: Arc<ResourceFactories>,
        destroy_delay: u32,
        frame_counter: Arc<AtomicU64>,
    ) -> Result<Self> {
        let skeletons_provider = ResourceProvider::from(
            SkeletonBackend::new(
                &limits,
                &resource_factories.buffer_factory,
                resource_reader.clone(),
                resource_transfer.clone(),
            )?,
            limits.max_skeletons,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_skeletons = PersistentSkeletons::create(
            &skeletons_provider,
            &limits,
        )?;

        let animation_provider = ResourceProvider::from(
            AnimationBackend::new(
                &limits,
                &resource_factories.buffer_factory,
                resource_reader.clone(),
                resource_transfer.clone(),
            )?,
            limits.max_animations,
            destroy_delay,
            frame_counter.clone(),
        );

        let image_provider = ResourceProvider::from(
            ImageBackend::new(
                device_context.texture_format,
                resource_factories.clone(),
                resource_reader.clone(),
                binding_layout.clone(),
                resource_transfer.clone(),
            ),
            limits.max_texture_descriptors,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_images = PersistentImages::create(
            &image_provider,
            &binding_layout.descriptor_set_manager,
            limits.max_texture_descriptors,
            swapchain_context.format,
            SampleCountFlags::TYPE_1,
        )?;

        let material_provider = ResourceProvider::from(
            MaterialBackend::new(
                &limits,
                &resource_factories.buffer_factory,
                image_provider.clone(),
                resource_reader.clone(),
                resource_transfer.clone(),
                &persistent_images,
            )?,
            limits.max_materials,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_materials = PersistentMaterials::create(
            &material_provider,
            &persistent_images,
        )?;

        let mesh_provider = ResourceProvider::from(
            MeshBackend::new(
                &limits,
                &resource_factories.buffer_factory,
                &persistent_materials,
                resource_reader.clone(),
                resource_transfer.clone(),
                material_provider.clone(),
                skeletons_provider.clone(),
            )?,
            limits.max_meshes,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_meshes = PersistentMeshes::create(
            &mesh_provider,
            &persistent_materials,
        )?;

        let pipeline_provider = ResourceProvider::from(
            PipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                resource_reader.clone(),
                binding_layout.clone(),
            ),
            128,
            destroy_delay,
            frame_counter.clone(),
        );

        let compute_pipeline_provider = ResourceProvider::from(
            ComputePipelineBackend::new(
                device_context.device.clone(),
                device_context.debug_utils.clone(),
                PipelineCache::null(),
                resource_reader.clone(),
                binding_layout.clone(),
            ),
            128,
            destroy_delay,
            frame_counter.clone(),
        );

        let persistent_resources = Arc::new(PersistentResources::create(
            persistent_skeletons,
            persistent_images,
            persistent_materials,
            persistent_meshes,
        )?);

        let resource_buffers = ResourceBuffers {
            index_buffer_handle: mesh_provider.backend.index_buffer.handle(),

            mesh_buffer: mesh_provider.backend.mesh_buffer.slice_at(SliceIndex::ZERO).device_address(),
            submesh_buffer: mesh_provider.backend.submesh_buffer.slice_at(SliceIndex::ZERO).device_address(),
            index_buffer: mesh_provider.backend.index_buffer.slice_at(SliceIndex::ZERO).device_address(),
            vertex_buffer: mesh_provider.backend.vertex_buffer.slice_at(SliceIndex::ZERO).device_address(),

            skeleton_buffer: skeletons_provider.backend.skeletons_buffer.slice_at(SliceIndex::ZERO).device_address(),
            skeleton_bone_buffer: skeletons_provider.backend.skeleton_bones_buffer.slice_at(SliceIndex::ZERO).device_address(),

            animation_buffer: animation_provider.backend.animation_buffer.slice_at(SliceIndex::ZERO).device_address(),
            animation_frame_buffer: animation_provider.backend.animation_frame_buffer.slice_at(SliceIndex::ZERO).device_address(),

            material_buffer: material_provider.backend.material_buffer.slice_at(SliceIndex::ZERO).device_address(),
        };

        Ok(Self {
            image_provider,
            material_provider,
            skeletons_provider,
            animation_provider,
            mesh_provider,
            pipeline_provider,
            compute_pipeline_provider,

            persistent_resources,

            resource_buffers,
        })
    }

    pub fn update(&self) {
        self.image_provider.update();
        self.material_provider.update();
        self.skeletons_provider.update();
        self.animation_provider.update();
        self.mesh_provider.update();
        self.pipeline_provider.update();
        self.compute_pipeline_provider.update();
    }

    pub fn statistics(&self) -> ResourcesStatistics {
        ResourcesStatistics {
            image_provider: self.image_provider.statistics(),
            skeleton_provider: self.skeletons_provider.statistics(),
            animation_provider: self.animation_provider.statistics(),
            material_provider: self.material_provider.statistics(),
            mesh_provider: self.mesh_provider.statistics(),
            pipeline_provider: self.pipeline_provider.statistics(),
            compute_pipeline_provider: self.compute_pipeline_provider.statistics(),
        }
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.pipeline_provider.try_unwrap()?.destroy(&resource_factories)?;
        self.compute_pipeline_provider.try_unwrap()?.destroy(&resource_factories)?;
        self.mesh_provider.try_unwrap()?.destroy(&resource_factories)?;
        self.animation_provider.try_unwrap()?.destroy(&resource_factories)?;
        self.skeletons_provider.try_unwrap()?.destroy(&resource_factories)?;
        self.material_provider.try_unwrap()?.destroy(&resource_factories)?;
        self.image_provider.try_unwrap()?.destroy(&resource_factories)?;

        Ok(())
    }
}
