use std::sync::Arc;
use ash::vk::{BufferMemoryBarrier, DependencyFlags, PipelineStageFlags};
use crate::ids::FrameIndex;
use crate::limits::AmberLumeLimits;
use crate::render::device::device_context::DeviceContext;
use crate::render::frame::command_recording::CommandRecording;
use crate::render::pass::pass_layout::RenderViewsLayout;
use crate::render::pass::ui::ui_snapshot::{UiSnapshot};
use crate::snapshot_handler::render_snapshot::RenderSnapshot;
use crate::ui::ui_context::UiContext;

pub struct FrameDataContext<'pass_prepare> {
    pub frame_index: FrameIndex,

    device_context: &'pass_prepare DeviceContext,
    command_recording: &'pass_prepare CommandRecording,

    pub limits: &'pass_prepare AmberLumeLimits,

    pub render_views_layout: &'pass_prepare RenderViewsLayout,

    pub render_snapshot: Arc<RenderSnapshot>,
    
    pub ui_snapshot: UiSnapshot,
    pub ui_context: &'pass_prepare UiContext,
}

impl<'pass_prepare> FrameDataContext<'pass_prepare> {
    pub fn create(
        frame_index: FrameIndex,
        device_context: &'pass_prepare DeviceContext,
        command_recording: &'pass_prepare CommandRecording,
        limits: &'pass_prepare AmberLumeLimits,
        render_views_layout: &'pass_prepare RenderViewsLayout,
        render_snapshot: Arc<RenderSnapshot>,
        ui_snapshot: UiSnapshot,
        ui_context: &'pass_prepare UiContext,
    ) -> Self {
        Self {
            frame_index,

            device_context,
            command_recording,

            limits,

            render_views_layout,

            render_snapshot,
            
            ui_snapshot,
            ui_context,
        }
    }

    pub fn pipeline_barrier(
        &self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        buffer_memory_barriers: &[BufferMemoryBarrier],
    ) {
        let device = &self.device_context.device;
        let command_buffer = self.command_recording.command_buffer;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                &[],
                buffer_memory_barriers,
                &[],
            )
        }
    }
}
