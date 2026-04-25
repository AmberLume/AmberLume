mod platform_providers;
mod input_event;

use crate::platform_providers::io_provider::AndroidIOProvider;
use crate::platform_providers::surface_provider::AndroidSurfaceProvider;
use amber_lume::platform_providers::providers::Providers;
use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use core::lume::Lume;
use std::sync::{Arc, Once};
use std::time::Duration;
use tracing::{error, info};
use tracing_android::layer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{registry, EnvFilter};
use amber_lume::limits::renderer_limits::{BufferLimits, ImageResourceLimits, RenderResourceLimits, RendererLimits, ShadowMapFormat, ShadowMapParams};
use crate::input_event::handle_input_event;

static INIT_LOGGER: Once = Once::new();

#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    INIT_LOGGER.call_once(init_tracing);

    std::panic::set_hook(Box::new(|info| {
        error!("panic: {info}");
    }));

    info!("android_main: started");

    let mut quit = false;

    let mut lume: Option<Lume> = None;

    while !quit {
        android_app.poll_events(Some(Duration::from_millis(16)), |event| match event {
            PollEvent::Main(MainEvent::InitWindow { .. }) => {
                info!("InitWindow");

                let surface_provider = AndroidSurfaceProvider::new(android_app.clone());
                let io_provider = AndroidIOProvider::new(android_app.clone());

                let providers = Providers {
                    surface_provider: Arc::new(surface_provider),
                    io_provider: Arc::new(io_provider),
                };

                let layers = vec![];

                let limits = RendererLimits {
                    frames_in_flight: 1,
                    buffer_limits: BufferLimits {
                        max_entities: 100_000,

                        max_staging_size: 32 * 1024 * 1024,
                    },
                    render_resource_limits: RenderResourceLimits {
                        max_indices: 500_000,
                        max_vertices: 1_500_000,

                        max_meshes: 100,
                        max_submeshes: 1_000,
                        max_materials: 1_000,

                        max_skeletons: 16,
                        max_skeleton_bones: 1024,
                        max_bones_per_skeleton: 128,

                        max_animations: 128,
                        max_animation_frames: 1048576,

                        max_skinning_instances: 128,
                        max_bone_transforms: 1024,

                        max_draw_calls: 1_000_000,
                        max_render_views: 5,
                    },
                    image_resource_limits: ImageResourceLimits {
                        max_texture_descriptors: 1024,
                        max_texture_array_descriptors: 16,
                        max_shadow_descriptors: 256,
                        max_shadow_array_descriptors: 16,
                    },
                    shadow_map_limits: ShadowMapParams {
                        global_cascades: vec![0.0..8.0, 8.0..32.0],
                        resolution: 2048,
                        format: ShadowMapFormat::D32,
                        bias: 0.00005,
                        pcf_count: 0,
                    },
                };

                lume = Some(Lume::create(providers, limits, layers).expect("Lume creation failed"));
            }
            PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                info!("TerminateWindow");

                lume = None;
            }
            PollEvent::Main(MainEvent::WindowResized { .. }) => {
                info!("WindowResized");

                if let Some(lume) = lume.as_mut() {
                    lume.on_update_surface();
                }
            }
            PollEvent::Main(MainEvent::Destroy) => {
                info!("Destroy");

                if let Some(lume) = lume.take() {
                    match lume.on_close() {
                        Ok(_) => info!("Window closed successfully"),
                        Err(error) => panic!("Window closed with error: {}", error),
                    }
                }

                quit = true
            }
            _ => {}
        });

        if let Some(lume_ref) = lume.as_mut() {
            match android_app.input_events_iter() {
                Ok(mut iter) => loop {
                    let read = iter.next(|event| {
                        handle_input_event(event, lume_ref);

                        InputStatus::Unhandled
                    });
                    if !read { break; }
                },
                Err(e) => error!("input_events_iter: {e:?}"),
            }

            if let Err(e) = lume_ref.draw() {
                error!("draw failed: {e:?}");
            }
        }

        if let Some(lume) = lume.as_mut() {
            if let Err(e) = lume.draw() {
                panic!("Failed to draw frame: {:?}", e);
            }
        }
    }

    info!("android_main: exited");
}

fn init_tracing() {
    let android_layer = layer("lume").expect("tracing_android init");

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,lume=debug"));

    registry().with(android_layer).with(filter).init();
}
