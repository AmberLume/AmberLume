# AmberLume

A modular 3D engine written in Rust, built around Vulkan rendering, an ECS architecture, and a custom asset pipeline.

---

## Architecture Overview

```
[lume/desktop]  — Desktop launcher (winit, X11/Wayland)  ┐
[lume/android]  — Android launcher (android-activity)     ├─ implement platform traits
    ↓                                                      ┘
[lume/core]     — Platform-agnostic app layer (scene, ECS workloads, UI)
    ↓ calls
[amber_lume]    — Engine core (Vulkan, ECS, rendering, physics)
    ↓ reads archives at runtime
[alpaca]        — Binary asset archive format (pack + unpack)
    ↑ writes archives at build time
[builder]       — Asset compilation pipeline (glTF, GLSL, images → packed archives)
```

Build-time and runtime are cleanly separated: `builder` produces `.alpaca` archives, and `amber_lume` reads them. The engine never touches raw source assets.

The engine itself has no dependency on any OS, window system, or filesystem — it relies entirely on two traits:

- **`SurfaceProvider`** — raw window handles and size for Vulkan surface creation
- **`IOProvider`** — list of available asset paths for resource discovery

Platform launchers implement these traits and hand them to `lume/core`, which wires the engine and runs it.

---

## Platforms

| Platform | Crate | Details |
|---|---|---|
| Desktop | [`lume/desktop`](lume/desktop/README.md) | Linux (X11 / Wayland), Windows, macOS via winit |
| Android | [`lume/android`](lume/android/README.md) | Requires Vulkan 1.3, real device only |

---

## Modules

### `alpaca` — Asset Archive Format

**What it does**

Packs and unpacks binary asset archives (`.alpaca` files). Each archive holds a collection of named blobs — compiled shaders, meshes, textures, animations — indexed for fast lookup.

**How it works**

The archive has three sections written sequentially:

```
[ Header 64 bytes ] [ Data blobs (aligned) ] [ Index (rkyv-serialized) ]
```

- **Header**: magic bytes, version, offset and size of the index section.
- **Data blobs**: raw binary, padded to a configurable alignment (32–64 bytes) for SIMD and GPU upload compatibility.
- **Index**: a `Vec<IndexEntry>` (name + offset + size) serialized with `rkyv` and appended at the end so it can be read without scanning the whole file.

At runtime, `AlpacaReader` memory-maps the file with `memmap2`, deserializes the index via `rkyv::access` (zero-copy, no allocation), then returns `&[u8]` slices directly from the mmap for any requested asset.

At build time, `AlpacaWriter` appends blobs one by one, keeps an in-memory index, and finalizes everything on `pack()`.

**Why this design**

- **Zero-copy reads**: mmap + rkyv means the engine can hand a GPU upload buffer a pointer straight into the file mapping — no intermediate `Vec<u8>`.
- **Single seek on open**: the index is at the end, so `AlpacaReader::parse` does two reads: header, then index. No scanning.
- **Alignment for GPU**: padded offsets make it safe to pass slices directly to Vulkan upload commands without re-aligning.
- **One archive per asset type** (meshes.alpaca, textures.alpaca, …): selective loading — loading a level doesn't force all textures into memory.

---

### `builder` — Asset Compilation Pipeline

**What it does**

Transforms raw source assets — `.gltf` models, GLSL shaders, PNG textures — into optimized, serialized formats and bundles them into `.alpaca` archives.

**How it works**

`main.rs` scans the `resources/` directory and dispatches a `BuildTarget` per file through a chain of processors:

```
RouteTargetProcessor
  ├── ShaderProcessor      (.vert/.frag/.comp → SPIR-V via shaderc)
  ├── ExtractAssetsProcessor  (.gltf → SceneData, MeshData, SkeletonData,
  │                             AnimationData, MaterialData, PhysicalBodyData)
  ├── ConvertKTX2Processor (PNG → KTX2 via basis-universal compression)
  └── WriteFileProcessor   (serialize with rkyv, write to generated/alpaca/)
```

Processors run in parallel via `rayon`. Each processor may enqueue downstream tasks (e.g. extracting a glTF file enqueues one WriteFile task per extracted asset type). A `DashMap`-backed `Dispatcher` tracks completion with an `AtomicUsize` counter and a `Condvar` for `wait_all()`.

Final step: `pack_all()` calls `AlpacaWriter` once per asset type to bundle the generated intermediates into `distribution/assets/*.alpaca`.

**Output layout**

```
generated/alpaca/
  meshes/*.MESH
  scenes/*.SCENE
  skeletons/*.SKELETON
  animations/*.ANIMATION
  materials/*.MATERIAL
  physical_bodies/*.PHYSICAL_BODY
  shaders/*.spv
  textures/*.ktx2

distribution/assets/
  meshes.alpaca
  scenes.alpaca
  ...
```

**Why this design**

