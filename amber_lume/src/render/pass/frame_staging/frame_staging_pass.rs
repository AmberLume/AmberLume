use render_graph::ReadbackScope;
use anyhow::Result;
use ash::vk::{AccessFlags, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::frame_data::culling_view_gpu::CullingViewGPU;
use crate::render::frame_data::entity_gpu::EntityGPU;
use crate::render::frame_data::scene_gpu::{MainCameraGPU, SceneGPU};
use crate::render::pass::pass_layout::RenderViewsLayout;
use render_graph::FrameContext;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::HeapAllocator;
use render_graph::VirtualBuffer;
use render_graph::VirtualData;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
use render_snapshot::RenderSnapshot;
use glam::Mat4;

pub struct FrameStagingPass {
    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    culling_view_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,
    render_views_layout: VirtualData<RenderViewsLayout>,
    previous_transforms: VirtualData<Vec<Mat4>>,
}

impl FrameStagingPass {
    pub fn create(
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        culling_view_buffer: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        render_views_layout: VirtualData<RenderViewsLayout>,
        previous_transforms: VirtualData<Vec<Mat4>>,
    ) -> Self {
        Self {
            scene_buffer,
            entity_buffer,
            culling_view_buffer,

            render_snapshot,
            render_views_layout,
            previous_transforms,
        }
    }
}

impl Pass for FrameStagingPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("frame_staging")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let render_snapshot = data_scope.get(self.render_snapshot);
        let previous_transforms = data_scope.get(self.previous_transforms);

        let render_views_layout = data_scope.get(self.render_views_layout);

        let entities_gpu: Vec<EntityGPU> = render_snapshot.entities.iter().enumerate().map(|(index, entity)| {
            let is_skinned = entity.animation.is_some();

            EntityGPU::create(
                entity.transform_matrix,
                entity.mesh_id,
                is_skinned,
                entity.animation.as_ref()
                    .map(|a| a.bone_transform_offset)
                    .unwrap_or(0),
                previous_transforms[index],
            )
        }).collect();

        self.entity_buffer.stage_slice(buffer_scope, allocator, &entities_gpu)?;

        let main_view = &render_views_layout.main;
        let main_projection_view = &main_view.view_projection;
        let main_inverse_view_projection = main_projection_view.inverted();
        let main_inverse_jittered_view_projection = main_view.jittered_view_projection.inverted();
        let main_camera_gpu = MainCameraGPU::new(
            main_projection_view,
            &main_view.previous_view_projection,
            &main_view.jittered_view_projection,
            &main_inverse_view_projection,
            &main_inverse_jittered_view_projection,
            &main_view.view,
            render_snapshot.camera.position,
            render_snapshot.camera.near,
            render_snapshot.camera.far,
            main_view.ndc_to_view_mul,
            main_view.ndc_to_view_add,
            main_view.mip_bias,
        );

        let scene_gpu: SceneGPU = SceneGPU::create(
            main_camera_gpu,
            render_snapshot.global_shadows_direction.to_array(),
            render_snapshot.global_shadows_color.to_array(),
            render_snapshot.global_shadows_intensity,
            render_snapshot.global_ibl_intensity,
            render_views_layout.cascade_count,
            render_snapshot.time,
        );

        self.scene_buffer.stage_slice(buffer_scope, allocator, &[scene_gpu])?;

        let culling_views = [CullingViewGPU::create(main_projection_view)];

        self.culling_view_buffer.stage_slice(buffer_scope, allocator, &culling_views)?;

        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.previous_transforms)
            .consume(self.render_snapshot)
            .consume(self.render_views_layout)
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
        _context: &FrameContext,
        _image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        _data: Self::PassData,
    ) -> Result<()> {
        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("FrameStagingPass destroyed");

        Ok(())
    }
}
