mod choreographer;
mod platform_providers;
mod input_event;
pub mod android_ui_renderer;
mod input_handler;

use std::ffi::c_void;
use std::panic::set_hook;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use crate::choreographer::{FrameRateBinding, VsyncDriver};
use crate::platform_providers::io_provider::AndroidIOProvider;
use crate::platform_providers::surface_provider::AndroidSurfaceProvider;
use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use core::lume::Lume;
use ndk::asset::AssetManager;
use raw_window_handle::{AndroidDisplayHandle, RawDisplayHandle};
use tracing::{error, info};
use tracing_android::layer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{registry, EnvFilter};
use amber_lume::amber_lume::AmberLume;
use amber_lume::input_handler::hardware_key_codes::HardwareKeyCode;
use amber_lume::input_handler::hardware_pointer_event::HardwarePointerEvent;
use amber_lume::input_handler::input_frame::PointerId;
use amber_lume::lifecycle::lifecycle::AmberLumeLifecycle;
use amber_lume::limits::{AmberLumeLimits, HiZFormat, HiZParams, PhysicsLimits, ProfilerLimits, ResourceLimits, ShadowMapFormat, ShadowMapParams};
use amber_lume::platform_providers::surface_provider::SurfaceProvider;
use amber_lume::settings::settings::EngineSettings;
use crate::android_ui_renderer::AndroidUiRenderer;
use crate::input_event::translate_input_event;
use crate::input_handler::InputHandler;

const PREFERRED_FRAME_RATE_HZ: f32 = 120.0;
const FRAME_RATE_COMPATIBILITY_DEFAULT: i8 = 0;
const POLL_TIMEOUT_VSYNC: Duration = Duration::from_millis(8);

static ENGINE_TX: OnceLock<Sender<EngineEvent>> = OnceLock::new();

pub enum EngineEvent {
    AttachSurface(Arc<dyn SurfaceProvider>),
    DetachSurface,
    Pause,
    Resume,
    UpdateSurface,
    Keycode { code: HardwareKeyCode, pressed: bool },
    Pointer { id: PointerId, event: HardwarePointerEvent },
    Tick,
}

fn limits() -> AmberLumeLimits {
    AmberLumeLimits {
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
            max_render_views: 2,

            max_texture_descriptors: 1024,
            max_shadow_array_descriptors: 16,
            max_storage_image_descriptors: 64,
            max_graph_texture_descriptors: 256,
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
        hiz_limits: HiZParams {
            format: HiZFormat::Rg16,
        },
        physics_limits: PhysicsLimits {
            fixed_delta_time: 1.0 / 40.0,
        },
        profiler_limits: ProfilerLimits {
            max_gpu_zones: 64,
        },
    }
}

fn engine_main(rx: Receiver<EngineEvent>, asset_manager: AssetManager) {
    info!("Engine thread: started");

    let input_handler = Arc::new(InputHandler::new());
    let ui_renderer = Arc::new(AndroidUiRenderer::new(input_handler.clone()));
    let io_provider = Arc::new(AndroidIOProvider::new(asset_manager));
    let display_handle = RawDisplayHandle::Android(AndroidDisplayHandle::new());

    let amber_lume = AmberLume::new(
        limits(),
        vec![],
        vec![],
        ui_renderer,
        io_provider,
        display_handle,
        EngineSettings::default(),
    ).expect("AmberLume creation failed");
    let mut lume = Lume::new(amber_lume).expect("Lume creation failed");

    info!("Engine thread: Lume ready");

    while let Ok(event) = rx.recv() {
        process_event(&mut lume, event);

        while let Ok(event) = rx.try_recv() {
            process_event(&mut lume, event);
        }

        for (key_event, state) in input_handler.drain() {
            lume.push_hardware_keycode_event(key_event, state);
        }
    }

    info!("Engine thread: channel closed, exiting");
}

