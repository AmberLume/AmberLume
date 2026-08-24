use crate::render::frame_data::terrain_chunk_view_gpu::TerrainChunkViewGPU;
use crate::render::frame_data::terrain_frame::TerrainFrame;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::terrain_points::terrain_points_push_constants::TerrainPointsPushConstants;
use crate::resource_manifest::shaders;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CompareOp, CullModeFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology};
use gpu::PipelineLayoutType;
use gpu::ResourceFactories;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::FrameContext;
use render_graph::HeapAllocator;
use render_graph::ImageResourceScope;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::ReadbackScope;
use render_graph::VirtualBuffer;
use render_graph::VirtualData;
use render_graph::VirtualImage;
use render_graph::{ColorTarget, DepthTarget, RenderTargets};
use resource_residency::ResRef;
use settings::RenderSettings;
use std::sync::Arc;
use terrain::ChunkGeometry;
use tracing::info;

pub struct TerrainPointsPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    depth_image: VirtualImage,

    terrain_chunk_buffer: VirtualBuffer,
    scene_buffer: VirtualBuffer,

    mesh_vertex_buffer: VirtualBuffer,
    mesh_buffer: VirtualBuffer,
    submesh_buffer: VirtualBuffer,

    terrain_frame: VirtualData<TerrainFrame>,
    render_settings: VirtualData<RenderSettings>,
}

impl TerrainPointsPass {
    pub const POINT_WORLD_SIZE: f32 = 0.1;

    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        depth_image: VirtualImage,
        terrain_chunk_buffer: VirtualBuffer,
        scene_buffer: VirtualBuffer,
        terrain_frame: VirtualData<TerrainFrame>,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "terrain_points".to_string(),

            stages: vec![
                PipelineStageConfig::fragment(shaders::TERRAIN_POINTS_FRAG),
                PipelineStageConfig::vertex(shaders::TERRAIN_POINTS_VERT),
            ],

            color_formats: vec![color_format],
            depth_format: Some(resources.render_context.depth_format),

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::POINT,
            primitive_topology: PrimitiveTopology::POINT_LIST,

            depth_test: true,
            depth_write: false,
            depth_compare_op: CompareOp::GREATER_OR_EQUAL,

            ..PipelineConfig::fullscreen()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config)?;
        let Some(pipeline) = resources.pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire Pipeline for TerrainPoints");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,
            depth_image,

            terrain_chunk_buffer,
            scene_buffer,

            mesh_vertex_buffer: resources.resource_buffer_handles.mesh_vertex_buffer,
            mesh_buffer: resources.resource_buffer_handles.mesh_buffer,
            submesh_buffer: resources.resource_buffer_handles.submesh_buffer,

            terrain_frame,
            render_settings,
        })
    }
}

pub struct TerrainPointsPassData {
    point_count: u32,
}

impl Pass for TerrainPointsPass {
    type PassData = TerrainPointsPassData;

    fn name(&self) -> String {
        String::from("terrain_points")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).terrain_vertex_points.value
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let terrain_frame = data_scope.get(self.terrain_frame);

        let chunks = terrain_frame
            .chunks
            .iter()
            .map(|chunk| TerrainChunkViewGPU::create(chunk.center, chunk.level, chunk.mesh_id.inner))
            .collect::<Vec<_>>();

        self.terrain_chunk_buffer.stage_slice(buffer_scope, allocator, &chunks)?;

        Ok(TerrainPointsPassData {
            point_count: chunks.len() as u32 * ChunkGeometry::NODE_COUNT,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.terrain_frame)
            .consume(self.render_settings)
            .read_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_image(
                self.depth_image,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .write_buffer(
                self.terrain_chunk_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.terrain_chunk_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.submesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.mesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget { image: self.target_image, mip: None, clear: None }],
            depth: Some(DepthTarget { image: self.depth_image, clear: None }),
            view_mask: 0,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let mesh_buffer = buffer_scope.get_physical_buffer(self.mesh_buffer);
        let mesh_vertex_buffer = buffer_scope.get_physical_buffer(self.mesh_vertex_buffer);
        let submesh_buffer = buffer_scope.get_physical_buffer(self.submesh_buffer);

        if data.point_count == 0 {
            return Ok(());
        }

        let target = image_scope.get_physical_image(self.target_image);

        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let terrain_chunk_buffer = buffer_scope.get_physical_buffer(self.terrain_chunk_buffer);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &TerrainPointsPushConstants::create(
                scene_buffer.range,
                terrain_chunk_buffer.range,
                mesh_vertex_buffer.range,
                mesh_buffer.range,
                submesh_buffer.range,
                ChunkGeometry::NODE_COUNT,
                Self::POINT_WORLD_SIZE,
                target.extent.height as f32,
            ),
        );

        context.draw(data.point_count);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("{} destroyed", self.name());

        Ok(())
    }
}
