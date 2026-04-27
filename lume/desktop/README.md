[← AmberLume](../../README.md) · [lume/core](../core/README.md) · [lume/android](../android/README.md)

# lume/desktop

Desktop platform target for AmberLume.

---

## What it does

Provides `SurfaceProvider` and `IOProvider` implementations for desktop operating systems, then drives the main loop via `winit`'s `EventLoop`. All application logic lives in `lume/core`; this crate is a thin adapter between the OS window system and the engine.

---

## How it works

`main.rs` creates a winit `EventLoop` and an `Application` that implements `ApplicationHandler`:

- **`resumed`**: creates the window, constructs `DesktopIOProvider` (walks `assets/`) and `VulkanSurfaceProvider` (wraps the winit window), then calls `Lume::create`.
- **`window_event`**: routes keyboard, mouse, scroll, and resize events to the engine; triggers a redraw on each `about_to_wait`.
- **`CloseRequested`**: calls `Lume::on_close()` for a clean Vulkan teardown.

---

## Platform support

The launcher targets Linux, Windows, and macOS via winit. On Linux, X11 can be forced instead of Wayland:

```bash
cargo run -p desktop --features x11
```

This is useful when Wayland compositing adds unwanted frame latency.

---

## Building

The launcher must be started from `target/distribution/` — `DesktopIOProvider` walks `assets/` relative to the working directory, and `builder` writes all `.alpaca` archives to `target/distribution/assets/`.

```bash
# Compile assets first (required once, or after changing source assets)
cargo run -p builder
```

```bash
# Run on the default backend (Wayland on Linux)
cd target/distribution && cargo run -p desktop
```

```bash
# Run with X11 backend
cd target/distribution && cargo run -p desktop --features x11
```

---

## Configuration

`AmberLumeLimits` is constructed in `Application::resumed` and passed to `Lume::create`. Key parameters:

| Parameter | Current value | Notes |
|---|---|---|
| `frames_in_flight` | 2 | Memory multiplier for all per-frame data — entity buffers, projection buffers, draw calls, etc. Each additional frame-in-flight duplicates that data. 2–3 is optimal: below 2 loses pipelining, above 3 wastes memory for no gain |
| `max_render_views` | 5 | A render view is a projection of the scene — main camera, each shadow cascade, IBL bakes, etc. Not the same as a render pass. Each view needs its own culled entity list, so this multiplies the entity buffer. Too few crashes; too many wastes memory. Must equal the number of active projections |
| `max_staging_size` | 64 MB | Size of the per-frame CPU→GPU staging buffer. Limits how much data can be uploaded in one frame |
| `shadow_map resolution` | 4096 | Shadow map size in texels per cascade. Largest single contributor to RT memory (4096² × 4 cascades × D32 = ~256 MB) |
| `global_cascades` | 4 (0–64 m) | Number of shadow cascade splits. Each cascade is one render view and one full shadow map |
| `pcf_count` | 1 | PCF kernel radius. **Sample count = (2n + 1)²** — so 0 = 1 sample, 1 = 9, 2 = 25, 10 = 441. Grows quadratically; values above 2–3 are expensive |
| `shadow bias` | 0.00005 | Depth bias to prevent shadow acne. Needs tuning per scene |
| `max_entities` | 100 000 | Upper bound for ECS entities. Contributes to per-frame buffer sizes |

---

## Distribution

On-disk size of the asset archives produced by `builder`:

| Archive | Size |
|---|---|
| textures.alpaca | 13 MB |
| meshes.alpaca | 954 KB |
| animations.alpaca | 365 KB |
| scenes.alpaca | 37 KB |
| shaders.alpaca | 50 KB |
| skeletons.alpaca | 12 KB |
| physical_bodies.alpaca | 5.3 KB |
| materials.alpaca | 4.3 KB |
| **Total assets** | **~14.7 MB** |
| Binary (`desktop`, release) | 5.5 MB |

These numbers are not fixed. Because both the asset pipeline (`builder`) and the archive format (`alpaca`) are fully custom, every asset type can be compressed and packed however makes sense for the target. Nothing here is locked to a third-party format or toolchain constraint.

**Textures (13 MB / 86% of total)**
Currently stored as KTX2 with Zstandard supercompression and transcoded to BC7/ASTC at load time. The supercompression gives good universal compatibility but requires a decode step on load. Baking platform-specific formats at build time (BC7 for desktop, ASTC for Android) would eliminate the transcode entirely — loading becomes a straight copy from the archive into the staging buffer.

**Meshes (954 KB)**
Stored as raw rkyv-serialized data, zero-copy at runtime. Could be compressed with any general-purpose codec (zstd, lz4) at build time and decompressed on load — trading a small CPU cost for smaller archives. Mesh data is typically very compressible.

**Animations (365 KB)**
Stored as raw frame data. Can be compressed at rest and decompressed on load, or re-encoded with a curve-fitting scheme (storing control points instead of per-frame samples) to reduce size at the cost of a reconstruction step at runtime.

**Everything else** (scenes, shaders, skeletons, physics, materials) is already small enough that compression would have no meaningful impact.

