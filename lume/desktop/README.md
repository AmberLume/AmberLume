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
| `max_render_views` | 2 | A render view is a projection of the scene — main camera, each shadow cascade, IBL bakes, etc. Not the same as a render pass. Each view needs its own culled entity list, so this multiplies the entity buffer. Too few crashes; too many wastes memory. Must equal the number of active projections |
| `max_staging_size` | 64 MB | Size of the per-frame CPU→GPU staging buffer. Limits how much data can be uploaded in one frame |
| `cascade_count` | 4 (SDSM) | Number of cascade splits. Cascade extents are derived per-frame by SDSM from the actual scene depth, so close-up scenes get tight, sharp cascades while deep vistas spread the same cascades out — coverage adapts to whatever is on screen instead of using fixed-distance splits. Rendered as a single multiview draw into a layered shadow map; cost no longer scales linearly with cascade count |
| `resolution` | 4096 | Shadow map size in texels per layer. Storage is `4096² × 4 layers × D32 = ~256 MB` in one layered image |
| `format` | D32 | Shadow map depth format. D16 cuts shadow RT memory in half at the cost of bias tuning |
| `pcf_sample_count` | 8 | Number of Poisson disk taps per shadow lookup. Cost is linear; visual quality saturates around 8–16 taps |
| `bias` / `normal_bias` | 0.02 / 0.08 | Depth and normal biases that prevent shadow acne. Need tuning per scene |
| `cascade_blend_range` | 0.05 | Fraction of cascade overlap used to hide seams between cascades |
| `split_lambda` | 0.7 | Mix between linear and logarithmic cascade splits; higher values push detail closer to the camera |
| `z_far_sample_stride` | 1 | SDSM depth sampling stride. `n` means every `n`-th pixel along each axis (so cost scales by `1/n²`). Desktop reads every pixel; mobile uses 4 |
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
- **Commands** — Vulkan command recording
- **Dispatch** — GPU shader execution time

All values in µs. Prep and Commands columns show **debug / release**; Dispatch is GPU time and is the same in both builds.

| Pass | Prep | Commands | Dispatch | Notes |
|---|---|---|---|---|
| Culling | ~33 / ~4 | ~21 / ~7 | ~13 | per-view frustum culling for the main camera |
| Сascade culling indirect | ~1 / ~0 | ~4 / ~3 | ~19 | one indirect cull dispatch covering every cascade view |
| Skinning | ~3 / ~0 | ~5 / ~3 | ~79 | compute pass; writes final bone transforms |
| Cascade compute | ~0 / ~0 | ~3 / ~2 | ~3 | derives cascade splits from the SDSM depth reduction |
| SDSM | ~1 / ~0 | ~3 / ~2 | ~29 | parallel reduction over the main depth buffer; sampling stride controlled by `z_far_sample_stride` |
| Shadows | ~0 / ~0 | ~13 / ~10 | ~28 | single multiview draw into a layered shadow map |
| Main | ~0 / ~0 | ~9 / ~7 | ~64 | depth prepass + lit forward pass; shadow filtering happens inline |
| Physics debug | ~20 / 0 | ~20 / ~10 | ~2 | unchanged in this pass; collect spikes due to mapping and uploading line geometry per entity, a dedicated debug buffer would fix this, not a priority |
| UI | ~12 / ~10 | ~6 / ~5 | ~5 | dispatch scales with UI complexity and window size |
| **Total** | **~50 / ~14** | **~64 / ~39** | **~256** | **~370 µs debug / ~309 µs release** |

> The shadow pass runs through Vulkan multiview: one recorded draw populates every cascade layer, the indirect cascade cull fills one shared buffer for all of them, and all cascades land in one layered shadow image. Cascades stay coherent (they are effectively the same pipeline running over the same geometry) without paying for duplicated buffers or extra record/read cycles.

---

### Shadows and ambient occlusion — raster and ray-traced

Both effects have two interchangeable branches, selected at runtime from **Debug → Render** (the ray-traced branches require a GPU with ray-query support and are gated on it). The rest of the frame — culling, depth prepass, main forward pass, bloom, tonemap — is identical either way; only the producer passes and the denoiser wiring change.

