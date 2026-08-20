use crate::render::frame_data::terrain_frame::TerrainFrame;
use crate::render::frame_data::terrain_stitch_request_gpu::TerrainStitchRequestGPU;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::terrain_stitch::terrain_stitch_push_constants::TerrainStitchPushConstants;
use crate::resource_manifest::shaders;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, DependencyFlags, DeviceAddress, MemoryBarrier, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use gpu::PipelineLayoutType;
use gpu::ResourceFactories;
use pipeline_store::ComputePipelineConfig;
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
use resource_residency::ResRef;
use std::sync::Arc;
use terrain::ChunkGeometry;
use tracing::info;

pub struct TerrainStitchPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    terrain_stitch_request: VirtualBuffer,
    terrain_edge_height: VirtualBuffer,

    vertex_buffer: DeviceAddress,
    mesh_buffer: DeviceAddress,
    submesh_buffer: DeviceAddress,

    terrain_frame: VirtualData<TerrainFrame>,
}

impl TerrainStitchPass {
    pub fn create(
        resources: &PassResources,
        terrain_stitch_request: VirtualBuffer,
        terrain_edge_height: VirtualBuffer,
        terrain_frame: VirtualData<TerrainFrame>,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: shaders::TERRAIN_STITCH_COMP,
            fn_name: String::from("main"),
            specialization_entries: Vec::new(),
        };

        let _handle = resources.compute_pipeline_provider.acquire_sync(compute_pipeline_config)?;
        let Some(pipeline) = resources.compute_pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire ComputePipeline for TerrainStitch");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            terrain_stitch_request,
            terrain_edge_height,

            vertex_buffer: resources.resource_buffers.vertex_buffer,
            mesh_buffer: resources.resource_buffers.mesh_buffer,
            submesh_buffer: resources.resource_buffers.submesh_buffer,

            terrain_frame,
        })
    }
}

pub struct TerrainStitchPassData {
    node_count: u32,
}

impl Pass for TerrainStitchPass {
    type PassData = TerrainStitchPassData;

    fn name(&self) -> String {
        String::from("terrain_stitch")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.terrain_frame)
            .write_buffer(
                self.terrain_stitch_request,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.terrain_stitch_request,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            )
            .write_buffer(
                self.terrain_edge_height,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.terrain_edge_height,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::COMPUTE_SHADER,
            );
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let terrain_frame = data_scope.get(self.terrain_frame);

        let mut requests = Vec::with_capacity(terrain_frame.stitch_requests.len());
        let mut edge_heights = Vec::with_capacity(
            terrain_frame.stitch_requests.len() * ChunkGeometry::EDGE_LENGTH,
        );

        for terrain_stitch_request in terrain_frame.stitch_requests.iter() {
            requests.push(TerrainStitchRequestGPU::create(
                terrain_stitch_request.mesh_id.inner,
                edge_heights.len() as u32,
                terrain_stitch_request.level_deltas,
            ));

            edge_heights.extend_from_slice(&terrain_stitch_request.edge_heights);
        }

        self.terrain_stitch_request.stage_slice(buffer_scope, allocator, &requests)?;
        self.terrain_edge_height.stage_slice(buffer_scope, allocator, &edge_heights)?;


        Ok(Self::PassData {
            node_count: requests.len() as u32 * ChunkGeometry::PERIMETER_NODE_COUNT,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let node_count = data.node_count;
        if node_count == 0 {
            return Ok(());
        }

        let terrain_stitch_request = buffer_scope.get_physical_buffer(self.terrain_stitch_request);
        let terrain_edge_height = buffer_scope.get_physical_buffer(self.terrain_edge_height);

        context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &TerrainStitchPushConstants::create(
                terrain_stitch_request,
                terrain_edge_height,
                self.vertex_buffer,
                self.mesh_buffer,
                self.submesh_buffer,
                node_count,
                ChunkGeometry::NODES,
            ),
        );

        context.dispatch(node_count);

        context.pipeline_barrier(
            PipelineStageFlags::COMPUTE_SHADER,
            PipelineStageFlags::VERTEX_SHADER
                | PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            DependencyFlags::empty(),
            &[MemoryBarrier::default()
                .src_access_mask(AccessFlags::SHADER_WRITE)
                .dst_access_mask(AccessFlags::SHADER_READ)],
            &[],
            &[],
        );

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("{} destroyed", self.name());

        Ok(())
    }
}
