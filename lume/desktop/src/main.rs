use std::env::var;
use std::sync::Arc;
use tracing::warn;
use crate::application::Application;
use crate::desktop_ui_renderer::DesktopUiRenderer;
use crate::platform_providers::desktop_io_provider::DesktopIOProvider;
use amber_lume::amber_lume::AmberLume;
use amber_lume::limits::{AmberLumeLimits, RenderLimits, HiZFormat, HiZParams, PhysicsLimits, ProfilerLimits, ResourceLimits, ShadowMapFormat, ShadowMapParams};
use gpu::VulkanLayer;
use gpu::ValidationFeatures;
use settings::EngineSettings;
use core::lume::Lume;
use core::tracing::Tracing;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
#[cfg(feature = "x11")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::WindowAttributes;
mod application;
mod cursor_capture;
mod platform_providers;
mod desktop_ui_renderer;

fn main() {
    Tracing::initialize();

    let size = Size::Physical(PhysicalSize::new(1280, 720));
    let config = WindowAttributes::default()
        .with_title(String::from("AmberLume"))
        .with_inner_size(size)
        .with_resizable(true);

    let event_loop = create_event_loop();
    let display_handle = event_loop.display_handle().expect("display_handle").as_raw();

    let (layers, validation_features) = parse_validation_env();
    let ui_renderer = Arc::new(DesktopUiRenderer::new());
    let io_provider = Arc::new(DesktopIOProvider::new());

    let amber_lume = AmberLume::new(
        build_limits(),
        layers,
        validation_features,
        ui_renderer,
        io_provider,
        display_handle,
        EngineSettings::default(),
    ).expect("AmberLume creation failed");
    let lume = Lume::new(amber_lume).expect("Lume creation failed");

    let mut application = Application::new(config, lume);

    event_loop.run_app(&mut application).unwrap();

    if let Err(e) = application.destroy() {
        panic!("failed to destroy application: {}", e);
    };
}

fn create_event_loop() -> EventLoop<()> {
    #[cfg(feature = "x11")]
    let event_loop = EventLoop::builder().with_x11().build().unwrap();

    #[cfg(not(feature = "x11"))]
    let event_loop = EventLoop::builder().build().unwrap();

    event_loop
}

fn parse_validation_env() -> (Vec<VulkanLayer>, Vec<ValidationFeatures>) {
    let Ok(raw) = var("AMBERLUME_VK_VALIDATION") else {
        return (vec![], vec![]);
    };

    let features = raw.split(",")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|token| match token {
            "synchronization" => Some(ValidationFeatures::Synchronization),
            "best_practices" => Some(ValidationFeatures::BestPractices),
            "gpu_assisted"  => Some(ValidationFeatures::GpuAssisted),
            other  => {
                warn!("unknown AMBERLUME_VK_VALIDATION token: {}", other);
                None
            },
        })
        .collect();

    (vec![VulkanLayer::Validation], features)
}

fn build_limits() -> AmberLumeLimits {
    AmberLumeLimits {
        render: RenderLimits {
            frames_in_flight: 2,
            resource_limits: ResourceLimits {
                max_frame_heap_size: 4 * 1024 * 1024,
    
                max_staging_size: 64 * 1024 * 1024,
    
                max_indices: 500_000,
                max_vertices: 2_000_000,
                max_vertex_attributes: 2_000_000,
                max_vertex_skins: 200_000,
    
                max_meshes: 1_536,
                max_submeshes: 2_048,
                max_materials: 1_000,
    
                max_skeletons: 16,
                max_skeleton_bones: 1024,
                max_bones_per_skeleton: 128,
    
                max_animations: 128,
                max_animation_frames: 16 * 1024,
    
                max_skinning_instances: 128,
                max_bone_transforms: 1024,
    
                max_draw_calls: 100_000,
                max_transparent_draw_calls: 8192,
                max_sorted_draw_calls: 1024,
                max_render_views: 2,
    
                max_texture_descriptors: 1024,
                max_shadow_array_descriptors: 16,
                max_storage_image_descriptors: 128,
                max_graph_texture_descriptors: 256,
            },
            shadow_map_limits: ShadowMapParams {
                cascade_count: 4,
                max_distance: 64.0,
                resolution: 4096,
                format: ShadowMapFormat::D32,
                bias: 0.02,
                normal_bias: 0.08,
                pcf_sample_count: 8,
                pcf_world_radius: 0.02,
                cascade_blend_range: 0.05,
                split_lambda: 0.9,
                shadow_caster_extension: 100.0,
                z_far_sample_stride: 1,
            },
            hiz_limits: HiZParams {
                format: HiZFormat::Rg32,
            },
            profiler_limits: ProfilerLimits {
                max_gpu_zones: 64,
            },
        },
        physics_limits: PhysicsLimits {
            fixed_delta_time: 1.0 / 60.0,
        },
    }
}