The general point: `alpaca` is a container — blobs with names and offsets. What goes inside, how it is encoded, and whether it is compressed is entirely up to `builder`. The same scene could be packed leaner for a mobile build, faster-loading for desktop, or more compressed for a distribution bundle, with no changes to the engine or runtime.

---

## Runtime memory

Measured via `/proc/<pid>/smaps_rollup` on the same hardware with the full test scene loaded.

| Category | Size | Notes |
|---|---|---|
| **Anonymous** | **~85 MB** | heap, stack — actual RAM that cannot be evicted |
| Shared_Clean | ~90 MB | shared libraries (Vulkan loader, libc, etc.) — shared with other processes |
| **RSS total (btop)** | **~305 MB** | all of the above combined; overstates real usage by ~3.5× |
| GPU VRAM | ~820 MB | see GPU frame stats below |

The engine's actual RAM footprint is the Anonymous figure (~85 MB). The rest is either evictable file-backed pages or memory shared across processes.

---

## Performance

Tested on **AMD Ryzen 9 9950X3D + RTX 5080** at **1440p**.

Each pass is measured across three stages:
- **Prep** — data remapping and buffer uploads
- **Collect** — Vulkan command recording
- **Dispatch** — GPU shader execution time

All values in µs. Prep and Collect columns show **debug / release**; Dispatch is GPU time and is the same in both builds.

| Pass | Prep | Collect | Dispatch | Notes |
|---|---|---|---|---|
| Culling | ~30 / ~2 | ~60 / ~30 | ~30 | runs once per render view |
| Skinning | ~4 / ~0 | ~15 / ~8 | ~110 | |
| Depth prepass | ~0.5 / ~0 | ~40 / ~20 | ~9 | runs once per render view |
| Shadow map | ~0.5 / ~0 | ~40 / ~20 | ~80 | |
| Shadow mask | ~0.5 / ~0 | ~35 / ~18 | ~70 | |
| Main pass | ~0.5 / ~0 | ~25 / ~13 | ~80 | |
| Physics debug | ~20 / 0–40 | 20–1000 / 10–60 | 2–6 | collect spikes due to mapping and uploading line geometry per entity; a dedicated debug buffer would fix this, not a priority |
| UI | ~12 / ~10 | ~30 / ~15 | 5–20 | dispatch scales with UI complexity and window size |
| **Total** | **~70 / ~12** | **~270 / ~124 (up to ~1250 / ~630)** | **~390** | **~730 µs debug / ~526 µs release** |

> Times may vary: passes like culling and depth prepass execute once per render view. Desktop uses **5 render views** (main camera + 4 shadow cascades), so those pass times scale accordingly.

---

### Asset loading times

Time from scene load request to each asset type being logged as ready. "Ready" here means: data read from the mmap'd alpaca archive + decoded + queued into the staging buffer. The actual GPU transfer (staging → device-local VRAM) runs on a dedicated transfer queue and is not captured here.

The staging flush does not wait to accumulate — it submits as soon as data arrives. In practice: the first asset in a batch flushes almost instantly; the second and third wait for the first submit to complete and then flush together. Because of this the numbers below can vary depending on queue position and batch size.

| Asset type | Count | Time from request | Notes |
|---|---|---|---|
| Meshes | 9 | ~4 ms | rkyv zero-copy from mmap, no decode step |
| Materials | 12 | ~4.5 ms | concurrent with meshes, tiny data |
| Animations | 6 | ~12 ms | rkyv zero-copy, loaded after meshes |
| Textures — small (< 300 KB) | 15 | ~10–25 ms | Zstandard decompress + BC7 transcode per mip |
| Textures — large (3–4 MB KTX2) | 2 | ~60–62 ms | same pipeline, dominates total load time |

Meshes, materials, and animations are loaded before the first frame. Textures load asynchronously — the engine renders with a placeholder until each one is ready.

**Texture load time could be significantly reduced** by baking platform-specific compressed formats at build time (BC7 for desktop, ASTC for Android) instead of universal KTX2+Zstandard. In that case loading a texture would be a straight `memcpy` from the archive into the staging buffer with no decode step, making even large textures near-instant.

---

### GPU frame stats (RenderDoc capture)

Captured at **1440p**, debug build. GPU metrics are independent of the Rust build configuration — shaders are compiled separately by the GPU driver and optimized through a different pipeline, so debug vs release has no effect on dispatch times or GPU memory layout.

| Metric | Value | Notes |
|---|---|---|
| Draw calls | 665 | |
| Dispatch calls | 2 | skinning compute + one auxiliary |
| API calls | 119 | fixed overhead — barriers and state changes don't scale with draw count |
| API : draw ratio | 0.178 | |
| Textures | 31 — 36 MB | includes built-in generated textures (white pixel, neutral normal, etc.) created in code at startup |
| Render targets | 7 — 355 MB | dominated by shadow map cascades |
| Buffers | 32 — 429 MB | index buffers 2.67 MB, vertex buffers **0 MB** |
| **Total GPU footprint** | **820 MB** | |

**Vertex buffers are 0 MB** — the engine uses buffer-based vertex pulling (SSBOs) rather than traditional Vulkan vertex buffer bindings, so all geometry lives in the buffer allocation.

The render target budget is almost entirely shadow maps; reducing cascade count or resolution is the main lever for cutting GPU memory.
