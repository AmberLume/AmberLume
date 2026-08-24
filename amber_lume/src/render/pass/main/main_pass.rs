use render_graph::VirtualReadback;
use crate::render::frame_data::picked_entity_gpu::PickedEntityGPU;
use render_graph::ReadbackScope;
use render_graph::VirtualData;
use crate::render::pass::main::main_push_constants::MainPushConstants;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use render_graph::PassResourceDeclaration;
use render_graph::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use render_graph::VirtualBuffer;
use render_graph::VirtualImage;
use resource_residency::ResRef;
use crate::resource_manifest::shaders;
use gpu::PipelineLayoutType;
use index_allocator::ResourceId;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use settings::RenderSettings;

pub struct MainPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    entity_id_image: VirtualImage,
    depth: VirtualImage,
    shadow_history_a: VirtualImage,
    shadow_history_b: VirtualImage,
    shadow_colored: bool,
    gtao_history_a: VirtualImage,
    gtao_history_b: VirtualImage,
    sh_image: VirtualImage,
    brdf_lut_image: VirtualImage,

    brdf_lut_descriptor: u32,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    pool: DrawPool,
    bucket: DrawBucket,
    bone_transform: VirtualBuffer,

    picked_entity: VirtualReadback<PickedEntityGPU>,

    render_settings: VirtualData<RenderSettings>,

    mesh_vertex_buffer: VirtualBuffer,

    mesh_vertex_skin_buffer: VirtualBuffer,

    mesh_vertex_attribute_buffer: VirtualBuffer,
    submesh_buffer: VirtualBuffer,
    material_buffer: VirtualBuffer,
    index_buffer: VirtualBuffer,
}

impl MainPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        entity_id_image: VirtualImage,
        depth: VirtualImage,
        shadow_history_a: VirtualImage,
        shadow_history_b: VirtualImage,
        shadow_colored: bool,
        gtao_history_a: VirtualImage,
        gtao_history_b: VirtualImage,
        sh_image: VirtualImage,
        brdf_lut_image: VirtualImage,
        brdf_lut_descriptor: u32,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        pool: DrawPool,
        bucket: DrawBucket,
        bone_transform: VirtualBuffer,
        picked_entity: VirtualReadback<PickedEntityGPU>,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "main".to_string(),
            stages: vec![
                PipelineStageConfig::fragment(shaders::MAIN_FRAG),
                PipelineStageConfig::vertex(shaders::MAIN_VERT),
            ],
            color_formats: vec![color_format, Format::R32_UINT],
            depth_format: Some(resources.render_context.depth_format),
            depth_write: false,
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config)?;
        let Some(pipeline) = resources.pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,
            entity_id_image,
            depth,
            shadow_history_a,
            shadow_history_b,
            shadow_colored,
            gtao_history_a,
            gtao_history_b,
            sh_image,
            brdf_lut_image,

            brdf_lut_descriptor,

            scene_buffer,
            entity_buffer,
            pool,
            bucket,
            bone_transform,

            picked_entity,

            render_settings,

            mesh_vertex_buffer: resources.resource_buffer_handles.mesh_vertex_buffer,

            mesh_vertex_skin_buffer: resources.resource_buffer_handles.mesh_vertex_skin_buffer,

            mesh_vertex_attribute_buffer: resources.resource_buffer_handles.mesh_vertex_attribute_buffer,
            submesh_buffer: resources.resource_buffer_handles.submesh_buffer,
            material_buffer: resources.resource_buffer_handles.material_buffer,
            index_buffer: resources.resource_buffer_handles.index_buffer,
        })
    }
}

pub struct MainPassData {
    ao_enabled: bool,
    shadow_enabled: bool,
}

impl Pass for MainPass {
    type PassData = MainPassData;

    fn name(&self) -> String {
        String::from("main")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let render_settings = data_scope.get(self.render_settings);

        Ok(MainPassData {
            ao_enabled: render_settings.ao_enabled.value,
            shadow_enabled: render_settings.shadow_enabled.value,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
            .write_image(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .read_image(
                self.shadow_history_a,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.shadow_history_b,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.gtao_history_a,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.gtao_history_b,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.sh_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.brdf_lut_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.entity_id_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.pool.draw_count,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.pool.indirect,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.pool.draw_data,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.bone_transform,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.index_buffer,
                AccessFlags::INDEX_READ,
                PipelineStageFlags::VERTEX_INPUT,
            )
            .read_buffer(
                self.mesh_vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_skin_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_attribute_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.submesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.material_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![
                ColorTarget {
                    image: self.target_image,
                    mip: None,
                    clear: None,
                },
                ColorTarget {
                    image: self.entity_id_image,
                    mip: None,
                    clear: Some(ClearColor::Uint([u32::MAX; 4])),
                },
            ],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: None,
            }),
            view_mask: 0,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let material_buffer = buffer_scope.get_physical_buffer(self.material_buffer);
        let index_buffer = buffer_scope.get_physical_buffer(self.index_buffer);
        let mesh_vertex_buffer = buffer_scope.get_physical_buffer(self.mesh_vertex_buffer);
        let mesh_vertex_skin_buffer = buffer_scope.get_physical_buffer(self.mesh_vertex_skin_buffer);
        let mesh_vertex_attribute_buffer = buffer_scope.get_physical_buffer(self.mesh_vertex_attribute_buffer);
        let submesh_buffer = buffer_scope.get_physical_buffer(self.submesh_buffer);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let draw_count = buffer_scope.get_physical_buffer(self.pool.draw_count);
        let indirect = buffer_scope.get_physical_buffer(self.pool.indirect);
        let draw_data = buffer_scope.get_physical_buffer(self.pool.draw_data);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);
        
        let shadow_history = if context.history_write_index == 0 {
            self.shadow_history_a
        } else {
            self.shadow_history_b
        };
        let shadow_factor_image = image_scope.get_physical_image(shadow_history);
        let shadow_factor_descriptor_id = shadow_factor_image
            .descriptors
            .full
            .expect("Main shadow history image must have a sampled descriptor");

        let gtao_history = if context.history_write_index == 0 {
            self.gtao_history_a
        } else {
            self.gtao_history_b
        };
        let gtao_image = image_scope.get_physical_image(gtao_history);

        let sh_image = image_scope.get_physical_image(self.sh_image);
        let sh_descriptor_id = sh_image.descriptors.full.unwrap_or(ResourceId::from(0));

        let gtao_descriptor_id = gtao_image.descriptors.full.unwrap_or(ResourceId::from(0));

        context.bind_index_buffer(index_buffer.range, 0);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);
        let picked_entity = readback_scope.get_physical_readback(self.picked_entity);

        let entity_id_extent = image_scope.get_physical_image(self.entity_id_image).extent;
        let pick_x = entity_id_extent.width / 2;
        let pick_y = entity_id_extent.height / 2;

        context.push_constants(
            self.pipeline_layout,
            &MainPushConstants::create(
                scene_buffer.range,
                draw_data.range,
                mesh_vertex_buffer.range,
                mesh_vertex_attribute_buffer.range,
                mesh_vertex_skin_buffer.range,
                entity_buffer.range,
                submesh_buffer.range,
                material_buffer.range,
                bone_transform_buffer.range,
                picked_entity.range,
                shadow_factor_descriptor_id,
                data.shadow_enabled as u32,
                self.shadow_colored as u32,
                gtao_descriptor_id,
                data.ao_enabled as u32,
                sh_descriptor_id,
                ResourceId::from(self.brdf_lut_descriptor),
                pick_x,
                pick_y,
            ),
        );
        context.draw_indirect_gpu_scene(&indirect, &draw_count, self.bucket);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
