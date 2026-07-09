use crate::render::pass::main::main_push_constants::MainPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::info;
use crate::settings::settings::EngineSettings;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_image::render_targets::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::resource_manifest::shaders;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::store::providers::pipeline::pipeline_config::{PipelineConfig, PipelineStageConfig};

pub struct MainPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    entity_id_image: VirtualImage,
    depth: VirtualImage,
    shadow_factor: VirtualImage,
    gtao_history_a: VirtualImage,
    gtao_history_b: VirtualImage,
    sh_image: VirtualImage,
    brdf_lut_image: VirtualImage,

    brdf_lut_descriptor: u32,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    draw_count_main: VirtualBuffer,
    indirect_main: VirtualBuffer,
    draw_data_main: VirtualBuffer,
    bone_transform: VirtualBuffer,

    settings: Arc<ArcSwap<EngineSettings>>,
}

impl MainPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        entity_id_image: VirtualImage,
        depth: VirtualImage,
        shadow_factor: VirtualImage,
        gtao_history_a: VirtualImage,
        gtao_history_b: VirtualImage,
        sh_image: VirtualImage,
        brdf_lut_image: VirtualImage,
        brdf_lut_descriptor: u32,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        draw_count_main: VirtualBuffer,
        indirect_main: VirtualBuffer,
        draw_data_main: VirtualBuffer,
        bone_transform: VirtualBuffer,
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

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,
            entity_id_image,
            depth,
            shadow_factor,
            gtao_history_a,
            gtao_history_b,
            sh_image,
            brdf_lut_image,

            brdf_lut_descriptor,

            scene_buffer,
            entity_buffer,
            draw_count_main,
            indirect_main,
            draw_data_main,
            bone_transform,

            settings: resources.settings.clone(),
        })
    }
}

impl Pass for MainPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("main")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_image(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .read_image(
                self.shadow_factor,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
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
                self.draw_count_main,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.indirect_main,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.draw_data_main,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.bone_transform,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
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

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let shadow_factor_image = image_scope.get_physical_image(self.shadow_factor);
        let shadow_factor_descriptor_id = shadow_factor_image
            .descriptors
            .full
            .expect("Main shadow factor image must have a sampled descriptor");

        let gtao_history = if context.history_write_index == 0 {
            self.gtao_history_a
        } else {
            self.gtao_history_b
        };
        let gtao_image = image_scope.get_physical_image(gtao_history);

        let sh_image = image_scope.get_physical_image(self.sh_image);
        let sh_descriptor_id = sh_image.descriptors.full.unwrap_or(0);

        let settings = self.settings.load();
        let gtao_enabled = settings.render.gtao_enabled.value;
        let gtao_descriptor_id = gtao_image.descriptors.full.unwrap_or(0);

        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let draw_count_main = buffer_scope.get_physical_buffer(self.draw_count_main);
        let indirect_main = buffer_scope.get_physical_buffer(self.indirect_main);
        let draw_data_main_buffer = buffer_scope.get_physical_buffer(self.draw_data_main);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);

        context.bind_index_buffer();

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &MainPushConstants::create(
                scene_buffer,
                draw_data_main_buffer,
                context.resource_buffers.vertex_buffer,
                entity_buffer,
                context.resource_buffers.submesh_buffer,
                context.resource_buffers.material_buffer,
                bone_transform_buffer,
                shadow_factor_descriptor_id,
                gtao_descriptor_id,
                gtao_enabled as u32,
                sh_descriptor_id,
                self.brdf_lut_descriptor,
            ),
        );
        context.draw_indirect_gpu_scene(
            &indirect_main,
            &draw_count_main,
        );

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
