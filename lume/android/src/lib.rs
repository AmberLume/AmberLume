mod choreographer;
mod platform_providers;
mod input_event;
pub mod android_ui_renderer;
mod input_handler;

use std::ffi::c_void;
use std::panic::set_hook;
use crate::choreographer::{FrameRateBinding, VsyncDriver};
use crate::platform_providers::io_provider::AndroidIOProvider;
use crate::platform_providers::surface_provider::AndroidSurfaceProvider;
use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use core::lume::Lume;
use raw_window_handle::{AndroidDisplayHandle, RawDisplayHandle};
use std::sync::{Arc, Once};
use std::time::Duration;
use tracing::{error, info};
use tracing_android::layer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{registry, EnvFilter};
use amber_lume::amber_lume::AmberLume;
use amber_lume::lifecycle::lifecycle::AmberLumeLifecycle;
use amber_lume::limits::{AmberLumeLimits, PhysicsLimits, ResourceLimits, ShadowMapFormat, ShadowMapParams};
use amber_lume::settings::settings::EngineSettings;
use crate::android_ui_renderer::AndroidUiRenderer;
use crate::input_event::handle_input_event;
use crate::input_handler::InputHandler;

const PREFERRED_FRAME_RATE_HZ: f32 = 120.0;
const FRAME_RATE_COMPATIBILITY_DEFAULT: i8 = 0;
const POLL_TIMEOUT_VSYNC: Duration = Duration::from_millis(8);

static INIT_LOGGER: Once = Once::new();

#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    INIT_LOGGER.call_once(init_tracing);

    set_hook(Box::new(|info| error!("panic: {info}")));

    info!("android_main: started");

    let mut quit = false;

    let input_handler = Arc::new(InputHandler::new());
    let ui_renderer = Arc::new(AndroidUiRenderer::new(input_handler.clone()));

    let vsync_driver = VsyncDriver::create();
    match vsync_driver {
        Some(_) => info!("Vsync driver initialized"),
        None => info!("Vsync driver unavailable — falling back to busy loop"),
    }

    let frame_rate_binding = FrameRateBinding::create();
    match frame_rate_binding {
        Some(_) => info!("FrameRate binding initialized"),
        None => info!("FrameRate binding unavailable on this device"),
    }

    let limits = AmberLumeLimits {
        frames_in_flight: 3,
        resource_limits: ResourceLimits {
            max_frame_heap_size: 4 * 1024 * 1024,

            max_staging_size: 32 * 1024 * 1024,

            max_indices: 500_000,
            max_vertices: 100_000,

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

            max_draw_calls: 100_000,
            max_render_views: 3,

            max_texture_descriptors: 1024,
            max_shadow_array_descriptors: 16,
        },
        shadow_map_limits: ShadowMapParams {
            cascade_count: 2,
            max_distance: 32.0,
            resolution: 2048,
            format: ShadowMapFormat::D16,
            bias: 0.02,
            normal_bias: 0.04,
            pcf_world_radius: 0.02,
            pcf_sample_count: 4,
            cascade_blend_range: 0.15,
            split_lambda: 0.7,
            shadow_caster_extension: 60.0,
            z_far_sample_stride: 4,
        },
        physics_limits: PhysicsLimits {
            fixed_delta_time: 1.0 / 40.0,
        },
    };

    let io_provider = Arc::new(AndroidIOProvider::new(android_app.clone()));
    let display_handle = RawDisplayHandle::Android(AndroidDisplayHandle::new());
    let amber_lume = AmberLume::new(
        limits,
        vec![],
        vec![],
        ui_renderer.clone(),
        io_provider,
        display_handle,
        EngineSettings::default(),
    ).expect("AmberLume creation failed");
    let mut lume = Lume::new(amber_lume).expect("Lume creation failed");

    while !quit {
        let poll_timeout = match (vsync_driver, lume.is_render_target_attached()) {
            (Some(_), true) => Some(POLL_TIMEOUT_VSYNC),
            _ => None,
        };

        android_app.poll_events(poll_timeout, |event| match event {
            PollEvent::Main(MainEvent::InitWindow { .. }) => {
                info!("InitWindow");
                if let (Some(binding), Some(native_window)) = (frame_rate_binding, android_app.native_window()) {
                    let result = binding.set(
                        native_window.ptr().as_ptr() as *mut c_void,
                        PREFERRED_FRAME_RATE_HZ,
                        FRAME_RATE_COMPATIBILITY_DEFAULT,
                    );
                    info!("ANativeWindow_setFrameRate({PREFERRED_FRAME_RATE_HZ}) -> {result}");
                }

                let surface_provider = Arc::new(AndroidSurfaceProvider::new(android_app.clone()));
                let target = match lume.create_surface_target(surface_provider) {
                    Ok(target) => target,
                    Err(error) => {
                        error!("Lume create surface target failed: {error:?}");
                        return;
                    }
                };
                if let Err(error) = lume.attach_render_target(target) {
                    error!("Lume attach failed: {error:?}");
                }
            }
            PollEvent::Main(MainEvent::Pause) => {
                info!("Pause");

                lume.pause();
            }
            PollEvent::Main(MainEvent::Resume { .. }) => {
                info!("Resume");

                lume.resume();
            }
            PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                info!("TerminateWindow");

                if let Err(error) = lume.detach_render_target() {
                    error!("Lume detach failed: {error:?}");
                }
            }
            PollEvent::Main(MainEvent::WindowResized { .. }) => {
                info!("WindowResized");

                lume.on_update_surface();
            }
            PollEvent::Main(MainEvent::Destroy) => {
                info!("Destroy");

                if let Err(error) = lume.detach_render_target() {
                    error!("Lume detach failed: {error:?}");
                }

                quit = true
            }
            _ => {}
        });

        if !lume.is_render_target_attached() {
            continue;
        }

        let should_draw = match vsync_driver {
            Some(driver) => driver.consume_frame(),
            None => true,
        };

        if !should_draw {
            continue;
        }

        match android_app.input_events_iter() {
            Ok(mut iter) => loop {
                let read = iter.next(|event| {
                    handle_input_event(event, &mut lume);

                    InputStatus::Unhandled
                });
                if !read { break; }
            },
            Err(e) => error!("input_events_iter: {e:?}"),
        }

        for (key_event, state) in input_handler.drain() {
            lume.push_hardware_keycode_event(key_event, state)
        }

        if let Err(e) = lume.draw() {
            error!("draw failed: {e:?}");
        }

        if let Some(driver) = vsync_driver {
            driver.request_next_frame();
        }
    }

    info!("android_main: exited");
}

fn init_tracing() {
    let android_layer = layer("lume").expect("tracing_android init");

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,lume=debug,android_activity::activity_impl=off"));

    registry().with(android_layer).with(filter).init();
}