- **Parallel pipeline**: asset compilation is CPU and I/O bound; rayon saturates all cores with no lock contention (DashMap).
- **Processor chain**: adding a new asset type means writing one new `Processor<T>` implementation without touching anything else.
- **rkyv for intermediates**: the same zero-copy format used at runtime means no re-serialization step; what the builder writes is what the engine reads.
- **Build-time vs runtime**: heavy work (shader compilation, KTX2 encoding, glTF parsing) happens once at build time, keeping runtime startup fast.

---

### `amber_lume` — Engine Core

**What it does**

Everything needed to run a 3D scene: Vulkan device management, a render graph, an ECS world, resource loading with a lazy cache, physics (Rapier), skeletal animation, and UI.

**How it works**

`AmberLume::new()` wires the subsystems together in order:

```
VulkanContext         — instance, physical device selection
RenderSurface         — window integration (via SurfaceProvider trait)
DeviceContext         — logical device, queues
SwapchainContext      — swapchain images, presentation
ResourceHub           — all asset managers (providers + alpaca readers)
Render                — render graph + per-frame submission
World                 — Shipyard ECS (entities, components, workloads)
```

**Resource loading**

Each asset type has a `ResourceProvider<B: ResourceBackend>`. Providers lazy-load on first access, cache via a weak-reference index keyed by `ResourceKey`, and clean up automatically when nothing holds a strong reference. Loading is async: a crossbeam channel queues requests, a background thread reads from the alpaca archive, and the provider returns a placeholder until the asset is ready.

**Render graph**

Render passes declare their virtual image inputs/outputs. A `PassGraph` topologically sorts them by dependency, resolves virtual images to real `VkImage`s, and inserts barriers via `ImageStateTracker`. Pass execution order is determined automatically — no manual sequencing. Defined passes:

- Skinning (compute — writes final bone transforms)
- Culling (per-view frustum + Hi-Z occlusion)
- Depth prepass (also writes world normals and motion vectors)
- Shadows — raster (SDSM cascades + PCF resolve) or ray-traced (TLAS trace + temporal denoise)
- Ambient occlusion — raster (screen-space GTAO) or ray-traced (TLAS trace), shared temporal denoise
- Environment / sky
- Main (lit, PBR forward)
- Bloom + tonemap
- Physics debug
- UI overlay

Shadows and ambient occlusion each have interchangeable raster and ray-traced branches, selected at runtime; see the performance chapter in `lume/desktop/README.md` for the trade-offs. Each pass is an independent struct; the graph wires them without manual synchronization.

**ECS layout (Shipyard)**

- **Components**: mesh ref, skeleton, animation state, transform (position/rotation/scale), physics body, camera
- **Systems**: animation evaluation, physics step + sync, render snapshot collection, input processing, time update
- **Workloads**: systems are grouped and dependency-ordered; engine calls one workload per frame

**Platform abstraction**

The engine depends on two traits, never on a concrete window or filesystem:

- `SurfaceProvider` — returns raw window handles and window size for Vulkan surface creation
- `IOProvider` — returns a list of available asset file paths for resource discovery

**Why this design**

- **Trait-based platform layer**: swappable backend without touching engine code. Desktop and Android are supported; Android support is actively being developed.
- **Render graph**: automatic barrier insertion and image layout management eliminates a class of GPU synchronization bugs that plague manual Vulkan code.
- **ECS**: compositional scene structure — adding a physics body or a new animation variant is a component addition, not a class hierarchy change.
- **Lazy + weak-ref cache**: assets load on demand and free themselves when no entity references them, keeping memory use proportional to what's on screen.

---

### `lume/core` — Platform-Agnostic Application Layer

See [`lume/core/README.md`](lume/core/README.md).

Owns scene setup, ECS workload composition, and the per-frame `draw()` loop. Shared between all platform targets. Has no direct dependency on OS APIs — it only calls `amber_lume` and receives the two platform trait implementations from the launcher.

---

### `lume/desktop` — Desktop Launcher

See [`lume/desktop/README.md`](lume/desktop/README.md).

Thin adapter between `winit` and `lume/core`. Implements `SurfaceProvider` (winit window handles) and `IOProvider` (directory walk). Supports X11 via a feature flag on Linux.

---

### `lume/android` — Android Launcher

See [`lume/android/README.md`](lume/android/README.md).

Thin adapter between the Android NDK and `lume/core`. Built as a `cdylib`, loaded by a Gradle activity. Requires Vulkan 1.3 and a real device.

---

## Building

```bash
# Compile assets (run once, or after changing source assets)
cargo run -p builder
```

```bash
# Run on desktop (Wayland on Linux) — must run from target/distribution/
cd target/distribution && cargo run -p desktop
```

```bash
# Run on desktop with X11
cd target/distribution && cargo run -p desktop --features x11
```

```bash
# Build for Android (from lume/android/)
cd lume/android && ./gradlew installDebug
```
