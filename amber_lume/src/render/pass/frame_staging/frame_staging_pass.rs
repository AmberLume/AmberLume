use anyhow::Result;
use ash::vk::{AccessFlags, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::culling_indirect::gpu::culling_view_gpu::CullingViewGPU;
use crate::render::pass::frame_staging::gpu::entity_gpu::EntityGPU;
use crate::render::pass::frame_staging::gpu::entity_motion_gpu::EntityMotionGPU;
use crate::render::pass::frame_staging::gpu::entity_outline_gpu::EntityOutlineGPU;
use crate::render::pass::frame_staging::gpu::camera_gpu::CameraGPU;
use crate::render::pass::frame_staging::gpu::scene_gpu::SceneGPU;
use crate::render::view::render_views_layout::RenderViewsLayout;
use render_graph::FrameContext;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualBuffer;
use render_graph::VirtualData;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use render_snapshot::RenderSnapshot;
use glam::Mat4;

pub struct FrameStagingPass {
    scene_buffer: VirtualBuffer,
    camera_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    entity_motion_buffer: VirtualBuffer,
    entity_outline_buffer: VirtualBuffer,
    main_culling_views_buffer: VirtualBuffer,
    mesh_vertex_buffer: VirtualBuffer,
    mesh_vertex_attribute_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,
    render_views_layout: VirtualData<RenderViewsLayout>,
    previous_transforms: VirtualData<Vec<Mat4>>,
}

impl FrameStagingPass {
    pub fn create(
        scene_buffer: VirtualBuffer,
        camera_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        entity_motion_buffer: VirtualBuffer,
        entity_outline_buffer: VirtualBuffer,
        main_culling_views_buffer: VirtualBuffer,
        mesh_vertex_buffer: VirtualBuffer,
        mesh_vertex_attribute_buffer: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        render_views_layout: VirtualData<RenderViewsLayout>,
        previous_transforms: VirtualData<Vec<Mat4>>,
    ) -> Self {
        Self {
            scene_buffer,
            camera_buffer,
            entity_buffer,
            entity_motion_buffer,
            entity_outline_buffer,
            main_culling_views_buffer,
            mesh_vertex_buffer,
            mesh_vertex_attribute_buffer,

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
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let mesh_vertex_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_buffer);
        let mesh_vertex_attribute_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_attribute_buffer);

        let render_snapshot = scopes.data.get(self.render_snapshot);
        let previous_transforms = scopes.data.get(self.previous_transforms);

        let render_views_layout = scopes.data.get(self.render_views_layout);

        let entity_count = render_snapshot.entities.len();

        let mut entities_gpu: Vec<EntityGPU> = Vec::with_capacity(entity_count);
        let mut entity_motions_gpu: Vec<EntityMotionGPU> = Vec::with_capacity(entity_count);
        let mut entity_outlines_gpu: Vec<EntityOutlineGPU> = Vec::with_capacity(entity_count);

        for (index, entity) in render_snapshot.entities.iter().enumerate() {
            entities_gpu.push(EntityGPU::create(
                entity.transform_matrix,
                entity.mesh_id,
                mesh_vertex_buffer.range,
                mesh_vertex_attribute_buffer.range,
            ));
            entity_motions_gpu.push(EntityMotionGPU::create(previous_transforms[index]));
            entity_outlines_gpu.push(EntityOutlineGPU::create(entity.outline));
        }

        self.entity_buffer.stage_slice(scopes.buffer, &entities_gpu)?;
        self.entity_motion_buffer.stage_slice(scopes.buffer, &entity_motions_gpu)?;
        self.entity_outline_buffer.stage_slice(scopes.buffer, &entity_outlines_gpu)?;

        let main_view = &render_views_layout.main;
        let main_projection_view = &main_view.view_projection;
        let main_inverse_view_projection = main_projection_view.inverted();
        let camera_gpu = CameraGPU::new(
            main_projection_view,
            &main_view.previous_view_projection,
            &main_inverse_view_projection,
            &main_view.view,
            render_snapshot.camera.position,
            render_snapshot.camera.near,
            render_snapshot.camera.far,
            main_view.tan_half_fov,
            main_view.jitter,
            main_view.mip_bias,
        );
        self.camera_buffer.stage_slice(scopes.buffer, &[camera_gpu])?;

        let scene_gpu: SceneGPU = SceneGPU::create(
            render_snapshot.global_shadows_direction.to_array(),
            render_snapshot.global_shadows_color.to_array(),
            render_snapshot.global_shadows_intensity,
            render_snapshot.global_ibl_intensity,
            render_views_layout.cascade_count,
            render_snapshot.time,
        );
        self.scene_buffer.stage_slice(scopes.buffer, &[scene_gpu])?;

        let culling_view = CullingViewGPU::create(main_projection_view);
        self.main_culling_views_buffer.stage_slice(scopes.buffer, &[culling_view])?;

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
                self.camera_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.entity_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.entity_motion_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.entity_outline_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.main_culling_views_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            );
    }

    fn record_commands(
        &self,
        _context: &FrameContext,
        _scopes: &RecordScopes,
        _data: Self::PassData,
    ) -> Result<()> {
        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("FrameStagingPass destroyed");

        Ok(())
    }
}
