use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BufferMemoryBarrier, DependencyFlags, DeviceAddress, MemoryBarrier, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, WHOLE_SIZE};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::buffer::typed::culling_views_buffer::CullingViewGpuData;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::buffer::typed::entity_buffer::EntityGpuData;
use crate::render::vulkan::buffer::typed::scene_buffer::{MainCameraGpuData, SceneGpuData, ShadowCascadeGpuData};
use crate::render::vulkan::render_pass::culling_indirect_pass::culling_indirect_push_constants::CullingIndirectPushConstants;
use crate::render::vulkan::render_pass::render_pass_layout::RenderView;
use crate::render::vulkan::renderer::stats::gpu_render_stats::GpuRenderStats;
use crate::render::vulkan::renderer::stats::gpu_render_stats_handler::GpuRenderStatsHandler;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct CullingIndirectRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _compute_pipeline_handle: Arc<ResRef>,

    gpu_render_stats_buffer_device_address: DeviceAddress,
    
    buffer_manager: Arc<BufferManager>,
}

impl CullingIndirectRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        compute_pipeline_provider: &ResourceProvider<ComputePipelineBackend>,
        persistent_resources: &PersistentResources,
        render_stats_reader: &GpuRenderStatsHandler,
    ) -> Result<Self> {
        let compute_pipeline_config = ComputePipelineConfig {
            shader_name: String::from("shaders/culling_indirect/culling_indirect.comp.spv"),
            fn_name: String::from("main"),
        };

        let compute_pipeline_handle = compute_pipeline_provider.acquire_sync(compute_pipeline_config);
        let Some(pipeline) = compute_pipeline_provider.get_resource(compute_pipeline_handle.id) else {
            bail!("Failed to acquire ComputePipeline");
        };

        Ok(Self {
            pipeline,
            pipeline_layout: persistent_resources.pipeline_layouts.global,

            _compute_pipeline_handle: compute_pipeline_handle,
            
            gpu_render_stats_buffer_device_address: render_stats_reader.buffer.device_address.unwrap(),
            
            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }

    fn push_to_culling_views(
        &self,
        render_view: &RenderView,
        culling_views: &mut Vec<CullingViewGpuData>,
    ) {
        let chunk_index = culling_views.len() as u32;

        culling_views.push(
            CullingViewGpuData::create(
                render_view.projection_view,
                self.buffer_manager.indirect_buffer.ptr_to_chunk(chunk_index),
                self.buffer_manager.draw_count_buffer.ptr_to_chunk(chunk_index),
                self.buffer_manager.draw_data_buffer.ptr_to_chunk(chunk_index),
            )
        );
    }
}

impl RenderPass for CullingIndirectRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        let mut culling_views = Vec::new();

        self.push_to_culling_views(&render_pass_context.render_views_layout.main, &mut culling_views);
        for render_view in &render_pass_context.render_views_layout.global_shadow_cascades {
            self.push_to_culling_views(&render_view, &mut culling_views);
        }

        self.buffer_manager.culling_views_buffer.replace_with(&culling_views)?;

        unsafe { 
            device.cmd_fill_buffer(
                command_buffer, 
                self.buffer_manager.draw_count_buffer.handle.handle, 
                0, 
                self.buffer_manager.draw_count_buffer.handle.size,
                0,
            ) 
        };

        let entities_gpu_data: Vec<EntityGpuData> = render_pass_context.world_snapshot.entities.iter().map(|entity| {
            EntityGpuData::create(entity.transform_matrix, entity.model_id)
        }).collect();
        self.buffer_manager.entity_buffer.replace_with(&entities_gpu_data)?;

        let main_projection_view = render_pass_context.render_views_layout.main.projection_view;
        let main_camera_gpu_data = MainCameraGpuData::new(
            main_projection_view.to_cols_array_2d(),
            render_pass_context.world_snapshot.camera_stamp.position.to_array(),
            render_pass_context.world_snapshot.camera_stamp.near,
            render_pass_context.world_snapshot.camera_stamp.far,
        );
        let main_projection_view_inverted = main_projection_view.inverse();

        let mut shadow_cascade_count: u32 = 0;
        let mut shadow_cascades = [ShadowCascadeGpuData::default(); 4];

        for render_view in &render_pass_context.render_views_layout.global_shadow_cascades {
            shadow_cascades[shadow_cascade_count as usize] = ShadowCascadeGpuData::new(
                render_view.projection_view.to_cols_array_2d(),
                (render_view.projection_view * main_projection_view_inverted).to_cols_array_2d(),
                render_pass_context.shadow_layout.shadow_cascades[shadow_cascade_count as usize].end,
            );

            shadow_cascade_count += 1;
        };

        let scene_gpu_data: SceneGpuData = SceneGpuData::create(
            main_camera_gpu_data,
            render_pass_context.world_snapshot.global_shadows_direction.to_array(),
            shadow_cascade_count,
            shadow_cascades,
        );

        render_pass_context.push_using_staging(&self.buffer_manager.scene_buffer.handle, scene_gpu_data)?;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::COMPUTE_SHADER,
                DependencyFlags::empty(),
                &[
                    MemoryBarrier::default()
                        .src_access_mask(AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE)
                ],
                &[],
                &[],
            )
        };

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        render_pass_context.bind_pipeline(PipelineBindPoint::COMPUTE, self.pipeline);

        let entity_count = render_pass_context.world_snapshot.entities.len() as u32;
        if entity_count == 0 {
            return Ok(());
        }

        let stats_size = size_of::<GpuRenderStats>() as DeviceAddress;
        let stats_buffer_device_address_offset = stats_size * render_pass_context.frame_index as DeviceAddress;

        render_pass_context.push_constants(
            self.pipeline_layout,
            &CullingIndirectPushConstants::create(
                self.buffer_manager.culling_views_buffer.handle.device_address.unwrap(),
                self.buffer_manager.entity_buffer.handle.device_address.unwrap(),
                self.buffer_manager.submesh_buffer.handle.device_address.unwrap(),
                self.buffer_manager.model_buffer.handle.device_address.unwrap(),
                self.gpu_render_stats_buffer_device_address + stats_buffer_device_address_offset,
                render_pass_context.render_views_layout.count(),
                entity_count,
            ),
        );

        let workgroups = (entity_count + 255) / 256;

        unsafe { device.cmd_dispatch(command_buffer, workgroups, 1, 1) };

        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let device = &render_pass_context.device_context.device;
        let command_buffer = render_pass_context.command_recording.command_buffer;

        let barriers = [
            BufferMemoryBarrier::default()
                .src_access_mask(AccessFlags::SHADER_WRITE)
                .dst_access_mask(AccessFlags::INDIRECT_COMMAND_READ)
                .buffer(self.buffer_manager.draw_count_buffer.handle.handle)
                .size(WHOLE_SIZE),
            BufferMemoryBarrier::default()
                .src_access_mask(AccessFlags::SHADER_WRITE)
                .dst_access_mask(AccessFlags::INDIRECT_COMMAND_READ)
                .buffer(self.buffer_manager.indirect_buffer.handle.handle)
                .size(WHOLE_SIZE),
            BufferMemoryBarrier::default()
                .src_access_mask(AccessFlags::SHADER_WRITE)
                .dst_access_mask(AccessFlags::SHADER_READ)
                .buffer(self.buffer_manager.draw_data_buffer.handle.handle)
                .size(WHOLE_SIZE),
        ];

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::COMPUTE_SHADER,
                PipelineStageFlags::DRAW_INDIRECT | PipelineStageFlags::VERTEX_SHADER,
                DependencyFlags::empty(),
                &[],
                &barriers,
                &[],
            );
        };

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("CullingRenderPass destroyed");

        Ok(())
    }
}