**Shadows**
- *Raster* — SDSM derives cascade splits from the actual scene depth, one multiview draw fills a layered shadow map, and `shadow_resolve` does the PCF lookup with receiver-plane depth bias. No temporal pass; filtering happens in the resolve.
- *Ray-traced* — `rt_shadow` traces a small disk of rays toward the sun against the TLAS (penumbra width comes from the sun's angular radius), then the shared temporal denoiser (`shadow_temporal`) accumulates the result over frames.

**Ambient occlusion**
- *Raster* — `gtao` is a screen-space XeGTAO port working off depth and G-buffer normals.
- *Ray-traced* — `rt_ao` traces a cosine-weighted hemisphere of short rays against the TLAS, bounded by the AO radius.

Both AO branches feed the same `denoise_guide` + `gtao_temporal` denoiser, so the two producers are drop-in interchangeable — the denoiser never learns which one wrote the raw signal. The shadow denoiser is the same generic pass under a second name.

**Measured cost** — RTX 5080, 1440p, GPU dispatch time in µs, averaged over ~14 stationary frames per branch at the same camera position. These numbers are close to a **worst case**: the camera looks straight into the scene with heavy overdraw and occlusion, most surfaces face the sun (so few shadow rays take the normal-facing early-out), and much of the frame sits in complex shadow.

| Stage | Raster | Ray-traced | Notes |
|---|--:|--:|---|
| AO trace | `gtao` 560 | `rt_ao` 407 | ray-traced AO is cheaper *and* more accurate here |
| AO denoise | `gtao_temporal` 118 + `denoise_guide` 33 | 119 + 37 | shared pass, identical between branches |
| Shadow trace | SDSM + cascades + `shadow_resolve` ≈ 288 | `rt_shadow` 674 + TLAS ≈ 26 | the ray-traced shadow trace is the whole RT premium |
| Shadow denoise | — | `shadow_temporal` 105 | raster PCF filters inline and needs no temporal pass |
| **Frame total** | **1647** | **2013** | **+366 µs (+22 %)** |

The entire ray-traced premium is shadows. `rt_shadow` at 674 µs is the single most expensive pass in the frame: a shadowed ray by definition descends into the BLAS and runs triangle intersections, while a lit ray leaves the BVH almost immediately, so a mostly-shadowed screen is the worst case — and this capture is one. Ambient occlusion goes the other way: ray tracing is ~150 µs *cheaper* than screen-space GTAO, which is why the two effects together net out to only +22 %.

Acceleration structures barely register in a steady-state frame: the TLAS is refit in place from updated transforms (~20 µs for the whole scene, with a full rebuild ~120 µs roughly once a second), the instance write is ~4 µs, and the BLAS is built lazily as meshes stream in (~120 µs total, once) and never appears again.

**Ambient occlusion — trade-offs.** Ray-traced AO is the better default here. It lies less than the screen-space version: occlusion comes from real geometry instead of reprojected depth, so it neither over- nor under-darkens on thin features, and it does not fall apart at screen edges or object silhouettes where GTAO runs out of samples. The cost is a little more noise. Two knobs trade that back — raise the sample count for a cleaner signal, or set `ao_trace_period` to trace each pixel every 1, 2, or 4 frames. Period 4 cuts the AO ray budget toward a quarter and reconstructs the gaps from history: a short trail on motion in exchange for a ~4× cheaper trace. AO is low-frequency enough that the trail is usually acceptable, and a mix of more samples plus amortization lands a cleaner result around ~200 µs.

**Shadows — trade-offs.** Ray-traced shadows cost more compute but buy things the cascade path cannot. The largest is memory: they need no shadow map, so a ray-traced-only configuration drops the whole cascade atlas — several 4K×4K D32 layers, ~256 MB (see the GPU frame stats below). They are also more physical: the penumbra is sized from the sun's actual angular radius and hardens with contact distance, instead of a fixed PCF kernel. Because the trace cost scales with the light's apparent size, a smaller sun needs fewer rays for the same quality, so it is tunable per scene. And there is headroom not yet taken — a proper temporal accumulation would cut the per-frame ray count sharply, trading some ghosting for a much cheaper trace. As it stands the ray-traced path is a deliberate balance: heavier per frame, lighter on memory, more physically correct, with room left to optimize.

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
| Dispatch calls | 5 | skinning + SDSM reduction + cascade compute + culling indirect dispatches |
| API calls | 115 | fixed overhead — barriers and state changes don't scale with draw count |
| API : draw ratio | 0.206 | |
| Textures | 30 — 36.4 MB | includes built-in generated textures (white pixel, neutral normal, etc.) created in code at startup |
| Render targets | 6 — 350.5 MB | shadow cascades share one layered image; the rest are main color, depth, and swapchain attachments |
| Buffers | 32 — 127.0 MB | index buffers 10.7 MB, vertex buffers 8.0 MB |
| **Total GPU footprint** | **~514 MB** | |

The shadow map is the largest single RT, so reducing layer count, resolution, or switching to D16 is the main lever for cutting it. The cull, draw, and view buffers are shared across all cascade layers — multiview drives all of them from one recorded draw, and the indirect cascade cull writes one shared buffer for the whole set.
