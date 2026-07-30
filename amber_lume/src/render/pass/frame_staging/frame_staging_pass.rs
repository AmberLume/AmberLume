use anyhow::Result;
use ash::vk::{AccessFlags, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::frame_data::culling_view_gpu::CullingViewGPU;
use crate::render::frame_data::entity_gpu::EntityGPU;
use crate::render::frame_data::scene_gpu::{MainCameraGPU, SceneGPU};
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;

pub struct FrameStagingPass {
    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,
}

impl FrameStagingPass {
    pub fn create(
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
    ) -> Self {
        Self {
            scene_buffer,
            entity_buffer,
            culling_view_buffer,
        }
    }
}

impl Pass for FrameStagingPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("frame_staging")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let entities_gpu: Vec<EntityGPU> = context.render_snapshot.entities.iter().enumerate().map(|(index, entity)| {
            let is_skinned = entity.animation.is_some();

            EntityGPU::create(
                entity.transform_matrix,
                entity.mesh_id,
                is_skinned,
                entity.animation.as_ref()
                    .map(|a| a.bone_transform_offset)
                    .unwrap_or(0),
                context.previous_transforms[index],
            )
        }).collect();

        self.entity_buffer.stage_slice(buffer_scope, allocator, &entities_gpu)?;

        let main_view = &context.render_views_layout.main;
        let main_projection_view = &main_view.view_projection;
        let main_camera_gpu = MainCameraGPU::new(
            main_projection_view,
            &main_view.previous_view_projection,
            &main_view.jittered_view_projection,
            &main_view.view,
            context.render_snapshot.camera.position,
            context.render_snapshot.camera.near,
            context.render_snapshot.camera.far,
            main_view.ndc_to_view_mul,
            main_view.ndc_to_view_add,
            main_view.mip_bias,
        );

        let scene_gpu: SceneGPU = SceneGPU::create(
            main_camera_gpu,
            context.render_snapshot.global_shadows_direction.to_array(),
            context.render_snapshot.global_shadows_color.to_array(),
            context.render_snapshot.global_shadows_intensity,
            context.render_snapshot.global_ibl_intensity,
            context.render_views_layout.cascade_count,
            context.render_snapshot.time,
        );

        self.scene_buffer.stage_slice(buffer_scope, allocator, &[scene_gpu])?;

        let culling_views = [CullingViewGPU::create(main_projection_view)];

        self.culling_view_buffer.stage_slice(buffer_scope, allocator, &culling_views)?;

        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_buffer(
                self.scene_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.entity_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.culling_view_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            );
    }

    fn record_commands(
        &self,
        _context: &PassContext,
        _image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _data: Self::PassData,
    ) -> Result<()> {
        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("FrameStagingPass destroyed");

        Ok(())
    }
}