fn process_event(lume: &mut Lume, event: EngineEvent) {
    match event {
        EngineEvent::AttachSurface(provider) => {
            let target = match lume.create_surface_target(provider) {
                Ok(target) => target,
                Err(error) => {
                    error!("create_surface_target failed: {error:?}");
                    return;
                }
            };
            if let Err(error) = lume.attach_render_target(target) {
                error!("attach_render_target failed: {error:?}");
            }
        }
        EngineEvent::DetachSurface => {
            if let Err(error) = lume.detach_render_target() {
                error!("detach_render_target failed: {error:?}");
            }
        }
        EngineEvent::Pause => lume.pause(),
        EngineEvent::Resume => lume.resume(),
        EngineEvent::UpdateSurface => lume.on_update_surface(),
        EngineEvent::Keycode { code, pressed } => lume.push_hardware_keycode_event(code, pressed),
        EngineEvent::Pointer { id, event } => lume.push_hardware_pointer_event(&id, event),
        EngineEvent::Tick => {
            if let Err(error) = lume.draw() {
                error!("draw failed: {error:?}");
            }
        }
    }
}

#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    let tx = ENGINE_TX
        .get_or_init(|| {
            init_tracing();
            set_hook(Box::new(|info| error!("panic: {info}")));

            let (tx, rx) = channel::<EngineEvent>();
            let asset_manager = android_app.asset_manager();
            std::thread::Builder::new()
                .name("lume-engine".into())
                .spawn(move || engine_main(rx, asset_manager))
                .expect("failed to spawn engine thread");
            tx
        })
        .clone();

    info!("android_main: started");

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

    let mut quit = false;
    let mut attached = false;
    let mut last_size: Option<(u32, u32)> = None;

    while !quit {
        let poll_timeout = match (vsync_driver, attached) {
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

                let provider = Arc::new(AndroidSurfaceProvider::new(android_app.clone()));
                last_size = Some(provider.size());
                tx.send(EngineEvent::AttachSurface(provider)).ok();

                attached = true;
            }
            PollEvent::Main(MainEvent::Pause) => {
                info!("Pause");

                tx.send(EngineEvent::Pause).ok();
            }
            PollEvent::Main(MainEvent::Resume { .. }) => {
                info!("Resume");

                tx.send(EngineEvent::Resume).ok();
            }
            PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                info!("TerminateWindow");

                tx.send(EngineEvent::DetachSurface).ok();

                attached = false;
            }
            PollEvent::Main(MainEvent::WindowResized { .. }) => {
                let new_size = android_app
                    .native_window()
                    .map(|w| (w.width() as u32, w.height() as u32));
                if new_size != last_size {
                    info!("WindowResized: {:?} -> {:?}", last_size, new_size);

                    last_size = new_size;

                    tx.send(EngineEvent::UpdateSurface).ok();
                }
            }
            PollEvent::Main(MainEvent::Destroy) => {
                info!("Destroy");

                tx.send(EngineEvent::DetachSurface).ok();

                attached = false;
                quit = true;
            }
            _ => {}
        });

        match android_app.input_events_iter() {
            Ok(mut iter) => loop {
                let read = iter.next(|event| {
                    for engine_event in translate_input_event(event) {
                        tx.send(engine_event).ok();
                    }
                    InputStatus::Unhandled
                });
                if !read { break; }
            },
            Err(e) => error!("input_events_iter: {e:?}"),
        }

        if !attached {
            continue;
        }

        let should_draw = match vsync_driver {
            Some(driver) => driver.consume_frame(),
            None => true,
        };

        if !should_draw {
            continue;
        }

        tx.send(EngineEvent::Tick).ok();

        if let Some(driver) = vsync_driver {
            driver.request_next_frame();
        }
    }

    info!("android_main: exited (engine thread keeps running)");
}

fn init_tracing() {
    let android_layer = layer("lume").expect("tracing_android init");

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,lume=debug,android_activity::activity_impl=off"));

    registry().with(android_layer).with(filter).init();
}
